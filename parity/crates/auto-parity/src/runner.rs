use crate::tap::{parse_tap, parse_tap_sorted, TapResult};
use std::path::{Path, PathBuf};
use std::process::{Child, Command};

/// Configuration for running a parity check on a single library.
#[derive(Debug, Clone)]
pub struct RunConfig {
    /// Path to the parity workspace root (the `parity/` directory).
    pub parity_root: PathBuf,
    /// Path to the auto binary (e.g. "auto" or a full path).
    pub auto_binary: String,
    /// Library name (e.g. "base64", "_dummy").
    pub library: String,
    /// If true, sort TAP results by test name before comparison.
    /// Used for async tests where completion order is non-deterministic.
    pub sort_results: bool,
}

impl RunConfig {
    /// Path to the library directory: `<parity_root>/libs/<library>`.
    pub fn lib_dir(&self) -> PathBuf {
        self.parity_root.join("libs").join(&self.library)
    }

    /// Return a clone of this config with the library name replaced.
    /// All other fields (including `sort_results`) are preserved.
    pub fn with_library(&self, name: &str) -> RunConfig {
        RunConfig {
            parity_root: self.parity_root.clone(),
            auto_binary: self.auto_binary.clone(),
            library: name.to_string(),
            sort_results: self.sort_results,
        }
    }

    /// Parse TAP output using the sort mode selected by this config.
    fn parse(&self, output: &str) -> Vec<TapResult> {
        if self.sort_results {
            parse_tap_sorted(output)
        } else {
            parse_tap(output)
        }
    }
}

// ──────────────────────────────────────────────────────────────────────────
// Plan 368 FU-4: mock-server setup/teardown for HTTP consumer parity libs.
//
// Some consumer libs (e.g. http_client_sync) need a live HTTP server to test
// against. The server lives at `<lib_dir>/mock-server/` as a standalone Cargo
// binary ( TcpListener on 127.0.0.1:18080). This RAII guard builds + spawns it
// before the three-way run, waits for the port to accept connections, and kills
// it on drop so the server always outlives all three backend runs.
// ──────────────────────────────────────────────────────────────────────────

/// Address the mock servers bind to (must match mock-server/src/main.rs).
const MOCK_HOST: &str = "127.0.0.1";
const MOCK_PORT: u16 = 18080;

/// True iff `<lib_dir>/mock-server/Cargo.toml` exists — i.e. this lib needs a
/// mock server for its three-way run.
pub fn lib_has_mock_server(lib_dir: &Path) -> bool {
    lib_dir.join("mock-server").join("Cargo.toml").is_file()
}

/// A guard that keeps the mock server alive until dropped. Spawned via
/// `cargo run --manifest-path <lib>/mock-server/Cargo.toml` so it builds on
/// first use and runs as a child process.
pub struct MockServer {
    child: Option<Child>,
}

impl MockServer {
    /// Build + spawn the mock server for `lib_dir` and wait (up to ~30s) for it
    /// to start accepting connections on MOCK_HOST:MOCK_PORT. Returns a guard
    /// that kills the server on drop. If the lib has no mock-server dir, returns
    /// an empty guard (no-op).
    pub fn start_for(lib_dir: &Path) -> Self {
        if !lib_has_mock_server(lib_dir) {
            return MockServer { child: None };
        }
        let manifest = lib_dir.join("mock-server").join("Cargo.toml");
        let child = Command::new("cargo")
            .args(["run", "--manifest-path", &manifest.to_string_lossy()])
            // Detach stdout/stderr so cargo's build/run chatter doesn't pollute
            // the parity TAP output (which is parsed from the backends' stdout).
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn();
        let child = match child {
            Ok(c) => c,
            Err(e) => {
                eprintln!(
                    "mock-server: failed to spawn (cargo run --manifest-path {}): {}",
                    manifest.display(),
                    e
                );
                return MockServer { child: None };
            }
        };
        let mut guard = MockServer { child: Some(child) };
        guard.wait_for_port();
        guard
    }

