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

    /// relay_store deep-dive: line-prefix bisection over the REAL file.
    /// Hand-crafted minimal repros don't trigger its cascade, so scan the
    /// real module at every brace-balanced cut point (top-level item
    /// boundaries) and report the item whose addition flips in the first
    /// parse error ("Expected end of statement, got Ident<编译期拦截>").
    #[test]
    #[ignore = "requires sibling auto-musk checkout; manual Phase-C gate"]
    fn musk_backend_relay_store_bisect() {
        let Some(src_dir) = locate_auto_src() else {
            eprintln!("plan442 musk backend probe: SKIPPED — auto-musk not found");
            return;
        };
        let code = std::fs::read_to_string(src_dir.join("relay_store.at")).expect("read");
        let lines: Vec<&str> = code.lines().collect();
        let mut depth = 0i32;
        let mut cuts: Vec<usize> = Vec::new();
        for (i, l) in lines.iter().enumerate() {
            for c in l.chars() {
                match c {
                    '{' => depth += 1,
                    '}' => depth -= 1,
                    _ => {}
                }
            }
            if depth == 0 {
                cuts.push(i + 1);
            }
        }
        let deps = corpus_dep_lines(&src_dir);
        let bis_path = src_dir.join("__plan442_bisect.at");
        let probe = |n: usize| -> Option<String> {
            std::fs::write(&bis_path, lines[..n].join("\n")).expect("write bisect");
            let r = run_driver(&src_dir, "__plan442_bisect", &deps);
            let _ = std::fs::remove_file(&bis_path);
            r.err()
        };
        let mut prev_err: Option<String> = None;
        for (idx, &n) in cuts.iter().enumerate() {
            let err = probe(n);
            let flipped = match (&err, &prev_err) {
                (Some(e), None) => Some(e.clone()),
                (Some(e), Some(p)) if !p.contains("编译期拦截") && e.contains("编译期拦截") => {
                    Some(e.clone())
                }
                _ => None,
            };
            if let Some(e) = flipped {
                let last = cuts[idx.saturating_sub(1)];
                eprintln!("═══ FLIP at cut {n} (prev cut {last}) — error: {e} ═══");
                eprintln!("── added lines [{}..{}] ──", last, n - 1);
                for l in &lines[last..n] {
                    eprintln!("  | {l}");
                }
            }
            prev_err = err;
        }
        eprintln!("═══ full-file error: {prev_err:?} ═══");

        // Greedy item-level shrink: drop top-level items (cut-to-cut spans,
        // from the front, never the last item) while the target error stays.
        let target = |e: &Option<String>| matches!(e, Some(s) if s.contains("编译期拦截"));
        let run_items = |keep: &[usize]| -> Option<String> {
            let mut text = String::new();
            for &i in keep {
                let end = cuts.get(i + 1).copied().unwrap_or(lines.len());
                text.push_str(&lines[cuts[i]..end].join("\n"));
                text.push('\n');
            }
            std::fs::write(&bis_path, &text).expect("write bisect");
            let r = run_driver(&src_dir, "__plan442_bisect", &deps);
            let _ = std::fs::remove_file(&bis_path);
            r.err()
        };
        let mut keep: Vec<usize> = (0..cuts.len()).collect();
        let mut i = 0;
        while i < keep.len().saturating_sub(1) {
            let mut trial = keep.clone();
            trial.remove(i);
            if target(&run_items(&trial)) {
                keep = trial;
            } else {
                i += 1;
            }
        }
        eprintln!(
            "═══ minimal item set ({}/{} items) — error: {:?} ═══",
            keep.len(),
            cuts.len(),
            run_items(&keep)
        );
        let mut text = String::new();
        for &i in &keep {
            let end = cuts.get(i + 1).copied().unwrap_or(lines.len());
            for l in &lines[cuts[i]..end] {
                eprintln!("  | {l}");
            }
            text.push_str(&lines[cuts[i]..end].join("\n"));
            text.push('\n');
        }
        eprintln!("══════ end minimal repro ══════");
    }

    /// C2 serve-adapter vertical: run the REAL relay_api.at `relay_routes()`
    /// on the VM — axum Router.new + app.route(path, get(h).post(h2)) chains —
    /// then drive a live HTTP request through the auto-started server.
    /// Asserts the full adapter pipe: route install (methods/paths/extractor
    /// shapes resolved from fn-ref closures) + extractor marshalling +
    /// call_closure dispatch.
    ///
    /// Shares process-global adapter/HTTP state with the other probes — run
    /// the probe batch serially: `-- --ignored --nocapture --test-threads=1`
    /// (the gap enumerator's pipeline resets would race this server thread).
    #[test]
    #[ignore = "requires sibling auto-musk checkout; manual Phase-C gate"]
    fn musk_backend_server_router_run() {
        use crate::vm::ffi::axum_adapter::{self, ExtractorKind};
        use std::io::{Read, Write};
        use std::net::TcpStream;

        let Some(src_dir) = locate_auto_src() else {
            eprintln!("plan442 musk backend probe: SKIPPED — auto-musk not found");
            return;
        };
        let deps = corpus_dep_lines(&src_dir);
        let driver = src_dir.join("__plan442_driver_router.at");
        let mut src = String::from("// Plan 442 C2 serve-adapter probe (generated; delete freely)\n");
        for d in &deps {
            src.push_str(d);
            src.push('\n');
        }
        src.push_str("use extern_sigs\nuse relay_api\n\n\
             fn main() {\n    let app = relay_routes()\n    print(\"router-built\")\n}\n");
        std::fs::write(&driver, src).expect("write driver");

        const PORT: u16 = 18442;
        std::env::set_var("AUTO_HTTP_PORT", PORT.to_string());
        crate::vm::ffi::stdlib::clear_http_routes();
        let driver_path = driver.to_string_lossy().to_string();
        let _server = std::thread::Builder::new()
            .stack_size(8 * 1024 * 1024)
            .spawn(move || {
                match crate::run_file(&driver_path) {
                    Ok(_) => eprintln!("plan442-c2-serve: run_file returned Ok"),
                    Err(e) => eprintln!("plan442-c2-serve: run_file error: {e}"),
                }
            })
            .expect("spawn server thread");

        // Phase 1: wait for route registration (happens during main). The
        // full-corpus driver compile can be slow — poll up to 120s with a
        // heartbeat so a stuck compile is distinguishable from a fast error.
        let mut routes = Vec::new();
        for i in 0..1200 {
            routes = axum_adapter::installed_routes();
            if !routes.is_empty() {
                break;
            }
            if i % 10 == 0 {
                eprintln!(
                    "plan442-c2-serve: waiting for routes... ({}s, stdlib table: {})",
                    i / 10,
                    crate::vm::ffi::stdlib::get_http_routes().len()
                );
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let _ = std::fs::remove_file(&driver);

        // Phase 2: assert the registered route table (real relay_routes
        // registers 15 handlers over 9 paths; spot-check the shapes).
        let find = |method: &str, path: &str| {
            routes
                .iter()
                .find(|r| r.method == method && r.path == path)
                .cloned()
        };
        let get_runs = find("GET", "/api/forge/relay/runs")
            .expect("GET /api/forge/relay/runs registered");
        assert_eq!(
            get_runs.params,
            vec![ExtractorKind::State, ExtractorKind::Query],
            "list_runs(s State, q Query) extractor shapes"
        );
        let post_runs = find("POST", "/api/forge/relay/runs")
            .expect("POST /api/forge/relay/runs registered (chained .post)");
        assert_eq!(
            post_runs.params,
            vec![ExtractorKind::State, ExtractorKind::Query, ExtractorKind::Json],
            "start_run(s State, q Query, body Json) extractor shapes"
        );
        assert!(
            find("DELETE", "/api/forge/relay/runs/:run_id").is_some(),
            "DELETE /api/forge/relay/runs/:run_id registered ({{run_id}} template conversion)"
        );
        eprintln!(
            "plan442-c2-serve: {} route(s) installed; first three:",
            routes.len()
        );
        for r in routes.iter().take(3) {
            eprintln!("  {} {} (closure #{}, {:?})", r.method, r.path, r.closure_id, r.params);
        }

        // Phase 3: wait for the listener, then GET through the whole dispatch
        // pipe (extractor marshalling + call_closure). Handler bodies call
        // extern no-ops, so a 200 with any body proves the closure ran.
        let mut stream = None;
        for _ in 0..100 {
            if let Ok(s) = TcpStream::connect(("127.0.0.1", PORT)) {
                stream = Some(s);
                break;
            }
            std::thread::sleep(std::time::Duration::from_millis(100));
        }
        let mut stream = stream.expect("connect to axum-adapter test server");
        stream
            .set_read_timeout(Some(std::time::Duration::from_secs(10)))
            .ok();
        write!(
            stream,
            "GET /api/forge/relay/runs HTTP/1.1\r\nHost: 127.0.0.1\r\nConnection: close\r\n\r\n"
        )
        .unwrap();
        let mut resp = String::new();
        stream.read_to_string(&mut resp).ok();
        eprintln!("plan442-c2-serve: GET /api/forge/relay/runs → {}", resp.lines().next().unwrap_or(""));
        assert!(
            resp.starts_with("HTTP/1.1 200"),
            "axum-route dispatch must answer 200, got: {}",
            resp.lines().next().unwrap_or("<empty>")
        );
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

    /// C2 minimal repro: `get(h1).post(h2)` chain on bare rust-imported
    /// routing fns — no server, no routes, isolates dispatch shaping.
    #[test]
    fn plan442_axum_get_post_chain_minimal() {
        let code = "dep axum\nuse.rust axum::Router\nuse.rust axum::routing::{get, post}\n\n\
             fn h1() int { return 1 }\n\
             fn h2() int { return 2 }\n\n\
             fn main() {\n    var mr = get(h1).post(h2)\n    print(\"chain-ok\")\n}\n";
        match crate::run_with_capture_and_bytecode(code) {
            Ok((_, _, lines)) => {
                for l in &lines {
                    let s = l.to_string();
                    if s.contains("CALL_NAT") || s.contains("CALL_SPEC") || s.contains("CLOSURE")
                        || s.contains("LOAD_STR") || s.contains("FN_") {
                        eprintln!("DISASM | {s}");
                    }
                }
            }
            Err(e) => panic!("get(h).post(h) chain failed: {e}"),
        }
    }
}
