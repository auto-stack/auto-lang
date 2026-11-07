//! PLAN-912 V-13 formal performance leg: the FUNCTION session across the
//! four incremental states (fresh / hit / restart / body-edit) at the
//! formal multi-function scale — ≥64 top-level functions, ≥3 sibling
//! lambdas per tested owner, entry-reachable, first/middle/last lambda
//! edits — with P90 wall gates, the retained-frontend pool gate, a
//! 512-revision churn leg and the parent working-set delta. Runs under the
//! `ace-performance` fixed group (`test(/::performance::/)`).

pub mod performance {
    use ace_av::closures_fn_session::{ClosuresFunctionSession, RouteReport};
    use std::path::PathBuf;

    /// The formal scale source: 62 plain helpers + `sum` (3 lambdas) +
    /// `main` (3 lambdas) = 68 top-level functions, 66 units.
    fn scale_source() -> String {
        let mut src = String::new();
        for i in 0..62 {
            src.push_str(&format!("fn h{i}(n int) int {{ return n + {i} }}\n"));
        }
        src.push_str(
            "fn sum(n int) int {\n\
             var y int = 4\n\
             var g0 = (k int) => { y = y + k; return y }\n\
             var g1 = (k int) => { return y - k }\n\
             var g2 = (k int) => { return k * 3 }\n\
             return g0(n) + g1(1) + g2(4) + h0(1)\n\
             }\n",
        );
        src.push_str(
            "fn main() int {\n\
             var x int = 10\n\
             var f0 = (n int) => { x = x + n; return x }\n\
             var f1 = (n int) => { return x - n }\n\
             var f2 = (n int) => { return n * 2 }\n\
             return f0(20) + f1(3) + f2(5) + sum(2) + h1(1) + h61(1)\n\
             }\n",
        );
        src
    }

    /// The independent behavioral model (hand-computed semantics; mirrors
    /// reference-style reasoning, not the implementation).
    fn scale_value(x: i32, f0_extra: i32, f1_mul: i32, f2_mul: i32) -> i32 {
        // main: f0(20) writes the shared cell (x + 20 + extra), f1(3) reads
        // it AFTER the write, f2(5) = 5 * mul.
        let cell = x + 20 + f0_extra;
        let f1 = cell - 3 * f1_mul;
        let f2 = 5 * f2_mul;
        // sum: y=4, g0(2) -> 6 (shared cell), g1(1) reads 6, g2(4) = 12.
        let y = 4 + 2;
        let s = y + (y - 1) + 12 + (1 + 0);
        cell + f1 + f2 + s + (1 + 1) + (1 + 61)
    }

