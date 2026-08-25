//! Plan 442 Phase C probes (auto-musk backend corpus, VM runtime).
//!
//! The musk backend (`backend/crates/musk/auto-src/*.at`) is Auto-sourced and
//! today reaches production via a2r transpilation. Phase C asks whether the
//! SAME sources can run directly on the AutoVM. Probes:
//!
//! 1. `musk_backend_app_config_vm_run` — the C1 vertical: VM-direct run of
//!    the purest module (`app_config.at`). Green since the env.var bridge +
//!    `.ok()` passthrough landed (the C1 ledger's gap ①).
//! 2. `musk_backend_gap_enumerator` — the C2 worklist generator: drives EVERY
//!    auto-src module through `run_file`, records each module's first
//!    blocker (link symbol / runtime error), and prints the mechanical
//!    shim-coverage checklist. Modules whose data layer is green may still
//!    fail when driven via dependents — the enumerator drives leaves first,
//!    so a PASS means "importable + module-init clean", the honest unit for
//!    shim-gap counting.
//!
//! `#[ignore]` — sibling-checkout assumption, manual-only:
//!   cargo test -p auto-lang --lib musk_backend -- --ignored --nocapture

#[cfg(test)]
mod plan442_musk_backend_probe {
    fn locate_auto_src() -> Option<std::path::PathBuf> {
        let rel = "../../../auto-musk/backend/crates/musk/auto-src";
        let candidates = [
            std::env::var("CARGO_MANIFEST_DIR")
                .ok()
                .map(|d| std::path::PathBuf::from(d).join(rel)),
            Some(std::path::PathBuf::from(rel)),
        ];
        candidates.into_iter().flatten().find(|p| p.is_dir())
    }

    /// Crate roots the corpus `use.rust`s that are NOT in the always-available
    /// set (BUILTIN_OPAQUE_CRATES + std) — the driver declares them as `dep`
    /// lines so the per-module use.rust gate passes, mirroring what the a2r
    /// build config provides (nativeize pipeline). Discovered by scanning the
    /// corpus on each run, so new use.rust targets are picked up for free.
    fn corpus_dep_lines(src_dir: &std::path::Path) -> Vec<String> {
        use std::collections::BTreeSet;
        let builtin: BTreeSet<&str> = [
            "regex", "url", "semver", "log", "env_logger", "tracing", "rand",
            "rand_distr", "chrono", "csv", "walkdir", "toml", "serde_json",
            "percent_encoding", "urlencoding", "base64", "hex", "sha2",
            "mime_guess", "same_file", "heapless", "clap", "ansi_term",
            "simplelog", "tar", "flate2", "crossbeam", "anyhow", "serde",
            "tokio", "num", "ndarray", "std", "core", "alloc", "proc_macro",
        ]
        .into_iter()
        .collect();
        let mut roots: BTreeSet<String> = BTreeSet::new();
        if let Ok(entries) = std::fs::read_dir(src_dir) {
            for e in entries.flatten() {
                let p = e.path();
                if p.extension().map_or(true, |x| x != "at") {
                    continue;
                }
                if let Ok(code) = std::fs::read_to_string(&p) {
                    for line in code.lines() {
                        let t = line.trim();
                        if let Some(rest) = t.strip_prefix("use.rust ") {
                            let root = rest
                                .split("::")
                                .next()
                                .unwrap_or(rest)
                                .trim_start_matches('{')
                                .to_string();
                            if !root.is_empty() && !builtin.contains(root.as_str()) {
                                roots.insert(root);
                            }
                        }
                    }
                }
            }
        }
        roots.into_iter().map(|r| format!("dep {}", r)).collect()
    }

    fn run_driver(src_dir: &std::path::Path, module: &str, deps: &[String]) -> Result<(), String> {
        let driver = src_dir.join(format!("__plan442_driver_{}.at", module));
        let mut src = String::from("// Plan 442 C probe driver (generated; delete freely)\n");
        for d in deps {
            src.push_str(d);
            src.push('\n');
        }
        src.push_str(&format!(
            "use extern_sigs\nuse {}\n\n\
             fn main() {{\n    print(\"plan442-c-ok\")\n}}\n",
            module
        ));
        std::fs::write(&driver, src).map_err(|e| format!("write driver: {e}"))?;
        let out = std::panic::catch_unwind(|| {
            crate::run_file(driver.to_string_lossy().as_ref()).map(|_| String::new())
        });
        let _ = std::fs::remove_file(&driver);
        match out {
            Ok(Ok(_)) => Ok(()),
            Ok(Err(e)) => Err(e.to_string()),
            Err(_) => Err("panicked".to_string()),
        }
    }

    /// C1 smoke: VM-direct run of a driver importing app_config.at.
    #[test]
    #[ignore = "requires sibling auto-musk checkout; manual Phase-C gate"]
    fn musk_backend_app_config_vm_run() {
        let Some(src_dir) = locate_auto_src() else {
            eprintln!("plan442 musk backend probe: SKIPPED — auto-musk not found");
            return;
        };
        let deps = corpus_dep_lines(&src_dir);
        match run_driver(&src_dir, "app_config", &deps) {
            Ok(()) => eprintln!("plan442-c1: VM run of app_config driver returned Ok"),
            Err(e) => panic!("VM run failed (C gap): {e}"),
        }
    }

    /// C2 worklist generator: per-module first-blocker enumeration over the
    /// whole auto-src corpus. PASS = importable + init-clean on the VM.
    #[test]
    #[ignore = "requires sibling auto-musk checkout; manual Phase-C gate"]
    fn musk_backend_gap_enumerator() {
        let Some(src_dir) = locate_auto_src() else {
            eprintln!("plan442 musk backend probe: SKIPPED — auto-musk not found");
            return;
        };
        // Silence the per-module print noise; the report is what matters.
        let mut entries: Vec<std::path::PathBuf> = std::fs::read_dir(&src_dir)
            .expect("read auto-src")
            .flatten()
            .map(|e| e.path())
            .filter(|p| {
                p.extension().map_or(false, |e| e == "at")
                    && !p
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map_or(false, |s| s.starts_with("__plan442"))
            })
            .collect();
        entries.sort();
        let deps = corpus_dep_lines(&src_dir);
        let mut pass = 0usize;
        let mut blocked: Vec<(String, String)> = Vec::new();
        for path in &entries {
            let module = path.file_stem().and_then(|s| s.to_str()).unwrap_or("?");
            if module == "extern_sigs" {
                continue; // sidecar — imported by the drivers, not driven
            }
            match run_driver(&src_dir, module, &deps) {
                Ok(()) => pass += 1,
                Err(e) => blocked.push((module.to_string(), e)),
            }
        }
        eprintln!("════ plan442 C2 worklist: {}/{} modules VM-clean ════", pass, entries.len());
        for (m, e) in &blocked {
            // Show up to 3 error lines — "aborting due to N" hides the real
            // compile errors listed before it.
            let mut lines: Vec<&str> = e
                .lines()
                .filter(|l| l.trim_start_matches(' ').starts_with("error["))
                .take(3)
                .map(|l| l.trim_start())
                .collect();
            if lines.is_empty() {
                // Single-error form (no MultipleErrors wrapper) — first line.
                lines = vec![e.lines().next().unwrap_or("?")];
            }

            eprintln!("  BLOCKED {:<24} {}", m, lines.join(" | "));
        }
        // The enumerator is a report, not a gate — any state is a pass.
    }
}
