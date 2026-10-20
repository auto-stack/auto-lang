//! PLAN-797 T-05: unified cache from the first case, precise
//! invalidation, restart/cross-project hits, tamper/missing controls,
//! 512-revision churn with lease/GC convergence, and the six-scenario
//! turnaround measurement with the two P95<1000ms hard gates
//! (plan §8 T-05; V-06..V-09).
//!
//! Naming: `abi_cache_`, `abi_turnaround_`.

#![cfg(windows)]

use auto_abi_probe::cache::{
    autoabi_object_key, c_object_key, canonical_decl_set, emitter_identity, produce_with_cache,
    LAYER_OBJECT_V01,
};
use auto_abi_probe::decl::{AbiScalar, BodyKind, FnDecl};
use auto_abi_probe::emit::emit_callee_suite;
use auto_abi_probe::layout::{self, REC_MIX};
use auto_abi_probe::link::link_pe;
use auto_abi_probe::run;
use auto_abi_probe::semref;
use auto_abi_probe::trace::{append_jsonl, percentiles, Trace};
use auto_abi_probe::validate::{admit, probe_expectation};

use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::store::{gc_config_default, run_gc, ManagedStore, StoreError};
use std::path::{Path, PathBuf};
use std::time::Instant;

fn manifest() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn t05_root() -> PathBuf {
    // Per-RUN root (pid namespaced): parallel tests inside the run use
    // distinct store names; the next `cargo test` process starts clean
    // without wiping a sibling run's stores mid-flight.
    let p = manifest()
        .join("target/autoabi-t05")
        .join(format!("run-{}", std::process::id()));
    std::fs::create_dir_all(&p).unwrap();
    p
}

fn mix_alt_def(seed: u64) -> FnDecl {
    FnDecl {
        symbol: "p797_mix",
        params: auto_abi_probe::decl::MIX_PARAMS,
        ret: AbiScalar::I64,
        body: BodyKind::MixAlt(seed),
    }
}

fn mix_def(body: BodyKind) -> FnDecl {
    FnDecl {
        symbol: "p797_mix",
        params: auto_abi_probe::decl::MIX_PARAMS,
        ret: AbiScalar::I64,
        body,
    }
}

fn fresh_store(root: &Path, name: &str) -> ManagedStore {
    ManagedStore::open(&root.join(name)).expect("open store")
}

/// F-797-R4-07 (R14): the artifact-exclusion sets per test family —
/// ONLY files each test's own pipeline creates in the search dirs
/// (compile obj, linked obj, probe exe); headers with ANY name
/// (config.tmp included) are real environment.
/// F-797-R4-07 (R15): artifact exclusion is by FULL PATH — declare the
/// files a test's own pipeline creates in one directory (compile obj,
/// linked obj, probe exe); staged-link temporaries are covered by the
/// `<output>.tmp-` path-level prefix rule bound to these outputs.
fn arts(dir: &Path) -> Vec<PathBuf> {
    vec![
        dir.join("caller.obj"),
        dir.join("linked.obj"),
        dir.join("probe.exe"),
    ]
}

/// F-797-R15-01/R14: tests that mutate the PROCESS working directory
/// must serialize — cargo test runs them on parallel threads and two
/// concurrent set_current_dir calls break each other's relative anchors
static CWD_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

/// artifacts for a source file: they live in the source's directory
fn arts_for(src: &Path) -> Vec<PathBuf> {
    let d = src
        .parent()
        .filter(|p| !p.as_os_str().is_empty())
        .unwrap_or(Path::new("."));
    arts(&d.to_path_buf())
}

// ---------------------------------------------------------------------------
// abi_cache_ — hit/miss discipline and precise invalidation (V-06)
// ---------------------------------------------------------------------------

/// Hit never invokes the producer; miss persists; bytes are identical.
#[test]
fn abi_cache_object_hit_never_calls_producer() {
    let _scen = SCEN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = t05_root();
    let store = fresh_store(&root, "cache-hit");
    let defs = [mix_def(BodyKind::MixV1)];
    let key = autoabi_object_key(&defs, false);

    let mut produced = 0;
    let (obj1, hit1) = produce_with_cache(&store, &key, || {
        produced += 1;
        emit_callee_suite("c1", &defs, false)
    })
    .expect("first produce");
    assert!(!hit1, "fresh store must miss");
    assert_eq!(produced, 1);

    let (obj2, hit2) = produce_with_cache(&store, &key, || {
        produced += 1; // must NOT run on a hit
        emit_callee_suite("c1", &defs, false)
    })
    .expect("second produce");
    assert!(hit2, "same inputs must hit");
    assert_eq!(
        produced, 1,
        "producer invoked on a hit — no-producer rule broken"
    );
    assert_eq!(obj1, obj2, "hit must return the persisted bytes");

    // Restart: a NEW store instance on the same root still hits.
    drop(store);
    let store2 = fresh_store(&root, "cache-hit");
    let (obj3, hit3) = produce_with_cache(&store2, &key, || {
        produced += 1;
        emit_callee_suite("c1", &defs, false)
    })
    .expect("post-restart produce");
    assert!(hit3, "restart (disk) must hit");
    assert_eq!(produced, 1);
    assert_eq!(obj1, obj3);
}

/// Real inputs change the key: body edit, import mode, C source content,
/// and (by construction of the key inputs) profile/toolchain/flags
/// dimensions all produce different hashes — stale reuse is impossible.
#[test]
fn abi_cache_invalidation_by_real_inputs() {
    let _scen = SCEN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let v1 = [mix_def(BodyKind::MixV1)];
    let v1b = [mix_def(BodyKind::MixV1Alt)];
    let extra = [FnDecl {
        symbol: "p797_extra",
        params: &[],
        ret: AbiScalar::I64,
        body: BodyKind::InnerV1,
    }];

    // body edit -> different canonical decl set -> different key
    assert_ne!(canonical_decl_set(&v1), canonical_decl_set(&v1b));
    assert_ne!(
        autoabi_object_key(&v1, false).hash(),
        autoabi_object_key(&v1b, false).hash()
    );
    // symbol set change -> different key
    assert_ne!(
        autoabi_object_key(&v1, false).hash(),
        autoabi_object_key(&extra, false).hash()
    );
    // link mode (import-inner) is a key input
    assert_ne!(
        autoabi_object_key(&v1, false).hash(),
        autoabi_object_key(&v1, true).hash()
    );
    // C source content is a key input
    assert_ne!(
        c_object_key("aaa", "c1").hash(),
        c_object_key("bbb", "c1").hash()
    );
    // the OBSERVED closure generation is part of the key
    assert_ne!(
        c_object_key("aaa", "c1").hash(),
        c_object_key("aaa", "c2").hash()
    );
    // profile / toolchain / semref dimensions are key inputs by
    // construction — assert two differently-tooled keys cannot collide.
    let q = |tool: &str| {
        QueryKey::new(
            &LAYER_OBJECT_V01,
            vec![
                ("kind".into(), "autoabi-emit".into()),
                ("decl-set".into(), canonical_decl_set(&v1)),
                ("toolchain".into(), tool.into()),
                ("profile".into(), layout::ABI_PROFILE.into()),
                ("semref".into(), semref::binding_hash().into()),
            ],
        )
    };
    assert_ne!(q(&emitter_identity()).hash(), q("cl:other|ver:x").hash());
    let qp = |profile: &str| {
        QueryKey::new(
            &LAYER_OBJECT_V01,
            vec![
                ("kind".into(), "autoabi-emit".into()),
                ("decl-set".into(), canonical_decl_set(&v1)),
                ("toolchain".into(), emitter_identity()),
                ("profile".into(), profile.into()),
                ("semref".into(), semref::binding_hash().into()),
            ],
        )
    };
    assert_ne!(
        qp(layout::ABI_PROFILE).hash(),
        qp("autoabi.win-x64.v0.2-candidate").hash()
    );
}

/// Cross-project sharing: two project labels over ONE store reuse the
/// same object (per-project duplication is not the model).
#[test]
fn abi_cache_cross_project_sharing() {
    let _scen = SCEN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = t05_root();
    let store = fresh_store(&root, "cache-xproj");
    let defs = [mix_def(BodyKind::MixV1)];
    let key = autoabi_object_key(&defs, false);
    let (_, hit_a) = produce_with_cache(&store, &key, || emit_callee_suite("cx", &defs, false))
        .expect("project A produces");
    assert!(!hit_a);
    // project B: same store, same content -> hit without producing
    let (_, hit_b) = produce_with_cache(&store, &key, || {
        panic!("project B must reuse the shared object");
    })
    .expect("project B reads");
    assert!(hit_b);
}

/// Good frame, wrong address (V-07): a tampered block cannot serve its
/// content — the store's CAS admission rejects it as a controlled
/// error, never as wrong bytes.
#[test]
fn abi_cache_tampered_block_is_controlled_error() {
    let _scen = SCEN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = t05_root();
    let store = fresh_store(&root, "cache-tamper");
    let defs = [mix_def(BodyKind::MixV1)];
    let key = autoabi_object_key(&defs, false);
    let (obj, _) = produce_with_cache(&store, &key, || emit_callee_suite("ct", &defs, false))
        .expect("produce for tamper test");

    // Flip one byte in the persisted block payload.
    // Block payloads live under blocks/blobs/<shard>/<chunk> — walk
    // to the actual payload files.
    let blobs = store.root.join("blocks/blobs");
    let mut flipped = 0;
    let mut files: Vec<PathBuf> = Vec::new();
    for shard in std::fs::read_dir(&blobs).unwrap().flatten() {
        if !shard.file_type().unwrap().is_dir() {
            continue;
        }
        for e in std::fs::read_dir(shard.path()).unwrap().flatten() {
            if e.file_type().unwrap().is_file() {
                files.push(e.path());
            }
        }
    }
    assert!(!files.is_empty(), "store has no block payloads to tamper");
    for p in files {
        if p.extension().and_then(|e| e.to_str()) == Some("tmp") {
            continue;
        }
        let mut bytes = std::fs::read(&p).unwrap();
        if bytes.len() > 60 && bytes[..8] == *b"ABIOBJ\0\0" || bytes.len() > 60 {
            let n = bytes.len();
            bytes[n - 1] ^= 0xFF;
            std::fs::write(&p, &bytes).unwrap();
            flipped += 1;
            break;
        }
    }
    assert_eq!(flipped, 1, "expected to tamper exactly one block");

    match store.get_artifact(LAYER_OBJECT_V01.id, &key) {
        Err(StoreError::ChunkAddressMismatch { .. }) => { /* controlled rejection */ }
        Err(StoreError::Codec(_)) => { /* framed-checksum rejection — equally controlled */ }
        other => panic!(
            "tampered block must be a controlled error, got {other:?} \
             (wrong bytes would be silently corrupt reuse)"
        ),
    }
    // The original object bytes are still known-good from the producer.
    assert!(!obj.is_empty());
}

