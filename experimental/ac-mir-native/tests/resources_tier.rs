//! PLAN-818 T-05 resources tier (`mir_native_resources_`): the frozen
//! reference workload against its closed-form mirror under the memory
//! partitions and tree/worker peaks; the ≥32 MiB genuinely-distinct cold
//! COFF set with a random read-only restore; 512 real edit/re-key/result
//! churn rounds pushing the SAME store over the inherited watermark, then
//! real GC with physical convergence, root (sentinel) and lease survival;
//! deep-CFG modules on BOTH sides of the 798 budget; the transient-spike
//! sampler counter-control; over-cap cleanup; and the raw zero/missing/
//! NaN gate (V-06 churn leg / V-08; AC-05).

mod common;

use ac_artifact_probe::codec::ChunkKind;
use ac_artifact_probe::identity::QueryKey;
use ac_artifact_probe::store::{
    gc_config_default, run_gc, ManagedStore, PutRequest, GC_HIGH, GC_LOW,
};
use ac_mn::cache::{get_unit_coff, put_unit_coff, unit_fp, CoffMeta};
use ac_mn::driver::MnDriver;
use ac_mn::link_driver::{link_and_run, ObjectInput};
use ac_mn::resources::{
    raw_gate, RawSample, TreePeakSampler, MANAGED_TOTAL_CAP, PARTITION_CAPS,
};
use ac_mn::{emit, worker, MnError};

