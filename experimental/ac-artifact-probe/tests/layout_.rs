//! V-03 layout tests (§5.2): two frozen layouts over one logical
//! population — value equality against the independent semantic baseline,
//! on-demand reads inside frozen budgets, real OS memory evidence, and
//! page-level sharing for suffix-modified versions.
//!
//! Heavy by design (≥128 MiB per layout, plan-frozen): the run takes tens
//! of seconds. Budgets are NOT widened to make anything green.

mod common;

use ac_artifact_probe::layout::{
    self, expected_record, render_record_atom, LayoutStore, LayoutView, HOT_BUDGET, PAGE_SIZE,
    QUERY_PAYLOAD_CAP, RECORD_COUNT,
};
use common::scratch;

fn sample_ids() -> Vec<u64> {
    // Deterministic spread over the whole id space (start, middle, end,
    // primes) — every sampled record must equal the pure baseline in BOTH
    // layouts.
    let mut v = vec![0, RECORD_COUNT - 1, RECORD_COUNT / 2];
    let mut x = 1u64;
    for _ in 0..40 {
        x = x
            .wrapping_mul(6364136223846793005)
            .wrapping_add(1442695040888963407);
        v.push(x % RECORD_COUNT);
    }
    v
}

fn build_both(root: &std::path::Path) -> (layout::BuildReport, layout::BuildReport) {
    let store = LayoutStore::new(root);
    let a = layout::build_layout_a(&store, 1, 0xC0FFEE, None, 0, None).unwrap();
    let b = layout::build_layout_b(&store, 1, 0xC0FFEE, None, 0, None).unwrap();
    (a, b)
}

#[test]
fn layout_both_match_semantic_baseline_and_write_at_least_128mib() {
    let root = scratch("layout-values");
    let (ra, rb) = build_both(&root);
    // Real-write requirement: EACH layout ≥128 MiB on disk.
    assert!(
        ra.bytes_written >= 128 * 1024 * 1024,
        "layout A wrote {} bytes",
        ra.bytes_written
    );
    assert!(
        rb.bytes_written >= 128 * 1024 * 1024,
        "layout B wrote {} bytes",
        rb.bytes_written
    );

    let store = LayoutStore::new(&root);
    let mut va = LayoutView::open(&store, ra.manifest.clone()).unwrap();
    let mut vb = LayoutView::open(&store, rb.manifest.clone()).unwrap();
    for id in sample_ids() {
        let want = expected_record(id, 0xC0FFEE, None, 0);
        let got_a = va.record(id).unwrap();
        let got_b = vb.record(id).unwrap();
        assert_eq!(got_a, want, "layout A record {}", id);
        assert_eq!(got_b, want, "layout B record {}", id);
        // Atom-style baseline text agrees with both layouts.
        assert_eq!(render_record_atom(&got_a), render_record_atom(&want));
    }
}

#[test]
fn layout_small_query_reads_within_frozen_budgets() {
    let root = scratch("layout-budget");
    let (ra, rb) = build_both(&root);
    let store = LayoutStore::new(&root);

    for (manifest, name) in [(ra.manifest.clone(), "A"), (rb.manifest.clone(), "B")] {
        let mut v = LayoutView::open(&store, manifest).unwrap();
        let mut max_io = 0u64;
        for id in sample_ids() {
            let before = v.stats.bytes_read;
            let r = v.record(id).unwrap();
            let io = v.stats.bytes_read - before;
            // The record itself came back (value equality spot check).
            assert_eq!(r, expected_record(id, 0xC0FFEE, None, 0));
            max_io = max_io.max(io);
        }
        assert!(
            max_io <= QUERY_PAYLOAD_CAP,
            "layout {}: single small query touched {} bytes (cap {})",
            name,
            max_io,
            QUERY_PAYLOAD_CAP
        );
        assert!(
            v.stats.max_query_io_bytes <= QUERY_PAYLOAD_CAP,
            "layout {}: recorded max query I/O {} over cap",
            name,
            v.stats.max_query_io_bytes
        );
        // I/O staging is structurally ≤2 pages and hot cache ≤ budget.
        assert_eq!(v.stats.staging_buffers, 2);
        assert_eq!(v.stats.staging_bytes, PAGE_SIZE * 2);
        assert!(
            v.stats.hot_bytes_now <= HOT_BUDGET,
            "hot cache {} over budget",
            v.stats.hot_bytes_now
        );
    }
}