    /// Poll-connect to MOCK_HOST:MOCK_PORT until it succeeds or ~30s elapse.
    fn wait_for_port(&self) {
        use std::net::TcpStream;
        use std::time::{Duration, Instant};
        let deadline = Instant::now() + Duration::from_secs(30);
        let addr = format!("{MOCK_HOST}:{MOCK_PORT}");
        while Instant::now() < deadline {
            if TcpStream::connect(&addr).is_ok() {
                return;
            }
            std::thread::sleep(Duration::from_millis(250));
        }
        eprintln!(
            "mock-server: port {MOCK_HOST}:{MOCK_PORT} did not come up within 30s; \
             HTTP consumer tests will likely fail"
        );
    }
}

impl Drop for MockServer {
    fn drop(&mut self) {
        if let Some(child) = self.child.as_mut() {
            // Kill the whole process tree: `cargo run` spawns the build + the
            // server binary as children. cargo itself forwards signals on Unix;
            // on Windows we kill the cargo process (which orphans the server on
            // some setups, but the port frees once the OS reaps it). Best-effort.
            let _ = child.kill();
            let _ = child.wait();
        }
    }
}

/// Run the AutoVM backend: `auto <test_file>` for each .at file in
/// `tests/auto/`. Returns combined TAP results.
pub fn run_vm(config: &RunConfig) -> Result<Vec<TapResult>, String> {
    let test_dir = config.lib_dir().join("tests").join("auto");
    if !test_dir.is_dir() {
        return Err(format!(
            "VM test dir not found: {}",
            test_dir.display()
        ));
    }
    let mut all_results = Vec::new();

    for entry in std::fs::read_dir(&test_dir)
        .map_err(|e| format!("failed to read test dir {}: {}", test_dir.display(), e))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("at") {
            continue;
        }

        // Canonicalise the test path so it is valid regardless of the child
        // process's working directory. The runner sets current_dir to the lib
        // dir (so `use auto.<lib>` resolves against `./auto/`), which would
        // invalidate a relative test path computed from the parity root.
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.clone());

        // Run from the lib directory so `use auto.<lib>` resolves against `./auto/`.
        let output = Command::new(&config.auto_binary)
            .arg(&abs_path)
            .current_dir(config.lib_dir())
            .output()
            .map_err(|e| format!("failed to run auto: {}", e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        // If auto crashes with no TAP output, capture as a single failure
        // keyed by the file stem so the comparison still surfaces it.
        if !output.status.success() && stdout.is_empty() {
            let stem = path
                .file_stem()
                .map(|s| s.to_string_lossy().to_string())
                .unwrap_or_else(|| path.display().to_string());
            all_results.push(TapResult {
                passed: false,
                number: 0,
                name: stem,
                diagnostics: Some(format!("auto crashed: {}", stderr.trim())),
            });
        } else {
            all_results.extend(config.parse(&stdout));
        }
    }

    Ok(all_results)
}