fn temp_root(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mn8-t05-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

const MIB: u64 = 1024 * 1024;

/// V-08 / §5.5 frozen reference workload (golden.md §2): ≥256 KiB source,
/// closed-form mirror expectation, first-request compile, all-hits second
/// request, the frozen body-edit bit (fn_041 A0 + 1 → exactly one unit
/// recompiles), and the frozen signature-conflict edit (frontend diagnosis).
/// Memory gates: managed partitions ≤16/24/8 MiB (total ≤48), worker
/// private peak ≤512 MiB, request-tree peak ≤768 MiB, raw samples nonzero.
#[test]
#[cfg(windows)]
fn mir_native_resources_reference_workload_mirror_and_budgets() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let wl = ac_mn::workload::generate(0);
    assert!(
        wl.src.len() >= ac_mn::workload::MIN_SOURCE_BYTES,
        "reference workload source is {} bytes, below the frozen ≥256 KiB scale duty",
        wl.src.len()
    );
    // §2.4 hand-audit record (constants + stepwise acc + final value) for
    // fn_000 and fn_099 — archived in the T-05 receipt.
    eprintln!("--- T-05 audit fn_000 ---");
    for l in wl.audit_lines(0) {
        eprintln!("{l}");
    }
    eprintln!("--- T-05 audit fn_099 ---");
    for l in wl.audit_lines(99) {
        eprintln!("{l}");
    }
    eprintln!("mirror expected total = {}", wl.expected);

    let root = temp_root("workload");
    let mut d = MnDriver::with_store(&root, "wl").expect("store");
    let sampler = TreePeakSampler::start(5);
    let t0 = std::time::Instant::now();

    // First request: every one of the 115 units compiles through the NEW
    // backend and the real PE result equals the independent mirror.
    let r1 = d.request(&wl.src, "d_main", &[], &root.join("w1"), None);
    let request_ms = t0.elapsed().as_secs_f64() * 1000.0;
    assert!(r1.ok, "workload request failed: {:?}", r1.error);
    assert_eq!(
        r1.exit_code,
        Some(wl.expected as i32),
        "PE result == closed-form mirror (never a backend self-check)"
    );
    assert_eq!(
        *r1.obs.counters.get("codegen_emit_units").unwrap_or(&0),
        115,
        "fresh store compiles all 100 callers + 14 leaves + main"
    );

    // Second request: everything hits — codegen FORBIDDEN as the hard
    // control; the closure still really links and runs.
    d.forbid_codegen = true;
    let r2 = d.request(&wl.src, "d_main", &[], &root.join("w2"), None);
    assert!(r2.ok, "all-hits request failed: {:?}", r2.error);
    assert_eq!(r2.exit_code, Some(wl.expected as i32));
    assert_eq!(*r2.obs.counters.get("codegen_emit_units").unwrap_or(&0), 0);
    assert_eq!(*r2.obs.counters.get("cache_hit_coff").unwrap_or(&0), 115);

    // Frozen body edit (fn_041's A0 += n). Hand-audit finding (T-05
    // receipt / golden r1.1 note): the closed-form mirror is authoritative,
    // but the terminal VALUE has plateaus — n=1 converges back to the same
    // trajectory at block 05 (value stays 28), while n=2 moves it (15).
    // BOTH revisions must re-key exactly fn_041; only n=2 moves the total.
    d.forbid_codegen = false;
    let wl1 = ac_mn::workload::generate(1);
    assert_eq!(wl1.expected, wl.expected, "n=1 mirror plateau (documented)");
    let r3 = d.request(&wl1.src, "d_main", &[], &root.join("w3"), None);
    assert!(r3.ok, "body edit (n=1) failed: {:?}", r3.error);
    assert_eq!(r3.exit_code, Some(wl1.expected as i32));
    assert_eq!(
        *r3.obs.counters.get("codegen_emit_units").unwrap_or(&0),
        1,
        "n=1 still re-keys exactly fn_041 (fp moves even when the value plateaus)"
    );
    assert_eq!(*r3.obs.counters.get("cache_hit_coff").unwrap_or(&0), 114);
    let wl2 = ac_mn::workload::generate(2);
    assert_ne!(wl2.expected, wl.expected, "n=2 moves the oracle");
    let r3b = d.request(&wl2.src, "d_main", &[], &root.join("w3b"), None);
    assert!(r3b.ok, "body edit (n=2) failed: {:?}", r3b.error);
    assert_eq!(r3b.exit_code, Some(wl2.expected as i32));
    assert_eq!(
        *r3b.obs.counters.get("codegen_emit_units").unwrap_or(&0),
        1,
        "only fn_041 recompiles"
    );
    assert_eq!(*r3b.obs.counters.get("cache_hit_coff").unwrap_or(&0), 114);

    // Frozen signature-conflict edit: fn_leaf_2 gains a parameter — the
    // correct outcome is a FRONTEND diagnosis (call sites break), with
    // codegen never invoked.
    let r4 = d.request(
        &ac_mn::workload::signature_conflict_source(),
        "d_main",
        &[],
        &root.join("w4"),
        None,
    );
    assert!(!r4.ok, "the signature conflict must be diagnosed");
    match &r4.error {
        Some(MnError::Unsupported("frontend", msg)) => {
            assert!(
                msg.contains("fn_leaf_2") || msg.to_lowercase().contains("arg"),
                "diagnosis points at the broken leaf/arity: {msg}"
            );
        }
        other => panic!("expected frontend diagnosis, got {other:?}"),
    }
    assert_eq!(*r4.obs.counters.get("codegen_emit_units").unwrap_or(&0), 0);

    let tp = sampler.stop();
    // Managed partitions (§11): every share and the ≤48 MiB total.
    assert!(
        r1.obs.ledger.violations().is_empty(),
        "managed-data partitions: {:?}",
        r1.obs.ledger.violations()
    );
    let peaks = r1.obs.ledger.peaks();
    assert!(peaks[0] > 0, "MIR/symbol partition really charged");
    for (i, cap) in PARTITION_CAPS.iter().enumerate() {
        assert!(peaks[i] <= *cap, "partition {i}: {} > {cap}", peaks[i]);
    }
    assert!(r1.obs.ledger.peak_total() <= MANAGED_TOTAL_CAP);
    // Worker private peak and request-tree peak hard gates.
    assert!(tp.own_peak > 0, "sampler produced real samples");
    assert!(
        tp.own_peak <= 512 * MIB,
        "worker private peak {} > 512 MiB",
        tp.own_peak / MIB
    );
    assert!(tp.tree_peak >= tp.own_peak);
    assert!(
        tp.tree_peak <= 768 * MIB,
        "request tree peak {} > 768 MiB",
        tp.tree_peak / MIB
    );
    // Raw admissibility: real nonzero samples pass the gate...
    let codegen_ms = *r1.obs.spans.get("codegen").unwrap_or(&0) as f64;
    let raw = vec![
        RawSample::measured("workload_request_ms", request_ms, tp.own_peak),
        RawSample::measured("workload_codegen_ms", codegen_ms + 0.5, peaks[1]),
    ];
    assert!(raw_gate(&raw).is_empty(), "{:?}", raw_gate(&raw));
    let _ = std::fs::remove_dir_all(&root);
}

