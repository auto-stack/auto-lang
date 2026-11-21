//! PLAN-1094 probe: the ordinary AV worker runs `use.py` sources through
//! the prepared py-bridge provider closure (the same foreign lane as the
//! 1093 providers). These tests bind the §6 contract shapes end-to-end:
//! nested python calls, the ValueError catch/uncaught pair with the
//! `python.value` classification, and the unicode/NUL text lane.

use ace_av::language_source_worker::serve;
use serde_json::json;
use std::path::PathBuf;

fn provider_root() -> String {
    // The prepare-av-providers bin writes the closures into its own exe
    // directory (the shared ACE_AV_PROVIDER_ROOT the harness forwards).
    let manifest = std::env::var("CARGO_MANIFEST_DIR").expect("cargo manifest dir");
    let root = PathBuf::from(manifest).join("../../target/debug");
    root.canonicalize()
        .unwrap_or(root)
        .to_string_lossy()
        .replace('/', "\\")
}

fn request(source: &str, path: &str) -> serde_json::Value {
    use base64::Engine as _;
    json!({
        "recipe": "ac-av/language-source-run-v1",
        "nonce": "0".repeat(64),
        "source_b64": base64::engine::general_purpose::STANDARD.encode(source),
        "steps": [{"op": "run"}],
        "source_path": path,
        "entry": "d_main",
        "caps": ["ac.source.language.av/v1", "foreign.scalar", "foreign.resource",
                 "console.stdout", "console.stderr"],
        "foreign": {"kind": "rust", "provider": "py", "artifact_root": provider_root()},
    })
}

#[test]
fn plan1094_py_nested_call_runs_42() {
    let source = r#"use.py builtins: int, str
fn main() int { return int(str(42)).to(int) }
"#;
    let step = serve(&request(source, "main.as")).expect("worker response")["steps"][0].clone();
    assert_eq!(step["outcome"], "success", "{step}");
    assert_eq!(step["value"], 42, "{step}");
}

#[test]
fn plan1094_py_error_catch_returns_42_and_classifies() {
    let source = r#"use.py builtins: int
fn main() int {
 try { return int("abc").to(int) } catch e { return 42 }
}
"#;
    let step = serve(&request(source, "main.as")).expect("worker response")["steps"][0].clone();
    assert_eq!(step["outcome"], "success", "{step}");
    assert_eq!(step["value"], 42, "{step}");
    assert_eq!(step["counters"]["handled_language_errors"], 1, "{step}");
}

#[test]
fn plan1094_py_error_uncaught_reports_python_value_with_span() {
    let source = r#"use.py builtins: int
fn main() int {
 return int("abc").to(int)
}
"#;
    let step = serve(&request(source, "main.as")).expect("worker response")["steps"][0].clone();
    assert_eq!(step["outcome"], "diag", "{step}");
    assert_eq!(step["diag"]["stage"], "run", "{step}");
    assert_eq!(step["diag"]["code"], "lang.error", "{step}");
    let language_error = step["language_error"].clone();
    assert_eq!(language_error["kind"], "python.5", "{step}");
    assert!(step["source_span"].is_object(), "{step}");
    // `int("abc")` starts at byte 45 in the frozen source (line 3).
    assert_eq!(step["source_span"]["start"], 45, "{step}");
}

#[test]
fn plan1094_py_unicode_nul_roundtrip() {
    let source = r#"use.py builtins: str
fn main() int {
 let p = str("中文\0尾")
 if p.to(str) == "中文\0尾" { return 42 }; return 0
}
"#;
    let step = serve(&request(source, "main.as")).expect("worker response")["steps"][0].clone();
    assert_eq!(step["outcome"], "success", "{step}");
    assert_eq!(step["value"], 42, "{step}");
}