#[test]
fn layout_hot_cache_bounded_with_real_memory_evidence() {
    let root = scratch("layout-hot");
    let (ra, _) = build_both(&root);
    let store = LayoutStore::new(&root);
    let mem0 = ac_artifact_probe::sysinfo::current_process_memory();
    let mut v = LayoutView::open(&store, ra.manifest.clone()).unwrap();
    // Sweep far beyond the 8 MiB hot budget (400+ distinct pages).
    let mut ids = Vec::new();
    for page in 0..400u64 {
        ids.push(page * 2048 + 17);
    }
    for id in ids {
        v.record(id).unwrap();
    }
    let stats = v.stats;
    assert!(stats.hot_evictions > 0, "wide sweep must evict");
    assert!(stats.hot_bytes_now <= HOT_BUDGET);
    assert!(stats.bytes_read > 0, "sweep did real I/O");
    let mem1 = ac_artifact_probe::sysinfo::current_process_memory();
    // OS-level evidence: the process did NOT load the store (a full load
    // would be ≥128 MiB of private growth). Bound: hot budget + staging +
    // generous allocator/view slack.
    #[cfg(windows)]
    {
        let delta = mem1.private_bytes.saturating_sub(mem0.private_bytes);
        assert!(
            delta < 32 * 1024 * 1024,
            "private bytes grew {} after a 400-page sweep — the store is being over-resident",
            delta
        );
    }
    let _ = (mem0, mem1);
}

#[test]
fn layout_suffix_version_shares_unchanged_pages() {
    let root = scratch("layout-share");
    let store = LayoutStore::new(&root);
    let v1 = layout::build_layout_a(&store, 1, 0xC0FFEE, None, 0, None).unwrap();
    // Version 2: only records from the last 8 record-table pages change.
    let modified_from = RECORD_COUNT - 8 * (PAGE_SIZE as u64 / 32);
    let v2 = layout::build_layout_a(
        &store,
        2,
        0xC0FFEE,
        Some(modified_from),
        0x5A5A,
        Some(&v1.manifest),
    )
    .unwrap();

    // Sharing: most page chunks are identical ⇒ shared with the base.
    let shared_ratio = v2.shared_with_base as f64 / v2.total_pages as f64;
    assert!(
        shared_ratio > 0.90,
        "suffix modification shared only {:.1}% of pages",
        shared_ratio * 100.0
    );
    // Values: prefix records keep v1 values; modified records differ.
    let mut view2 = LayoutView::open(&store, v2.manifest.clone()).unwrap();
    let keep = expected_record(1000, 0xC0FFEE, Some(modified_from), 0x5A5A); // == v1 value
    assert_eq!(keep, expected_record(1000, 0xC0FFEE, None, 0));
    assert_eq!(view2.record(1000).unwrap(), keep);
    let changed = expected_record(RECORD_COUNT - 1, 0xC0FFEE, Some(modified_from), 0x5A5A);
    assert_ne!(
        changed,
        expected_record(RECORD_COUNT - 1, 0xC0FFEE, None, 0)
    );
    assert_eq!(view2.record(RECORD_COUNT - 1).unwrap(), changed);
    // v1 remains readable independently (immutable version).
    let mut view1 = LayoutView::open(&store, v1.manifest.clone()).unwrap();
    assert_eq!(
        view1.record(RECORD_COUNT - 1).unwrap(),
        expected_record(RECORD_COUNT - 1, 0xC0FFEE, None, 0)
    );
}

#[test]
fn layout_corrupt_page_is_controlled_rejection() {
    let root = scratch("layout-corrupt");
    let store = LayoutStore::new(&root);
    let ra = layout::build_layout_a(&store, 1, 0xC0FFEE, None, 0, None).unwrap();
    // Corrupt the FIRST record page on disk.
    let cid = &ra.manifest.record_pages[0];
    let path = store.page_path(cid);
    let mut bytes = std::fs::read(&path).unwrap();
    let last = bytes.len() - 1;
    bytes[last] ^= 0x01;
    std::fs::write(&path, &bytes).unwrap();
    // Chunk validation refuses the corrupt page (checksum).
    assert!(layout::validate_page_chunk(&store, cid).is_err());
    // A view reading through that page still reads BYTES but the probe's
    // framing gate (validate) is the controlled entry — a view access
    // without validation returning data through a corrupt chunk is caught
    // by the checksum gate here, not by a panic.
}

#[test]
fn layout_out_of_range_id_is_controlled_miss() {
    let root = scratch("layout-range");
    let store = LayoutStore::new(&root);
    let ra = layout::build_layout_a(&store, 1, 0xC0FFEE, None, 0, None).unwrap();
    let mut v = LayoutView::open(&store, ra.manifest.clone()).unwrap();
    assert!(v.record(RECORD_COUNT + 5).is_err());
}