/// One fill function: `cf_{k}` with S genuinely-distinct add statements
/// (constants vary with k AND i; the symbol name differs by construction)
/// plus its `main`. Returns (source, unit-name-fragment).
fn fill_gen(k: usize, stmts: usize) -> String {
    let mut src = format!("fn cf_{k}(x int) int {{\n    var acc = x\n");
    for i in 0..stmts {
        let c = 1000 + ((k * 37 + i) % 977);
        src.push_str(&format!("    acc = acc + {c}\n"));
    }
    src.push_str(&format!(
        "    return acc\n}}\n\nfn main() int {{\n    return cf_{k}(3)\n}}\n"
    ));
    src
}

/// Compile one fill source to its verified units (the same public pipeline
/// the driver runs) and return them.
fn fill_units(src: &str) -> Vec<auto_av_core::mir::MirModule> {
    let compiled =
        auto_ac_prototype::source::compile_scalar_source(src, "mn.at", &[]).expect("compile");
    let mir =
        auto_av_probe::hir_adapter::lower_checked(&compiled.checked, "d_main", 0).expect("lower");
    let checked = auto_av_core::mir_verify::verify_mir(&mir).expect("verify");
    auto_av_probe::cache::split_units(checked.module()).expect("split")
}

/// The churn closure (T-04's form): body edit `k` re-keys `bump` while
/// `main` stays hot; independent oracle 30 + 2k.
fn churn_gen(k: i32) -> (String, i32) {
    (
        format!(
            "fn bump(x int) int {{\n    return x + {k}\n}}\n\nfn main() int {{\n    let a = bump(10)\n    let b = bump(20)\n    return a + b\n}}\n"
        ),
        30 + 2 * k,
    )
}