    fn isolated_store(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "plan912-perf-{tag}-{}-{}",
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
    fn closures_fn_formal_states_scale_and_churn() {
        const SAMPLES: usize = 30;
        const POOL: usize = 16 * 1024 * 1024;
        let mut fresh_t = Vec::new();
        let mut hit_t = Vec::new();
        let mut restart_t = Vec::new();
        let mut edit_t = Vec::new();
        let base = scale_value(10, 0, 1, 2);
        let edit_first = scale_value(10, 1, 1, 2); // f0 body: + n -> + n + 1
        let edit_middle = scale_value(10, 0, 2, 2); // f1 body: x - n -> x - n * 2
        let edit_last = scale_value(10, 0, 1, 3); // f2 body: n * 2 -> n * 3
                                                  // Hand arithmetic for the base (the model's independent check):
                                                  // cell=30, f1=27, f2=10, sum=6+5+12+1=24, helpers=2+62=64 → 155.
        assert_eq!(
            (base, edit_first, edit_middle, edit_last),
            (155, 157, 152, 160)
        );

        for id in 0..SAMPLES {
            let source = scale_source();
            let edited_first = source.replace("x = x + n; return x", "x = x + n + 1; return x");
            let root = isolated_store("states");

            let mut session = ClosuresFunctionSession::new(&root, "scale.at");
            let t = std::time::Instant::now();
            let fresh = session.apply_source(&source).expect("fresh");
            fresh_t.push(ms(t.elapsed()));
            assert_eq!(fresh.value, base, "sample {id}");
            assert!(
                matches!(fresh.route, RouteReport::Cold { units: 70 }),
                "{:?}",
                fresh.route
            );
            assert!(fresh.retained_bytes <= POOL);
            // The code-index build axis is WIRED at the real link seam and
            // honestly reports ZERO for the closures profile (it is
            // index-gated in ResidentProgram::prepare); a future profile
            // change would surface here instead of silently vanishing.
            assert_eq!(
                fresh.stages.total(ace_core::body_work::Stage::IndexBuild),
                0
            );

            let t = std::time::Instant::now();
            let hit = session.apply_source(&source).expect("hit");
            hit_t.push(ms(t.elapsed()));
            assert_eq!(hit.route, RouteReport::ZeroHit);
            assert_eq!(hit.submitted_units, 0);
            assert!(
                hit.warm_gate.as_ref().unwrap().pass(),
                "sample {id} hit gate"
            );
            assert_eq!(hit.stages.whole_total(), 0);
            assert!(hit.retained_bytes <= POOL);

            // FIRST lambda edit: exactly one unit, every sibling untouched.
            let t = std::time::Instant::now();
            let edit = session.apply_source(&edited_first).expect("edit-first");
            edit_t.push(ms(t.elapsed()));
            assert_eq!(edit.value, edit_first, "sample {id}");
            assert_eq!(
                edit.route,
                RouteReport::FastUnit {
                    unit: "d___closure_main_0".to_string()
                },
                "sample {id}: {:?}",
                edit.route
            );
            assert_eq!(edit.submitted_units, 1);
            assert!(
                edit.warm_gate.as_ref().unwrap().pass(),
                "sample {id} edit gate"
            );
            session.close().expect("close");

            let mut restart_session = ClosuresFunctionSession::new(&root, "scale.at");
            let t = std::time::Instant::now();
            let restart = restart_session
                .apply_source(&edited_first)
                .expect("restart");
            restart_t.push(ms(t.elapsed()));
            assert_eq!(restart.value, edit_first);
            assert!(matches!(restart.route, RouteReport::Cold { units: 70 }));
            // R3, honestly: the session's store wire is SPAN-BEARING, so a
            // restart over the EDITED source re-admits exactly the units
            // whose spans moved past the edit (here: main + f1 + f2 — the
            // same shape as PLAN-909's `store_writes <= 1` on a smaller
            // sample); every other unit hits the store and nothing hides.
            assert!(
                restart.production.cache_hits >= 60,
                "restart hits {}",
                restart.production.cache_hits
            );
            assert!(
                restart.production.store_writes <= 5,
                "restart wrote {} span-shifted units",
                restart.production.store_writes
            );
            assert_eq!(
                restart.stages.total(ace_core::body_work::Stage::MirLower),
                70
            );
            restart_session.close().expect("close");
            std::fs::remove_dir_all(&root).ok();
        }

        let report = |name: &str, v: &Vec<u128>| {
            format!(
                "{name}: P50={}ms P90={}ms P99={}ms max={}ms",
                percentile(v.clone(), 50.0),
                percentile(v.clone(), 90.0),
                percentile(v.clone(), 99.0),
                v.iter().max().copied().unwrap_or(0),
            )
        };
        eprintln!(
            "PLAN-912 V-13 formal (64-fn/70-unit scale, 30 samples x 4 states):\n  {}\n  {}\n  {}\n  {}",
            report("fresh", &fresh_t),
            report("hit", &hit_t),
            report("restart", &restart_t),
            report("body-edit", &edit_t),
        );
        for (name, v) in [
            ("fresh", &fresh_t),
            ("hit", &hit_t),
            ("restart", &restart_t),
            ("body-edit", &edit_t),
        ] {
            let p90 = percentile(v.clone(), 90.0);
            assert!(p90 <= 2000, "{name} P90 {p90}ms above the 2000ms gate");
        }

        // ── middle / last lambda + owner edits at the same scale ──
        let source = scale_source();
        let root = isolated_store("edges");
        let mut session = ClosuresFunctionSession::new(&root, "scale.at");
        assert_eq!(session.apply_source(&source).expect("cold").value, base);
        let middle_src = source.replace("return x - n }", "return x - n * 2 }");
        let middle = session.apply_source(&middle_src).expect("middle");
        assert_eq!(middle.value, edit_middle);
        assert_eq!(
            middle.route,
            RouteReport::FastUnit {
                unit: "d___closure_main_1".to_string()
            }
        );
        let last_src = middle_src.replace("return n * 2 }", "return n * 3 }");
        let last = session.apply_source(&last_src).expect("last");
        assert_eq!(last.value, scale_value(10, 0, 2, 3));
        assert_eq!(
            last.route,
            RouteReport::FastUnit {
                unit: "d___closure_main_2".to_string()
            }
        );
        // Owner body edit: x 10 -> 20 reuses every lambda (six total).
        let owner = session
            .apply_source(&last_src.replace("var x int = 10", "var x int = 20"))
            .expect("owner");
        assert_eq!(owner.value, scale_value(20, 0, 2, 3));
        match &owner.route {
            RouteReport::FastOwner {
                owner: o,
                changed,
                reused,
            } => {
                assert_eq!(o, "d_main");
                assert!(changed.is_empty());
                assert_eq!(reused.len(), 3, "main's three lambdas reused");
            }
            other => panic!("owner edit route: {other:?}"),
        }
        assert!(owner.warm_gate.as_ref().unwrap().pass());
        assert_eq!(owner.submitted_unit_names, vec!["d_main"]);

        // ── 512-revision churn: alternating first-lambda edits/flips ──
        let sampler = ace_av::mempeek::PeakSampler::start(50);
        let cold_peak_baseline = {
            let churn_root = isolated_store("baseline");
            let mut s = ClosuresFunctionSession::new(&churn_root, "churn.at");
            let _ = s.apply_source(&source).expect("baseline cold");
            let peak = sampler.stop();
            s.close().expect("close");
            std::fs::remove_dir_all(&churn_root).ok();
            peak
        };
        let sampler = ace_av::mempeek::PeakSampler::start(50);
        let mut churn = ClosuresFunctionSession::new(&root, "churn.at");
        churn.apply_source(&source).expect("churn cold");
        // §5.5: the churn rotates across DIFFERENT lambdas (f0's writer
        // body, f1's reader body, f2's pure body) and revisits prior
        // contents (history hits), flipping back to the base each 4th leg.
        let variants = [
            ("x = x + n; return x", "x = x + n + 1; return x"),
            ("return x - n }", "return x - n - 1 }"),
            ("return n * 2 }", "return n * 3 }"),
        ];
        let mut last_value = 0;
        for rev in 0..512 {
            let next = if rev % 2 == 0 {
                let (from, to) = variants[(rev / 2) % variants.len()];
                source.replace(from, to)
            } else {
                source.clone()
            };
            let run = churn.apply_source(&next).expect("churn rev");
            if rev % 64 == 0 {
                assert!(
                    run.retained_bytes <= POOL,
                    "rev {rev}: retained frontend {} above the 16 MiB pool",
                    run.retained_bytes
                );
                assert!(run.history_hot_bytes <= 8 * 1024 * 1024);
            }
            last_value = run.value;
        }
        let churn_peak = sampler.stop();
        assert_eq!(last_value, base, "the churn ends on the base revision");
        // Parent working-set delta vs the cold baseline (§5.5: ≤128 MiB).
        assert!(
            churn_peak <= cold_peak_baseline + 128 * 1024 * 1024,
            "churn peak {churn_peak} vs cold baseline {cold_peak_baseline} (+{})",
            churn_peak.saturating_sub(cold_peak_baseline)
        );
        churn.close().expect("close");
        std::fs::remove_dir_all(&root).ok();
    }
}
