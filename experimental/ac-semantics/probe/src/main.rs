//! `sema-probe` — cross-process probe entry (PLAN-796 T-01+).
//!
//! Subprocess surface used by the integration tests and the turnaround
//! harness. `put` performs a REAL inference and publishes; `get` looks
//! up WITHOUT any compute path and prints the hit/stale/miss outcome as
//! JSON lines. Process exit between the two is the cross-process
//! persistence proof (store write → process exit → restore).

use ac_artifact_probe::store::ManagedStore;
use ac_sema_core::query::QueryKind;
use ac_sema_probe::{open_store, publish_infer_result, run_inference, FetchOutcome};
use std::io::Write;
use std::path::PathBuf;
use std::process::exit;

fn usage() -> ! {
    eprintln!(
        "usage:\n  sema-probe put <store-root> <source.at> <fn-name> [project]\n  sema-probe get <store-root> <source.at> <fn-name>
  sema-probe bench-turnaround <out-dir> [store-root]"
    );
    exit(2);
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 2 {
        usage();
    }
    if args[1] == "bench-turnaround" {
        return bench_turnaround(&args);
    }
    if args.len() == 2 && args[1] == "serve" {
        // semaworker/1 bounded single-shot session (T-06)
        std::process::exit(ac_sema_probe::ipc::serve());
    }
    if args.len() < 5 {
        usage();
    }
    let cmd = args[1].as_str();
    let store_root = PathBuf::from(&args[2]);
    let src_path = PathBuf::from(&args[3]);
    let fn_name = args[4].clone();

    let src = std::fs::read_to_string(&src_path).unwrap_or_else(|e| {
        eprintln!("fatal: read {}: {e}", src_path.display());
        exit(3);
    });
    let input = ac_sema_core::adapter::parse_into_input(
        &src,
        "probe",
        src_path.file_stem().unwrap().to_string_lossy().as_ref(),
        auto_ac_prototype::source::SourceProfile::Core,
    )
    .unwrap_or_else(|e| {
        eprintln!("fatal: {e}");
        exit(4);
    });
    let store: ManagedStore = open_store(&store_root).unwrap_or_else(|e| {
        eprintln!("fatal: open store: {e}");
        exit(5);
    });
    let query = QueryKind::InferBody { fn_name: fn_name.clone() };

    match cmd {
        "put" => {
            let project = args.get(5).cloned();
            let outcome = run_inference(&input, &fn_name);
            let ac_sema_core::InferOutcome::Ok(summary) = outcome else {
                eprintln!("fatal: inference did not succeed cleanly");
                exit(6);
            };
            let art = publish_infer_result(
                &store,
                &input,
                &query,
                &summary,
                project.as_deref(),
            )
            .unwrap_or_else(|e| {
                eprintln!("fatal: publish: {e}");
                exit(7);
            });
            let mut out = std::io::stdout().lock();
            writeln!(out, "{{\"event\":\"published\",\"artifact\":\"{art}\"}}").unwrap();
            writeln!(
                out,
                "{{\"event\":\"bindings\",\"items\":[{}]}}",
                summary
                    .facts
                    .bindings
                    .iter()
                    .map(|(n, t)| format!("{{\"name\":\"{n}\",\"ty\":\"{}\"}}", desc_name(t)))
                    .collect::<Vec<_>>()
                    .join(",")
            )
            .unwrap();
        }
        "get" => {
            // NO compute path exists in this branch — structurally.
            match ac_sema_probe::lookup_only(&store, &input, &query) {
                Ok(FetchOutcome::Hit { page, artifact_id }) => {
                    let mut out = std::io::stdout().lock();
                    writeln!(out, "{{\"event\":\"hit\",\"compute_called\":false,\"artifact\":\"{artifact_id}\"}}").unwrap();
                    if let ac_sema_contract::PageBody::CachedResult(c) = &page.body {
                        if let ac_sema_contract::ResultFacts::Infer(f) = &c.facts {
                            writeln!(
                                out,
                                "{{\"event\":\"bindings\",\"items\":[{}]}}",
                                f.bindings
                                    .iter()
                                    .map(|(n, t)| format!(
                                        "{{\"name\":\"{n}\",\"ty\":\"{}\"}}",
                                        desc_name(t)
                                    ))
                                    .collect::<Vec<_>>()
                                    .join(",")
                            )
                            .unwrap();
                        }
                    }
                }
                Ok(FetchOutcome::Stale { reason }) => {
                    println!("{{\"event\":\"stale\",\"reason\":\"{reason}\"}}");
                }
                Ok(FetchOutcome::Miss) => {
                    println!("{{\"event\":\"miss\"}}");
                }
                Err(e) => {
                    eprintln!("fatal: lookup: {e}");
                    exit(8);
                }
            }
        }
        _ => usage(),
    }
}

fn desc_name(t: &ac_sema_contract::TypeDesc) -> String {
    match t {
        ac_sema_contract::TypeDesc::I32 => "i32".into(),
        ac_sema_contract::TypeDesc::Bool => "bool".into(),
        ac_sema_contract::TypeDesc::Opaque { name } => format!("opaque:{name}"),
        other => format!("{other:?}"),
    }
}

/// T-05: run the six measured scenarios and write the raw-samples
/// report (machine info + stats + per-sample traces) to <out-dir>.
fn bench_turnaround(args: &[String]) {
    let out_dir = std::path::PathBuf::from(&args[2]);
    let store_root = args
        .get(3)
        .map(std::path::PathBuf::from)
        .unwrap_or_else(|| std::env::temp_dir().join("ac796-turnaround-bench"));
    let _ = std::fs::remove_dir_all(&store_root);
    let harness = ac_sema_probe::turnaround::Harness::new(store_root.clone());
    let store = ac_sema_probe::open_store(&store_root).expect("open store");
    let t_prep = std::time::Instant::now();

    let mut all: Vec<(&'static str, Vec<ac_sema_probe::turnaround::Sample>)> = Vec::new();
    all.push(("no-op", harness.scenario_no_op(&store)));
    all.push(("local-body-edit", harness.scenario_local_body_edit(&store)));
    all.push(("sig-change", harness.scenario_sig_change(&store)));
    all.push(("restart-disk-hot", harness.scenario_restart_disk_hot()));
    all.push(("cross-project", harness.scenario_cross_project(&store)));
    all.push(("gc-concurrent", harness.scenario_gc_concurrent(&store)));
    all.push(("full-cold", harness.scenario_full_cold()));
    all.push(("wide-invalidation", harness.scenario_wide_invalidation(&store)));

    let machine = ac_sema_probe::turnaround::machine_info();
    let path = ac_sema_probe::turnaround::write_report(&out_dir, &machine, &all)
        .expect("write report");
    println!(
        "{{\"event\":\"bench-done\",\"report\":\"{}\",\"prep_total_ms\":{}}}",
        path.display(),
        t_prep.elapsed().as_millis()
    );
}
