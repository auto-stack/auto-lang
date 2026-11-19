//! PLAN-1107 T-05: the 978 engineering entry builds a `use.py` project for the
//! A2PY target and the emitted native Python really runs its training program.
#![cfg(windows)]

use std::path::{Path, PathBuf};
use std::process::Command;

use ace_m2_build::driver::{M2Build, ProjectInputs};
use ace_m2_build::identity::TargetKind;

/// The ONE shared PLAN-1107 corpus (byte-identical for every target; the
/// AV/A2PY/m2-build lanes must never keep hand-written copies).
fn section_5_1() -> String {
    std::fs::read_to_string(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../../../crates/auto-lang/test/cases/ace-m2/python-1107/pytorch_train.as"
    ))
    .expect("the shared PLAN-1107 corpus `pytorch_train.as`")
}

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
        sources: vec![("main.at".to_string(), section_5_1())],
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
fn copy_tree(from: &Path, to: &Path) {
    std::fs::create_dir_all(to).unwrap();
    for entry in std::fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let src = entry.path();
        let dst = to.join(entry.file_name());
        if src.is_dir() {
            copy_tree(&src, &dst);
        } else {
            std::fs::copy(&src, &dst).unwrap();
        }
    }
}

fn training_lines(stdout: &str) -> Vec<String> {
    stdout
        .lines()
        .filter(|l| !l.starts_with("value:"))
        .map(|l| l.to_string())
        .collect()
}

fn assert_training(lines: &[String], label: &str) {
    assert_eq!(
        lines.len(),
        62,
        "{label}: 60 losses + shape + pred0, got {lines:?}"
    );
    let first: f32 = lines[0].parse().expect("first loss");
    let last: f32 = lines[59].parse().expect("final loss");
    assert!((first - 0.2370201051235199f32).abs() < 1e-6, "{label} first");
    assert!(
        (last - 0.1013089045882225f32).abs() < 1e-6,
        "{label} final"
    );
    assert_eq!(lines[60], "torch.Size([4, 1])", "{label} shape");
    let pred0: f32 = lines[61].parse().expect("pred0");
    assert!((pred0 - 0.4728480279445648f32).abs() < 1e-5, "{label} pred0");
}

#[test]
fn p1107_t05_sdk_use_py_project_runs_relocates_and_refuses_missing_artifact() {
    let root = scratch("sdk");
    let inputs = ProjectInputs {
        name: "usepy1107sdk".to_string(),
        entry: "main".to_string(),
        caps: vec![
            ace_foreign_contract::manual::SOURCE_CAP.to_string(),
            ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
        ],
        sources: vec![("main.at".to_string(), section_5_1())],
        stdlib: Vec::new(),
    };
    let mut m2 = M2Build::new(&root.join("store"));
    let sdk = root.join("sdk");
    m2.build_sdk(&inputs, &[TargetKind::A2py], &sdk, vec![], "run:py")
        .expect("the a2py SDK builds");

    let run = m2
        .run_sdk(&sdk, TargetKind::A2py, "run:py")
        .unwrap_or_else(|e| panic!("SDK run: {e:?}"));
    assert_eq!(run.exit_code, Some(0), "the entry returns 0: {}", run.stdout);
    assert_training(&training_lines(&run.stdout), "SDK run");

    // V-07: the SDK is relocatable — move it to another root with no dev
    // cache around it and run again.
    let moved = root.join("relocated");
    copy_tree(&sdk, &moved);
    let run2 = m2
        .run_sdk(&moved, TargetKind::A2py, "run:py")
        .unwrap_or_else(|e| panic!("relocated SDK run: {e:?}"));
    assert_eq!(run2.exit_code, Some(0));
    assert_training(&training_lines(&run2.stdout), "relocated SDK run");

    // V-07 negative: a missing declared artifact is a NAMED failure, never a
    // silent/partial run.
    let broken = root.join("broken");
    copy_tree(&sdk, &broken);
    std::fs::remove_file(broken.join("targets").join("a2py").join("a2x_py_runtime.py")).unwrap();
    let err = m2
        .run_sdk(&broken, TargetKind::A2py, "run:py")
        .expect_err("a missing helper must be refused");
    assert!(
        err.code.starts_with("m2build."),
        "named engineering rejection: {:?}",
        err
    );
}
#[test]
fn p1107_t05_sdk_missing_python_package_is_named_at_run_not_emit() {
    // emit never executes the program (so an unavailable package cannot fail
    // the build); the RUN names the missing declared dependency instead of
    // counting a traceback as a run.
    let root = scratch("missingdep");
    let src = "use.py no_such_module_1107: member\nfn main() int {\n    var v = member()\n    return 0\n}";
    let inputs = ProjectInputs {
        name: "usepy1107missing".to_string(),
        entry: "main".to_string(),
        caps: vec![
            ace_foreign_contract::manual::SOURCE_CAP.to_string(),
            ace_foreign_contract::resource::RESOURCE_CAP.to_string(),
        ],
        sources: vec![("main.at".to_string(), src.to_string())],
        stdlib: Vec::new(),
    };
    let mut m2 = M2Build::new(&root.join("store"));
    let sdk = root.join("sdk");
    m2.build_sdk(&inputs, &[TargetKind::A2py], &sdk, vec![], "run:py")
        .expect("emit does not execute the program, so the build is clean");
    let err = m2
        .run_sdk(&sdk, TargetKind::A2py, "run:py")
        .expect_err("a missing runtime module must not be a pass");
    assert_eq!(
        err.code, "m2build.run.runtime-dependency",
        "named dependency rejection: {err:?}"
    );
    assert!(
        err.detail.contains("no_such_module_1107"),
        "the missing module is named: {err:?}"
    );
}
fn corpus_file(name: &str) -> String {
    std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../../crates/auto-lang/test/cases/ace-m2/python-1107")
            .join(name),
    )
    .unwrap_or_else(|e| panic!("shared corpus `{name}`: {e}"))
}

