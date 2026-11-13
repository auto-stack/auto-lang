use std::path::PathBuf;
use std::process::Command;

use ace_a2x_core::TargetProvider;
use ace_a2x_python::{
    default_stdlib_port_catalog, python_runtime_source, verify_python_fileset_manifest,
    ParamDirection, PythonTargetProvider, PY_CERT_SCHEMA, PY_FILESET_SCHEMA, PY_PRODUCER,
    PY_RUNTIME_VERSION,
};
use ace_core::runtime_contract::sha256_hex;

fn find_python() -> String {
    if let Ok(p) = std::env::var("A2PY_PYTHON") {
        if !p.trim().is_empty() {
            return p;
        }
    }
    for cand in ["python3", "python"] {
        if let Ok(out) = Command::new(cand).arg("--version").output() {
            if out.status.success() {
                return cand.to_string();
            }
        }
    }
    panic!("INFRA ERROR: Python interpreter not found on PATH and A2PY_PYTHON not set");
}

fn scratch_dir(name: &str) -> PathBuf {
    let d = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/a2py-unit")
        .join(format!("{name}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&d);
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn run_py_snippet(dir: &PathBuf, runtime: &str, body: &str, envs: &[(&str, &str)]) -> (i32, Vec<u8>, Vec<u8>) {
    std::fs::write(dir.join("a2x_py_runtime.py"), runtime).unwrap();
    std::fs::write(dir.join("driver.py"), body).unwrap();
    let py = find_python();
    let mut cmd = Command::new(py);
    cmd.arg("driver.py").current_dir(dir);
    for (k, v) in envs {
        cmd.env(k, v);
    }
    let out = cmd.output().expect("spawn python");
    (
        out.status.code().unwrap_or(-1),
        out.stdout,
        out.stderr,
    )
}

#[test]
fn pyready_02_utf8_nul_zero_gc_handles_and_traps() {
    let dir = scratch_dir("pyready02_runtime");
    let rt = python_runtime_source(&["hir.test.mark_a", "hir.test.mark_b"]);
    let trace_file = dir.join("trace.json");
    let trace_str = trace_file.to_str().unwrap().to_string();

    // Nominal run: multibyte UTF-8 ("A中😀"), combining ("e\u{0301}"),
    // embedded NUL ("a\0b" and "ab\0cd"), math ports, explicit free & resource handle lifecycle.
    let script = r#"
import json
from a2x_py_runtime import (
    A2X_PY_RUNTIME_BODY_SHA256,
    A2X_PY_RUNTIME_VERSION,
    a2x_div,
    a2x_port_call,
    a2x_rem,
    a2x_rt_reset,
    a2x_rt_snapshot,
    a2x_run_main,
    a2x_str_free,
    a2x_str_len,
)

dep = f"{A2X_PY_RUNTIME_VERSION}@{A2X_PY_RUNTIME_BODY_SHA256}"

def entry():
    assert a2x_div(-7, 2) == -3
    assert a2x_rem(-7, 2) == -1
    assert a2x_port_call("math.private_square", 1, ("View",), dep, 9) == 81
    assert a2x_port_call("math.private_cube", 1, ("View",), dep, 3) == 27

    s_uni = "A中😀".encode("utf-8")
    s_comb = "e\u0301".encode("utf-8")
    s_nul1 = b"a\x00b"
    s_nul2 = b"ab\x00cd"
    s_short = b"ab"

    assert a2x_port_call("text.byte_length", 1, ("View",), dep, s_uni) == 8
    assert a2x_str_len(s_uni) == 3
    assert a2x_port_call("text.byte_length", 1, ("View",), dep, s_comb) == 3
    assert a2x_str_len(s_comb) == 2
    assert a2x_port_call("text.byte_length", 1, ("View",), dep, s_nul1) == 3
    assert a2x_port_call("text.byte_length", 1, ("View",), dep, s_nul2) == 5
    assert not a2x_port_call("text.equal", 1, ("View", "View"), dep, s_nul2, s_short)

    owned = a2x_port_call("text.clone", 1, ("View",), dep, s_nul2)
    assert owned.as_bytes() == b"ab\x00cd"
    a2x_str_free(owned)

    h = a2x_port_call("io.open_buffer", 1, ("View",), dep, b"alpha\nbeta\r\n")
    l1 = a2x_port_call("io.read_line_handle", 1, ("Mut",), dep, h)
    l2 = a2x_port_call("io.read_line_handle", 1, ("Mut",), dep, h)
    l3 = a2x_port_call("io.read_line_handle", 1, ("Mut",), dep, h)
    assert l1 == b"alpha" and l2 == b"beta" and l3 is None
    a2x_port_call("io.close_handle", 1, ("Move",), dep, h)

    a2x_port_call("console.say", 1, ("View",), dep, s_uni)
    snap = a2x_rt_snapshot()
    assert snap["alloc_count"] == 1
    assert snap["free_count"] == 1
    assert snap["live_strs"] == []
    assert snap["live_handles"] == []
    return len(snap["handle_trace"])

a2x_run_main(entry)
"#;

    let (code, stdout, stderr) = run_py_snippet(
        &dir,
        &rt,
        script,
        &[("A2X_PY_TRACE_JSON", &trace_str), ("A2X_RT_EVENTS", "1")],
    );
    assert_eq!(code, 0, "stderr: {}", String::from_utf8_lossy(&stderr));
    let expected_stdout = ["A中😀".as_bytes(), b"\nvalue:5\n"].concat();
    assert_eq!(stdout, expected_stdout);

    let trace_raw = std::fs::read_to_string(&trace_file).unwrap();
    assert!(trace_raw.contains("\"alloc_count\": 1"));
    assert!(trace_raw.contains("\"free_count\": 1"));
    assert!(trace_raw.contains("\"live_handles\": []"));
    assert!(trace_raw.contains("\"live_strs\": []"));

    // Negative semantic controls: DoubleClose, ResourceClosed, alloc fault, overflow, wrong port contract, invalid UTF-8
    let neg_script = r#"
from a2x_py_runtime import (
    A2X_I32_MAX,
    A2X_PY_RUNTIME_BODY_SHA256,
    A2X_PY_RUNTIME_VERSION,
    A2xTrap,
    a2x_add,
    a2x_port_call,
    a2x_rt_reset,
    a2x_str_view_lit,
)

dep = f"{A2X_PY_RUNTIME_VERSION}@{A2X_PY_RUNTIME_BODY_SHA256}"

# 1) DoubleClose
a2x_rt_reset()
h = a2x_port_call("io.open_buffer", 1, ("View",), dep, b"x\n")
a2x_port_call("io.close_handle", 1, ("Move",), dep, h)
try:
    a2x_port_call("io.close_handle", 1, ("Move",), dep, h)
    raise AssertionError("expected DoubleClose")
except A2xTrap as t:
    assert t.code == "DoubleClose", t.code

# 2) ResourceClosed on read-after-close
try:
    a2x_port_call("io.read_line_handle", 1, ("Mut",), dep, h)
    raise AssertionError("expected ResourceClosed")
except A2xTrap as t:
    assert t.code == "ResourceClosed", t.code

# 3) Overflow trap
try:
    a2x_add(A2X_I32_MAX, 1)
    raise AssertionError("expected Overflow")
except A2xTrap as t:
    assert t.code == "Overflow", t.code

# 4) Invalid UTF-8 rejected
try:
    a2x_str_view_lit(b"\xff\xfe")
    raise AssertionError("expected InvalidScalar")
except A2xTrap as t:
    assert t.code == "InvalidScalar", t.code

# 5) Port contract version mismatch rejected
try:
    a2x_port_call("text.byte_length", 2, ("View",), dep, b"hi")
    raise AssertionError("expected PortContract")
except A2xTrap as t:
    assert t.code == "PortContract", t.code

print("NEGATIVE_CONTROLS_OK")
"#;
    let (ncode, nstdout, nstderr) = run_py_snippet(&dir, &rt, neg_script, &[]);
    assert_eq!(ncode, 0, "stderr: {}", String::from_utf8_lossy(&nstderr));
    assert!(String::from_utf8_lossy(&nstdout).contains("NEGATIVE_CONTROLS_OK"));

    // Alloc fault injection via env
    let fault_script = r#"
from a2x_py_runtime import A2X_PY_RUNTIME_BODY_SHA256, A2X_PY_RUNTIME_VERSION, a2x_port_call, a2x_run_main
dep = f"{A2X_PY_RUNTIME_VERSION}@{A2X_PY_RUNTIME_BODY_SHA256}"
def entry():
    _ = a2x_port_call("text.clone", 1, ("View",), dep, b"abc")
    return 0
a2x_run_main(entry)
"#;
    let (fcode, fstdout, fstderr) =
        run_py_snippet(&dir, &rt, fault_script, &[("A2X_RT_FAULT_ALLOC", "1")]);
    assert_eq!(fcode, 0, "stderr: {}", String::from_utf8_lossy(&fstderr));
    assert_eq!(fstdout, b"trap:Resource\n");
}

#[test]
fn port_catalog_and_manifest_reject_wrong_producer_or_tamper() {
    let rt = python_runtime_source(&[]);
    let cat = default_stdlib_port_catalog(&rt);
    let rt_hash = sha256_hex(rt.as_bytes());
    assert!(cat
        .verify_call(
            "io.close_handle",
            1,
            &[ParamDirection::Move],
            &rt_hash
        )
        .is_ok());
    assert!(cat
        .verify_call(
            "io.close_handle",
            1,
            &[ParamDirection::View],
            &rt_hash
        )
        .is_err());

    let p = PythonTargetProvider;
    assert_eq!(p.descriptor().backend, "auto-a2x-python");
    let key_inputs = p.fragment_key_inputs(&rt_hash);
    assert!(key_inputs.iter().any(|(k, _)| k == "stdlib_port_catalog"));

    let files = vec![("generated.py".to_string(), "def f():\n    return 1\n".to_string())];
    let digest = sha256_hex(files[0].1.as_bytes());
    let valid_manifest = format!(
        "{{\n  \"schema\": \"{PY_FILESET_SCHEMA}\",\n  \"producer\": \"{PY_PRODUCER}\",\n  \"files\": [ {{ \"path\": \"generated.py\", \"sha256\": \"{digest}\" }} ]\n}}"
    );
    assert!(verify_python_fileset_manifest(
        &valid_manifest,
        &files,
        PY_PRODUCER,
        PY_FILESET_SCHEMA
    )
    .is_ok());
    assert!(verify_python_fileset_manifest(
        &valid_manifest,
        &files,
        "legacy-PythonTrans",
        PY_FILESET_SCHEMA
    )
    .is_err());
    assert!(verify_python_fileset_manifest(
        &valid_manifest,
        &files,
        PY_PRODUCER,
        PY_CERT_SCHEMA
    )
    .is_err());
    assert_eq!(cat.runtime_version, PY_RUNTIME_VERSION);
}