/// Missing block (V-07): a vanished blob reads as a MISS (recompute),
/// never as an error the caller cannot recover from.
#[test]
fn abi_cache_missing_block_is_miss_then_recompute() {
    let _scen = SCEN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = t05_root();
    let store = fresh_store(&root, "cache-missing");
    let defs = [mix_def(BodyKind::MixV1)];
    let key = autoabi_object_key(&defs, false);
    let _ = produce_with_cache(&store, &key, || emit_callee_suite("cm", &defs, false))
        .expect("produce");

    // Delete the persisted block bytes.
    let blobs = store.root.join("blocks/blobs");
    let mut removed = 0;
    let mut files: Vec<PathBuf> = Vec::new();
    for shard in std::fs::read_dir(&blobs).unwrap().flatten() {
        if !shard.file_type().unwrap().is_dir() {
            continue;
        }
        for e in std::fs::read_dir(shard.path()).unwrap().flatten() {
            if e.file_type().unwrap().is_file() {
                files.push(e.path());
            }
        }
    }
    for p in files {
        if p.extension().and_then(|e| e.to_str()) == Some("tmp") {
            continue;
        }
        std::fs::remove_file(&p).unwrap();
        removed += 1;
    }
    assert!(removed >= 1, "expected to remove the object block");

    // Reads miss; produce_with_cache recomputes and re-persists.
    let mut produced = 0;
    let (_, hit) = produce_with_cache(&store, &key, || {
        produced += 1;
        emit_callee_suite("cm", &defs, false)
    })
    .expect("recompute after miss");
    assert!(!hit, "missing block must read as a miss");
    assert_eq!(produced, 1);
    let (_, hit2) = produce_with_cache(&store, &key, || unreachable!()).unwrap();
    assert!(hit2, "re-persisted object hits again");
}

/// V-08: 512 interface revisions — rooted newest + one leased old
/// revision survive GC; everything unreferenced is reclaimed; the total
/// converges under the watermark; user-owned files outside the managed
/// namespaces are untouched.
#[test]
fn abi_cache_512_revision_churn_gc_convergence() {
    let _scen = SCEN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = t05_root();
    let store = fresh_store(&root, "cache-churn");

    let payload: Vec<u8> = (0..2048).map(|i| (i % 251) as u8).collect();
    let mut last_art = String::new();
    let mut leased_art = String::new();
    for i in 0..512 {
        let key = c_object_key(&format!("rev-{i:04}"), &format!("closure-{i:04}"));
        // Distinct content per revision: real invalidation behavior
        // needs distinct keys, and distinct payloads keep CAS honest.
        let mut p = payload.clone();
        let n = p.len();
        p[n - 1] = (i % 251) as u8;
        p[n - 2] = ((i >> 8) % 251) as u8;
        let art = store
            .put_artifact(ac_artifact_probe::store::PutRequest {
                layer: LAYER_OBJECT_V01.id,
                key,
                kind: ac_artifact_probe::codec::ChunkKind::Object,
                payload: p,
                edges: vec![],
                project: None,
            })
            .expect("put revision");
        if i == 511 {
            last_art = art.clone();
        }
        if i == 100 {
            leased_art = art;
        }
    }

    // Root the newest; lease revision 100 (an OLD version) live.
    store
        .register_root("autoabi-churn", &last_art, "newest")
        .expect("root newest");
    let lease = store
        .acquire_lease("churn-live-old", 60, Some(&leased_art))
        .expect("lease old revision");

    // User sentinel: OUTSIDE the store's managed namespaces but inside
    // the scenario dir — GC must not touch it.
    let sentinel_path = root.join("cache-churn").with_extension("user.exe");
    let sentinel = b"USER-OWNED-DELIVERABLE-SENTINEL";
    std::fs::write(&sentinel_path, sentinel).unwrap();

    // Force collection with watermarks measured against the REAL
    // uncompressible floor (defaults are 64MiB/48MiB; the churn set is
    // ~1MiB, so the probe-sized config is the honest way to exercise
    // the path on this data volume). The floor is the main DB + locks +
    // logs; `low` must sit above it or GC correctly backs off.
    let space0 = store.space();
    let floor = space0.db_main + space0.locks + space0.logs;
    let total0 = space0.total;
    assert!(total0 > floor, "churn must add block bytes over the floor");
    let cfg = ac_artifact_probe::store::GcConfig {
        high: total0 - 1,       // force the run
        low: floor + 64 * 1024, // achievable: live bytes are tiny
    };
    let receipt = run_gc(&store, cfg).expect("gc");
    assert!(
        receipt.ran,
        "GC must run over the watermark: {}",
        receipt.reason
    );
    assert!(
        receipt.deleted_chunks >= 500,
        "unreferenced revisions reclaimed (deleted={})",
        receipt.deleted_chunks
    );

    // Newest (rooted) and revision 100 (leased) still readable.
    let newest = store
        .get_artifact(
            LAYER_OBJECT_V01.id,
            &c_object_key("rev-0511", "closure-0511"),
        )
        .expect("read newest")
        .expect("rooted newest survives GC");
    assert_eq!(newest.payload.len(), 2048);
    let old = store
        .get_artifact(
            LAYER_OBJECT_V01.id,
            &c_object_key("rev-0100", "closure-0100"),
        )
        .expect("read leased")
        .expect("leased old revision survives GC");
    assert_eq!(old.payload.len(), 2048);

    // Convergence: the managed BLOCK layer collapses to the live set
    // (the ~1MiB of dead revisions is gone) and the total shrinks net.
    // The SQLite main file is an uncompressible floor under incremental
    // auto_vacuum — it does not synchronously shrink within one GC run,
    // so convergence is measured as total minus floor (blocks+staging),
    // not as raw total.
    let space1 = store.space();
    let floor1 = space1.db_main + space1.locks + space1.logs;
    let managed_live = space1.total.saturating_sub(floor1);
    assert!(
        managed_live <= 128 * 1024,
        "block layer must converge to the live set (~2 chunks), got {managed_live}B"
    );
    assert!(
        space1.total < total0,
        "GC reclaimed nothing ({} vs {total0})",
        space1.total
    );

    // User sentinel untouched, byte-identical.
    assert_eq!(std::fs::read(&sentinel_path).unwrap(), sentinel);

    // Release the lease; the liveness observation flips (the revision is
    // now reclaimable by any later GC — the plan's "取消释放lease" rule).
    let lease_id = lease.lease_id.clone();
    lease.release();
    assert!(!store.lease_live(&lease_id).expect("lease liveness check"));
    let _ = gc_config_default(); // default watermarks kept for reference runs
}

// ---------------------------------------------------------------------------
// abi_turnaround_ — six scenarios x 30 samples, two hard P95 gates (V-09)
// ---------------------------------------------------------------------------

/// One probe request: admission -> layout -> callee produce (cached) ->
/// C consumer (cached compile) -> link -> run -> exit-code compare.
/// Every stage is traced; the PE always runs and re-compares (PASS is
/// never cached). Returns (ok, trace).
fn probe_request(
    case: &str,
    store: Option<&ManagedStore>,
    defs: &[FnDecl],
    lay_def: &auto_abi_probe::layout::RecordDef,
    scenario: &'static str,
    iter: u32,
) -> (bool, auto_abi_probe::trace::RequestTrace) {
    let dir = run::scratch(&format!("t05-{case}"));
    let mut tr = Trace::new(scenario, iter);
    let mut ok = true;

    // descriptor admission (the consumer side gate)
    let descriptor = auto_abi_probe::descriptor::ModuleDescriptor::probe_module();
    tr.stage("admit", || {
        match admit(&descriptor.encode().unwrap(), &probe_expectation()) {
            Ok(_) => auto_abi_probe::trace::StageMeta::default(),
            Err(e) => {
                ok = false;
                auto_abi_probe::trace::StageMeta {
                    note: Some(format!("{e}")),
                    ..Default::default()
                }
            }
        }
    });

    // layout (cached since T-01)
    tr.stage("layout", || {
        let computed = layout::layout_record(lay_def).expect("layout");
        if let Some(st) = store {
            if layout::get_layout(st, lay_def)
                .expect("get layout")
                .is_none()
            {
                layout::put_layout(st, lay_def, &computed).expect("put layout");
                auto_abi_probe::trace::StageMeta::hit(false)
            } else {
                auto_abi_probe::trace::StageMeta::hit(true)
            }
        } else {
            auto_abi_probe::trace::StageMeta::hit(false)
        }
    });

    // callee object (produce-through-cache)
    let mut callee_obj = Vec::new();
    tr.stage("emit-callee", || {
        let key = autoabi_object_key(defs, false);
        let (obj, hit) = match store {
            Some(st) => {
                produce_with_cache(st, &key, || emit_callee_suite("t05-callee", defs, false))
                    .expect("callee produce")
            }
            None => (
                emit_callee_suite("t05-callee", defs, false).expect("emit"),
                false,
            ),
        };
        callee_obj = obj;
        auto_abi_probe::trace::StageMeta {
            hit: Some(hit),
            bytes: Some(callee_obj.len() as u64),
            note: None,
        }
    });

    // C consumer (caller_main.c) — compile once, then object hits
    let mut c_obj_path = PathBuf::new();
    tr.stage("c-consumer", || {
        let src = manifest().join("cside/caller_main.c");
        let content = std::fs::read(&src).expect("read C consumer");
        let hash = semref::blake3_hex(&content);
        let key = c_object_key(&hash, &hash);
        let (obj, hit) = match store {
            Some(st) => produce_with_cache(st, &key, || {
                let obj = dir.join("consumer-once.obj");
                auto_abi_probe::ctool::compile_c(&src, &manifest().join("cside"), &obj)
                    .expect("compile C consumer");
                Ok(std::fs::read(&obj).unwrap())
            })
            .expect("C consumer produce"),
            None => {
                let obj = dir.join("consumer-once.obj");
                auto_abi_probe::ctool::compile_c(&src, &manifest().join("cside"), &obj)
                    .expect("compile C consumer");
                (std::fs::read(&obj).unwrap(), false)
            }
        };
        let p = dir.join("consumer.obj");
        std::fs::write(&p, &obj).unwrap();
        c_obj_path = p;
        auto_abi_probe::trace::StageMeta {
            hit: Some(hit),
            bytes: Some(obj.len() as u64),
            note: None,
        }
    });

    // link + run + compare (ALWAYS real; the C consumer carries the mix
    // oracle itself — exit 0 IS the compare)
    tr.stage("link-run-compare", || {
        let callee_p = dir.join("callee.obj");
        std::fs::write(&callee_p, &callee_obj).unwrap();
        match link_pe(&dir, &[c_obj_path.clone(), callee_p], "p797_c_entry") {
            Ok(exe) => match run::run_pe(&exe) {
                Ok(pe) => {
                    if pe.code != 0 {
                        ok = false;
                        auto_abi_probe::trace::StageMeta {
                            note: Some(format!("exit={}", pe.code)),
                            ..Default::default()
                        }
                    } else {
                        auto_abi_probe::trace::StageMeta::default()
                    }
                }
                Err(e) => {
                    ok = false;
                    auto_abi_probe::trace::StageMeta {
                        note: Some(format!("{e}")),
                        ..Default::default()
                    }
                }
            },
            Err(e) => {
                ok = false;
                auto_abi_probe::trace::StageMeta {
                    note: Some(format!("{e}")),
                    ..Default::default()
                }
            }
        }
    });

    (ok, tr.finish(ok))
}

