//! PLAN-909 V-15 formal performance leg: 30 original-scale samples across
//! the four incremental states (fresh / hit / restart / body-edit) with
//! P50/P90/P95/P99 wall times, plus a 512-revision churn run against the
//! frozen per-unit history caps. Runs under the `ace-perf` fixed
//! group (`test(/::performance::/)`).

pub mod performance {
    use ace_av::closures_865_support::ClosuresSourceSession;
    use std::collections::BTreeMap;
    use std::path::PathBuf;

    /// One original-scale sample: `owners` free functions plus a shared-cell
    /// writer/read family in main (mirrors the K/S corpus shape; the exact
    /// program varies deterministically per sample id).
    fn sample_source(id: usize, owners: usize) -> String {
        let mut src = String::new();
        for i in 0..owners {
            src.push_str(&format!(
                "fn helper{i}(n int) int {{ var acc int = n; var add = (k int) => {{ acc = acc + k; return acc }}; return add({i}) + acc }}\n"
            ));
        }
        let calls: Vec<String> = (0..owners).map(|i| format!("helper{i}(1)")).collect();
        src.push_str(&format!(
            "fn main() int {{ var x int = 1; var w = (n int) => {{ x = x + n; return x }}; let r = () => {{ return x }}; return w({id}) + r() + {} }}\n",
            calls.join(" + ")
        ));
        src
    }

    fn isolated_store(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "plan909-perf-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("create perf store");
        root
    }

    fn percentile(mut sorted: Vec<u128>, p: f64) -> u128 {
        if sorted.is_empty() {
            return 0;
        }
        sorted.sort_unstable();
        let idx = ((p / 100.0) * (sorted.len() - 1) as f64).round() as usize;
        sorted[idx.min(sorted.len() - 1)]
    }

    fn ms(d: std::time::Duration) -> u128 {
        d.as_millis()
    }