/// Run the a2r backend: transpile each test .at to Rust, prepend the
/// transpiled library source, wrap in a binary crate that depends on
/// `auto-lang` and `a2r-std` via path, compile, run, collect TAP output.
pub fn run_a2r(config: &RunConfig) -> Result<Vec<TapResult>, String> {
    let test_dir = config.lib_dir().join("tests").join("auto");
    if !test_dir.is_dir() {
        return Err(format!(
            "a2r test dir not found: {}",
            test_dir.display()
        ));
    }
    let lib_auto_dir = config.lib_dir().join("auto");
    let build_dir = config.lib_dir().join("build_a2r");
    std::fs::create_dir_all(&build_dir).map_err(|e| e.to_string())?;

    // Transpile the library source once and reuse it for every test binary.
    let lib_rs = transpile_library(config, &lib_auto_dir)?;

    let mut all_results = Vec::new();

    for entry in std::fs::read_dir(&test_dir)
        .map_err(|e| format!("failed to read test dir {}: {}", test_dir.display(), e))?
    {
        let entry = entry.map_err(|e| e.to_string())?;
        let test_path = entry.path();
        if test_path.extension().and_then(|s| s.to_str()) != Some("at") {
            continue;
        }
        let test_stem = test_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        // Step 1: Transpile the test .at -> Rust source string.
        // `auto trans --path <f> rust` (without --output) writes the source to
        // a sibling `<stem>.a2r.rs` file and returns only a status line on
        // stdout. We canonicalise the test path so it resolves regardless of
        // the child's working directory, then read back the generated file.
        let abs_test_path = test_path
            .canonicalize()
            .unwrap_or_else(|_| test_path.clone());
        let trans_output = Command::new(&config.auto_binary)
            .args([
                "trans",
                "--path",
                &abs_test_path.to_string_lossy(),
                "rust",
            ])
            .current_dir(config.lib_dir())
            .output()
            .map_err(|e| format!("failed to run auto trans: {}", e))?;

        if !trans_output.status.success() {
            let stderr = String::from_utf8_lossy(&trans_output.stderr);
            all_results.push(TapResult {
                passed: false,
                number: 0,
                name: test_stem.clone(),
                diagnostics: Some(format!(
                    "a2r transpile failed: {}",
                    stderr.trim()
                )),
            });
            continue;
        }

        // The transpiler writes `<stem>.a2r.rs` next to the source. Read it back.
        let transpiled = abs_test_path.with_extension("a2r.rs");
        let test_rs = match std::fs::read_to_string(&transpiled) {
            Ok(s) => s,
            Err(e) => {
                // Some auto versions print source to stdout instead. Fall back
                // to stdout if it looks like Rust source (contains "fn" or "}").
                let stdout = String::from_utf8_lossy(&trans_output.stdout).to_string();
                if stdout.contains("fn ") || stdout.contains('}') {
                    stdout
                } else {
                    all_results.push(TapResult {
                        passed: false,
                        number: 0,
                        name: test_stem.clone(),
                        diagnostics: Some(format!(
                            "a2r output not found at {}: {}",
                            transpiled.display(),
                            e
                        )),
                    });
                    continue;
                }
            }
        };

        // Step 2: Build a binary crate that depends on auto-lang + a2r-std.
        let bin_name = test_stem.replace('-', "_");
        let bin_dir = build_dir.join(&bin_name);
        std::fs::create_dir_all(bin_dir.join("src")).map_err(|e| e.to_string())?;

        // Cargo path values are Rust string literals, so backslashes must be
        // escaped. Convert to forward slashes for cross-platform safety.
        // Canonicalise the dependency paths so they are absolute — the build
        // runs with current_dir set to bin_dir, which would invalidate a
        // relative dependency path.
        let crate_path = |name: &str| -> std::path::PathBuf {
            let raw = config
                .parity_root
                .join("..")
                .join("crates")
                .join(name);
            match raw.canonicalize() {
                // On Windows, canonicalize() returns a \\?\ extended-length
                // path that Cargo rejects. Strip the verbatim prefix.
                Ok(p) => {
                    let s = p.to_string_lossy().into_owned();
                    let stripped = s.strip_prefix(r"\\?\").unwrap_or(&s).to_string();
                    std::path::PathBuf::from(stripped)
                }
                Err(_) => raw,
            }
        };
        let auto_lang_path = crate_path("auto-lang");
        let a2r_std_path = crate_path("a2r-std");
        let to_fwd = |p: &std::path::Path| {
            p.to_string_lossy().replace('\\', "/")
        };
        let cargo_toml = format!(
            r#"[package]
name = "{bin_name}"
version = "0.1.0"
edition = "2021"

[dependencies]
auto-lang = {{ path = "{auto_lang}" }}
a2r-std = {{ path = "{a2r_std}" }}
# Plan 347: the a2r transpiler emits `use once_cell::sync::Lazy;` (and
# `std::sync::Mutex`) for module-level `var` globals, so any a2r test binary
# whose library uses globals needs once_cell on the dep list.
once_cell = "1"
# Plan 347 P4: async tests transpile to `async fn main()` with `#[tokio::main]`,
# so the test binary needs tokio as a dependency.
tokio = {{ version = "1", features = ["rt", "macros"] }}

[[bin]]
name = "{bin_name}"
path = "src/main.rs"

# Keep this generated crate out of the parity workspace.
[workspace]
"#,
            bin_name = bin_name,
            auto_lang = to_fwd(&auto_lang_path),
            a2r_std = to_fwd(&a2r_std_path),
        );
        std::fs::write(bin_dir.join("Cargo.toml"), cargo_toml)
            .map_err(|e| e.to_string())?;

        // Prepend the library's transpiled source so the test code resolves
        // the library's symbols at compile time. The transpiled test imports
        // symbols via `use crate::<lib>:{...}` (mirroring the Auto `use <lib>`
        // clause), so the library source must be wrapped in a matching
        // `pub mod <lib> { ... }` with public functions for the import to work.
        //
        // Plan 347 P4: when the library name collides with an extern crate
        // (e.g. `tokio`), wrapping as `pub mod tokio` shadows the tokio crate,
        // breaking `#[tokio::main]`. We wrap as `pub mod auto_<lib>` and
        // rewrite the test's `use crate::<lib>::` imports to match.
        let main_rs = if lib_rs.is_empty() {
            test_rs
        } else {
            let wrapped = wrap_as_module(&config.library, &lib_rs);
            let combined = format!("{}\n\n{}", wrapped, test_rs);
            // Rewrite `crate::<lib>::` → `crate::auto_<lib>::` in the test code
            // to match the prefixed module name.
            let mod_name = config.library.replace('-', "_");
            let prefixed = format!("auto_{}", mod_name);
            combined.replace(
                &format!("crate::{}::", mod_name),
                &format!("crate::{}::", prefixed),
            )
        };
        std::fs::write(bin_dir.join("src").join("main.rs"), main_rs)
            .map_err(|e| e.to_string())?;

        // Step 3: Build and run the test binary.
        let build_output = Command::new("cargo")
            .args(["build", "--release"])
            .current_dir(&bin_dir)
            .output()
            .map_err(|e| format!("failed to run cargo build: {}", e))?;

        if !build_output.status.success() {
            let stderr = String::from_utf8_lossy(&build_output.stderr);
            all_results.push(TapResult {
                passed: false,
                number: 0,
                name: test_stem.clone(),
                diagnostics: Some(format!("a2r compile failed: {}", stderr.trim())),
            });
            continue;
        }

        let bin_path = bin_dir
            .join("target")
            .join("release")
            .join(&bin_name);
        let run_output = Command::new(&bin_path)
            .current_dir(config.lib_dir())
            .output()
            .map_err(|e| format!("failed to run a2r binary: {}", e))?;

        let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();

        if !run_output.status.success() && stdout.is_empty() {
            all_results.push(TapResult {
                passed: false,
                number: 0,
                name: test_stem,
                diagnostics: Some(format!("a2r binary crashed: {}", stderr.trim())),
            });
        } else {
            all_results.extend(config.parse(&stdout));
        }
    }

    Ok(all_results)
}

/// Wrap transpiled library source in a `pub mod <name> { ... }` block so the
/// test's generated `use crate::<name>:{...}` import resolves. Top-level
/// function signatures are made `pub` (a no-op in Auto but required by Rust's
/// visibility rules for cross-module imports).
fn wrap_as_module(lib_name: &str, src: &str) -> String {
    // Normalise lib names so they are valid Rust identifiers, and prefix with
    // `auto_` to avoid shadowing extern crates (e.g. `tokio`, `regex`).
    let mod_name = format!("auto_{}", lib_name.replace('-', "_"));
    // Promote top-level (zero-indent) function declarations to `pub fn`, and
    // struct/type declarations to `pub struct`/`pub enum`, so the imported
    // symbols are visible outside the module. Handles both `fn` and
    // `async fn` (the a2r transpiler emits `async fn` for `~T` functions).
    // Struct *fields* are also promoted to `pub`: in the AutoVM a returned
    // user-defined struct's fields are readable from any module (Plan 348 B1),
    // and the parity tests now exercise that (e.g. `u.scheme` on a `Url`
    // returned across the boundary), so the a2r module wrap must expose the
    // fields with matching visibility.
    let promoted = src
        .lines()
        .map(|line| {
            let trimmed = line.trim_start();
            let indent_len = line.len() - trimmed.len();
            let indent = &line[..indent_len];
            if indent.is_empty() {
                if let Some(rest) = trimmed.strip_prefix("fn ") {
                    return format!("pub fn {}", rest);
                }
                if let Some(rest) = trimmed.strip_prefix("async fn ") {
                    return format!("pub async fn {}", rest);
                }
                if let Some(rest) = trimmed.strip_prefix("struct ") {
                    return format!("pub struct {}", rest);
                }
                if let Some(rest) = trimmed.strip_prefix("enum ") {
                    return format!("pub enum {}", rest);
                }
            } else if indent_len == 4 {
                // Inside a struct body: promote `field: Type,` lines to
                // `pub field: Type,` so cross-module field reads compile.
                // (The a2r transpiler indents struct fields by 4 spaces.)
                if let Some(rest) = trimmed.strip_prefix("pub ") {
                    let _ = rest; // already pub; leave as-is
                } else if is_struct_field_line(trimmed) {
                    return format!("{}pub {}", indent, trimmed);
                }
            }
            line.to_string()
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("pub mod {} {{\n{}\n}}", mod_name, promoted)
}

/// Heuristic: does this trimmed line look like a struct field declaration
/// (`name: Type,`)? Used by `wrap_as_module` to promote fields to `pub`.
/// Rejects derive attributes, braces, and blank lines.
fn is_struct_field_line(trimmed: &str) -> bool {
    if trimmed.is_empty() || trimmed.starts_with('#') {
        return false;
    }
    // Must contain `: ` (field separator) and not be a brace line.
    if !trimmed.contains(": ") || trimmed.starts_with('{') || trimmed.starts_with('}') {
        return false;
    }
    // The token before the first `:` must be a valid Rust identifier (the
    // field name), not e.g. an `impl` or a nested item.
    let name = trimmed.split(':').next().unwrap_or("").trim();
    if name.is_empty() {
        return false;
    }
    name.chars().all(|c| c.is_alphanumeric() || c == '_')
        && name.chars().next().map_or(false, |c| c.is_alphabetic() || c == '_')
}

/// Transpile all .at files in the library's `auto/` directory into a single
/// Rust source string (concatenated). Returns an empty string if the
/// directory does not exist.
fn transpile_library(config: &RunConfig, lib_auto_dir: &Path) -> Result<String, String> {
    let mut combined = String::new();
    if !lib_auto_dir.exists() {
        return Ok(combined);
    }
    // Sort directory entries so the concatenated output is deterministic.
    let mut entries: Vec<_> = std::fs::read_dir(lib_auto_dir)
        .map_err(|e| e.to_string())?
        .filter_map(|e| e.ok())
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("at") {
            continue;
        }
        // Canonicalise so the path resolves after the child changes CWD.
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.clone());
        let output = Command::new(&config.auto_binary)
            .args([
                "trans",
                "--path",
                &abs_path.to_string_lossy(),
                "rust",
            ])
            .current_dir(config.lib_dir())
            .output()
            .map_err(|e| format!("failed to transpile library: {}", e))?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(format!(
                "library transpile failed for {}: {}",
                path.display(),
                stderr.trim()
            ));
        }
        // `auto trans` writes `<stem>.a2r.rs` next to the source. Read it back;
        // fall back to stdout for auto versions that print source directly.
        let transpiled = abs_path.with_extension("a2r.rs");
        let rs = match std::fs::read_to_string(&transpiled) {
            Ok(s) => s,
            Err(_) => {
                let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                if stdout.contains("fn ") || stdout.contains('}') {
                    stdout
                } else {
                    String::new()
                }
            }
        };
        combined.push_str(&rs);
        combined.push('\n');
    }
    Ok(combined)
}