/// The descriptor-only-reject request: an edited descriptor is rejected
/// at admission — the whole request is the diagnosis, nothing links.
fn reject_request(
    scenario: &'static str,
    iter: u32,
) -> (bool, auto_abi_probe::trace::RequestTrace) {
    let mut tr = Trace::new(scenario, iter);
    let mut ok = false;
    let mut descriptor = auto_abi_probe::descriptor::ModuleDescriptor::probe_module();
    descriptor.semref_hash = semref::blake3_hex(b"edited-descriptor");
    tr.stage("admit", || {
        match admit(&descriptor.encode().unwrap(), &probe_expectation()) {
            Err(auto_abi_probe::validate::Rejection::SemRefMismatch { .. }) => {
                ok = true; // the CORRECT diagnosis
                auto_abi_probe::trace::StageMeta::default()
            }
            other => {
                ok = false;
                auto_abi_probe::trace::StageMeta {
                    note: Some(format!("wrong diagnosis: {other:?}")),
                    ..Default::default()
                }
            }
        }
    });
    (ok, tr.finish(ok))
}

fn p95_of(samples: &[f64]) -> f64 {
    percentiles(samples.to_vec()).1
}

/// Six scenarios x 30 samples each, serially in one test (parallel
/// sampling would contend and inflate the numbers). Hard gates:
/// body-edit request -> correct PE result P95 < 1000ms; descriptor-only
/// reject request -> correct diagnosis P95 < 1000ms. Raw JSONL lines
/// and a summary land under target/autoabi-t05/turnaround/.
/// R4: the six-scenario run builds 210 PEs — serialize it against the
/// cache tests in this binary so machine contention cannot flip the
/// hard-gate P95 (observed flake in full-suite runs).
static SCEN_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn abi_turnaround_six_scenarios_30_samples() {
    let _scen = SCEN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = t05_root().join("turnaround");
    std::fs::create_dir_all(&root).unwrap();
    let mut summary = String::from(
        "# PLAN-797 T-05 turnaround raw summary

",
    );

    // Prewarm tool discovery OUTSIDE the timed requests (tool prep is
    // reported separately, per contract-decision §9).
    let t0 = Instant::now();
    auto_abi_probe::ctool::find_cl().expect("cl prewarm");
    let cl_prewarm_ms = t0.elapsed().as_secs_f64() * 1000.0;
    summary.push_str(&format!(
        "cl discovery prewarm (untimed): {:.1} ms

",
        cl_prewarm_ms,
    ));

    struct Scen {
        name: &'static str,
        totals: Vec<f64>,
        oks: usize,
    }

    let mut scenarios: Vec<Scen> = Vec::new();

    // 1) fresh: a new empty store per sample — everything cold.
    {
        let mut s = Scen {
            name: "fresh",
            totals: Vec::new(),
            oks: 0,
        };
        for i in 0..30 {
            let store = ManagedStore::open(&root.join(format!("fresh-{i}"))).unwrap();
            let defs = [mix_def(BodyKind::MixV1)];
            let (ok, tr) = probe_request(
                &format!("fresh-{i}"),
                Some(&store),
                &defs,
                &REC_MIX,
                "fresh",
                i,
            );
            append_jsonl(&root, &tr).unwrap();
            if ok {
                s.oks += 1;
            }
            s.totals.push(tr.total_ms);
        }
        scenarios.push(s);
    }

    // 2) layout-hot: shared store; layout hits, callee body alternates
    //    (object misses) — isolates the layout-cache tier.
    {
        let store = fresh_store(&root, "layout-hot-store");
        // warm the layout once, untimed
        {
            let defs = [mix_def(BodyKind::MixV1)];
            let _ = probe_request("layout-warm", Some(&store), &defs, &REC_MIX, "warm", 0);
        }
        let mut s = Scen {
            name: "layout-hot",
            totals: Vec::new(),
            oks: 0,
        };
        for i in 0..30 {
            let body = if i % 2 == 0 {
                BodyKind::MixV1Alt
            } else {
                BodyKind::MixV1
            };
            let defs = [mix_def(body)];
            let (ok, tr) = probe_request(
                &format!("lh-{i}"),
                Some(&store),
                &defs,
                &REC_MIX,
                "layout-hot",
                i,
            );
            append_jsonl(&root, &tr).unwrap();
            if ok {
                s.oks += 1;
            }
            s.totals.push(tr.total_ms);
        }
        scenarios.push(s);
    }

    // 3) object-hot: shared store, everything hits; link+run stay real.
    {
        let store = fresh_store(&root, "object-hot-store");
        let defs = [mix_def(BodyKind::MixV1)];
        let _ = probe_request("oh-warm", Some(&store), &defs, &REC_MIX, "warm", 0);
        let mut s = Scen {
            name: "object-hot",
            totals: Vec::new(),
            oks: 0,
        };
        for i in 0..30 {
            let (ok, tr) = probe_request(
                &format!("oh-{i}"),
                Some(&store),
                &defs,
                &REC_MIX,
                "object-hot",
                i,
            );
            append_jsonl(&root, &tr).unwrap();
            if ok {
                s.oks += 1;
            }
            s.totals.push(tr.total_ms);
        }
        scenarios.push(s);
    }

    // 4) restart-disk-hot: reopen the store instance before every
    //    sample — all tiers hit from disk, nothing process-resident.
    {
        let store_dir = root.join("restart-store");
        {
            let store = ManagedStore::open(&store_dir).unwrap();
            let defs = [mix_def(BodyKind::MixV1)];
            let _ = probe_request("rs-warm", Some(&store), &defs, &REC_MIX, "warm", 0);
        }
        let mut s = Scen {
            name: "restart-disk-hot",
            totals: Vec::new(),
            oks: 0,
        };
        for i in 0..30 {
            let store = ManagedStore::open(&store_dir).unwrap();
            let defs = [mix_def(BodyKind::MixV1)];
            let (ok, tr) = probe_request(
                &format!("rs-{i}"),
                Some(&store),
                &defs,
                &REC_MIX,
                "restart-disk-hot",
                i,
            );
            append_jsonl(&root, &tr).unwrap();
            if ok {
                s.oks += 1;
            }
            s.totals.push(tr.total_ms);
        }
        scenarios.push(s);
    }

    // 5) body-edit (HARD GATE, R4 uniform-cold): every sample runs a
    //    DISTINCT mix-alt seed — an exactly-semantics-preserving body
    //    whose canonical form differs, so each sample is a COLD producer
    //    run (no hot-hit majority); the C consumer object is reused from
    //    the store. P95 of request->correct PE result < 1000ms.
    {
        let store = fresh_store(&root, "body-edit-store");
        {
            let defs = [mix_def(BodyKind::MixV1)];
            let _ = probe_request("be-warm", Some(&store), &defs, &REC_MIX, "warm", 0);
        }
        let mut s = Scen {
            name: "body-edit",
            totals: Vec::new(),
            oks: 0,
        };
        for i in 0..30 {
            let defs = [mix_alt_def(i as u64)];
            let (ok, tr) = probe_request(
                &format!("be-{i}"),
                Some(&store),
                &defs,
                &REC_MIX,
                "body-edit",
                i,
            );
            append_jsonl(&root, &tr).unwrap();
            if ok {
                s.oks += 1;
            }
            s.totals.push(tr.total_ms);
        }
        scenarios.push(s);
    }

    // 6) abi/layout-edit: the layout input changes per request (field
    //    order variant -> layout tier misses) while the call surface
    //    stays correct; GC observation recorded after the run.
    {
        let store = fresh_store(&root, "abi-edit-store");
        let mut s = Scen {
            name: "abi-layout-edit",
            totals: Vec::new(),
            oks: 0,
        };
        for i in 0..30 {
            let lay = if i % 2 == 0 {
                &REC_MIX
            } else {
                &auto_abi_probe::layout::REC_MIX_SWAPPED
            };
            let defs = [mix_def(BodyKind::MixV1)];
            let (ok, tr) = probe_request(
                &format!("ae-{i}"),
                Some(&store),
                &defs,
                lay,
                "abi-layout-edit",
                i,
            );
            append_jsonl(&root, &tr).unwrap();
            if ok {
                s.oks += 1;
            }
            s.totals.push(tr.total_ms);
        }
        // GC observation (post-scenario): under this volume the default
        // watermarks leave GC dormant — recorded as-is, no threshold games.
        let receipt = run_gc(&store, gc_config_default()).expect("gc observation");
        summary.push_str(&format!(
            "abi-layout-edit GC observation: ran={} reason=\"{}\" total_before={}B total_after={}B
",
            receipt.ran, receipt.reason, receipt.total_before, receipt.total_after
        ));
        scenarios.push(s);
    }

    // Summary + assertions. All samples must be green (no deleting
    // failures to pass a gate).
    for s in &scenarios {
        let (p50, p95, p99) = percentiles(s.totals.clone());
        summary.push_str(&format!(
            "{:16} n={} ok={} P50={:.1}ms P95={:.1}ms P99={:.1}ms
",
            s.name,
            s.totals.len(),
            s.oks,
            p50,
            p95,
            p99
        ));
        assert_eq!(s.oks, 30, "scenario {} lost green samples", s.name);
    }

    // HARD GATES (contract-decision §9): both P95 < 1000ms.
    let body_edit = scenarios.iter().find(|s| s.name == "body-edit").unwrap();
    assert!(
        p95_of(&body_edit.totals) < 1000.0,
        "HARD GATE body-edit P95={:.1}ms >= 1000ms",
        p95_of(&body_edit.totals)
    );

    let mut reject_totals = Vec::new();
    for i in 0..30 {
        let (ok, tr) = reject_request("descriptor-reject", i);
        append_jsonl(&root, &tr).unwrap();
        assert!(ok, "reject request {i} produced the wrong diagnosis");
        reject_totals.push(tr.total_ms);
    }
    let (r50, r95, r99) = percentiles(reject_totals.clone());
    summary.push_str(&format!(
        "{:16} n=30 ok=30 P50={:.1}ms P95={:.1}ms P99={:.1}ms
",
        "descriptor-reject", r50, r95, r99
    ));
    assert!(
        p95_of(&reject_totals) < 1000.0,
        "HARD GATE descriptor-reject P95={:.1}ms >= 1000ms",
        p95_of(&reject_totals)
    );

    std::fs::write(root.join("summary.md"), &summary).unwrap();
    println!("{summary}");
}

