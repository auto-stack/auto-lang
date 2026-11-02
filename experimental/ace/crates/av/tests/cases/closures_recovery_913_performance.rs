//! PLAN-913 V-14 formal performance leg: source-bound recovery across the
//! four states (fresh / hit / restart / restore-then-edit) at the formal
//! multi-function scale — 68 top-level functions / 70 units, ≥3 sibling
//! lambdas in two owners, entry-reachable — with P90 wall gates, the
//! caller-visible fresh-process cost, the restore's zero-production and
//! deep-verification axes, the post-restore warm edit's identity counters and
//! the required-data byte budget.
//!
//! Runs under the `ace-performance` fixed group (`test(/::performance::/)`).

pub mod performance {
    use ace_av::closures_fn_session::{ClosuresFunctionSession, RouteReport};
    use std::path::{Path, PathBuf};

    const OWNER: &str = "scale.at";

    /// The formal scale source: 62 plain helpers + `sum` (3 lambdas) +
    /// `main` (3 lambdas) = 68 top-level functions, 70 units.
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

    /// The independent behavioral model (hand-computed semantics).
    fn scale_value(x: i32, f0_extra: i32, f1_mul: i32, f2_mul: i32) -> i32 {
        let cell = x + 20 + f0_extra;
        let f1 = cell - 3 * f1_mul;
        let f2 = 5 * f2_mul;
        let y = 4 + 2;
        let s = y + (y - 1) + 12 + (1 + 0);
        cell + f1 + f2 + s + (1 + 1) + (1 + 61)
    }

