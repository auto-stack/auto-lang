mod compare;
mod report;
mod runner;
mod tap;

use clap::{Parser, Subcommand};
use compare::{ComparisonReport, TestCaseComparison};
use runner::{lib_has_mock_server, MockServer, RunConfig};
use std::path::PathBuf;

/// Three-way parity checker: AutoVM vs a2r vs native Rust.
#[derive(Parser)]
#[command(name = "auto-parity", version, about)]
struct Cli {
    /// Path to the parity workspace root (default: auto-detect ./parity).
    #[arg(long, env = "PARITY_ROOT")]
    root: Option<PathBuf>,

    /// Path to the auto binary (default: "auto").
    #[arg(long, env = "AUTO_BINARY", default_value = "auto")]
    auto_binary: String,

    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
    /// Run parity check for a specific library.
    Run {
        /// Library name (e.g. "base64", "_dummy").
        library: String,
    },
    /// Run parity check for all libraries in a phase.
    Phase {
        /// Phase name (p0, p1, p2, p3, p4).
        phase: String,
    },
    /// Run parity check for all libraries.
    All,
    /// List discovered libraries.
    List,
    /// Generate a static HTML dashboard of parity results.
    Report {
        /// Path to write the HTML dashboard to.
        #[arg(short, long, default_value = "docs/parity-dashboard.html")]
        output: PathBuf,
    },
}

fn main() {
    let cli = Cli::parse();
    let root = cli.root.unwrap_or_else(detect_parity_root);

    if !root.is_dir() {
        eprintln!("Error: parity root not found: {}", root.display());
        std::process::exit(1);
    }

    // Async libraries (P4) need sorted output because their completion order
    // is non-deterministic. We also default sort_results to false and turn it
    // on per-library inside run_library.
    let base_config = RunConfig {
        parity_root: root.clone(),
        auto_binary: cli.auto_binary.clone(),
        library: String::new(),
        sort_results: false,
    };

    match cli.command {
        Command::Run { library } => {
            run_library(&base_config.with_library(&library));
        }
        Command::Phase { phase } => {
            let libs = discover_libraries_by_phase(&root, &phase);
            if libs.is_empty() {
                eprintln!("No libraries found for phase {}", phase);
                std::process::exit(1);
            }
            for lib in libs {
                run_library(&base_config.with_library(&lib));
            }
        }
        Command::All => {
            let libs = discover_all_libraries(&root);
            if libs.is_empty() {
                eprintln!("No libraries found under {}", root.join("libs").display());
                std::process::exit(1);
            }
            for lib in libs {
                run_library(&base_config.with_library(&lib));
            }
        }
        Command::List => {
            let libs = discover_all_libraries(&root);
            for lib in libs {
                println!("{}", lib);
            }
        }
        Command::Report { output } => {
            // Verified phases included in the dashboard. P1/P2 are the Plan
            // 355 core; D1/D2 are Plan 358 additions; P4's tokio is L1 (reqwest
            // is auto-skipped if absent). P3 (sha2/rusqlite) stays L3 roadmap.
            // D5 is Plan 367 consumer-mode parity (Auto as library consumer).
            let phases = ["p1", "p2", "d1", "d2", "d4", "d5", "p4"];
            let mut reports = Vec::new();
            for phase in &phases {
                let libs = discover_libraries_by_phase(&root, phase);
                for lib in libs {
                    let mut cfg = base_config.clone();
                    cfg.library = lib;
                    cfg.sort_results = is_async_library(&cfg.library);
                    match build_comparison_report(&cfg) {
                        Ok(r) => reports.push(r),
                        Err(e) => eprintln!("Warning: {} report failed: {}", cfg.library, e),
                    }
                }
            }
            let kd = root.join("docs/known-divergences.md");
            match report::generate_dashboard(&reports, &kd, &output) {
                Ok(()) => {
                    println!("Dashboard written to {}", output.display());
                }
                Err(e) => {
                    eprintln!("report error: {}", e);
                    std::process::exit(1);
                }
            }
        }
    }
}

/// Walk current directory and its parents looking for a `parity/` directory.
/// Falls back to `"parity"` (relative) if nothing is found.
fn detect_parity_root() -> PathBuf {
    let mut dir = match std::env::current_dir() {
        Ok(d) => d,
        Err(_) => return PathBuf::from("parity"),
    };
    loop {
        let candidate = dir.join("parity");
        if candidate.is_dir() {
            return candidate;
        }
        if !dir.pop() {
            return PathBuf::from("parity");
        }
    }
}

