//! PLAN-1129 T-04: the 978 engineering entry builds a `use.py` project whose
//! Python call takes a RUNTIME float for the A2PY target, and the emitted
//! native Python really runs it — the same author source the corpus runs on
//! both ends.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

use ace_m2_build::driver::{M2Build, ProjectInputs};
use ace_m2_build::identity::TargetKind;

const CORPUS: &str = include_str!(concat!(
    env!("CARGO_MANIFEST_DIR"),
    "/../../../../crates/auto-lang/test/cases/language/script/python-float-args.case.ad"
));

fn case(name: &str) -> &'static str {
    CORPUS
        .split_once(&format!("## Case: {name}\n"))
        .expect("named case")
        .1
        .split("\n## Case: ")
        .next()
        .expect("case body")
}

fn fence<'a>(section: &'a str, language: &str) -> &'a str {
    let marker = format!("```{language}\n");
    let (_, body) = section.split_once(&marker).expect("opening fence");
    body.split_once("```").expect("closing fence").0
}

fn scratch(name: &str) -> PathBuf {
    let scope = std::env::var_os("ACE_PLAN_1129_SCRATCH")
        .map(PathBuf::from)
        .unwrap_or_else(std::env::temp_dir);
    let dir = scope.join(format!("p1129_m2_{name}_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    dir
}

fn find_python() -> String {
    for cand in [
        std::env::var("A2PY_PYTHON").unwrap_or_default(),
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
    panic!("no usable Python interpreter (set A2PY_PYTHON)");
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

fn product<'a>(products: &'a [(String, Vec<u8>)], suffix: &str) -> &'a str {
    products
        .iter()
        .find(|(rel, _)| rel.ends_with(suffix))
        .map(|(_, bytes)| std::str::from_utf8(bytes).expect("utf-8 product"))
        .unwrap_or_else(|| panic!("product `{suffix}`"))
}

/// The engineering entry compiles the corpus's own PyTorch source and the
/// emitted native Python reproduces the corpus oracle: the optimizer holds
/// the learning rate each run computed, and both runs train as they do on
/// the case runner.
#[test]
fn p1129_t04_engineering_entry_passes_the_computed_learning_rate() {
    let began = std::time::Instant::now();
    let training = case("torch-lr-computed");
    let source = fence(training, "auto");
    let (_, av) = training.split_once("### Probe av\n").expect("AV probe");
    let oracle: Vec<&str> = fence(av, "text").lines().collect();
    assert_eq!(oracle.len(), 12, "two runs of six observations");

    let root = scratch("project");
    let inputs = ProjectInputs {
        name: "floatargs1129".to_string(),
        entry: "main".into(),
        caps: vec![
            ace_foreign_contract::manual::SOURCE_CAP.to_string(),
            ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
        ],
        sources: vec![("main.at".into(), source.to_string())],
        stdlib: Vec::new(),
        rust_provider: None,
    };
    let mut m2 = M2Build::new(&root.join("store"));
    let outcome = m2
        .build(TargetKind::A2py, &inputs)
        .expect("the a2py build of a use.py project with a runtime float argument");
    assert!(outcome.run.is_none(), "emit must not run torch");
    drop(m2);

    let generated = product(&outcome.products, "targets/a2py/generated.py");
    // the rate reaches SGD as the computed value at f32 precision — never
    // as a number the compiler remembered
    let sgd = generated
        .lines()
        .find(|line| line.contains(r#""lr": "#))
        .expect("the `SGD(model.parameters(), lr: rate)` call");
    assert!(sgd.contains("a2x_f32_round("), "{sgd}");
    assert!(
        !sgd.contains(r#""lr": 0."#),
        "no folded learning rate: {sgd}"
    );

    let run_dir = scratch("run");
    write_tree(&run_dir, &outcome.products);
    let started = std::time::Instant::now();
    let out = Command::new(find_python())
        .args(["-I", "-B", "a2x_main.py"])
        .current_dir(run_dir.join("targets").join("a2py"))
        .output()
        .expect("run python");
    let elapsed = started.elapsed();
    let stdout = String::from_utf8_lossy(&out.stdout)
        .to_string()
        .replace("\r\n", "\n");
    let stderr = String::from_utf8_lossy(&out.stderr).to_string();
    assert!(
        out.status.success(),
        "native python run failed: {stderr}\n{generated}"
    );
    let lines: Vec<&str> = stdout
        .lines()
        .filter(|line| !line.starts_with("value:"))
        .collect();
    assert_eq!(lines, oracle, "the corpus oracle, line for line");
    // the optimizer really holds what each run computed
    assert_eq!(lines[0], "0.03125", "run 1: the rate Auto computed");
    assert_eq!(lines[1], "0.03125", "run 1: the optimizer's own lr");
    assert_eq!(lines[6], "0.09375", "run 2: the rate Auto computed");
    assert_eq!(lines[7], "0.09375", "run 2: the optimizer's own lr");
    assert_ne!(
        lines[3], lines[9],
        "the two rates train different loss paths"
    );
    assert_ne!(lines[5], lines[11], "and reach different predictions");

    let _ = std::fs::remove_dir_all(&root);
    let _ = std::fs::remove_dir_all(&run_dir);
    eprintln!(
        "p1129 engineering entry: {} lines, python {} ms, whole test {:?}",
        lines.len(),
        elapsed.as_millis(),
        began.elapsed()
    );
}