/// Run the Rust native backend: `cargo test` in the library's
/// `tests/rust/` directory. Returns TAP-equivalent results.
pub fn run_rust(config: &RunConfig) -> Result<Vec<TapResult>, String> {
    let rust_dir = config.lib_dir().join("tests").join("rust");
    if !rust_dir.is_dir() {
        return Err(format!(
            "rust test dir not found: {}",
            rust_dir.display()
        ));
    }

    let output = Command::new("cargo")
        .args(["test"])
        .current_dir(&rust_dir)
        .output()
        .map_err(|e| format!("failed to run cargo test: {}", e))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
    let combined = format!("{}\n{}", stdout, stderr);

    // Convert cargo test output to TAP-equivalent results.
    // Cargo prints "test <name> ... ok" or "test <name> ... FAILED".
    // The sort_results flag (set for async libraries) is honoured here too.
    let mut results = parse_cargo_test_output(&combined);
    if config.sort_results {
        results.sort_by(|a, b| a.name.cmp(&b.name));
        for (i, r) in results.iter_mut().enumerate() {
            r.number = i + 1;
        }
    }
    Ok(results)
}

/// Parse cargo test output into TAP results.
///
/// Recognises lines like:
/// - `test test_encode_empty ... ok`
/// - `test test_decode_bad ... FAILED`
pub fn parse_cargo_test_output(output: &str) -> Vec<TapResult> {
    let mut results = Vec::new();
    let mut number = 0;
    for line in output.lines() {
        let line = line.trim();
        if let Some(rest) = line.strip_prefix("test ") {
            if let Some(name_end) = rest.rfind(" ... ") {
                let name = &rest[..name_end];
                let status = &rest[name_end + 5..];
                number += 1;
                let passed = status.trim() == "ok";
                results.push(TapResult {
                    passed,
                    number,
                    name: name.to_string(),
                    diagnostics: if passed {
                        None
                    } else {
                        Some("cargo test FAILED".to_string())
                    },
                });
            }
        }
    }
    results
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_cargo_test_pass() {
        let output = "test test_encode_empty ... ok\ntest test_encode_single ... ok\n";
        let results = parse_cargo_test_output(output);
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert_eq!(results[0].name, "test_encode_empty");
        assert_eq!(results[0].number, 1);
        assert_eq!(results[1].number, 2);
    }

    #[test]
    fn test_parse_cargo_test_fail() {
        let output = "test test_decode_bad ... FAILED\n";
        let results = parse_cargo_test_output(output);
        assert_eq!(results.len(), 1);
        assert!(!results[0].passed);
        assert_eq!(results[0].name, "test_decode_bad");
        assert_eq!(
            results[0].diagnostics.as_deref(),
            Some("cargo test FAILED")
        );
    }

    #[test]
    fn test_parse_cargo_test_ignores_other_lines() {
        let output = "\nrunning 2 tests\n\
            test test_a ... ok\n\
            test test_b ... FAILED\n\
            test result: FAILED. 1 passed; 1 failed\n";
        let results = parse_cargo_test_output(output);
        assert_eq!(results.len(), 2);
        assert!(results[0].passed);
        assert!(!results[1].passed);
    }

    #[test]
    fn test_run_config_with_library_preserves_sort_flag() {
        let base = RunConfig {
            parity_root: PathBuf::from("/tmp/parity"),
            auto_binary: "auto".to_string(),
            library: "_dummy".to_string(),
            sort_results: true,
        };
        let next = base.with_library("reqwest");
        assert_eq!(next.library, "reqwest");
        assert!(next.sort_results);
        assert_eq!(next.parity_root, base.parity_root);
        // Original is untouched.
        assert_eq!(base.library, "_dummy");
    }

    #[test]
    fn test_run_config_lib_dir() {
        let cfg = RunConfig {
            parity_root: PathBuf::from("/tmp/parity"),
            auto_binary: "auto".to_string(),
            library: "base64".to_string(),
            sort_results: false,
        };
        assert_eq!(
            cfg.lib_dir(),
            PathBuf::from("/tmp/parity/libs/base64")
        );
    }
}
fn python_interpreter() -> &'static str {
    use std::sync::OnceLock;
    static INTERP: OnceLock<String> = OnceLock::new();
    INTERP.get_or_init(|| {
        for candidate in ["python3", "python"] {
            if Command::new(candidate)
                .arg("--version")
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false)
            {
                return candidate.to_string();
            }
        }
        // Default to python3 even if the probe failed — the actual run will
        // surface a clear error if neither interpreter is on PATH.
        "python3".to_string()
    })
}

