//! PLAN-1000 (I10): the mn churn+GC face — 512 REAL body revisions through
//! the D-828 session with a REAL managed store under GC pressure, the av
//! closures face's sibling (`av_closures_incremental_store_512_revisions_
//! gc_bounded`). The four I10 observations, judged here parent-side:
//!
//! 1. **活 root/lease 保留** — every GC checkpoint re-admits ALL current
//!    unit editions from the store (restore by identity) and the session
//!    roots survive each GC;
//! 2. **最后引用后物理回收** — editions evicted from the bounded history
//!    window lose their last reference and the GC physically removes them
//!    (restore by the evicted edition's meta returns None; reclaimed bytes
//!    accumulate);
//! 3. **内存/磁盘水位有界** — the store's live roots stay at
//!    units × (1 + HISTORY_WINDOW) and the settled GC watermark does not
//!    grow without bound (end ≤ middle + 64 KiB);
//! 4. **用户哨兵不动** — a user sentinel file inside the store root is
//!    byte-identical after every GC (GC only touches managed namespaces).
//!
//! Churn conservation is judged too: revisions - live window == evicted
//! (every mint either stays in the bounded window or was reclaimed).

#[path = "../common/mod.rs"]
mod common;

use ace_native::incremental::{MnIncrementalSession, HISTORY_WINDOW};

// Every round is a real link/run subprocess pair — share the incremental
// tier's lock: parallel link storms pollute the tier's P90 wall gates
// (documented there: P95 1131 vs ~850 solo).

const REVISIONS: i64 = 512;
const GC_EVERY: i64 = 64;

mod performance {
    use super::*;

    // PLAN-1000 I10: 512 × real link+run rounds (≈2-3 min of link storms) —
    // stress-tier by nature. Uncontrolled, its link storms pollute the tier's
    // P90 wall gates (documented there: P95 1131 vs ~850 solo) and push the
    // ace-native group past its wall budget. Formal home is the explicit
    // single-test ignored replay:
    //   cargo t ace-perf --ignored --test-name mn_incremental_store_512_revisions_gc_bounded
    // The daily/full group records it as an ignored:true conditional exclusion.
    #[ignore = "PLAN-1000 I10 churn+GC stress leg: explicit single-test replay (see comment)"]
    #[test]
    fn mn_incremental_store_512_revisions_gc_bounded() {
        let _lock = crate::incremental_tier::T11_LOCK
            .lock()
            .unwrap_or_else(|p| p.into_inner());
        if !common::tools_available() {
            eprintln!("infra-blocked: skipped");
            return;
        }
        let src = "fn helper(x int) int {\n    return x * 2 + 0\n}\n\nfn main() int {\n    return helper(6)\n}\n";
        let root = std::env::temp_dir().join(format!("mn-i10-churn-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).unwrap();
        let store_root = root.join("store");
        let (mut s, cold) =
            MnIncrementalSession::warmup(src, "d_main", &[], &store_root, None).expect("warmup");
        assert_eq!(cold.exit_code, Some(12), "helper(6) = 6*2 + 0");
        let project = s.store_project().to_owned();
        let sentinel = store_root.join("user-sentinel.txt");
        std::fs::write(&sentinel, b"do not remove").unwrap();
        let store = ace_artifacts::store::ManagedStore::open(&store_root).unwrap();

        // The warm-up helper edition — the FIRST evicted edition once the
        // history window overflows; its physical removal is observation 2.
        let warm_fp = s.units().get("d_helper").expect("helper unit").fp.clone();

        let mut gc_watermarks = Vec::new();
        let mut reclaimed_bytes_total = 0u64;
        let mut evicted_seen = false;
        for factor in 1..=REVISIONS {
            let cur = s.source();
            let site = cur.find("x * 2 + ").expect("churn site") + "x * 2 + ".len();
            let end = cur[site..]
                .find('\n')
                .map(|i| site + i)
                .unwrap_or(cur.len());
            let obs = s.edit(site as u64, end as u64, format!("{factor}").into_bytes());
            assert!(
                obs.ok,
                "rev {factor}: {:?}",
                obs.infra.or(obs.refused_reason)
            );
            assert_eq!(
                obs.exit_code,
                Some((12 + factor) as i32),
                "rev {factor} value"
            );
            assert_eq!(obs.codegen_units, 1, "bounded unit emission");
            assert_eq!(obs.fallback_whole_lower, 0, "no degenerate fallback");
            assert_eq!(obs.unaffected_stage_delta, 0, "main untouched");
            assert_eq!(obs.whole_module_stage_delta, 0, "no whole-module stage");
            // Bounded live roots: 2 units × (1 current + HISTORY_WINDOW history).
            let roots = store.root_artifacts(&project, "soft").unwrap();
            assert!(
                roots.len() <= 2 * (1 + HISTORY_WINDOW),
                "live roots unbounded at rev {factor}: {}",
                roots.len()
            );
            if factor % GC_EVERY == 0 {
                let receipt = s.gc().expect("GC");
                reclaimed_bytes_total += receipt.freed_bytes;
                gc_watermarks.push(receipt.total_after);
                // Observation 1: live roots preserved — every CURRENT edition
                // still restores from the store after GC.
                for (name, entry) in s.units() {
                    let meta = s.unit_meta(name, &entry.fp);
                    let restored =
                        ace_native::cache::get_unit_coff(&store, &meta).expect("restore probe");
                    assert!(restored.is_some(), "current edition {name} lost after GC");
                }
                // Observation 2 (once the window overflowed): the warm-up
                // edition is PHYSICALLY gone — restore by its meta misses.
                if factor as usize > HISTORY_WINDOW {
                    let meta = s.unit_meta("d_helper", &warm_fp);
                    let gone =
                        ace_native::cache::get_unit_coff(&store, &meta).expect("restore probe");
                    assert!(gone.is_none(), "evicted warm-up edition still on disk");
                    evicted_seen = true;
                }
                // Observation 4: the user sentinel is untouched.
                assert_eq!(std::fs::read(&sentinel).unwrap(), b"do not remove");
            }
        }
        assert!(evicted_seen, "the history window must have overflowed");
        // Churn conservation: 512 mints − live window == evicted (reclaimed).
        let evicted = REVISIONS as u64 - HISTORY_WINDOW as u64;
        assert_eq!(
            s.reclaimed_history(),
            evicted,
            "mints − live window == evicted editions"
        );
        assert!(
            reclaimed_bytes_total > 0,
            "physical reclaim must actually free bytes"
        );
        // Observation 3: the settled watermark is bounded (av face's bound).
        assert_eq!(
            gc_watermarks.len(),
            8,
            "periodic GC checkpoints must execute"
        );
        let middle = gc_watermarks[3];
        let end = gc_watermarks[7];
        eprintln!("mn store GC settled bytes: {gc_watermarks:?}");
        assert!(
            end <= middle + 64 * 1024,
            "store watermark grew without bound: mid={middle}, end={end}"
        );
        // The session remains usable after the churn.
        let cur = s.source();
        let site = cur.find("x * 2 + ").expect("churn site") + "x * 2 + ".len();
        let end_site = cur[site..]
            .find('\n')
            .map(|i| site + i)
            .unwrap_or(cur.len());
        let obs = s.edit(site as u64, end_site as u64, b"1000".to_vec());
        assert!(
            obs.ok,
            "post-churn edit: {:?}",
            obs.infra.or(obs.refused_reason)
        );
        assert_eq!(obs.exit_code, Some(1012i32));
        s.close();
        drop(store);
        drop(s);
        let _ = std::fs::remove_dir_all(&root);
    }
}