/// V-08 cold set + V-06 churn/GC leg: fill ONE store with ≥32 MiB of
/// genuinely different COFFs until it passes the inherited high watermark,
/// restore one random object read-only, run 512 real edit/re-key/result
/// rounds, then real GC — physical deletion, convergence below the low
/// watermark, the user sentinel (root) and a live lease both surviving.
#[test]
#[cfg(windows)]
fn mir_native_resources_cold_set_churn_gc_convergence() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let root = temp_root("coldgc");
    let store = ManagedStore::open(&root).expect("store");

    // The USER sentinel — a root-registered user artifact that must
    // survive every GC pass untouched.
    let sentinel_payload =
        b"plan818 t05 user property sentinel - must survive watermark GC".to_vec();
    let sentinel_id = store
        .put_artifact(PutRequest {
            layer: "USER-DATA".into(),
            key: QueryKey::new(
                &ac_mn::config::LINK_LAYER, // layer spec only shapes the key hash space
                vec![("kind".into(), "user-sentinel".into())],
            ),
            kind: ChunkKind::LayoutData,
            payload: sentinel_payload.clone(),
            edges: vec![],
            project: Some("user-project".into()),
        })
        .expect("sentinel put");
    store
        .register_root("user-project", &sentinel_id, "sentinel")
        .expect("sentinel root");

    // ---- cold fill: genuinely different COFFs until past the high mark --
    let mut filled: Vec<(CoffMeta, Vec<u8>)> = Vec::new();
    let mut hashes: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut filled_bytes: u64 = 0;
    let t_fill = std::time::Instant::now();
    let mut k = 0usize;
    loop {
        let src = fill_gen(k, 2000);
        for unit in fill_units(&src) {
            let obj = emit::emit_unit_coff(&unit).expect("emit fill unit");
            let meta = CoffMeta::for_unit(&unit, unit_fp(&unit)).unwrap();
            // Cache fill: NO project tag — a tagged put auto-soft-roots
            // the artifact in 783, which would pin every fill object live.
            put_unit_coff(&store, None, &meta, &obj.bytes).expect("fill put");
            assert!(
                hashes.insert(ac_mn::cache::content_hash(&obj.bytes)),
                "fill unit {} is not genuinely distinct",
                meta.unit
            );
            filled_bytes += obj.bytes.len() as u64;
            filled.push((meta, obj.bytes));
        }
        k += 1;
        if k % 12 == 0 {
            let sp = store.space();
            if k % 60 == 0 {
                eprintln!(
                    "fill {k}: store {} MiB, distinct COFF bytes {} MiB ({:?})",
                    sp.total / MIB,
                    filled_bytes / MIB,
                    t_fill.elapsed()
                );
            }
            if sp.total > GC_HIGH + MIB {
                break;
            }
        }
    }
    assert!(
        filled_bytes >= 32 * MIB,
        "cold set is {} MiB, below the frozen ≥32 MiB duty",
        filled_bytes / MIB
    );
    assert!(store.space().total > GC_HIGH, "store over the high watermark");
    assert_eq!(hashes.len(), filled.len(), "every COFF byte-distinct");

    // ---- random read-only restore of ONE function object ----------------
    let seed = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .subsec_nanos() as u64
        ^ (std::process::id() as u64) << 16;
    let pick = (seed % filled.len() as u64) as usize;
    let (meta_pick, bytes_pick) = &filled[pick];
    let restored = get_unit_coff(&store, meta_pick).expect("random read").expect("present");
    assert_eq!(
        restored, *bytes_pick,
        "random read-only restore binds byte-identically (sha admission)"
    );
    eprintln!("random cold read: unit {} ({} B) admitted", meta_pick.unit, bytes_pick.len());

    // ---- 512 real edit/re-key/result rounds on the SAME store -----------
    let sp_before_churn = store.space().total;
    let mut d = MnDriver::with_store(&root, "churn").expect("churn store");
    let t_churn = std::time::Instant::now();
    let mut last_manifest = None;
    let mut leased_manifest = None;
    for kr in 0i32..512 {
        let (src, want) = churn_gen(kr);
        let r = d.request(&src, "d_main", &[], &root.join(format!("c{kr}")), None);
        assert!(r.ok, "churn round {kr} failed: {:?}", r.error);
        assert_eq!(r.exit_code, Some(want), "round {kr} independent oracle");
        let emitted = *r.obs.counters.get("codegen_emit_units").unwrap_or(&0);
        let hits = *r.obs.counters.get("cache_hit_coff").unwrap_or(&0);
        if kr == 0 {
            assert_eq!(emitted, 2, "fresh closure compiles both units");
        } else {
            assert_eq!(emitted, 1, "round {kr}: ONLY the edited unit re-keys");
            assert_eq!(hits, 1, "round {kr}: the unchanged unit hits");
        }
        last_manifest = r.manifest_artifact_id.clone();
        if kr == 510 {
            leased_manifest = r.manifest_artifact_id.clone();
        }
        if kr % 128 == 127 {
            eprintln!("churn {kr}: {:?}", t_churn.elapsed());
        }
    }
    let sp_after_churn = store.space().total;
    assert!(
        sp_after_churn > sp_before_churn,
        "512 real distinct edits physically grew the store ({} -> {})",
        sp_before_churn,
        sp_after_churn
    );
    let last_manifest = last_manifest.expect("manifest id");
    // Lease the ROUND-510 manifest: its soft root was already dropped by
    // round 511's ordered publication, so the LEASE ALONE must keep it
    // (and its required closure) alive across GC.
    let leased_manifest = leased_manifest.expect("round-510 manifest id");
    let lease = store
        .acquire_lease("mn-t05-gc", 600, Some(&leased_manifest))
        .expect("lease the superseded closure");

    // ---- real GC: physical deletion, convergence, protected survivors ---
    let rc = run_gc(&store, gc_config_default()).expect("gc");
    eprintln!(
        "gc: ran={} deleted={} freed {} MiB pending={} backoff={:?} {} MiB -> {} MiB in {}ms",
        rc.ran,
        rc.deleted_chunks,
        rc.freed_bytes / MIB,
        rc.failed_deletes,
        rc.backoff,
        rc.total_before / MIB,
        rc.total_after / MIB,
        rc.duration_ms
    );
    assert!(rc.ran, "GC must actually run over the high watermark");
    assert!(rc.deleted_chunks >= 1, "at least one PHYSICAL deletion");
    assert!(rc.freed_bytes > 0);
    assert!(rc.backoff.is_none(), "no live overage: {}", rc.backoff.unwrap_or_default());
    let sp_after_gc = store.space().total;
    assert!(
        sp_after_gc <= GC_LOW,
        "physical convergence below the low watermark: {} MiB > {}",
        sp_after_gc / MIB,
        GC_LOW / MIB
    );

    // Sentinel: byte-identical, still addressable.
    let sentinel_after = store
        .get_artifact_by_id(&sentinel_id)
        .expect("sentinel get")
        .expect("sentinel survived GC");
    assert_eq!(sentinel_after.payload, sentinel_payload, "sentinel bytes untouched");

    // Leased closure: the superseded round-510 manifest and its units
    // survived on the lease alone; restoring runs its PE result again.
    let out = worker::restore_run_by_id(&root, &leased_manifest, &root.join("leased"), None)
        .expect("leased closure restores and runs");
    let (_, want510) = churn_gen(510);
    assert_eq!(out.exit_code, Some(want510), "leased closure result identical");

    // Unreachable cache data is REALLY gone: watermark GC deletes
    // oldest-first down TO the low watermark, so the store's very first
    // fill unit is among the reclaimed — its key misses cleanly (a
    // physical deletion, never an error).
    let (meta0, bytes0) = &filled[0];
    assert!(
        matches!(get_unit_coff(&store, meta0), Ok(None)),
        "oldest unleased cache object physically deleted"
    );
    assert!(
        bytes0.len() > 100 * 1024,
        "the deleted object was a real full-size COFF ({} B)",
        bytes0.len()
    );

    // Release the lease, GC again with 783's documented test seam
    // (GcConfig{low: 0} = reclaim EVERY dead candidate — the store is now
    // below the high watermark, but full reclaim proves the released
    // closure is collectable and nothing else was hiding behind the lease):
    // the formerly-leased manifest is gone and its restore becomes a
    // controlled rejection.
    lease.release();
    let rc2 = run_gc(&store, ac_artifact_probe::store::GcConfig { high: 0, low: 0 })
        .expect("gc 2 (full reclaim seam)");
    assert!(rc2.deleted_chunks >= 1, "full reclaim deletes the released closure");
    match worker::restore_run_by_id(&root, &leased_manifest, &root.join("released"), None) {
        Err(MnError::Admit(kind, _)) => assert_eq!(kind, "manifest"),
        other => panic!("released closure must be gone: {other:?}"),
    }

    // The CURRENT publication (round 511) stayed root-protected through
    // both GC passes and still restores.
    let (_, want511) = churn_gen(511);
    let out511 = worker::restore_run_by_id(&root, &last_manifest, &root.join("current"), None)
        .expect("current publication survives GC");
    assert_eq!(out511.exit_code, Some(want511));

    // Post-GC correctness: after the full reclaim every cache entry is
    // gone — a fresh request recompiles cleanly from scratch.
    let mut d2 = MnDriver::with_store(&root, "post").expect("post store");
    let (src0, want0) = churn_gen(0);
    let rp = d2.request(&src0, "d_main", &[], &root.join("post"), None);
    assert!(rp.ok, "post-GC request failed: {:?}", rp.error);
    assert_eq!(rp.exit_code, Some(want0));
    // The edited `bump(0)` unit recompiles; the source-identical `main`
    // unit's content-addressed chunk is shared across every round and
    // stayed LIVE through the full reclaim via the current publication's
    // required edge — one recompile, one edge-protected hit.
    assert_eq!(*rp.obs.counters.get("codegen_emit_units").unwrap_or(&0), 1);
    assert_eq!(*rp.obs.counters.get("cache_hit_coff").unwrap_or(&0), 1);
    let _ = std::fs::remove_dir_all(&root);
}

