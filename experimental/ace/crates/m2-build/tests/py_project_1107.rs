//! PLAN-1107 T-05: the 978 engineering entry builds a `use.py` project for the
//! A2PY target and the emitted native Python really runs its training program.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

use ace_m2_build::driver::{M2Build, ProjectInputs};
use ace_m2_build::identity::TargetKind;

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

fn scratch(name: &str) -> PathBuf {
    let dir = std::env::temp_dir().join(format!("p1107_m2_{name}_{}", std::process::id()));
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

fn write_tree(dir: &Path, products: &[(String, Vec<u8>)]) {
    for (rel, bytes) in products {
        let path = dir.join(rel);
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        std::fs::write(&path, bytes).unwrap();
    }
}

#[test]
fn p1107_t05_ac_build_a2py_use_py_project_runs_native_python() {
    let root = scratch("project");
    let inputs = ProjectInputs {
        name: "usepy1107".to_string(),
        entry: "main".to_string(),
        caps: vec![
            ace_foreign_contract::manual::SOURCE_CAP.to_string(),
            ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
        ],
        sources: vec![("main.at".to_string(), SECTION_5_1.to_string())],
        stdlib: Vec::new(),
    };
    let mut m2 = M2Build::new(&root.join("store"));
    let outcome = m2
        .build(TargetKind::A2py, &inputs)
        .expect("the a2py build of a use.py project");
    assert!(outcome.run.is_none(), "emit must not run torch");

    // the emitted products are the native Python fileset (no ctypes wrapper)
    let generated = outcome
        .products
        .iter()
        .find(|(rel, _)| rel.ends_with("targets/a2py/generated.py"))
        .map(|(_, b)| String::from_utf8_lossy(b).to_string())
        .expect("generated.py product");
    assert!(
        !generated.contains("a2x_foreign_py_"),
        "py operations must be native, not ctypes wrappers"
    );
    assert!(
        generated.contains("a2x_py_import") && generated.contains("a2x_py_call"),
        "native operations emitted"
    );
    let runtime = outcome
        .products
        .iter()
        .find(|(rel, _)| rel.ends_with("targets/a2py/a2x_py_runtime.py"))
        .map(|(_, b)| String::from_utf8_lossy(b).to_string())
        .expect("runtime product");
    assert!(
        !runtime.contains("f0_text_provider.dll"),
        "no ctypes provider is loaded by the py lane"
    );

    // run the declared interpreter over the emitted fileset
    let run_dir = scratch("run");
    write_tree(&run_dir, &outcome.products);
    let target_dir = run_dir.join("targets").join("a2py");
    let started = std::time::Instant::now();
    let out = Command::new(find_python())
        .arg("a2x_main.py")
        .current_dir(&target_dir)
        .output()
        .expect("run python");
    let elapsed = started.elapsed();
    let stdout = String::from_utf8_lossy(&out.stdout).to_string();
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "native python run failed: {stderr}\n{}",
        generated
    );
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
        "T-05 ac-build a2py use.py project: {} lines in {} ms",
        lines.len(),
        elapsed.as_millis()
    );
}