#[test]
fn p1107_v04_sdk_run_of_an_uncaught_python_error_is_not_a_pass() {
    let root = scratch("sdkerr");
    let inputs = ProjectInputs {
        name: "usepy1107err".to_string(),
        entry: "main".to_string(),
        caps: vec![],
        sources: vec![("main.at".to_string(), corpus_file("error_uncaught_1107.at"))],
        stdlib: Vec::new(),
    };
    let mut m2 = M2Build::new(&root.join("store"));
    let sdk = root.join("sdk");
    m2.build_sdk(&inputs, &[TargetKind::A2py], &sdk, vec![], "run:py")
        .expect("emit does not execute the program");
    let run = m2
        .run_sdk(&sdk, TargetKind::A2py, "run:py")
        .expect("the interpreter runs and its failure is observable");
    assert_ne!(
        run.exit_code,
        Some(0),
        "an uncaught Python error never counts as a successful run: {:?}",
        run
    );
    assert!(
        !run.stdout.contains("value:"),
        "no success marker for a failed run: {:?}",
        run.stdout
    );
}

#[test]
fn p1107_v08_plain_auto_project_on_the_a2py_lane_is_unaffected() {
    // Control (V-08): a non-Python program keeps the frozen scalar cold path —
    // no py declaration contract, no py helpers, no native object runtime.
    let root = scratch("plain");
    let inputs = ProjectInputs {
        name: "plain1107".to_string(),
        entry: "main".to_string(),
        caps: vec![],
        sources: vec![(
            "main.at".to_string(),
            "fn main() int {\n    var a = 14\n    return a\n}".to_string(),
        )],
        stdlib: Vec::new(),
    };
    let mut m2 = M2Build::new(&root.join("store"));
    let outcome = m2
        .build(TargetKind::A2py, &inputs)
        .expect("the plain project builds on the a2py lane");
    let runtime = outcome
        .products
        .iter()
        .find(|(rel, _)| rel.ends_with("targets/a2py/a2x_py_runtime.py"))
        .map(|(_, b)| String::from_utf8_lossy(b).to_string())
        .expect("runtime product");
    assert!(
        !runtime.contains("def a2x_py_import("),
        "a non-Python module must not carry the py object helpers"
    );
    assert!(
        !runtime.contains("f0_text_provider.dll"),
        "and must not load the ctypes provider"
    );
    let sdk = root.join("sdk-plain");
    m2.build_sdk(&inputs, &[TargetKind::A2py], &sdk, vec![], "run:py")
        .expect("the plain SDK builds");
    let run = m2
        .run_sdk(&sdk, TargetKind::A2py, "run:py")
        .expect("the plain program runs");
    assert_eq!(run.exit_code, Some(14), "the entry value is observed: {run:?}");
}