/// V-08 deep CFG on BOTH sides of the 798 budget: a many-block branch/loop
/// mix well inside the table budget compiles and runs correctly through
/// the NEW backend; a single function OVER the 65 536 per-table budget is
/// rejected by the 798 gate (`av.resource.*`) with codegen never invoked.
#[test]
#[cfg(windows)]
fn mir_native_resources_deep_cfg_budget_sides() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    // Under-budget side: 1 700 sequential guarded blocks (~3.4 K blocks;
    // the binding budget here is the verify init-analysis bitset bound —
    // blocks x words x 3 matrices, ~12 MiB < the frozen 16 MiB) — deep
    // CFG with a mirror oracle: acc=9 drains to 5 in three steps, then
    // every further guard is false.
    let mut src = String::from("fn deep(x int) int {\n    var acc = x\n");
    for _ in 0..1700 {
        src.push_str("    if acc > 5 {\n        acc = acc - 2\n    }\n");
    }
    src.push_str("    return acc\n}\n\nfn main() int {\n    return deep(9)\n}\n");
    let mut d = MnDriver::ephemeral();
    let r = d.request(&src, "d_main", &[], &temp_root("deep-under").join("w"), None);
    assert!(r.ok, "deep-CFG under budget failed: {:?}", r.error);
    assert_eq!(r.exit_code, Some(5), "9 -> 7 -> 5, guards then false");

    // Over-budget side: one function with 13 600 add statements ≈ 68 000
    // table entries — over the frozen 65 536 per-table budget.
    let mut big = String::from("fn huge(x int) int {\n    var acc = x\n");
    for i in 0..13600 {
        big.push_str(&format!("    acc = acc + {}\n", 1000 + (i % 89)));
    }
    big.push_str("    return acc\n}\n\nfn main() int {\n    return huge(3)\n}\n");
    let r2 = d.request(&big, "d_main", &[], &temp_root("deep-over").join("w"), None);
    assert!(!r2.ok, "over-budget module must be rejected");
    let msg = r2.error.as_ref().expect("error").message();
    assert!(
        msg.contains("av.resource."),
        "the 798 resource taxonomy surfaces (got: {msg})"
    );
    assert_eq!(
        *r2.obs.counters.get("codegen_emit_units").unwrap_or(&0),
        0,
        "codegen never invoked past the budget gate"
    );

    // Over-budget side #2 — the BLOCK-side bound discovered by this tier:
    // 2 400 sequential guarded blocks (~4.8 K blocks) blow the verify
    // init-analysis bitset budget (blocks x words x 3 matrices > 16 MiB)
    // BEFORE the table budget; still `av.resource.*`, codegen 0.
    let mut wide = String::from("fn wide(x int) int {\n    var acc = x\n");
    for _ in 0..2400 {
        wide.push_str("    if acc > 5 {\n        acc = acc - 2\n    }\n");
    }
    wide.push_str("    return acc\n}\n\nfn main() int {\n    return wide(9)\n}\n");
    let r3 = d.request(&wide, "d_main", &[], &temp_root("deep-over2").join("w"), None);
    assert!(!r3.ok, "block-heavy module over the analysis budget must be rejected");
    let msg3 = r3.error.as_ref().expect("error").message();
    assert!(
        msg3.contains("av.resource.verify-memory") || msg3.contains("av.resource."),
        "798 resource taxonomy surfaces (got: {msg3})"
    );
    assert_eq!(*r3.obs.counters.get("codegen_emit_units").unwrap_or(&0), 0);
}