    fn isolated_store(tag: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!(
            "plan913-perf-{tag}-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_nanos()
        ));
        std::fs::create_dir_all(&root).expect("create perf store");
        root
    }

    fn write_source(dir: &Path, name: &str, src: &str) -> PathBuf {
        let path = dir.join(name);
        std::fs::write(&path, src).expect("write source");
        path
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

    fn run_probe(args: &[&str]) -> serde_json::Value {
        let exe = env!("CARGO_BIN_EXE_av_closures_913_restore");
        let out = std::process::Command::new(exe)
            .args(args)
            .output()
            .expect("spawn recovery probe");
        let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let json: serde_json::Value = serde_json::from_str(
            stdout
                .lines()
                .last()
                .unwrap_or("{\"ok\":false,\"code\":\"no-output\"}"),
        )
        .unwrap_or_else(|e| panic!("probe output not JSON ({e}): {stdout}"));
        assert!(out.status.success(), "probe failed: {stdout}");
        json
    }

    #[test]
    fn closures_recovery_formal_states_scale() {
        const SAMPLES: usize = 30;
        const POOL: usize = 16 * 1024 * 1024;
        const UNITS: u64 = 70;
        let mut fresh_t = Vec::new();
        let mut hit_t = Vec::new();
        let mut restart_t = Vec::new();
        let mut edit_t = Vec::new();
        let mut manifest_bytes_max = 0u64;
        let mut wire_bytes_max = 0u64;
        // The restore's own execution leg (decode + deep verify + context
        // rebuild + install + execute), reported separately from the
        // caller-visible fresh-process cost (spawn + process setup).
        let mut restart_exec_ms: Vec<u128> = Vec::new();
        let base = scale_value(10, 0, 1, 2);
        let edited = scale_value(10, 0, 2, 2);
        // Hand arithmetic for the base: cell=30, f1=27, f2=10, sum=24,
        // helpers=2+62=64 → 155; the middle-lambda edit (f1: x-n → x-n*2)
        // makes f1 = 30 - 6 = 24 → 152.
        assert_eq!((base, edited), (155, 152));

        for id in 0..SAMPLES {
            let source = scale_source();
            let edited_source = source.replace("return x - n }", "return x - n * 2 }");
            let root = isolated_store("states");
            let src_path = write_source(&root, "scale.at", &source);
            let edited_path = write_source(&root, "scale_edited.at", &edited_source);

            // ── fresh: real cold production + publication (preparation) ──
            let mut session = ClosuresFunctionSession::new(&root, OWNER);
            let t = std::time::Instant::now();
            let fresh = session.apply_source(&source).expect("fresh");
            fresh_t.push(ms(t.elapsed()));
            assert_eq!(fresh.value, base, "sample {id}");
            assert!(
                matches!(fresh.route, RouteReport::Cold { units: 70 }),
                "sample {id}: {:?}",
                fresh.route
            );
            assert!(fresh.retained_bytes <= POOL);
            let ticket = session
                .publish_source_recovery("ac913-perf")
                .expect("publish");
            assert_eq!(ticket.units, UNITS as usize);
            assert!(ticket.manifest_bytes > 0 && ticket.manifest_bytes <= 4 * 1024 * 1024);

            // ── hit: identical re-apply (zero production) ──
            let t = std::time::Instant::now();
            let hit = session.apply_source(&source).expect("hit");
            hit_t.push(ms(t.elapsed()));
            assert_eq!(hit.route, RouteReport::ZeroHit);
            assert_eq!(hit.submitted_units, 0);
            assert!(hit.warm_gate.as_ref().unwrap().pass(), "sample {id} hit gate");
            assert!(hit.retained_bytes <= POOL);
            session.close().expect("close");

            // ── restart: a FRESH PROCESS restores from the caller binding ──
            let store_arg = root.to_string_lossy().to_string();
            let src_arg = src_path.to_string_lossy().to_string();
            let t = std::time::Instant::now();
            let restored = run_probe(&[
                &store_arg,
                &ticket.manifest_artifact,
                &ticket.manifest_digest,
                &src_arg,
                OWNER,
                "d_main",
                ace_core::source::CLOSURES_SOURCE_PROFILE,
            ]);
            restart_t.push(ms(t.elapsed()));
            restart_exec_ms.push(
                restored["restore_wall_ms"]
                    .as_f64()
                    .expect("restore_wall_ms") as u128,
            );
            assert_eq!(restored["ok"], true);
            assert_eq!(restored["restore_value"], base, "sample {id} restart value");
            assert_eq!(restored["p_axis_attempts"], 0, "sample {id} restart P axis");
            assert_eq!(restored["r_unit_reads"], UNITS);
            assert_eq!(restored["r_deep_verifies"], UNITS);
            assert_eq!(restored["r_admits"], UNITS);
            assert_eq!(restored["r_context_reads"], 1);
            manifest_bytes_max = manifest_bytes_max.max(
                restored["manifest_bytes"].as_u64().unwrap_or(0),
            );
            wire_bytes_max =
                wire_bytes_max.max(restored["wire_bytes"].as_u64().unwrap_or(0));

            // ── restore-then-edit: the SAME fresh process takes one edit ──
            let edited_arg = edited_path.to_string_lossy().to_string();
            let t = std::time::Instant::now();
            let edited_run = run_probe(&[
                &store_arg,
                &ticket.manifest_artifact,
                &ticket.manifest_digest,
                &src_arg,
                OWNER,
                "d_main",
                ace_core::source::CLOSURES_SOURCE_PROFILE,
                "",
                &edited_arg,
            ]);
            edit_t.push(ms(t.elapsed()));
            let edit = &edited_run["edit"];
            assert_eq!(edited_run["base"]["p_axis_attempts"], 0);
            assert_eq!(edit["value"], edited, "sample {id} edited value");
            assert_eq!(edit["route"], "fast-unit", "sample {id}: {edit:?}");
            assert_eq!(edit["submitted"], 1);
            assert_eq!(edit["whole_total"], 0);
            assert_eq!(edit["warm_gate_pass"], true, "sample {id} edit gate");
            assert_eq!(edit["unchanged_total"], UNITS - 1);
            assert_eq!(edit["unchanged_same_object"], UNITS - 1);
            assert_eq!(edit["unchanged_production_attempts"], 0);
            assert_eq!(edit["store_writes"], 0);
            assert_eq!(edit["live_after_release"], 0);
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
            "PLAN-913 V-14 formal (68-fn/70-unit scale, 30 samples x 4 states):\n  {}\n  {}\n  {}\n  {}\n  restart execution leg (no spawn): {}\n  required data: manifest_bytes<={manifest_bytes_max}, wire_bytes<={wire_bytes_max}",
            report("fresh", &fresh_t),
            report("hit", &hit_t),
            report("restart (fresh process, caller-visible)", &restart_t),
            report("restore+edit (fresh process)", &edit_t),
            report("restart", &restart_exec_ms),
        );
        for (name, v) in [
            ("fresh", &fresh_t),
            ("hit", &hit_t),
            ("restart", &restart_t),
            ("restore+edit", &edit_t),
        ] {
            let p90 = percentile(v.clone(), 90.0);
            assert!(p90 <= 2000, "{name} P90 {p90}ms above the 2000ms gate");
        }
        // Resource axes at the formal scale: the required closure stays within
        // the per-manifest budget and the wire set stays bounded.
        assert!(manifest_bytes_max <= 4 * 1024 * 1024);
        assert!(wire_bytes_max <= 8 * 1024 * 1024);
    }
}