/// Run a parity check for a single library: invoke all three backends, build
/// a per-test comparison, and print the report.
fn run_library(config: &RunConfig) {
    // Async libraries have non-deterministic completion order; sort TAP
    // output by test name before comparison.
    let mut config = config.clone();
    config.sort_results = is_async_library(&config.library);

    println!();
    println!("{}", "=".repeat(60));
    let mode = detect_parity_mode(&config);
    let mode_label = match mode {
        ParityMode::Python => " [Python parity]",
        ParityMode::Rust => "",
    };
    println!("Checking library: {}{}", config.library, mode_label);
    println!("{}", "=".repeat(60));

    match build_comparison_report(&config) {
        Ok(report) => {
            let text = report::format_report(&report);
            println!("{}", text);
        }
        Err(e) => {
            eprintln!("Failed to build report for {}: {}", config.library, e);
        }
    }
}

/// Whether a library's tests are async and therefore need sorted TAP output.
///
/// Async libraries (P4: reqwest, tokio) have non-deterministic completion
/// order, so their results must be sorted by test name before comparison.
fn is_async_library(library: &str) -> bool {
    matches!(library, "reqwest" | "tokio")
}

/// Run all three backends for a single library and build the per-test
/// `ComparisonReport` (without printing). Backend errors are logged to stderr
/// and treated as an empty result set for that backend, so the comparison
/// still surfaces the missing cases as divergences rather than crashing.
fn build_comparison_report(config: &RunConfig) -> Result<ComparisonReport, String> {
    // Plan 368 FU-4: if this lib has a mock-server/ dir, spawn it now so all
    // three backend runs can hit it; it's killed automatically when `_mock`
    // goes out of scope (after the three runs below).
    let _mock = MockServer::start_for(&config.lib_dir());
    if lib_has_mock_server(&config.lib_dir()) {
        eprintln!("mock-server: started for {} (will be killed after the run)", config.library);
    }

    // Detect the parity mode (Rust vs Python) from the library's on-disk test
    // layout. The mode determines which backends are dispatched and how their
    // results map into the three-way comparison slots (vm/a2r/rust).
    let mode = detect_parity_mode(config);

    // Run the appropriate backends. Log + swallow per-backend failures so a
    // single broken backend doesn't abort an aggregate `report` run; the
    // missing results show up as divergences in the comparison.
    let (vm_results, a2r_results, rust_results) = match mode {
        ParityMode::Rust => {
            let vm = runner::run_vm(config).unwrap_or_else(|e| {
                eprintln!("VM backend error for {}: {}", config.library, e);
                Vec::new()
            });
            let a2r = runner::run_a2r(config).unwrap_or_else(|e| {
                eprintln!("a2r backend error for {}: {}", config.library, e);
                Vec::new()
            });
            let rust = runner::run_rust(config).unwrap_or_else(|e| {
                eprintln!("Rust backend error for {}: {}", config.library, e);
                Vec::new()
            });
            (vm, a2r, rust)
        }
        ParityMode::Python => {
            // Python parity mapping (Plan 369):
            //   vm slot   -> AutoVM (run_vm)
            //   a2r slot  -> a2py (run_a2py) — the transpiler slot
            //   rust slot -> Python oracle (run_python_oracle) — the oracle slot
            let vm = runner::run_vm(config).unwrap_or_else(|e| {
                eprintln!("VM backend error for {}: {}", config.library, e);
                Vec::new()
            });
            let a2py = runner::run_a2py(config).unwrap_or_else(|e| {
                eprintln!("a2py backend error for {}: {}", config.library, e);
                Vec::new()
            });
            let oracle = runner::run_python_oracle(config).unwrap_or_else(|e| {
                eprintln!("Python oracle backend error for {}: {}", config.library, e);
                Vec::new()
            });
            (vm, a2py, oracle)
        }
    };

    // Build per-test comparison across all backends.
    let vm_map = tap::tap_map_from_results(&vm_results);
    let a2r_map = tap::tap_map_from_results(&a2r_results);
    let rust_map = tap::tap_map_from_results(&rust_results);

    let all_names: std::collections::BTreeSet<String> = vm_map
        .keys()
        .chain(a2r_map.keys())
        .chain(rust_map.keys())
        .cloned()
        .collect();

    let cases: Vec<TestCaseComparison> = all_names
        .iter()
        .map(|name| TestCaseComparison {
            name: name.clone(),
            vm: vm_map.get(name).cloned(),
            a2r: a2r_map.get(name).cloned(),
            rust: rust_map.get(name).cloned(),
        })
        .collect();

    Ok(ComparisonReport {
        library: config.library.clone(),
        cases,
    })
}

/// Which parity variant a library exercises.
///
/// - `Rust`: the original three-way parity (AutoVM vs a2r vs native Rust).
///   Used when the library has a `tests/rust/` directory.
/// - `Python`: three-way parity against a Python oracle (AutoVM vs a2py vs
///   native Python). Used when the library has a `tests/python/` directory
///   instead of (or in addition to) `tests/rust/`. Plan 369.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ParityMode {
    Rust,
    Python,
}