// ---------------------------------------------------------------------------
// F-797-R4-07 (R8): REAL dependency invalidation — the closure is
// OBSERVED from `cl -showIncludes` (stored beside the object) and
// RE-VERIFIED on every hit. The old text-derived key could not see
// comment-hidden / line-spliced / macro includes: those sibling forms
// kept the same key while the compiled PE changed (R8 counterexample).
// ---------------------------------------------------------------------------

/// The value-tracking oracle over a REAL compile/link/run per spelling:
/// with the SOURCE byte-identical throughout, changing the header VALUE
/// 17→29 must invalidate the hit (recompile) and change the PE exit —
/// for every legal preprocessor spelling, INCLUDING the three R8
/// siblings and the macro include.
#[test]
fn abi_cache_include_real_deps_invalidated_and_reused() {
    let _scen = SCEN_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let root = t05_root().join("inc-real");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let store = fresh_store(&root, "store");

    let forms: &[(&str, &str)] = &[
        ("plain", "#include \"dep.h\""),
        ("trailing_comment", "#include \"dep.h\" // c"),
        ("spaced", "#  include \"dep.h\""),
        ("angle_local", "#include <dep.h>"),
        // R8 siblings: legal preprocessing the R5 text scanner silently
        // dropped — the header VALUE changed the compiled behavior but
        // not the text-derived key.
        (
            "leading_comment",
            "/* legal preprocessing whitespace */ #include \"dep.h\"",
        ),
        ("hash_comment", "#/**/include \"dep.h\""),
        ("spliced", "#inc\\\nlude \"dep.h\""),
        // macro include: unenumerable from text — observable by the
        // compiler's own preprocessor, so it caches SAFELY now
        ("macro", "#define HDR \"dep.h\"\n#include HDR"),
    ];

    for (name, directive) in forms {
        let dir = root.join(name);
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("caller.c");
        std::fs::write(
            &src,
            format!(
                "{directive}\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) {{ ExitProcess(VALUE); }}\n"
            ),
        )
        .unwrap();
        let write_header = |value: u32| {
            std::fs::write(dir.join("dep.h"), format!("#define VALUE {value}\n")).unwrap()
        };
        let obj_path = dir.join("caller.obj");
        let mut producer_runs = 0u32;
        // a fresh FnOnce per call (borrowed counter — assertable between
        // calls); `runs` counts REAL compiler invocations.
        fn compile_once(
            src: &Path,
            dir: &Path,
            obj_path: &Path,
            runs: &mut u32,
        ) -> Result<(Vec<u8>, Vec<PathBuf>), String> {
            *runs += 1;
            let deps = auto_abi_probe::ctool::compile_c_with_deps(src, dir, obj_path)
                .map_err(|e| format!("cl: {e}"))?;
            let bytes = std::fs::read(obj_path).map_err(|e| format!("read: {e}"))?;
            Ok((bytes, deps))
        }

        write_header(17);
        let (obj17, hit1) =
            auto_abi_probe::cache::produce_c_object(&store, &src, &dir, &arts_for(&src), || {
                compile_once(&src, &dir, &obj_path, &mut producer_runs)
            })
            .expect("compile v17");
        assert!(!hit1, "{name}: first compile is a miss");
        std::fs::write(dir.join("linked.obj"), &obj17).unwrap();
        let pe = run::run_pe(&link_pe(&dir, &[dir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap();
        assert_eq!(pe.code, 17, "{name}: PE exits with the header VALUE (17)");

        // repeat: hit, ZERO compiler invocations, identical bytes
        let (obj17b, hit2) =
            auto_abi_probe::cache::produce_c_object(&store, &src, &dir, &arts_for(&src), || {
                compile_once(&src, &dir, &obj_path, &mut producer_runs)
            })
            .expect("hit v17");
        assert!(hit2, "{name}: unchanged deps must hit");
        assert_eq!(producer_runs, 1, "{name}: a hit never invokes the compiler");
        assert_eq!(obj17, obj17b);

        // header VALUE change with the SOURCE untouched: MISS + real
        // recompile + the PE now exits 29 (the R8 counterexample shape)
        write_header(29);
        let (obj29, hit3) =
            auto_abi_probe::cache::produce_c_object(&store, &src, &dir, &arts_for(&src), || {
                compile_once(&src, &dir, &obj_path, &mut producer_runs)
            })
            .expect("recompile v29");
        assert!(!hit3, "{name}: a changed dependency must NOT serve a hit");
        assert_eq!(producer_runs, 2, "{name}: real recompile ran");
        assert_ne!(obj17, obj29, "{name}: the object itself changed");
        std::fs::write(dir.join("linked.obj"), &obj29).unwrap();
        let pe = run::run_pe(&link_pe(&dir, &[dir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap();
        assert_eq!(pe.code, 29, "{name}: PE exits the NEW header VALUE (29)");

        // a DELETED dependency is also a miss, and the recompile fails
        // through the real compiler (controlled error, never stale-serve)
        std::fs::remove_file(dir.join("dep.h")).unwrap();
        match auto_abi_probe::cache::produce_c_object(&store, &src, &dir, &arts_for(&src), || {
            compile_once(&src, &dir, &obj_path, &mut producer_runs)
        }) {
            Err(_) => {}
            Ok((_, hit)) => panic!("{name}: deleted dependency must not serve a hit (hit={hit})"),
        }
        assert_eq!(producer_runs, 3, "{name}: the miss really invoked cl");
    }
}

/// The system/MSVC header closure is RECORDED and hashed, not assumed by
/// the cl.exe identity: a compile pulling `<stdint.h>` stores deps
/// OUTSIDE the source directory (the toolchain exe hash is not a header
/// content hash — F-797-R4-07 R8).
#[test]
fn abi_cache_include_records_system_header_closure() {
    let root = t05_root().join("inc-sys");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    // the store lives OUTSIDE the search dir: the recursive listing is
    // the include environment, and a mutating store inside it would
    // (correctly) invalidate every lookup
    let store = fresh_store(&t05_root(), "inc-sys-store");
    let src = root.join("caller.c");
    std::fs::write(
        &src,
        "#include <stdint.h>\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess((unsigned)(uint64_t)7); }\n",
    )
    .unwrap();
    let obj_path = root.join("caller.obj");
    let ident = auto_abi_probe::cache::c_source_identity(&src).unwrap();
    let (_, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &root, &arts_for(&src), || {
            let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &root, &obj_path)
                .map_err(|e| e.to_string())?;
            Ok((std::fs::read(&obj_path).map_err(|e| e.to_string())?, deps))
        })
        .expect("compile with system headers");
    assert!(!hit);
    // read the stored generation's manifest through the store's public
    // db handle (the same read-only discovery produce_c_object uses)
    let payload = {
        let db = store.db();
        // same substring discovery produce_c_object uses (canonical
        // keys sort fields alphabetically)
        let needle = format!("%source={},target=%", ident);
        let art_id: String = db
            .query_row(
                "SELECT artifact_id FROM artifacts WHERE layer = ?1 AND key_canonical LIKE ?2",
                [LAYER_OBJECT_V01.id, needle.as_str()],
                |row| row.get(0),
            )
            .expect("the compiled generation is stored under its source identity");
        drop(db);
        store
            .get_artifact_by_id(&art_id)
            .expect("artifact readable")
            .expect("artifact present")
            .payload
    };
    let entry: serde_json::Value = serde_json::from_slice(&payload).expect("envelope");
    let deps = entry["deps"].as_array().expect("deps recorded");
    assert!(!deps.is_empty(), "the closure must be recorded");
    let outside = deps.iter().any(|d| {
        d[0].as_str()
            .is_some_and(|p| !Path::new(p).starts_with(&root))
    });
    assert!(
        outside,
        "the MSVC/ucrt closure (outside the source dir) must be among the recorded deps: {deps:?}"
    );
    // and the recorded closure satisfies the re-verification: a second
    // produce is a HIT (every recorded dep still hashes equal)
    let (_, hit2) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &root, &arts_for(&src), || {
            panic!("a verified closure must hit without compiling")
        })
        .expect("hit");
    assert!(hit2);
}

// ---------------------------------------------------------------------------
// F-797-R9-01 (R9 repair): generation discovery must bind the FULL recipe
// input domain — a same-name+content source at a DIFFERENT directory (or
// under a different toolchain/target) is a DIFFERENT compilation.
// ---------------------------------------------------------------------------

/// Cross-project wrong-object counterexample (the R9 probe): projects A
/// and B both carry a byte-identical `same.c` including "dep.h"; A's
/// header says 17, B's says 29. Over ONE shared store, B's request must
/// NOT serve A's object — it is a miss (real recompile) and B's PE exits
/// 29. Correct-reuse control: the SAME source at the SAME location with
/// unchanged inputs still hits.
#[test]
fn abi_cache_cross_project_same_content_different_deps() {
    let root = t05_root().join("xproj");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let store = fresh_store(&root, "store");
    let body = "#include \"dep.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n";
    let mk = |proj: &str, value: u32| {
        let dir = root.join(proj);
        std::fs::create_dir_all(&dir).unwrap();
        std::fs::write(dir.join("same.c"), body).unwrap();
        std::fs::write(dir.join("dep.h"), format!("#define VALUE {value}\n")).unwrap();
        dir
    };
    let runs = std::cell::Cell::new(0u32);
    let produce = |dir: &Path, obj_name: &str| -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = dir.join(obj_name);
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&dir.join("same.c"), dir, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };
    let link_run = |dir: &Path, obj: &[u8]| -> i32 {
        std::fs::write(dir.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(dir, &[dir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };

    let a = mk("projA", 17);
    let (obj_a, hit_a) = auto_abi_probe::cache::produce_c_object(
        &store,
        &a.join("same.c"),
        &a,
        &arts_for(&a.join("same.c")),
        || produce(&a, "caller.obj"),
    )
    .expect("A compiles");
    assert!(!hit_a);
    assert_eq!(link_run(&a, &obj_a), 17, "A's PE exits 17");

    // correct-reuse control FIRST: same source, same dirs, unchanged
    // inputs -> hit, zero compiler invocations
    let (_, hit_a2) = auto_abi_probe::cache::produce_c_object(
        &store,
        &a.join("same.c"),
        &a,
        &arts_for(&a.join("same.c")),
        || produce(&a, "caller.obj"),
    )
    .expect("A repeat");
    assert!(hit_a2, "same project inputs must reuse");
    assert_eq!(runs.get(), 1);

    // the counterexample: B has the SAME source identity but a different
    // directory + header — serving A's object would exit 17
    let b = mk("projB", 29);
    let (obj_b, hit_b) = auto_abi_probe::cache::produce_c_object(
        &store,
        &b.join("same.c"),
        &b,
        &arts_for(&b.join("same.c")),
        || produce(&b, "caller.obj"),
    )
    .expect("B compiles");
    assert!(
        !hit_b,
        "a different source directory is a MISS, never A's object"
    );
    assert_eq!(runs.get(), 2, "B really compiled");
    assert_eq!(
        link_run(&b, &obj_b),
        29,
        "B's PE exits 29 (A's cached 17-object must not be served)"
    );
}

/// Foreign-toolchain generation control (the R9 positive control): a
/// legally-stored generation under a DIFFERENT toolchain fingerprint
/// (public store API, well-formed envelope) must not be served to the
/// CURRENT toolchain's request — discovery pre-filters by the current
/// target+toolchain before any payload is trusted.
#[test]
fn abi_cache_foreign_toolchain_generation_not_served() {
    use ac_artifact_probe::codec::ChunkKind;
    let root = t05_root().join("foreigntc");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let store = fresh_store(&t05_root(), "foreigntc-store");
    let src = root.join("caller.c");
    std::fs::write(
        &src,
        "#include \"dep.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    std::fs::write(root.join("dep.h"), "#define VALUE 17\n").unwrap();

    // a well-formed v2 envelope for a DIFFERENT toolchain + target,
    // stored under the public API with its own closure key
    let ident = auto_abi_probe::cache::c_source_identity(&src).unwrap();
    use base64::Engine as _;
    let foreign = serde_json::json!({
        "v": 2,
        "source": ident,
        "src_dir": root.display().to_string(),
        "include_dir": root.display().to_string(),
        "target": "x86_64-unknown-linux-gnu",
        "toolchain": "cl:C:\\foreign\\cl.exe|ver:x|hash:00|flags:-c",
        "obj_b64": base64::engine::general_purpose::STANDARD.encode(b"\0foreign-object-payload"),
        "deps": [],
        "dir_listings": [
            (
                root.display().to_string(),
                vec!["caller.c".to_string(), "dep.h".to_string()],
            ),
        ],
    });
    let key = auto_abi_probe::cache::c_object_key(&ident, "foreign-closure");
    store
        .put_artifact(ac_artifact_probe::store::PutRequest {
            layer: auto_abi_probe::cache::LAYER_OBJECT_V01.id,
            key,
            kind: ChunkKind::Object,
            payload: serde_json::to_vec(&foreign).unwrap(),
            edges: vec![],
            project: None,
        })
        .expect("foreign generation stored via public API");

    let runs = std::cell::Cell::new(0u32);
    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &root, &arts_for(&src), || {
            runs.set(runs.get() + 1);
            let obj = root.join("caller.obj");
            let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &root, &obj)
                .map_err(|e| format!("cl: {e}"))?;
            Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
        })
        .expect("current-toolchain request");
    assert!(
        !hit,
        "a foreign toolchain/target generation must NOT be served"
    );
    assert_eq!(runs.get(), 1, "the current toolchain really compiled");
    std::fs::write(root.join("linked.obj"), &obj).unwrap();
    let pe = run::run_pe(&link_pe(&root, &[root.join("linked.obj")], "p797_inc_entry").unwrap())
        .unwrap();
    assert_eq!(pe.code, 17);
}

/// F-797-R4-07 (R9): SHADOWING — a NEW header next to the source changes
/// include resolution without touching any recorded dep. Three states:
/// resolve-via -I shared (17) -> hit; shadow/dep.h appears (29) -> MISS
/// and PE flips to 29; the shadow is removed -> MISS and PE returns to
/// 17. Same-input repeats between the transitions must still HIT.
#[test]
fn abi_cache_nearby_shadow_invalidates() {
    let root = t05_root().join("shadow");
    let _ = std::fs::remove_dir_all(&root);
    let shared = root.join("shared");
    let shadow = root.join("shadow");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::create_dir_all(&shadow).unwrap();
    std::fs::write(shared.join("dep.h"), "#define VALUE 17\n").unwrap();
    let src = shadow.join("same.c");
    std::fs::write(
        &src,
        "#include \"dep.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    let store = fresh_store(&root, "store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |obj: &[u8]| -> i32 {
        std::fs::write(shadow.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(&shadow, &[shadow.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = shadow.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &shared, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    // state 1: dep.h resolves via -I shared -> VALUE 17
    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(link_run(&obj), 17);
    let (_, hit_again) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("repeat");
    assert!(hit_again, "unchanged search environment must hit");
    assert_eq!(runs.get(), 1);

    // state 2: a NEW shadowing header appears next to the source — the
    // source and every previously recorded dep are unchanged, yet the
    // resolution changes; the listing snapshot must force a MISS
    std::fs::write(shadow.join("dep.h"), "#define VALUE 29\n").unwrap();
    let (obj29, hit29) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("shadow compile");
    assert!(!hit29, "a new nearby header must invalidate (shadowing)");
    assert_eq!(runs.get(), 2);
    assert_eq!(link_run(&obj29), 29, "the shadowed resolution wins (29)");
    let (_, hit29b) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("shadow repeat");
    assert!(hit29b, "stable shadowed environment hits");

    // state 3: removing the shadow restores the shared resolution. The
    // listing no longer matches the SHADOWED generation, so it can never
    // be served again; the environment equals the ORIGINAL state-1
    // world, whose generation re-verifies (listing + deps) — serving 17
    // again is CORRECT reuse of a re-appeared environment, and the
    // shadowed-29 object is dead. The oracle is behavioral: PE is 17.
    std::fs::remove_file(shadow.join("dep.h")).unwrap();
    let (obj17, _hit17) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("unshadow serve");
    assert_eq!(link_run(&obj17), 17, "back to the shared resolution (17)");
    // and a NOVEL shadowed world (new shadow CONTENT) is a real miss —
    // neither the 17 nor the 29 generation applies:
    std::fs::write(
        shadow.join("dep.h"),
        "#define VALUE 31
",
    )
    .unwrap();
    let (obj31, hit31) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("novel-shadow compile");
    assert!(
        !hit31,
        "a novel shadow content must miss every earlier generation"
    );
    assert_eq!(link_run(&obj31), 31);
}

// ---------------------------------------------------------------------------
// F-797-R4-07 (R10): nested-path and extension-less includes are part of
// the search environment; both shadowing forms must invalidate.
// ---------------------------------------------------------------------------

/// `#include "sub/dep.h"` — the search dirs' sub/ exists on BOTH sides;
/// the source-side sub/ is initially empty (17 resolves via the include
/// dir). Adding source-side sub/dep.h (29) changes resolution without
/// touching the source or any recorded dep — the recursive listing
/// snapshot must force a miss.
#[test]
fn abi_cache_nested_subpath_shadow_invalidates() {
    let root = t05_root().join("nested");
    let _ = std::fs::remove_dir_all(&root);
    let shared = root.join("shared");
    let srcdir = root.join("src");
    std::fs::create_dir_all(shared.join("sub")).unwrap();
    std::fs::create_dir_all(srcdir.join("sub")).unwrap();
    std::fs::write(shared.join("sub/dep.h"), "#define VALUE 17\n").unwrap();
    let src = srcdir.join("same.c");
    std::fs::write(
        &src,
        "#include \"sub/dep.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    let store = fresh_store(&root, "store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |obj: &[u8]| -> i32 {
        std::fs::write(srcdir.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(&srcdir, &[srcdir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = srcdir.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &shared, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(link_run(&obj), 17, "resolves via the include dir (17)");

    std::fs::write(srcdir.join("sub/dep.h"), "#define VALUE 29\n").unwrap();
    let (obj29, hit29) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("nested shadow compile");
    assert!(!hit29, "a NEW nested nearby header must invalidate");
    assert_eq!(runs.get(), 2);
    assert_eq!(link_run(&obj29), 29, "the nested shadow wins (29)");
}

/// `#include "settings"` — extension-less headers resolve exactly like
/// named ones; the listing must not filter them out.
#[test]
fn abi_cache_extensionless_header_shadow_invalidates() {
    let root = t05_root().join("extless");
    let _ = std::fs::remove_dir_all(&root);
    let shared = root.join("shared");
    let srcdir = root.join("src");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::create_dir_all(&srcdir).unwrap();
    std::fs::write(shared.join("settings"), "#define VALUE 17\n").unwrap();
    let src = srcdir.join("same.c");
    std::fs::write(
        &src,
        "#include \"settings\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    let store = fresh_store(&root, "store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |obj: &[u8]| -> i32 {
        std::fs::write(srcdir.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(&srcdir, &[srcdir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = srcdir.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &shared, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(link_run(&obj), 17);

    std::fs::write(srcdir.join("settings"), "#define VALUE 29\n").unwrap();
    let (obj29, hit29) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("extension-less shadow compile");
    assert!(!hit29, "a NEW extension-less nearby header must invalidate");
    assert_eq!(runs.get(), 2);
    assert_eq!(link_run(&obj29), 29);
}

// ---------------------------------------------------------------------------
// F-797-R10-01: the applicability domain is part of the GENERATION KEY —
// a legal recompile after a listing-only change must SUCCEED (its own new
// key), never collide with the old content-bound entry.
// ---------------------------------------------------------------------------

/// Adding a completely UNUSED nearby header changes only the listing:
/// the recompile must succeed and still run 17 (the R10 counterexample
/// previously dead-ended in a permanent Conflict/Internal).
#[test]
fn abi_cache_unused_nearby_file_legal_recompile() {
    let root = t05_root().join("unused");
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("src");
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("dep.h"), "#define VALUE 17\n").unwrap();
    let src = dir.join("value.c");
    std::fs::write(
        &src,
        "#include \"dep.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    let store = fresh_store(&root, "store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |obj: &[u8]| -> i32 {
        std::fs::write(dir.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(&dir, &[dir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = dir.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &dir, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &dir, &arts_for(&src), produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(link_run(&obj), 17);

    // listing-only change: an unused file appears; deps and source are
    // unchanged — the recompile must SUCCEED under its own key
    std::fs::write(dir.join("unused.h"), "// never included\n").unwrap();
    let (obj2, hit2) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &dir, &arts_for(&src), produce)
            .expect("legal recompile after a listing change must succeed");
    assert!(!hit2, "listing change invalidates the old generation");
    assert_eq!(runs.get(), 2);
    assert_eq!(link_run(&obj2), 17, "still compiles and runs 17");
    // stable now: repeat hits
    let (_, hit3) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &dir, &arts_for(&src), produce)
            .expect("repeat");
    assert!(hit3);
    assert_eq!(runs.get(), 2);
}

/// Two projects with byte-identical include-FREE sources over one store:
/// both compilations must succeed (the R10 counterexample dead-ended B
/// in a permanent Conflict because the empty closure collided).
#[test]
fn abi_cache_no_include_cross_project_both_succeed() {
    let root = t05_root().join("noincl");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    let store = fresh_store(&root, "store");
    let body = "__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(17); }\n";
    let runs = std::cell::Cell::new(0u32);
    for proj in ["projA", "projB"] {
        let dir = root.join(proj);
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("same.c");
        std::fs::write(&src, body).unwrap();
        let (obj, hit) =
            auto_abi_probe::cache::produce_c_object(&store, &src, &dir, &arts_for(&src), || {
                runs.set(runs.get() + 1);
                let obj = dir.join("caller.obj");
                let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &dir, &obj)
                    .map_err(|e| format!("cl: {e}"))?;
                Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
            })
            .expect("each project's legal domain compiles successfully");
        assert!(!hit, "{proj}: distinct src dirs are distinct generations");
        std::fs::write(dir.join("linked.obj"), &obj).unwrap();
        let pe = run::run_pe(&link_pe(&dir, &[dir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap();
        assert_eq!(pe.code, 17, "{proj}");
    }
    assert_eq!(runs.get(), 2, "both projects really compiled");
}

/// F-797-R4-07 (R11): TRANSITIVE include directories are part of the
/// search environment — the reviewer's construction: src/value.c
/// includes ../common/outer.h; outer.h includes "dep.h" which resolves
/// via -I include/ (17). A NEW common/dep.h (29) sits in the INCLUDING
/// header's directory — outside both snapshot roots — and wins the
/// resolution; the dependency-dir listings must force a miss.
#[test]
fn abi_cache_transitive_header_dir_shadow_invalidates() {
    let root = t05_root().join("transitive");
    let _ = std::fs::remove_dir_all(&root);
    let srcdir = root.join("project/src");
    let common = root.join("project/common");
    let include = root.join("project/include");
    std::fs::create_dir_all(&srcdir).unwrap();
    std::fs::create_dir_all(&common).unwrap();
    std::fs::create_dir_all(&include).unwrap();
    std::fs::write(
        srcdir.join("value.c"),
        "#include \"../common/outer.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    std::fs::write(common.join("outer.h"), "#include \"dep.h\"\n").unwrap();
    std::fs::write(include.join("dep.h"), "#define VALUE 17\n").unwrap();
    let src = srcdir.join("value.c");
    let store = fresh_store(&t05_root(), "transitive-store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |obj: &[u8]| -> i32 {
        std::fs::write(srcdir.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(&srcdir, &[srcdir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = srcdir.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &include, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(link_run(&obj), 17, "dep.h resolves via the -I dir (17)");
    let (_, hit_again) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("repeat");
    assert!(hit_again, "unchanged environment hits");
    assert_eq!(runs.get(), 1);

    // the R11 counterexample: a NEW header in the TRANSITIVE header's
    // own directory (not under src/ or include/) shadows dep.h
    std::fs::write(common.join("dep.h"), "#define VALUE 29\n").unwrap();
    let (obj29, hit29) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("transitive shadow compile");
    assert!(
        !hit29,
        "a new header in a transitive dependency's directory must invalidate"
    );
    assert_eq!(runs.get(), 2);
    assert_eq!(link_run(&obj29), 29, "the transitive shadow wins (29)");
}

/// F-797-R4-07 (R12): TRANSITIVE + SUB-PATH shadow — outer.h includes
/// "sub/dep.h" resolving via -I include/sub (17); a NEW common/sub/dep.h
/// (29) sits in the INCLUDING header's subdirectory, outside the two
/// explicit roots. The dependency-parent dir (common, from outer.h)
/// gets a RECURSIVE listing, so the new file under common/sub is
/// visible and the lookup is a miss.
#[test]
fn abi_cache_transitive_subpath_shadow_invalidates() {
    let root = t05_root().join("transub");
    let _ = std::fs::remove_dir_all(&root);
    let srcdir = root.join("project/src");
    let common = root.join("project/common");
    let include = root.join("project/include");
    for d in [&srcdir, &common, &include] {
        std::fs::create_dir_all(d).unwrap();
    }
    std::fs::create_dir_all(common.join("sub")).unwrap();
    std::fs::create_dir_all(include.join("sub")).unwrap();
    std::fs::write(
        srcdir.join("value.c"),
        "#include \"../common/outer.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    std::fs::write(common.join("outer.h"), "#include \"sub/dep.h\"\n").unwrap();
    std::fs::write(include.join("sub/dep.h"), "#define VALUE 17\n").unwrap();
    let src = srcdir.join("value.c");
    let store = fresh_store(&t05_root(), "transub-store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |obj: &[u8]| -> i32 {
        std::fs::write(srcdir.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(&srcdir, &[srcdir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = srcdir.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &include, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(link_run(&obj), 17, "sub/dep.h resolves via the -I dir (17)");
    let (_, hit_again) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("repeat");
    assert!(hit_again, "unchanged environment hits");
    assert_eq!(runs.get(), 1);

    // the R12 counterexample: a NEW header in the transitive header's
    // SUBDIRECTORY (outside both explicit roots) shadows sub/dep.h
    std::fs::write(common.join("sub/dep.h"), "#define VALUE 29\n").unwrap();
    let (obj29, hit29) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("transitive sub-path shadow compile");
    assert!(
        !hit29,
        "a new header in a transitive dependency's SUBDIRECTORY must invalidate"
    );
    assert_eq!(runs.get(), 2);
    assert_eq!(
        link_run(&obj29),
        29,
        "the transitive sub-path shadow wins (29)"
    );
}

/// F-797-R4-07 (R13): PARENT-PATH candidates — value.c includes
/// "../external/headers/outer.h"; outer.h includes "../shared/dep.h"
/// which resolves via the -I dir's parent escape
/// (project/include/../shared/dep.h = 17). A NEW
/// project/external/shared/dep.h (29) sits at the INCLUDING header's
/// parent-escape position — outside EVERY recorded dir's subtree. The
/// (dep, resolution-dir, chain-dir) candidate derivation covers it:
/// the spec that resolved shared/dep.h from the include dir is
/// "../shared/dep.h", and applying it to the chain dir
/// external/headers yields exactly external/shared — recorded with an
/// empty listing, so the new file forces a miss.
#[test]
fn abi_cache_parent_path_escape_shadow_invalidates() {
    let root = t05_root().join("paresc");
    let _ = std::fs::remove_dir_all(&root);
    let srcdir = root.join("project/src");
    let headers = root.join("project/external/headers");
    let esc_shared = root.join("project/external/shared");
    let include = root.join("project/include");
    let incl_shared = root.join("project/shared");
    for d in [&srcdir, &headers, &include] {
        std::fs::create_dir_all(d).unwrap();
    }
    // both ../shared targets start existing (empty on the external side)
    std::fs::create_dir_all(&esc_shared).unwrap();
    std::fs::create_dir_all(&incl_shared).unwrap();
    std::fs::write(
        srcdir.join("value.c"),
        "#include \"../external/headers/outer.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    std::fs::write(headers.join("outer.h"), "#include \"../shared/dep.h\"\n").unwrap();
    std::fs::write(incl_shared.join("dep.h"), "#define VALUE 17\n").unwrap();
    let src = srcdir.join("value.c");
    let store = fresh_store(&t05_root(), "paresc-store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |obj: &[u8]| -> i32 {
        std::fs::write(srcdir.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(&srcdir, &[srcdir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = srcdir.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &include, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(link_run(&obj), 17, "resolves via the -I parent escape (17)");
    let (_, hit_again) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("repeat");
    assert!(
        hit_again,
        "unchanged environment (both shared trees empty of shadows) hits"
    );
    assert_eq!(runs.get(), 1);

    // the R13 counterexample: a NEW file at the INCLUDING header's
    // parent-escape position wins resolution
    std::fs::write(esc_shared.join("dep.h"), "#define VALUE 29\n").unwrap();
    let (obj29, hit29) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("parent-escape shadow compile");
    assert!(
        !hit29,
        "a new header at a derived parent-path candidate must invalidate"
    );
    assert_eq!(runs.get(), 2);
    assert_eq!(link_run(&obj29), 29, "the parent-escape shadow wins (29)");
}

/// F-797-R4-07 (R14): a REAL header named `config.tmp` — the artifact
/// filter no longer drops whole suffixes; only caller-declared outputs
/// and our staged-link temporaries are excluded. `#include "config.tmp"`
/// resolves via -I (17); a NEW source-side config.tmp (29) must
/// invalidate.
#[test]
fn abi_cache_tmp_named_header_shadow_invalidates() {
    let root = t05_root().join("tmpname");
    let _ = std::fs::remove_dir_all(&root);
    let shared = root.join("shared");
    let srcdir = root.join("src");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::create_dir_all(&srcdir).unwrap();
    std::fs::write(shared.join("config.tmp"), "#define VALUE 17\n").unwrap();
    let src = srcdir.join("same.c");
    std::fs::write(
        &src,
        "#include \"config.tmp\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    let store = fresh_store(&t05_root(), "tmpname-store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |obj: &[u8]| -> i32 {
        std::fs::write(srcdir.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(&srcdir, &[srcdir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = srcdir.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &shared, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(
        link_run(&obj),
        17,
        "config.tmp resolves via the -I dir (17)"
    );
    let (_, hit_again) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("repeat");
    assert!(hit_again, "same-input hit control");
    assert_eq!(runs.get(), 1);

    // the R14 counterexample: a NEW text header with an artifact-looking
    // NAME next to the source wins resolution
    std::fs::write(srcdir.join("config.tmp"), "#define VALUE 29\n").unwrap();
    let (obj29, hit29) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &shared, &arts_for(&src), produce)
            .expect("tmp-named shadow compile");
    assert!(!hit29, "a new config.tmp header must invalidate");
    assert_eq!(runs.get(), 2);
    assert_eq!(link_run(&obj29), 29);
}

/// F-797-R4-07 (R14): a LEGACY v3 envelope — stored legally under the
/// OLD (narrower) applicability rule via the public store API, with a
/// genuine object, genuine observed deps, and the R13-era domain (4
/// roots: source/-I/dep parents, suffix-filtered listings) — must NEVER
/// be served by the current rule: it misses and re-proves under v4.
#[test]
fn abi_cache_legacy_v3_envelope_never_served() {
    use ac_artifact_probe::codec::ChunkKind;
    use base64::Engine as _;
    let root = t05_root().join("legacyv3");
    let _ = std::fs::remove_dir_all(&root);
    let shared = root.join("shared");
    let headers = root.join("external/headers");
    let esc_shared = root.join("external/shared");
    let include = root.join("include");
    std::fs::create_dir_all(&shared).unwrap();
    std::fs::create_dir_all(&headers).unwrap();
    std::fs::create_dir_all(&esc_shared).unwrap();
    std::fs::create_dir_all(&include).unwrap();
    std::fs::write(headers.join("outer.h"), "#include \"../shared/dep.h\"\n").unwrap();
    std::fs::write(shared.join("dep.h"), "#define VALUE 17\n").unwrap();
    let srcdir = root.join("src");
    std::fs::create_dir_all(&srcdir).unwrap();
    let src = srcdir.join("value.c");
    std::fs::write(
        &src,
        "#include \"../external/headers/outer.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    let store = fresh_store(&t05_root(), "legacyv3-store");
    let runs = std::cell::Cell::new(0u32);
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = srcdir.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &include, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    // a real compile happens (we need its genuine object + deps)
    let (obj, _hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("initial compile");
    assert_eq!(runs.get(), 1);

    // hand-store the LEGACY v3 shape under the OLD generation rule:
    // domain = {src, -I} ∪ dep parents, listings suffix-filtered (a
    // `.tmp`/`.obj`-named file would be dropped there), generation hash
    // over that narrower set — the exact R13-era envelope this entry
    // would have carried
    let ident = auto_abi_probe::cache::c_source_identity(&src).unwrap();
    let deps: Vec<String> = vec![
        headers.join("outer.h").display().to_string(),
        shared.join("dep.h").display().to_string(),
    ];
    let dep_hashes: Vec<(String, String)> = deps
        .iter()
        .map(|p| {
            (
                p.clone(),
                auto_abi_probe::semref::blake3_hex(&std::fs::read(p).unwrap()),
            )
        })
        .collect();
    let mut old_dirs: Vec<String> = vec![
        srcdir.display().to_string(),
        include.display().to_string(),
        headers.display().to_string(),
        shared.display().to_string(),
    ];
    old_dirs.sort();
    old_dirs.dedup();
    let old_listings: Vec<(String, Vec<String>)> = old_dirs
        .iter()
        .map(|d| {
            (
                d.clone(),
                // suffix-filtered listing of the OLD rule
                std::fs::read_dir(d)
                    .into_iter()
                    .flatten()
                    .flatten()
                    .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
                    .filter_map(|e| e.file_name().into_string().ok())
                    .filter(|n| {
                        let l = n.to_ascii_lowercase();
                        !(l.ends_with(".obj")
                            || l.ends_with(".exe")
                            || l.ends_with(".tmp")
                            || l.contains(".tmp-"))
                    })
                    .collect(),
            )
        })
        .collect();
    let mut gen: Vec<String> = dep_hashes
        .iter()
        .map(|(p, h)| format!("dep|{p}|{h}"))
        .collect();
    gen.push(format!("src_dir|{}", srcdir.display()));
    gen.push(format!("inc_dir|{}", include.display()));
    gen.push("target|x86_64-pc-windows-msvc".to_string());
    gen.push(format!(
        "toolchain|{}",
        std::env::var("PLAN797_TEST_TC").unwrap_or_default()
    ));
    for (d, l) in &old_listings {
        gen.push(format!("dir|{d}"));
        for n in l {
            gen.push(format!("file|{d}|{n}"));
        }
    }
    gen.sort();
    gen.dedup();
    // NOTE: generation hash intentionally NON-matching to the current
    // algorithm — the store key just needs to be unique for this entry
    let legacy_key = auto_abi_probe::cache::c_object_key(&ident, "legacy-v3-generation");
    let legacy = serde_json::json!({
        "v": 3,
        "source": ident,
        "src_dir": srcdir.display().to_string(),
        "include_dir": include.display().to_string(),
        "target": "x86_64-pc-windows-msvc",
        "toolchain": "",
        "obj_b64": base64::engine::general_purpose::STANDARD.encode(&obj),
        "deps": dep_hashes,
        "dir_listings": old_listings,
    });
    store
        .put_artifact(ac_artifact_probe::store::PutRequest {
            layer: auto_abi_probe::cache::LAYER_OBJECT_V01.id,
            key: legacy_key,
            kind: ChunkKind::Object,
            payload: serde_json::to_vec(&legacy).unwrap(),
            edges: vec![],
            project: None,
        })
        .expect("legacy v3 stored via public API");

    // the current rule must NEVER serve the legacy envelope (v!=4) —
    // and after the parent-escape shadow appears, the fresh compile
    // gives 29 while the legacy 17-object sits in the store
    std::fs::write(esc_shared.join("dep.h"), "#define VALUE 29\n").unwrap();
    let (obj29, hit29) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &arts_for(&src), produce)
            .expect("post-shadow compile");
    assert!(!hit29);
    assert_eq!(
        runs.get(),
        2,
        "a real recompile happened (legacy never served)"
    );
    std::fs::write(srcdir.join("linked.obj"), &obj29).unwrap();
    let pe =
        run::run_pe(&link_pe(&srcdir, &[srcdir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap();
    assert_eq!(pe.code, 29, "the shadow wins — the stale 17 never served");
    let _ = legacy; // silences unused when json! moves
}

/// F-797-R9-01 (R14 reopen): RELATIVE path anchors bind to the CURRENT
/// working directory — two projects sharing the same relative literals
/// (src/same.c + include) over one store must NOT alias: B misses and
/// gets its own object (29), not A's 17.
#[test]
fn abi_cache_relative_paths_bind_cwd() {
    let root = t05_root().join("relcwd");
    let _ = std::fs::remove_dir_all(&root);
    let body = "#include \"dep.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n";
    let mk = |proj: &str, value: u32| {
        let dir = root.join(proj);
        std::fs::create_dir_all(dir.join("src")).unwrap();
        std::fs::create_dir_all(dir.join("include")).unwrap();
        std::fs::write(dir.join("src/same.c"), body).unwrap();
        std::fs::write(
            dir.join("include/dep.h"),
            format!("#define VALUE {value}\n"),
        )
        .unwrap();
        dir
    };
    let a = mk("projA", 17);
    let b = mk("projB", 29);
    let store = fresh_store(&t05_root(), "relcwd-store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |dir: &Path, obj: &[u8]| -> i32 {
        std::fs::write(dir.join("src/linked.obj"), obj).unwrap();
        run::run_pe(
            &link_pe(
                &dir.join("src"),
                &[dir.join("src/linked.obj")],
                "p797_inc_entry",
            )
            .unwrap(),
        )
        .unwrap()
        .code
    };
    let _cwd_lock = CWD_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let cwd_guard = std::env::current_dir().unwrap();
    let run_one = |dir: &Path| -> (bool, i32) {
        std::env::set_current_dir(dir).unwrap();
        let src = std::path::Path::new("src/same.c");
        let inc = std::path::Path::new("include");
        let (obj, hit) =
            auto_abi_probe::cache::produce_c_object(&store, src, inc, &arts_for(src), || {
                runs.set(runs.get() + 1);
                let obj = dir.join("src/caller.obj");
                let deps = auto_abi_probe::ctool::compile_c_with_deps(
                    &dir.join("src/same.c"),
                    &dir.join("include"),
                    &obj,
                )
                .map_err(|e| format!("cl: {e}"))?;
                Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
            })
            .expect("compile");
        (hit, link_run(dir, &obj))
    };

    let (hit_a, pe_a) = run_one(&a);
    assert!(!hit_a);
    assert_eq!(pe_a, 17, "A compiles and runs 17");
    // same-input hit control: A again (relative anchors, same cwd)
    let (hit_a2, pe_a2) = run_one(&a);
    assert!(hit_a2, "same project + same literals still hit");
    assert_eq!(pe_a2, 17);

    // the R14 counterexample: B shares the LITERALS but not the
    // directory — canonicalized anchors must separate the domains
    let (hit_b, pe_b) = run_one(&b);
    std::env::set_current_dir(&cwd_guard).unwrap();
    assert!(
        !hit_b,
        "a different project must MISS, never serve A's object"
    );
    assert_eq!(runs.get(), 2, "B really compiled");
    assert_eq!(
        pe_b, 29,
        "B's PE exits 29 (A's cached 17-object must not be served)"
    );
}

/// F-797-R4-07 (R15): artifact exclusion is by FULL PATH — the reviewer's
/// construction: the source includes `sub/value.obj` (a REAL text header
/// under include/sub), while the compiler's output is `src/value.obj`,
/// declared by exact path. A NEW `src/sub/value.obj` header shares only
/// the BASENAME with the declared output — it must stay a visible
/// candidate and its appearance must invalidate.
#[test]
fn abi_cache_artifact_path_not_basename() {
    let root = t05_root().join("artpath");
    let _ = std::fs::remove_dir_all(&root);
    let srcdir = root.join("src");
    let include = root.join("include");
    std::fs::create_dir_all(srcdir.join("sub")).unwrap();
    std::fs::create_dir_all(include.join("sub")).unwrap();
    std::fs::write(
        srcdir.join("value.c"),
        "#include \"sub/value.obj\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n",
    )
    .unwrap();
    std::fs::write(include.join("sub/value.obj"), "#define VALUE 17\n").unwrap();
    let src = srcdir.join("value.c");
    let store = fresh_store(&t05_root(), "artpath-store");
    let runs = std::cell::Cell::new(0u32);
    let link_run = |obj: &[u8]| -> i32 {
        std::fs::write(srcdir.join("linked.obj"), obj).unwrap();
        run::run_pe(&link_pe(&srcdir, &[srcdir.join("linked.obj")], "p797_inc_entry").unwrap())
            .unwrap()
            .code
    };
    let artifacts = arts(&srcdir);
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = srcdir.join("value.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &include, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &artifacts, produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(link_run(&obj), 17, "sub/value.obj resolves via -I (17)");
    let (_, hit_again) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &artifacts, produce)
            .expect("repeat");
    assert!(hit_again, "same-input hit control");
    assert_eq!(runs.get(), 1);

    // the R15 counterexample: a NEW header sharing only the BASENAME
    // with the declared output must NOT be excluded
    std::fs::write(srcdir.join("sub/value.obj"), "#define VALUE 29\n").unwrap();
    let (obj29, hit29) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &include, &artifacts, produce)
            .expect("same-basename shadow compile");
    assert!(!hit29, "a new same-basename header must invalidate");
    assert_eq!(runs.get(), 2);
    assert_eq!(link_run(&obj29), 29);
}

/// F-797-R15-01: a BARE source path — `Path("same.c")` has an EMPTY
/// parent; the anchors resolve to the current directory and the compile
/// proceeds (hit control included), never an Internal rejection.
#[test]
fn abi_cache_bare_source_path_resolves_cwd() {
    let root = t05_root().join("baresrc");
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(root.join("include")).unwrap();
    std::fs::write(root.join("same.c"), "#include \"dep.h\"\n__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(VALUE); }\n").unwrap();
    std::fs::write(root.join("include/dep.h"), "#define VALUE 17\n").unwrap();
    let store = fresh_store(&t05_root(), "baresrc-store");
    let runs = std::cell::Cell::new(0u32);
    let artifacts = arts(&root);
    let _cwd_lock = CWD_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let cwd_guard = std::env::current_dir().unwrap();
    std::env::set_current_dir(&root).unwrap();
    let src = std::path::Path::new("same.c");
    let inc = std::path::Path::new("include");
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = root.join("caller.obj");
        let deps = auto_abi_probe::ctool::compile_c_with_deps(
            &root.join("same.c"),
            &root.join("include"),
            &obj,
        )
        .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };
    let (obj, hit) = auto_abi_probe::cache::produce_c_object(&store, src, inc, &artifacts, produce)
        .expect("bare-path compile must proceed, not Internal-reject");
    std::env::set_current_dir(&cwd_guard).unwrap();
    assert!(!hit);
    assert_eq!(runs.get(), 1);
    std::fs::write(root.join("linked.obj"), &obj).unwrap();
    let pe = run::run_pe(&link_pe(&root, &[root.join("linked.obj")], "p797_inc_entry").unwrap())
        .unwrap();
    assert_eq!(pe.code, 17);

    // same-input hit control through the bare path again
    std::env::set_current_dir(&root).unwrap();
    let (_, hit2) = auto_abi_probe::cache::produce_c_object(&store, src, inc, &artifacts, produce)
        .expect("bare-path repeat");
    std::env::set_current_dir(&cwd_guard).unwrap();
    assert!(hit2, "same-input hit through the bare path");
    assert_eq!(runs.get(), 1);
}

/// F-797-R15-02: an old v3 entry stored under the SAME materials the
/// current algorithm would hash (identical anchors/deps/toolchain,
/// domain equal because this input's derivation adds nothing) — with the
/// envelope VERSION now part of the generation identity, the v4 upgrade
/// recompile PUBLISHES under its own key instead of dead-ending in a
/// permanent Conflict; the legacy entry is simply never served.
#[test]
fn abi_cache_v3_same_materials_upgrade_publishes() {
    use ac_artifact_probe::codec::ChunkKind;
    use base64::Engine as _;
    let root = t05_root().join("v3samekey");
    let _ = std::fs::remove_dir_all(&root);
    let dir = root.join("proj");
    std::fs::create_dir_all(&dir).unwrap();
    // include-FREE source: the dependency set is empty, so the old and
    // new domain derivations coincide exactly
    std::fs::write(
        dir.join("value.c"),
        "__declspec(dllimport) void __stdcall ExitProcess(unsigned int code);\nvoid p797_inc_entry(void) { ExitProcess(17); }\n",
    )
    .unwrap();
    let src = dir.join("value.c");
    let store = fresh_store(&t05_root(), "v3samekey-store");
    let runs = std::cell::Cell::new(0u32);
    let artifacts = arts(&dir);
    let produce = || -> Result<(Vec<u8>, Vec<PathBuf>), String> {
        runs.set(runs.get() + 1);
        let obj = root.join("scratch/caller.obj");
        std::fs::create_dir_all(root.join("scratch")).unwrap();
        let deps = auto_abi_probe::ctool::compile_c_with_deps(&src, &dir, &obj)
            .map_err(|e| format!("cl: {e}"))?;
        Ok((std::fs::read(&obj).map_err(|e| e.to_string())?, deps))
    };

    // initial real compile under the CURRENT rule (v4, published)
    let (obj, hit) =
        auto_abi_probe::cache::produce_c_object(&store, &src, &dir, &artifacts, produce)
            .expect("initial compile");
    assert!(!hit);
    assert_eq!(runs.get(), 1);

    // store an honest v3 legacy entry under the key the OLD algorithm
    // computed for the SAME materials (no env-version line, no
    // artifact-path lines): for this input the anchors/deps/toolchain
    // are identical, so without the env-version segregation this key
    // WOULD collide with the v4 generation — exactly the reviewer's
    // dead-end shape
    let ident = auto_abi_probe::cache::c_source_identity(&src).unwrap();
    let src_dir_c = std::fs::canonicalize(&dir).unwrap().display().to_string();
    let src_dir_plain = src_dir_c
        .strip_prefix(r"\\?\")
        .unwrap_or(&src_dir_c)
        .to_string();
    let tc = auto_abi_probe::ctool::toolchain_fingerprint().unwrap();
    let mut gen: Vec<String> = Vec::new(); // empty deps
    gen.push(format!("src_dir|{src_dir_plain}"));
    gen.push(format!("inc_dir|{src_dir_plain}"));
    gen.push("target|x86_64-pc-windows-msvc".to_string());
    gen.push(format!("toolchain|{tc}"));
    for (d, _) in [(src_dir_plain.clone(), 0u8), (src_dir_plain.clone(), 0u8)] {
        gen.push(format!("dir|{d}"));
    }
    gen.sort();
    gen.dedup();
    let legacy_gen = auto_abi_probe::semref::blake3_hex(gen.join("\n").as_bytes());
    let legacy = serde_json::json!({
        "v": 3,
        "source": ident,
        "src_dir": src_dir_plain,
        "include_dir": src_dir_plain,
        "target": "x86_64-pc-windows-msvc",
        "toolchain": tc,
        "obj_b64": base64::engine::general_purpose::STANDARD.encode(&obj),
        "deps": [],
        "dir_listings": [],
    });
    store
        .put_artifact(ac_artifact_probe::store::PutRequest {
            layer: auto_abi_probe::cache::LAYER_OBJECT_V01.id,
            key: auto_abi_probe::cache::c_object_key(&ident, &legacy_gen),
            kind: ChunkKind::Object,
            payload: serde_json::to_vec(&legacy).unwrap(),
            edges: vec![],
            project: None,
        })
        .expect("legacy v3 stored under the old-materials key");

    // a fresh store-less miss path: force a recompile by consulting a
    // SECOND store that only has the legacy entry — the current rule
    // must (a) never serve v3, (b) PUBLISH its own v4 generation
    // without Internal Conflict
    let store2 = fresh_store(&t05_root(), "v3samekey-store2");
    store2
        .put_artifact(ac_artifact_probe::store::PutRequest {
            layer: auto_abi_probe::cache::LAYER_OBJECT_V01.id,
            key: auto_abi_probe::cache::c_object_key(&ident, &legacy_gen),
            kind: ChunkKind::Object,
            payload: serde_json::to_vec(&legacy).unwrap(),
            edges: vec![],
            project: None,
        })
        .expect("legacy v3 seeded into store2");
    let (obj2, hit2) =
        auto_abi_probe::cache::produce_c_object(&store2, &src, &dir, &artifacts, produce)
            .expect("upgrade recompile must publish, never Internal-conflict");
    assert!(!hit2, "the legacy v3 entry is never served");
    assert_eq!(runs.get(), 2, "a real recompile happened and published v4");
    std::fs::write(root.join("scratch/linked.obj"), &obj2).unwrap();
    let pe = run::run_pe(
        &link_pe(
            &root.join("scratch"),
            &[root.join("scratch/linked.obj")],
            "p797_inc_entry",
        )
        .unwrap(),
    )
    .unwrap();
    assert_eq!(pe.code, 17, "same source, same correct object");
    // and a repeat now HITS the published v4 (not the legacy v3)
    let (_, hit3) =
        auto_abi_probe::cache::produce_c_object(&store2, &src, &dir, &artifacts, produce)
            .expect("v4 repeat");
    assert!(hit3);
    assert_eq!(runs.get(), 2);
}
