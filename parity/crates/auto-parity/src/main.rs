mod compare;
mod report;
mod runner;
mod tap;

use clap::{Parser, Subcommand};
use compare::{ComparisonReport, TestCaseComparison};
use runner::RunConfig;
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
    config.sort_results = matches!(config.library.as_str(), "reqwest" | "tokio");

    println!();
    println!("{}", "=".repeat(60));
    println!("Checking library: {}", config.library);
    println!("{}", "=".repeat(60));

    // Run all three backends.
    let vm_results = runner::run_vm(&config).unwrap_or_else(|e| {
        eprintln!("VM backend error: {}", e);
        Vec::new()
    });
    let a2r_results = runner::run_a2r(&config).unwrap_or_else(|e| {
        eprintln!("a2r backend error: {}", e);
        Vec::new()
    });
    let rust_results = runner::run_rust(&config).unwrap_or_else(|e| {
        eprintln!("Rust backend error: {}", e);
        Vec::new()
    });

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

    let report = ComparisonReport {
        library: config.library.clone(),
        cases,
    };

    let text = report::format_report(&report);
    println!("{}", text);
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
fn discover_libraries_by_phase(root: &PathBuf, phase: &str) -> Vec<String> {
    let phase_map: &[(&str, &[&str])] = &[
        ("p0", &["_dummy"]),
        ("p1", &["base64", "url"]),
        ("p2", &["serde_json", "regex"]),
        ("p3", &["sha2", "rusqlite"]),
        ("p4", &["reqwest", "tokio"]),
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
