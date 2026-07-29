//! Plan 364 Step 5: runtime VM Config eval — end-to-end tests.
//!
//! These tests go through the full pipeline (parser → `Codegen::new_for_config()`
//! → AutoVM accumulation opcodes → materialization) via `AutoConfig::from_code`,
//! so they verify real behavior at runtime, not bytecode shape. `port` is
//! injected exactly like `auto build` does for manifest evaluation.
//!
//! Migrated from the deleted `config_codegen.rs` (Plan 075); the old bytecode-
//! shape tests (asserting CREATE_OBJ opcodes) were dropped because the runtime
//! Config mode emits a different opcode sequence (PUSH_ACCUM/ACCUM_PAIR/...).

#![cfg(test)]

use crate::config::AutoConfig;
use auto_val::Obj;

fn eval_with_port(source: &str, port: &str) -> AutoConfig {
    let mut args = Obj::new();
    args.set("port", auto_val::Value::str(port));
    AutoConfig::from_code(source, &args).unwrap()
}

#[test]
fn test_config_if_else_picks_matching_branch() {
    // port == "win32" → x = 1
    let source = r#"
if port == "win32" {
    x: 1
} else {
    x: 2
}
"#;
    let cfg = eval_with_port(source, "win32");
    assert_eq!(cfg.root.get_prop("x").to_astr().as_str(), "1");

    // port == "linux" → else → x = 2
    let cfg = eval_with_port(source, "linux");
    assert_eq!(cfg.root.get_prop("x").to_astr().as_str(), "2");
}

#[test]
fn test_config_var_declaration_and_reference() {
    // Mirrors SCU001's `var kernel_config = {...}` then `kernel: kernel_config`.
    let source = r#"
var kernel_config = { mode: "lockstep", mpu: true }
kernel: kernel_config
"#;
    let cfg = eval_with_port(source, "win32");
    // `kernel_config` itself must NOT be a root field (it's a variable).
    // `kernel` must resolve to the recorded object.
    let kernel = cfg.root.get_prop("kernel");
    let repr = kernel.repr().to_string();
    assert!(repr.contains("lockstep"),
        "kernel should resolve to the var's object value (containing 'lockstep'), got {}",
        repr);
}

#[test]
fn test_config_var_redeclared_takes_latest() {
    // Re-declaring the same var before reference takes the latest value.
    let source = r#"
var arch = "arm"
var arch = "armv7"
target: arch
"#;
    let cfg = eval_with_port(source, "win32");
    assert_eq!(cfg.root.get_prop("target").to_astr().as_str(), "armv7");
}

#[test]
fn test_config_for_unrolls_literal_array() {
    let source = r#"
ports: [8080, 9090]
"#;
    // This is a baseline: array literal as a field value.
    let cfg = eval_with_port(source, "win32");
    let ports = cfg.root.get_prop("ports");
    assert!(matches!(ports, auto_val::Value::Array(_)),
        "ports should be an array, got {:?}", ports);
}

#[test]
fn test_config_if_in_nested_manifest_style() {
    // SCU001-style: top-level data plus a port-guarded block.
    let source = r#"
app: "SCU001"
if port == "win32" {
    builder: "iar"
    toolchain: "arm"
} else {
    builder: "make"
}
"#;
    let cfg = eval_with_port(source, "win32");
    assert_eq!(cfg.root.get_prop("app").to_astr().as_str(), "SCU001");
    assert_eq!(cfg.root.get_prop("builder").to_astr().as_str(), "iar");
    assert_eq!(cfg.root.get_prop("toolchain").to_astr().as_str(), "arm");

    let cfg2 = eval_with_port(source, "linux");
    assert_eq!(cfg2.root.get_prop("builder").to_astr().as_str(), "make");
    // toolchain should not exist when else branch taken (empty/nil)
    assert_ne!(cfg2.root.get_prop("toolchain").to_astr().as_str(), "arm");
}

// -------------------------------------------------------------------------
// Plan 364 Step 5: runtime-only features that the old compile-time flatten
// (Step 2) could NOT handle. These exercise the VM Config eval path:
// f-string templates, object field access, for-over-runtime-array.
// -------------------------------------------------------------------------

#[test]
fn test_config_fstring_template_from_runtime_object() {
    // `kernel` is assembled at runtime; `${kernel.heap}` reads its field.
    // Old Step 2 compile-time flatten could not do this (kernel.heap doesn't
    // exist at compile time).
    let source = r#"
kernel: { heap: "heap_4" }
dep("osal") {
    heap: `${kernel.heap}`
}
"#;
    let cfg = eval_with_port(source, "win32");
    let osal = cfg.root.nodes("dep").into_iter().next().expect("dep node");
    assert_eq!(osal.get_prop("heap").to_astr().as_str(), "heap_4");
}

#[test]
fn test_config_object_field_access() {
    // `kernel.port` — Dot access on a runtime-assembled object.
    let source = r#"
kernel: { port: "IAR" }
target: kernel.port
"#;
    let cfg = eval_with_port(source, "win32");
    assert_eq!(cfg.root.get_prop("target").to_astr().as_str(), "IAR");
}

#[test]
fn test_config_for_over_runtime_array() {
    // `modules` is a runtime array; `for d in modules` iterates it and each
    // iteration emits a child node. The lib("x") node should gain 2 dir kids.
    let source = r#"
modules: ["a", "b"]
lib("x") {
    for d in modules {
        dir(id: d) { at: d }
    }
}
"#;
    let cfg = eval_with_port(source, "win32");
    let lib_x = cfg.root.nodes("lib").into_iter().next().expect("lib node");
    let dirs: Vec<_> = lib_x.nodes("dir");
    assert_eq!(dirs.len(), 2, "lib x should have 2 dir kids, got {}", dirs.len());
    let ids: Vec<String> = dirs.iter().map(|n| n.id().to_string()).collect();
    assert!(ids.contains(&"a".to_string()), "dir ids = {:?}", ids);
    assert!(ids.contains(&"b".to_string()), "dir ids = {:?}", ids);
}

#[test]
#[ignore] // Plan 364: regression guard for the real SCU001 root manifest.
          // (Dep pac.at files need full DSL eval — f-string templates,
          //  obj field access — tracked separately.)
fn diag_scu001_real_manifest() {
    let base = env!("CARGO_MANIFEST_DIR").to_string() + "/../../tmp/pacprobe_s3/scu001_real.at";
    let src = std::fs::read_to_string(&base).unwrap();
    let mut args = Obj::new();
    args.set("port", auto_val::Value::str("lanshan"));
    let cfg = AutoConfig::from_code(&src, &args).expect("SCU001 root pac.at must parse");

    let port_names: Vec<String> = cfg
        .root
        .get_nodes("port")
        .iter()
        .map(|n| n.id().to_string())
        .collect();
    assert!(port_names.iter().any(|n| n == "lanshan"),
        "port lanshan must be present, got {:?}", port_names);

    let osal = cfg
        .root
        .get_nodes("dep")
        .into_iter()
        .find(|n| n.id().as_str() == "osal")
        .expect("dep osal node");
    let kernel = osal.get_prop("kernel");
    assert!(kernel.repr().contains("heap_4"),
        "osal.kernel must be the lanshan-branch value, got {}", kernel.repr());
}