/// Run the Python oracle backend: `python3 tests/python/*.py`
/// Returns TAP results parsed from stdout.
pub fn run_python_oracle(config: &RunConfig) -> Result<Vec<TapResult>, String> {
    let python_dir = config.lib_dir().join("tests").join("python");
    if !python_dir.is_dir() {
        return Err(format!(
            "python test dir not found: {}",
            python_dir.display()
        ));
    }

    let mut all_results = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir(&python_dir)
        .map_err(|e| format!("failed to read python test dir: {}", e))?
        .filter_map(|e| e.ok())
        .filter(|e| {
            e.path().extension().and_then(|s| s.to_str()) == Some("py")
                && e.file_name().to_string_lossy().starts_with("test_")
        })
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let path = entry.path();
        let abs_path = path.canonicalize().unwrap_or_else(|_| path.clone());

        let output = Command::new(python_interpreter())
            .arg(&abs_path)
            .current_dir(config.lib_dir())
            .output()
            .map_err(|e| format!("failed to run {}: {}", python_interpreter(), e))?;

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&output.stderr).to_string();

        if !output.status.success() && stdout.is_empty() {
            all_results.push(TapResult {
                passed: false,
                number: 0,
                name: path
                    .file_stem()
                    .unwrap()
                    .to_string_lossy()
                    .to_string(),
                diagnostics: Some(format!("python crashed: {}", stderr.trim())),
            });
        } else {
            all_results.extend(config.parse(&stdout));
        }
    }

    Ok(all_results)
}

