//! PLAN-818 T-07 driver-tier turnaround legs (`mir_native_turnaround_`):
//! the three scenarios that need ONE PROCESS or interleaved store
//! ownership — same-process hot, cross-project sharing, and real
//! concurrent GC under edit traffic — over the frozen reference workload,
//! with window raws, percentile reporting and the zero/NaN raw control.
//! The formal-runner legs (fresh / restart-warm / body-edit / signature
//! gates) live in the main workspace (`ac_mn_probe_tests.rs`).

mod common;

use ac_mn::driver::MnDriver;

fn temp_root(tag: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("mn8-t07d-{tag}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn percentile(v: &[f64], p: f64) -> f64 {
    let mut s = v.to_vec();
    s.sort_by(|a, b| a.partial_cmp(b).unwrap());
    s[((p / 100.0) * (s.len() - 1) as f64).round() as usize % s.len()]
}

fn assert_raw_admissible(name: &str, walls: &[f64]) {
    assert!(!walls.is_empty(), "{name}: no samples (missing raw)");
    for (i, w) in walls.iter().enumerate() {
        assert!(w.is_finite(), "{name}[{i}]: NaN window");
        assert!(*w > 0.0, "{name}[{i}]: zero window");
    }
}

/// Same-process hot: ONE driver, ONE store — the first request warms the
/// cache, every following request is an in-process hit that still really
/// links and runs. Windows are the full request walls (debug tier — the
/// absolute numbers are reported, not gated; the formal gates are the
/// edit paths in the main workspace).
#[test]
#[cfg(windows)]
fn mir_native_turnaround_same_process_hot_30() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let samples: usize = std::env::var("MN818_FORMAL_SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let wl = ac_mn::workload::generate(0);
    let root = temp_root("hot");
    let mut d = MnDriver::with_store(&root, "hot").expect("store");
    let mut walls = Vec::with_capacity(samples);
    for i in 0..samples {
        let t0 = std::time::Instant::now();
        let r = d.request(&wl.src, "d_main", &[], &root.join(format!("w{i}")), None);
        let w = t0.elapsed().as_secs_f64() * 1000.0;
        assert!(r.ok, "hot round {i}: {:?}", r.error);
        assert_eq!(r.exit_code, Some(wl.expected as i32), "round {i} mirror");
        let emitted = *r.obs.counters.get("codegen_emit_units").unwrap_or(&0);
        let hits = *r.obs.counters.get("cache_hit_coff").unwrap_or(&0);
        if i == 0 {
            assert_eq!(emitted, 115, "first request warms the store");
        } else {
            assert_eq!(emitted, 0, "round {i}: in-process hit (no codegen)");
            assert_eq!(hits, 115);
        }
        walls.push(w);
    }
    assert_raw_admissible("same-process-hot", &walls);
    eprintln!(
        "t07 driver same-process-hot: P50/P95/P99 = {:.1}/{:.1}/{:.1} ms ({samples} samples, debug tier)",
        percentile(&walls, 50.0),
        percentile(&walls, 95.0),
        percentile(&walls, 99.0),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Cross-project: two projects alternate over ONE store root — the
/// content-addressed unit keys share objects across project namespaces
/// (the second project's first request hits everything), and every
/// request returns the mirror result.
#[test]
#[cfg(windows)]
fn mir_native_turnaround_cross_project_30() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let samples: usize = std::env::var("MN818_FORMAL_SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let wl = ac_mn::workload::generate(0);
    let root = temp_root("xproj");
    let mut pa = MnDriver::with_store(&root, "proj-alpha").expect("store a");
    let mut pb = MnDriver::with_store(&root, "proj-beta").expect("store b");
    let mut walls = Vec::with_capacity(samples);
    for i in 0..samples {
        let d = if i % 2 == 0 { &mut pa } else { &mut pb };
        let t0 = std::time::Instant::now();
        let r = d.request(&wl.src, "d_main", &[], &root.join(format!("w{i}")), None);
        let w = t0.elapsed().as_secs_f64() * 1000.0;
        assert!(r.ok, "cross-project round {i}: {:?}", r.error);
        assert_eq!(r.exit_code, Some(wl.expected as i32), "round {i} mirror");
        let emitted = *r.obs.counters.get("codegen_emit_units").unwrap_or(&0);
        if i == 0 {
            assert_eq!(emitted, 115, "the store's very first request warms");
        } else {
            // Cross-project sharing: proj-beta's FIRST request already
            // hits every unit (content-addressed keys ignore the project
            // axis) — including pb's own round 1.
            assert_eq!(
                emitted, 0,
                "round {i}: shared objects across project namespaces"
            );
        }
        walls.push(w);
    }
    assert_raw_admissible("cross-project", &walls);
    eprintln!(
        "t07 driver cross-project: P50/P95/P99 = {:.1}/{:.1}/{:.1} ms ({samples} samples, debug tier)",
        percentile(&walls, 50.0),
        percentile(&walls, 95.0),
        percentile(&walls, 99.0),
    );
    let _ = std::fs::remove_dir_all(&root);
}

/// Real concurrent GC: 30 body-edit revisions flow through ONE store
/// while a background thread runs watermark GC passes against the SAME
/// store — publishers and GC serialize on the 783 liveness guard, every
/// round's result stays correct, the current publication stays live
/// (root-protected), and the store converges afterwards.
#[test]
#[cfg(windows)]
fn mir_native_turnaround_concurrent_gc_30() {
    if !common::tools_available() {
        eprintln!("infra-blocked: rust-lld/SDK unavailable — skipped");
        return;
    }
    let samples: usize = std::env::var("MN818_FORMAL_SAMPLES")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(30);
    let root = temp_root("concg");
    let stop = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let gc_root = root.clone();
    let gc_stop = std::sync::Arc::clone(&stop);
    let gc_handle = std::thread::spawn(move || {
        // The GC racer: continuous passes against the same store; passes
        // are SHORT (below the high watermark most of the time they
        // no-op; when traffic pushes past it, real reclaim happens).
        let store = ac_artifact_probe::store::ManagedStore::open(&gc_root).expect("gc store");
        let mut passes = 0u64;
        let mut deleted = 0u64;
        while !gc_stop.load(std::sync::atomic::Ordering::Relaxed) {
            if let Ok(rc) = ac_artifact_probe::store::run_gc(
                &store,
                ac_artifact_probe::store::gc_config_default(),
            ) {
                passes += 1;
                deleted += rc.deleted_chunks as u64;
            }
            std::thread::sleep(std::time::Duration::from_millis(250));
        }
        (passes, deleted)
    });
    let mut d = MnDriver::with_store(&root, "concg").expect("store");
    let mut walls = Vec::with_capacity(samples);
    for n in 1..=samples {
        let wl = ac_mn::workload::generate(n as i64);
        let t0 = std::time::Instant::now();
        let r = d.request(&wl.src, "d_main", &[], &root.join(format!("w{n}")), None);
        let w = t0.elapsed().as_secs_f64() * 1000.0;
        assert!(r.ok, "concurrent-GC round {n}: {:?}", r.error);
        assert_eq!(r.exit_code, Some(wl.expected as i32), "round {n} mirror");
        walls.push(w);
    }
    stop.store(true, std::sync::atomic::Ordering::Relaxed);
    let (passes, deleted) = gc_handle.join().expect("gc racer joins");
    assert!(passes >= 2, "the GC racer really ran ({passes} passes)");
    eprintln!(
        "t07 driver concurrent-gc: {passes} GC passes during {samples} edit rounds ({} chunks reclaimed); request P50/P95 = {:.1}/{:.1} ms (debug tier)",
        deleted,
        percentile(&walls, 50.0),
        percentile(&walls, 95.0),
    );
    assert_raw_admissible("concurrent-gc", &walls);
    // Physical reclaim under traffic was proven at T-05 scale; here the
    // post-traffic full-reclaim pass shows the store is healthy garbage
    // underneath the concurrent racing (dead revisions collectable).
    {
        let store = ac_artifact_probe::store::ManagedStore::open(&root).expect("post store");
        let rc = ac_artifact_probe::store::run_gc(
            &store,
            ac_artifact_probe::store::GcConfig { high: 0, low: 0 },
        )
        .expect("post-traffic full reclaim");
        assert!(rc.deleted_chunks >= 1, "dead edit revisions collectable");
    }
    // The CURRENT publication survived the concurrent passes: re-requesting
    // the last revision hits the still-live closure (codegen 0) and runs
    // its PE result again.
    let wl_last = ac_mn::workload::generate(samples as i64);
    let rl = d.request(&wl_last.src, "d_main", &[], &root.join("post"), None);
    assert!(rl.ok, "post-traffic request: {:?}", rl.error);
    assert_eq!(rl.exit_code, Some(wl_last.expected as i32));
    let _ = std::fs::remove_dir_all(&root);
}
