//! PLAN-1094 probe: the ordinary AV worker runs `use.py` sources through
//! the prepared py-bridge provider closure (the same foreign lane as the
//! 1093 providers). These tests bind the §6 contract shapes end-to-end:
//! nested python calls, the ValueError catch/uncaught pair with the
//! `python.value` classification, the unicode/NUL text lane, and the
//! `with` context-manager statement (enter/exit exactly once).
//!
//! The py bridge is ONE SERIAL SESSION per process (the frozen v1 support
//! boundary) — tests sharing this process serialize through PY_LOCK so
//! parallel nextest threads cannot corrupt the interpreter state.

use ace_av::language_source_worker::serve;
use serde_json::json;
use std::path::PathBuf;
use std::sync::Mutex;

static PY_LOCK: Mutex<()> = Mutex::new(());

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
    let _guard = PY_LOCK.lock().expect("py lock");
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
    let _guard = PY_LOCK.lock().expect("py lock");
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
    let _guard = PY_LOCK.lock().expect("py lock");
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
fn plan1094_py_with_context_manager_close() {
    // §6 context-manager-close: `with closing(buf) as r` enters, the body
    // raises a language error, `__exit__` closes buf EXACTLY ONCE, and the
    // error propagates to the outer catch (no suppression by closing).
    let source = r#"use.py io: StringIO
use.py contextlib: closing
fn main() int {
 let buf = StringIO()
 try { with closing(buf) as r { r.write("ok"); fail("stop") } } catch e { }
 if buf.closed.to(bool) { return 42 }; return 0
}
"#;
    let _guard = PY_LOCK.lock().expect("py lock");
    let step = serve(&request(source, "main.as")).expect("worker response")["steps"][0].clone();
    assert_eq!(step["outcome"], "success", "{step}");
    assert_eq!(step["value"], 42, "{step}");
}

#[test]
fn plan1094_py_with_normal_exit() {
    let source = r#"use.py io: StringIO
use.py contextlib: closing
fn main() int {
 let buf = StringIO()
 with closing(buf) as r { r.write("ok") }
 if buf.closed.to(bool) { return 42 }; return 0
}
"#;
    let _guard = PY_LOCK.lock().expect("py lock");
    let step = serve(&request(source, "main.as")).expect("worker response")["steps"][0].clone();
    assert_eq!(step["outcome"], "success", "{step}");
    assert_eq!(step["value"], 42, "{step}");
}

#[test]
fn plan1094_py_with_enter_failure_no_exit() {
    // An `__enter__` failure must NOT run `__exit__` (Python semantics).
    let source = r#"use.py builtins: int
fn main() int {
 try { with int("abc") as x { } } catch e { return 42 }; return 0
}
"#;
    let _guard = PY_LOCK.lock().expect("py lock");
    let step = serve(&request(source, "main.as")).expect("worker response")["steps"][0].clone();
    assert_eq!(step["outcome"], "success", "{step}");
    assert_eq!(step["value"], 42, "{step}");
}

#[test]
fn plan1094_dbg_legacy_shapes() {
    use base64::Engine as _;
    let cases = [
        ("dual-len", "use.py numpy: arange\nfn main() int {\n var t = arange(6)\n print(t.len())\n var s = \"abc\"\n print(s.len())\n return 0\n}\n"),
        ("err-oracle", "use.py builtins: int\nfn main() int {\n try { var x = int(\"abc\"); print(\"no-error\") } catch e { print(\"caught: \" + e.to(str)) }\n print(\"end\")\n return 0\n}\n"),
        ("across-fn", "use.py builtins: int\nfn getnum(s str) int { return int(s).to(int) }\nfn main() int { return getnum(\"42\") }\n"),
        ("native-len", "fn main() int {\n var s = \"abcdef\"\n if s.len() != 6 { return 0 }\n var xs = [1, 2, 3, 4]\n if xs.len() != 4 { return 0 }\n var t = \"x\" + s\n if t.len() != 7 { return 0 }\n return 42\n}\n"),
    ];
    for (name, src) in cases {
        let mut req = request(src, "main.as");
        if name == "native-len" {
            req.as_object_mut().unwrap().remove("foreign");
        }
        let step = serve(&req).expect("worker")["steps"][0].clone();
        let stdout = base64::engine::general_purpose::STANDARD
            .decode(step["console"]["stdout_b64"].as_str().unwrap_or(""))
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        println!("DBG legacy {name}: outcome={} value={:?} stdout={:?} diag={:?} lang_err={:?}", step["outcome"], step["value"], stdout, step["diag"], step["language_error"]);
    }
}

#[test]
fn plan1094_dbg_legacy_shapes2() {
    use base64::Engine as _;
    let cases = [
        ("len-attr", "use.py builtins: list\nfn main() int {\n var xs = list(\"xy\")\n print(xs.len())\n return 0\n}\n"),
        ("len-pylist", "use.py builtins: list\nfn main() int {\n var xs = list((1, 2, 3))\n print(xs.len())\n return 0\n}\n"),
        ("nested-fn", "use.py builtins: int\nfn main() int {\n fn getnum(s str) int { return int(s).to(int) }\n return getnum(\"42\")\n}\n"),
    ];
    for (name, src) in cases {
        let step = serve(&request(src, "main.as")).expect("worker")["steps"][0].clone();
        let stdout = base64::engine::general_purpose::STANDARD
            .decode(step["console"]["stdout_b64"].as_str().unwrap_or(""))
            .map(|b| String::from_utf8_lossy(&b).into_owned())
            .unwrap_or_default();
        println!("DBG legacy2 {name}: outcome={} value={:?} stdout={:?} diag={:?}", step["outcome"], step["value"], stdout, step["diag"]);
    }
}

#[test]
fn plan1094_py_unicode_nul_roundtrip() {
    let source = r#"use.py builtins: str
fn main() int {
 let p = str("中文\0尾")
 if p.to(str) == "中文\0尾" { return 42 }; return 0
}
"#;
    let _guard = PY_LOCK.lock().expect("py lock");
    let step = serve(&request(source, "main.as")).expect("worker response")["steps"][0].clone();
    assert_eq!(step["outcome"], "success", "{step}");
    assert_eq!(step["value"], 42, "{step}");
}