/// Run the a2py backend: transpile each .at test to Python, then run with the
/// Python interpreter. Returns TAP results parsed from stdout.
pub fn run_a2py(config: &RunConfig) -> Result<Vec<TapResult>, String> {
    let test_dir = config.lib_dir().join("tests").join("auto");
    let build_dir = config.lib_dir().join("build_a2py");
    std::fs::create_dir_all(&build_dir).map_err(|e| e.to_string())?;

    let mut all_results = Vec::new();

    let mut entries: Vec<_> = std::fs::read_dir(&test_dir)
        .map_err(|e| format!("failed to read test dir {}: {}", test_dir.display(), e))?
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().and_then(|s| s.to_str()) == Some("at"))
        .collect();
    entries.sort_by_key(|e| e.path());

    for entry in entries {
        let test_path = entry.path();
        let test_stem = test_path
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .to_string();

        // Transpile .at -> .py.
        //
        // Note: `auto trans ... python --output <f>` does NOT honour `--output`
        // for the code payload — `trans_python` always writes the generated
        // source to a sibling `<stem>.py` next to the input, and `--output`
        // only receives the `[trans] ... -> ...` status string. So we invoke
        // the transpiler WITHOUT `--output` and read back the sibling file
        // (mirroring how `run_a2r` reads `<stem>.a2r.rs`). We then copy it
        // into the build dir so a2py artefacts are isolated from the source
        // tree.
        let py_path = build_dir.join(format!("{}.py", test_stem));
        let abs_test = test_path
            .canonicalize()
            .unwrap_or_else(|_| test_path.clone());

        let trans_output = Command::new(&config.auto_binary)
            .args([
                "trans",
                "--path",
                &abs_test.to_string_lossy(),
                "python",
            ])
            .current_dir(config.lib_dir())
            .output()
            .map_err(|e| format!("failed to run auto trans python: {}", e))?;

        if !trans_output.status.success() {
            let stderr = String::from_utf8_lossy(&trans_output.stderr);
            all_results.push(TapResult {
                passed: false,
                number: 0,
                name: test_stem.clone(),
                diagnostics: Some(format!("a2py transpile failed: {}", stderr.trim())),
            });
            continue;
        }

        // The transpiler writes `<stem>.py` next to the source. Read it back.
        let sibling_py = abs_test.with_extension("py");
        let py_source = match std::fs::read_to_string(&sibling_py) {
            Ok(s) => s,
            Err(_) => {
                // Fall back to stdout for auto versions that print source
                // directly instead of writing a sibling file.
                let stdout = String::from_utf8_lossy(&trans_output.stdout).to_string();
                if stdout.contains("def ") || stdout.contains("import ") {
                    stdout
                } else {
                    all_results.push(TapResult {
                        passed: false,
                        number: 0,
                        name: test_stem.clone(),
                        diagnostics: Some(format!(
                            "a2py output not found at {}",
                            sibling_py.display()
                        )),
                    });
                    continue;
                }
            }
        };
        // The transpiler unconditionally writes the sibling `<stem>.py` next
        // to the source; remove it after reading so a2py artefacts do not
        // pollute the library's source tree. Ignore errors — the file may
        // legitimately not exist when we took the stdout fallback above.
        let _ = std::fs::remove_file(&sibling_py);

        // Copy the generated source into the isolated build dir and run it
        // from there so the source tree is not polluted by a2py artefacts.
        std::fs::write(&py_path, &py_source).map_err(|e| e.to_string())?;

        // Run the transpiled Python.
        //
        // `-P` (PYTHONSAFEPATH) prevents Python from prepending the script's
        // own directory to sys.path. The a2py build dir holds the transpiled
        // `<stem>.py`; when a test stem matches a pure-Python stdlib module
        // (e.g. `random`, `datetime`, `uuid`), the build-dir file would
        // otherwise shadow the stdlib module and `from random import ...` would
        // fail with ImportError. Built-in modules like `math` are unaffected
        // (they resolve before the path search), which is why py_math worked
        // without this flag. This makes a2py usable for all Python-parity libs.
        let abs_py = py_path.canonicalize().unwrap_or_else(|_| py_path.clone());
        let run_output = Command::new(python_interpreter())
            .arg("-P")
            .arg(&abs_py)
            .current_dir(config.lib_dir())
            .output()
            .map_err(|e| {
                format!(
                    "failed to run {} on transpiled: {}",
                    python_interpreter(),
                    e
                )
            })?;

        let stdout = String::from_utf8_lossy(&run_output.stdout).to_string();
        let stderr = String::from_utf8_lossy(&run_output.stderr).to_string();

        if !run_output.status.success() && stdout.is_empty() {
            all_results.push(TapResult {
                passed: false,
                number: 0,
                name: test_stem,
                diagnostics: Some(format!("a2py python crashed: {}", stderr.trim())),
            });
        } else {
            all_results.extend(config.parse(&stdout));
        }
    }

    Ok(all_results)
}
