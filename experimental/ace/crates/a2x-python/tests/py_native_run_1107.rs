//! PLAN-1107 T-03: the SAME §5.1 source compiled by the shared front door and
//! emitted as NATIVE Python by the A2PY provider, then run by the declared OS
//! interpreter (no PyO3, no ctypes provider, no AV session).
#![cfg(windows)]

use std::path::PathBuf;
use std::process::Command;

use ace_a2x_core::{Family, FilesetRequest, FragmentText, SemanticView, TargetProvider};
use ace_a2x_python::py_contract::py_declaration_contract;
use ace_a2x_python::PythonTargetProvider;
use ace_foreign_contract::manual::SOURCE_CAP;
use ace_foreign_contract::resource::RESOURCE_CAP;

const SECTION_5_1: &str = r#"use.py torch: manual_seed, arange, ones
use.py torch.nn: Linear, MSELoss
use.py torch.optim: SGD

fn main() int {
    manual_seed(0)
    var x = arange(8).reshape(4, 2) / 2
    var target = ones(4, 1)
    var model = Linear(2, 1, bias: false)
    var loss_fn = MSELoss()
    var optimizer = SGD(model.parameters(), lr: 0.1)
    var step = 0
    for step < 60 {
        var prediction = model(x)
        var loss = loss_fn(prediction, target)
        optimizer.zero_grad()
        loss.backward()
        optimizer.step()
        print(loss.to(float))
        step = step + 1
    }
    var prediction = model(x)
    print(prediction.shape)
    print(prediction[0].to(float))
    return 0
}"#;

fn scratch_dir(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("p1107_a2py_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn find_python() -> String {
    for cand in [
        std::env::var("AUTO_PYTHON_HOME")
            .map(|h| format!("{h}\\python.exe"))
            .unwrap_or_default(),
        r"C:\Python314\python.exe".to_string(),
        "python".to_string(),
    ] {
        if cand.is_empty() {
            continue;
        }
        if let Ok(out) = Command::new(&cand).arg("--version").output() {
            if out.status.success() {
                return cand;
            }
        }
    }
    panic!("no usable Python interpreter (set AUTO_PYTHON_HOME)");
}

fn emit_fileset(dir: &PathBuf) -> Vec<String> {
    let contract = py_declaration_contract().expect("declaration contract");
    let caps = vec![SOURCE_CAP.to_string(), RESOURCE_CAP.to_string()];
    let compiled = ace_core::source::compile_aggregates_source_with_foreign(
        SECTION_5_1,
        "p1107-a2py.at",
        &caps,
        &contract,
    )
    .unwrap_or_else(|d| panic!("compile: {d:?}"));
    let view = SemanticView::from_checked(&compiled.checked).expect("view");
    let provider = PythonTargetProvider;
    provider
        .validate_view(&view)
        .expect("py declarations are language operations, not ctypes entries");
    let runtime = provider
        .runtime_source(&view, Family::Collections)
        .expect("runtime");
    assert!(
        runtime.contains("def a2x_py_import("),
        "the py lane appends the native object helpers"
    );
    assert!(
        !runtime.contains("f0_text_provider.dll"),
        "the py lane must not load the ctypes provider"
    );
    let fns: Vec<&ace_a2x_core::FnFacts> = view.functions().iter().collect();
    let texts = provider
        .produce_fragments(&view, &fns, Family::Collections)
        .unwrap_or_else(|e| panic!("lower: {e:?}"));
    let defs: Vec<FragmentText> = fns
        .iter()
        .zip(texts.iter())
        .map(|(f, t)| FragmentText {
            def_index: f.def_index,
            source_name: f.source_name,
            sem_fp: "",
            text: t.as_str(),
        })
        .collect();
    let fileset = provider
        .assemble_fileset(FilesetRequest {
            entry: "main",
            input_hash: "p1107",
            family: Family::Collections,
            fragments: &defs,
            view: Some(&view),
            runtime: &runtime,
            prelude_seed: None,
            origins: None,
            source_sha256: None,
        })
        .unwrap_or_else(|e| panic!("assemble: {e:?}"));
    let mut paths = Vec::new();
    for (name, content) in &fileset.files {
        let p = dir.join(name);
        std::fs::write(&p, content).unwrap();
        paths.push(name.clone());
    }
    paths
}

#[test]
fn p1107_t03_section51_runs_as_native_python() {
    let dir = scratch_dir("train");
    let files = emit_fileset(&dir);
    assert!(files.contains(&"a2x_main.py".to_string()));
    let generated = std::fs::read_to_string(dir.join("generated.py")).unwrap();
    assert!(
        !generated.contains("a2x_foreign_py_"),
        "no ctypes wrapper for py ops: {generated}"
    );
    assert!(
        generated.contains("a2x_py_call") || generated.contains("a2x_py_import"),
        "the native operations are emitted: {generated}"
    );
    let started = std::time::Instant::now();
    let out = Command::new(find_python())
        .arg("a2x_main.py")
        .current_dir(&dir)
        .output()
        .expect("run python");
    let elapsed = started.elapsed();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "native python run failed: {stderr}\n--- generated.py ---\n{generated}"
    );
    // The run-observation marker (`value:<i32>`) is the target run channel
    // (the A2PY analogue of the AV session's success value), not program
    // output; the language-level 62 lines are asserted below it.
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|l| !l.starts_with("value:"))
        .collect();
    assert_eq!(
        lines.len(),
        62,
        "60 losses + shape + pred0; got {lines:?}\nstderr: {stderr}"
    );
    let first: f32 = lines[0].parse().expect("first loss");
    let last: f32 = lines[59].parse().expect("final loss");
    assert!((first - 0.2370201051235199f32).abs() < 1e-6, "first {first}");
    assert!(
        (last - 0.1013089045882225f32).abs() < 1e-6,
        "final {last}"
    );
    assert_eq!(lines[60], "torch.Size([4, 1])", "shape display");
    let pred0: f32 = lines[61].parse().expect("pred0");
    assert!((pred0 - 0.4728480279445648f32).abs() < 1e-5, "pred0 {pred0}");
    eprintln!(
        "T-03 native python training: {} lines in {} ms",
        lines.len(),
        elapsed.as_millis()
    );
}
