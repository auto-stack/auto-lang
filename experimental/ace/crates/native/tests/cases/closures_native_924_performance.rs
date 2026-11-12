//! PLAN-924 V-10 formal performance leg: the NATIVE closures session
//! across the four incremental states (fresh / hit / restart-restore /
//! body-edit) at the formal multi-function scale — ≥64 top-level
//! functions, ≥3 sibling lambdas per tested owner — with P90 wall gates,
//! a 512-distinct-revision churn leg, the budget/cancel refusal faces and
//! the stable-edit zero-rustc/cargo invariant. Runs under the
//! `ace-perf` fixed group (`test(/::performance::/)`).

pub mod performance {
    use ace_native::collections_lower::WrapperKnobs;
    use std::path::PathBuf;

    /// The formal scale source: 62 plain helpers + `make` (1 move-lambda)
    /// + `sum` (3 sibling lambdas) + `main` (3 sibling lambdas) = 65
    /// top-level functions, 72 units.
    fn scale_source() -> String {
        let mut src = String::new();
        for i in 0..62 {
            src.push_str(&format!("fn h{i}(n int) int {{ return n + {i} }}\n"));
        }
        src.push_str(
            "fn make(move s Str) fn(int) int {\n\
             return move (n int) => { return s.len() + n }\n\
             }\n",
        );
        src.push_str(
            "fn sum(n int) int {\n\
             var y int = 4\n\
             var g0 = (k int) => { return y + k }\n\
             var g1 = (k int) => { return y - k }\n\
             var g2 = (k int) => { return k * 3 }\n\
             return g0(n) + g1(1) + g2(4) + h0(1)\n\
             }\n",
        );
        src.push_str(
            "fn main() int {\n\
             let base int = 10\n\
             let s Str = \"abc\"\n\
             let f0 = (n int) => { return base + n }\n\
             let f1 = make(s.move)\n\
             let f2 = (n int) => { return n * 2 }\n\
             let f3 = (n int) => { return base - n }\n\
             return f0(20) + f1(3) + f2(5) + f3(4) + sum(2) + h0(1) + h61(1)\n\
             }\n",
        );
        src
    }