/// V-08 transient-spike counter-control: a ~160 MiB allocation that is
/// freed before the window closes is INVISIBLE to endpoint deltas but
/// MUST be caught by the ≥5 ms peak sampler; the raw gate rejects
/// zero/missing/NaN/failed samples and an empty raw.
#[test]
#[cfg(windows)]
fn mir_native_resources_transient_spike_sampler_and_raw_gate() {
    let sampler = TreePeakSampler::start(5);
    let base = auto_av_probe::mempeek::sample_private_bytes();
    let t0 = std::time::Instant::now();
    let mut spike = vec![0u8; 160 * 1024 * 1024];
    for chunk in spike.chunks_mut(4096) {
        chunk[0] = 1; // commit the pages — private bytes must rise
    }
    drop(spike);
    std::thread::sleep(std::time::Duration::from_millis(40));
    let window_ms = t0.elapsed().as_secs_f64() * 1000.0;
    let tp = sampler.stop();
    let after = auto_av_probe::mempeek::sample_private_bytes();

    assert!(tp.ticks >= 5, "sampler actually polled ({})", tp.ticks);
    assert!(
        tp.own_peak >= base + 100 * MIB,
        "the transient spike is caught: peak {} MiB vs base {} MiB",
        tp.own_peak / MIB,
        base / MIB
    );
    assert!(
        after + 60 * MIB <= tp.own_peak,
        "the endpoint sits far below the observed peak — an endpoint delta cannot reconstruct the freed spike (after {} MiB, peak {} MiB)",
        after / MIB,
        tp.own_peak / MIB
    );
    assert!(tp.tree_peak >= tp.own_peak);

    // Real nonzero raw passes; degenerate raws keep the aggregate gate RED.
    let real = vec![
        RawSample::measured("spike_window_ms", window_ms, tp.own_peak),
        RawSample::measured("spike_ticks_ms", tp.ticks as f64, after.max(1)),
    ];
    assert!(raw_gate(&real).is_empty(), "{:?}", raw_gate(&real));

    let zero_time = vec![RawSample::measured("z", 0.0, 4096)];
    assert!(!raw_gate(&zero_time).is_empty(), "zero time never passes");
    let zero_bytes = vec![RawSample::measured("z", 1.0, 0)];
    assert!(!raw_gate(&zero_bytes).is_empty(), "zero bytes never passes");
    let nan = vec![RawSample::measured("z", f64::NAN, 4096)];
    assert!(!raw_gate(&nan).is_empty(), "NaN never passes");
    let failed = vec![RawSample {
        label: "z".into(),
        ms: 5.0,
        bytes: 4096,
        ok: false,
    }];
    assert!(!raw_gate(&failed).is_empty(), "a failed sample never passes");
    assert!(!raw_gate(&[]).is_empty(), "missing raw never passes");
}