/// Detect the parity mode for a library by inspecting its test layout.
///
/// A `tests/python/` directory selects Python parity; otherwise the default is
/// the original Rust parity. If both directories exist, Python wins (the
/// library was explicitly migrated to Python oracle parity).
fn detect_parity_mode(config: &RunConfig) -> ParityMode {
    if config.lib_dir().join("tests").join("python").is_dir() {
        ParityMode::Python
    } else {
        ParityMode::Rust
    }
}

/// Discover all library directories under `<root>/libs/`, skipping `_dummy`
/// (which is a framework smoke test, not a real library under test).
fn discover_all_libraries(root: &PathBuf) -> Vec<String> {
    let libs_dir = root.join("libs");
    let mut libs = Vec::new();
    if let Ok(entries) = std::fs::read_dir(&libs_dir) {
        for entry in entries.flatten() {
            if entry.path().is_dir() {
                if let Some(name) = entry.file_name().to_str() {
                    if name != "_dummy" {
                        libs.push(name.to_string());
                    }
                }
            }
        }
    }
    libs.sort();
    libs
}

/// Return the library list for a given phase.
///
/// Phase mapping per Plan 355:
/// - p0: `_dummy` (framework smoke test)
/// - p1: `base64`, `url`
/// - p2: `serde_json`, `regex`
/// - p3: `sha2`, `rusqlite`
/// - p4: `reqwest`, `tokio`
/// - p5: `py_math`, `py_random` (Plan 369 Python parity)
/// - p6: `py_datetime`, `py_struct`, `py_uuid` (Plan 369 Python parity)
fn discover_libraries_by_phase(root: &PathBuf, phase: &str) -> Vec<String> {
    let phase_map: &[(&str, &[&str])] = &[
        ("p0", &["_dummy"]),
        ("p1", &["base64", "url"]),
        ("p2", &["serde_json", "regex"]),
        ("p3", &["sha2", "rusqlite"]),
        ("p4", &["reqwest", "tokio"]),
        // Plan 358 additions:
        ("d1", &["cli_app"]),
        ("d2", &["trait_advanced"]),
        ("d4", &["string_utils"]),
        // Plan 367 (consumer-mode parity): Layer 1 consumer apps. Each calls
        // `auto.<module>` stdlib and is compared three-way with a native Rust
        // oracle that calls the same underlying crate directly.
        ("d5", &["c_fs_app", "c_env_app", "c_process_app", "c_text_app"]),
        // Plan 368 FU-4 (Layer 2): HTTP consumer apps. Need a live mock server
        // (parity runner auto-spawns libs/<name>/mock-server/ via MockServer).
        ("d6", &["http_client_sync"]),
        // Plan 369 (Python parity): three-way parity against a Python oracle
        // (AutoVM vs a2py vs native Python). The mode is auto-detected from the
        // library's `tests/python/` directory by `detect_parity_mode`.
        ("p5", &["py_math", "py_random"]),
        ("p6", &["py_datetime", "py_struct", "py_uuid"]),
    ];

    for (p, libs) in phase_map {
        if *p == phase {
            // Filter to libraries that actually exist on disk so a missing
            // library does not crash the run.
            return libs
                .iter()
                .filter_map(|s| {
                    if root.join("libs").join(s).is_dir() {
                        Some(s.to_string())
                    } else {
                        None
                    }
                })
                .collect();
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_phase_lookup_returns_known_phases() {
        let tmp = tempfile_dir();
        // No libs exist on disk -> the filter drops everything.
        assert!(discover_libraries_by_phase(&tmp, "p0").is_empty());
        assert!(discover_libraries_by_phase(&tmp, "p1").is_empty());

        // Unknown phase -> empty.
        assert!(discover_libraries_by_phase(&tmp, "px").is_empty());
    }

    #[test]
    fn test_phase_lookup_picks_up_existing_lib() {
        let tmp = tempfile_dir();
        let libs = tmp.join("libs");
        std::fs::create_dir_all(libs.join("_dummy")).unwrap();
        std::fs::create_dir_all(libs.join("base64")).unwrap();
        let p0 = discover_libraries_by_phase(&tmp, "p0");
        assert_eq!(p0, vec!["_dummy".to_string()]);
        let p1 = discover_libraries_by_phase(&tmp, "p1");
        assert_eq!(p1, vec!["base64".to_string()]);
    }

    #[test]
    fn test_discover_all_libraries_skips_dummy() {
        let tmp = tempfile_dir();
        let libs = tmp.join("libs");
        std::fs::create_dir_all(libs.join("_dummy")).unwrap();
        std::fs::create_dir_all(libs.join("base64")).unwrap();
        std::fs::create_dir_all(libs.join("url")).unwrap();
        let found = discover_all_libraries(&tmp);
        assert_eq!(found, vec!["base64".to_string(), "url".to_string()]);
    }

    fn tempfile_dir() -> PathBuf {
        // Use a unique subdirectory under the system temp dir.
        let mut p = std::env::temp_dir();
        p.push(format!(
            "auto-parity-test-{}-{}",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        std::fs::create_dir_all(&p).unwrap();
        p
    }
}