    fn isolated(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "plan924-perf-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("create perf root");
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

    /// The four formal states over 30 samples each with the P90 gate, then
    /// the 512-distinct-revision churn, then the budget/cancel refusals.
    #[test]
    fn closures_native_formal_states_churn_and_refusals() {
        const SAMPLES: usize = 30;
        const P90_CAP_MS: u128 = 2000;
        let mut fresh_t = Vec::new();
        let mut hit_t = Vec::new();
        let mut restart_t = Vec::new();
        let mut edit_t = Vec::new();
        let source = scale_source();
        let edited_base = source.replace("return base + n }", "return base + n + 1 }");

        for id in 0..SAMPLES {
            let root = isolated("states");
            // fresh: the cold session produces all 72 units and runs 137.
            let t = std::time::Instant::now();
            let (mut session, fresh) =
                ace_native::closures_incremental::ClosuresNativeSession::new(
                    &source,
                    "main",
                    &root.join("out"),
                    None,
                )
                .expect("fresh session");
            fresh_t.push(t.elapsed().as_millis());
            assert_eq!(fresh.exit_code, Some(137), "sample {id}");
            assert_eq!(fresh.work.codegen_units.len(), 72);
            // hit: zero production over the installed PE.
            let t = std::time::Instant::now();
            let hit = session.apply_source(&source).expect("hit");
            hit_t.push(t.elapsed().as_millis());
            assert_eq!(hit.exit_code, Some(137));
            assert!(hit.work.codegen_units.is_empty());
            // body-edit: exactly main's FIRST lambda body re-codegens.
            let t = std::time::Instant::now();
            let edit = session.apply_source(&edited_base).expect("edit");
            edit_t.push(t.elapsed().as_millis());
            assert_eq!(edit.exit_code, Some(138), "sample {id}");
            assert_eq!(
                edit.work.codegen_units,
                vec!["d___closure_main_0".to_string()]
            );
            // stable editing NEVER spawns rustc/cargo: the warm applies
            // carry no runtime_cargo_invocations counter at all.
            assert!(!edit
                .work
                .obs
                .counters
                .contains_key("runtime_cargo_invocations"));
            session.close().expect("close");

            // restart: a NEW process identity restores the published
            // manifest into a warm session (P0 first run). The publish is
            // SETUP — the restart state measures the restore chain only
            // (bounded read -> deep verify -> admit -> link -> run).
            let mut publisher =
                ace_native::driver::MnDriver::with_store(&root.join("store"), "mn-closures-perf")
                    .unwrap();
            let build = publisher
                .build_closures(&source, "main", &root.join("pub"), None)
                .expect("publish");
            let t = std::time::Instant::now();
            let mut restorer = ace_native::driver::MnDriver::with_store(
                &root.join("store"),
                "mn-closures-perf-restore",
            )
            .unwrap();
            let (mut restored, run) = restorer
                .restore_closures_session(
                    build.manifest_artifact_id.as_deref().unwrap(),
                    &root.join("rst"),
                    None,
                )
                .expect("restore session");
            restart_t.push(t.elapsed().as_millis());
            assert_eq!(run.exit_code, Some(137));
            assert!(run.work.codegen_units.is_empty(), "restart is P0");
            let next = restored
                .apply_source(&edited_base)
                .expect("post-restart edit");
            assert_eq!(next.exit_code, Some(138));
            restored.close().expect("close restored");
            let _ = std::fs::remove_dir_all(&root);
        }

        // The frozen gates: warm P90 ≤ 2000ms on every state.
        for (name, times) in [
            ("fresh", &fresh_t),
            ("hit", &hit_t),
            ("restart", &restart_t),
            ("body-edit", &edit_t),
        ] {
            let p90 = percentile(times.clone(), 90.0);
            assert!(
                p90 <= P90_CAP_MS,
                "{name} P90 {p90}ms exceeds the {P90_CAP_MS}ms gate (max {:?})",
                times.iter().max()
            );
        }

        // The 512-DISTINCT-revision churn: every revision is a different
        // source (a unique constant rides the closure body); every apply
        // must stay exactly-one-unit production with the modeled value.
        let root = isolated("churn");
        let (mut session, first) = ace_native::closures_incremental::ClosuresNativeSession::new(
            &source,
            "main",
            &root.join("out"),
            None,
        )
        .expect("churn session");
        assert_eq!(first.exit_code, Some(137));
        let mut current = source.clone();
        for k in 1..=512u32 {
            let next = if k == 1 {
                edited_base.clone()
            } else {
                source.replace("return base + n }", &format!("return base + n + {k} }}"))
            };
            let run = session.apply_source(&next).expect("churn edit");
            assert_eq!(
                run.exit_code,
                Some(137 + k as i32),
                "revision {k}: {}",
                String::from_utf8_lossy(&run.stderr)
            );
            assert_eq!(
                run.work.codegen_units,
                vec!["d___closure_main_0".to_string()],
                "revision {k} route: {}",
                run.work.route
            );
            current = next;
        }
        // The hit after the churn is still zero production.
        let hit = session.apply_source(&current).expect("churn hit");
        assert_eq!(hit.exit_code, Some(137 + 512));
        assert!(hit.work.codegen_units.is_empty());
        session.close().expect("close churn");
        let _ = std::fs::remove_dir_all(&root);

        // The budget/cancel refusal faces through the closures recipe: a
        // zero budget refuses BEFORE any mutation (trap 22, clean); a
        // cancellation after real CLONE work traps CANCELLED (26, clean) —
        // K05's explicit deep clone is the polling workload.
        let k05 = {
            let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
                .join("../../../../crates/auto-lang/test/cases/ace-m1/src/K05.at");
            std::fs::read_to_string(&path).expect("K05 corpus source")
        };
        for (tag, source, knobs, code) in [
            (
                "budget",
                &source,
                WrapperKnobs {
                    budget_bytes: Some(0),
                    ..Default::default()
                },
                22,
            ),
            (
                "cancel",
                &k05,
                WrapperKnobs {
                    cancel_after_work: Some(2),
                    ..Default::default()
                },
                26,
            ),
        ] {
            let root = isolated(tag);
            let mut driver = ace_native::driver::MnDriver::ephemeral();
            let res =
                driver.request_closures_resource(source, "main", &root.join("out"), None, knobs);
            let text = String::from_utf8_lossy(&res.stderr);
            assert_eq!(res.exit_code, Some(70), "{tag}: {}", text);
            assert!(
                text.contains(&format!("MN-COLL-TRAP code={code}\n")),
                "{tag}: {text}"
            );
            for field in [
                "MN-COLL live=0",
                "MN-COLL bytes=0",
                "MN-COLL invalid=0",
                "MN-COLL arena_live=0",
            ] {
                assert!(
                    text.contains(&format!("{field}\n")),
                    "{tag} missing {field}: {text}"
                );
            }
            let _ = std::fs::remove_dir_all(&root);
        }
    }
}