    #[test]
    fn closures_shared_mutable_formal_states_and_churn() {
        const SAMPLES: usize = 30;
        let mut fresh_t = Vec::new();
        let mut hit_t = Vec::new();
        let mut restart_t = Vec::new();
        let mut edit_t = Vec::new();
        let mut max_history_hot = 0usize;

        for id in 0..SAMPLES {
            let source = sample_source(id + 1, 4);
            let edited = source.replacen("x = x + n", "x = x + n + 1", 1);
            let root = isolated_store("states");

            // fresh: cold apply over an empty store.
            let mut session = ClosuresSourceSession::new(&root);
            let t = std::time::Instant::now();
            let fresh = session.apply_source(&source, "perf.at").expect("fresh");
            fresh_t.push(ms(t.elapsed()));
            assert_eq!(
                fresh.submitted_units,
                fresh
                    .frontend
                    .changed_owners
                    .len()
                    .max(1)
                    .max(fresh.submitted_units)
            );
            assert!(fresh.value > 0);

            // hit: identical re-apply — zero production of every kind.
            let t = std::time::Instant::now();
            let hit = session.apply_source(&source, "perf.at").expect("hit");
            hit_t.push(ms(t.elapsed()));
            assert_eq!(hit.submitted_units, 0, "sample {id}: hit produced units");
            assert!(
                hit.lowered_units.is_empty(),
                "sample {id}: hit lowered units"
            );
            assert_eq!(
                hit.production.store_writes, 0,
                "sample {id}: hit wrote artifacts"
            );
            max_history_hot = max_history_hot.max(hit.history_hot_bytes);

            // body-edit: exactly the edited lambda's unit is produced.
            let t = std::time::Instant::now();
            let edit = session.apply_source(&edited, "perf.at").expect("edit");
            edit_t.push(ms(t.elapsed()));
            assert_eq!(
                edit.submitted_units, 1,
                "sample {id}: {:#?}",
                edit.submitted_unit_names
            );
            session.close().expect("close");

            // restart: a fresh session over the same store re-admits from
            // persisted artifacts (read/decode/re-verify), zero writes.
            let mut fresh_session = ClosuresSourceSession::new(&root);
            let t = std::time::Instant::now();
            let restart = fresh_session
                .apply_source(&edited, "perf.at")
                .expect("restart");
            restart_t.push(ms(t.elapsed()));
            // The store holds the PRE-edit artifacts; the edited lambda's
            // unit is genuinely new — at most that one write on restart.
            assert!(
                restart.production.store_writes <= 1,
                "sample {id}: restart wrote {} artifacts",
                restart.production.store_writes
            );
            assert!(
                restart.production.cache_hits >= 1,
                "sample {id}: restart hit the store"
            );
            fresh_session.close().expect("close");
            std::fs::remove_dir_all(&root).ok();
        }

        let report = |name: &str, v: &Vec<u128>| {
            format!(
                "{name}: P50={}ms P90={}ms P95={}ms P99={}ms max={}ms",
                percentile(v.clone(), 50.0),
                percentile(v.clone(), 90.0),
                percentile(v.clone(), 95.0),
                percentile(v.clone(), 99.0),
                v.iter().max().copied().unwrap_or(0),
            )
        };
        let lines = [
            report("fresh", &fresh_t),
            report("hit", &hit_t),
            report("restart", &restart_t),
            report("body-edit", &edit_t),
        ];
        eprintln!(
            "PLAN-909 V-15 formal (30 samples x 4 states):\n  {}",
            lines.join("\n  ")
        );

        // The development-period gate: P90 within 2000ms for every state.
        for (name, v) in [
            ("fresh", &fresh_t),
            ("hit", &hit_t),
            ("restart", &restart_t),
            ("body-edit", &edit_t),
        ] {
            let p90 = percentile(v.clone(), 90.0);
            assert!(p90 <= 2000, "{name} P90 {p90}ms above the 2000ms gate");
        }

        // 512-revision churn on one sample: alternating body edits against
        // the frozen history caps (HISTORY_PER_UNIT = 32, code+history hot
        // budget 8 MiB) and a stable final oracle.
        let source = sample_source(1, 4);
        let root = isolated_store("churn");
        let mut session = ClosuresSourceSession::new(&root);
        session
            .apply_source(&source, "churn.at")
            .expect("churn cold");
        let baseline = session
            .apply_source(&source, "churn.at")
            .expect("churn base")
            .value;
        let mut variant_a = source.replacen("x = x + n", "x = x + n + 1", 1);
        let mut flip = true;
        let mut last_value = baseline;
        for rev in 0..512 {
            let next = if flip {
                source.replacen("x = x + n", "x = x + n + 1", 1)
            } else {
                source.clone()
            };
            flip = !flip;
            let run = session.apply_source(&next, "churn.at").expect("churn rev");
            // Every other revision is an EDIT (1 unit); the flip-back is a
            // full hit (0 units) — history stays bounded by the frozen cap.
            if rev % 64 == 0 {
                assert!(
                    run.history_hot_bytes <= 8 * 1024 * 1024,
                    "history hot bytes {} above the 8 MiB budget",
                    run.history_hot_bytes
                );
                max_history_hot = max_history_hot.max(run.history_hot_bytes);
            }
            last_value = run.value;
        }
        variant_a.clear();
        assert_eq!(
            last_value, baseline,
            "the churn family returns to the base value"
        );
        let final_hit = session
            .apply_source(&source, "churn.at")
            .expect("churn final");
        assert_eq!(final_hit.submitted_units, 0);
        session.close().expect("close");
        std::fs::remove_dir_all(&root).ok();
        eprintln!(
            "PLAN-909 V-15 churn: 512 revisions, max history hot {max_history_hot} bytes (cap 8 MiB), final hit zero-production"
        );
        // Silence unused warnings for the report map when assertions compile out.
        let _ = BTreeMap::<usize, u128>::new();
    }
}
