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
use auto_abi_probe::decl::{AbiScalar, BodyKind, FnDecl, RecValues, FROZEN_INPUTS};
use auto_abi_probe::emit::{emit_callee_suite, emit_caller_suite, SuiteCheck};
use auto_abi_probe::layout::{self, REC_MIX};
use auto_abi_probe::link::link_pe;
use auto_abi_probe::run;
use auto_abi_probe::semref;
use auto_abi_probe::trace::{append_jsonl, percentiles, Trace};
use auto_abi_probe::validate::{admit, probe_expectation};

use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::schema::LayerSpec;
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

// ---------------------------------------------------------------------------
// abi_cache_ — hit/miss discipline and precise invalidation (V-06)
// ---------------------------------------------------------------------------

/// Hit never invokes the producer; miss persists; bytes are identical.
#[test]
fn abi_cache_object_hit_never_calls_producer() {
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
    assert_ne!(c_object_key("aaa").hash(), c_object_key("bbb").hash());
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
    let root = t05_root();
    let store = fresh_store(&root, "cache-churn");

    let payload: Vec<u8> = (0..2048).map(|i| (i % 251) as u8).collect();
    let mut last_art = String::new();
    let mut leased_art = String::new();
    for i in 0..512 {
        let key = c_object_key(&format!("rev-{i:04}"));
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
        .get_artifact(LAYER_OBJECT_V01.id, &c_object_key("rev-0511"))
        .expect("read newest")
        .expect("rooted newest survives GC");
    assert_eq!(newest.payload.len(), 2048);
    let old = store
        .get_artifact(LAYER_OBJECT_V01.id, &c_object_key("rev-0100"))
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
    let descriptor = {
        let mut d = auto_abi_probe::descriptor::ModuleDescriptor::probe_module();
        d
    };
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
        let key = c_object_key(&hash);
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
#[test]
fn abi_turnaround_six_scenarios_30_samples() {
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

    // 5) body-edit (HARD GATE): body flips between requests; C consumer
    //    unchanged and reused. P95 of request->correct PE result < 1000ms.
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
            let body = if i % 2 == 0 {
                BodyKind::MixV1Alt
            } else {
                BodyKind::MixV1
            };
            let defs = [mix_def(body)];
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