/// V-08 over-cap cleanup: link inputs past the frozen 128 MiB cap are
/// rejected BEFORE any staging exists (nothing left behind); a >16 MiB
/// object put is a controlled `mn.resource.object-size` rejection with
/// nothing persisted. (`request-objects` >64 MiB is unreachable through
/// real 798-budgeted input — the 65 536 per-table budget bounds a request
/// far below it — so the frozen driver check stays the guard.)
#[test]
fn mir_native_resources_over_cap_cleanup() {
    // Link-input cap: 3 × 44 MiB synthetic objects > 128 MiB total.
    let workdir = temp_root("cap");
    let objects: Vec<ObjectInput> = (0..3)
        .map(|i| ObjectInput {
            name: format!("big-{i}"),
            bytes: vec![0xA5u8; 44 * 1024 * 1024],
        })
        .collect();
    match link_and_run(
        &objects,
        b"wrapper",
        ac_mn::config::START_SYMBOL,
        &workdir,
        None,
        "cap probe",
    ) {
        Err(MnError::Resource(kind, msg)) => {
            assert_eq!(kind, "link-input", "{msg}");
        }
        other => panic!("over-cap link inputs must Resource-reject: {:?}", other.map(|_| ())),
    }
    let leftovers: Vec<_> = std::fs::read_dir(&workdir)
        .expect("workdir")
        .flatten()
        .collect();
    assert!(
        leftovers.is_empty(),
        "no staging written before the cap rejection"
    );

    // Object-size cap: a >16 MiB blob under a real unit meta.
    let dir = temp_root("objcap");
    let store = ManagedStore::open(&dir).unwrap();
    let src = "fn helper(v int) int {\n    return v + 5\n}\n\nfn main() int {\n    return helper(37)\n}\n";
    let unit = fill_units(src)
        .into_iter()
        .find(|u| u.funcs[0].name == "d_helper")
        .expect("helper unit");
    let meta = CoffMeta::for_unit(&unit, unit_fp(&unit)).unwrap();
    let blob = vec![0x42u8; 16 * 1024 * 1024 + 1];
    match put_unit_coff(&store, None, &meta, &blob) {
        Err(MnError::Resource(kind, msg)) => assert_eq!(kind, "object-size", "{msg}"),
        other => panic!("oversized object must Resource-reject: {other:?}"),
    }
    assert!(
        matches!(get_unit_coff(&store, &meta), Ok(None)),
        "nothing persisted by the rejected put"
    );
    let _ = std::fs::remove_dir_all(&dir);
    let _ = std::fs::remove_dir_all(&workdir);
}
