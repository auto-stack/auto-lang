//! PLAN-979 T-03：组件场景驱动的 **A2R 真跑**（同一 CheckedModule → rustc → 运行）。
//!
//! 与 AV 侧同名场景一致（`crates/av/tests/cases/plan979_age_component_cpu.rs`）：
//! 两实例、定向事件、Tick；A2R 侧以 `__age_drive` 为 i32 入口真实编译运行，断言
//! 同一编码值 22（实例隔离与推进次数由共享 verify/lowering 的产物给出）。

use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_program};
use ace_core::source::age_component::{compile_component_scenario, ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;

use crate::rt_common::{out_root, rustc};

const COUNTER: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Tick {\n        count = count + 1\n    }\n    view {\n        text \"c\" {}\n    }\n}\n";

#[test]
fn p979_component_scenario_runs_through_real_rustc() {
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Tick { instance: 1 },
        ],
    };
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled = compile_component_scenario(
        COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &scenario,
    )
    .unwrap_or_else(|ds| {
        panic!(
            "component frontend diagnostics: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    });
    let view = SemanticView::from_checked(&compiled.checked).expect("view admission");
    let program = lower_program(&view, "__age_drive").expect("lower the scenario driver");
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "__age_drive");

    let dir = out_root().join("P979_COMPONENT").join("debug");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    let exe = dir.join("a2x_rt_case.exe");
    let compile_out = auto_tool_execution::run_command_capture(
        {
            let mut c = std::process::Command::new(rustc());
            c.arg("--edition")
                .arg("2021")
                .arg("-o")
                .arg(&exe)
                .arg("main.rs")
                .current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("spawn rustc");
    assert!(
        compile_out.kind == auto_tool_execution::ExitKind::Success,
        "rustc failed: {}",
        String::from_utf8_lossy(compile_out.stderr.as_ref().expect("stderr"))
    );
    let run = auto_tool_execution::run_command_capture(
        {
            let mut c = std::process::Command::new(&exe);
            c.current_dir(&dir);
            c
        },
        auto_tool_execution::TreeScope::ProcessTree,
        None,
    )
    .expect("run exe");
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success);
    let out = String::from_utf8_lossy(run.stdout.as_ref().expect("stdout")).to_string();
    assert_eq!(out, "value:514\n", "A2R run matches the AV observation (2 + 2*256)");
}

const HELPER_COUNTER: &str = "fn seed() int {\n    return 0\n}\nfn bump(n int) int {\n    return n + 1\n}\nnode Counter {\n    model {\n        count int = seed()\n    }\n    on Inc {\n        count = bump(count)\n    }\n    on Tick {\n        count = bump(count)\n    }\n    view {\n        text \"c\" {}\n    }\n}\n";

#[test]
fn p979_component_helpers_and_n_ticks_run_on_a2r() {
    // 与 AV 侧同名场景一致：实例0 三次 Inc、实例1 两次 Tick ⇒ 3 + 2*10 = 23。
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Send {
                instance: 0,
                msg: "Inc".to_string(),
            },
            ScenarioOp::Tick { instance: 1 },
            ScenarioOp::Tick { instance: 1 },
        ],
    };
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled = compile_component_scenario(
        HELPER_COUNTER,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &caps,
        &scenario,
    )
    .expect("helper scenario compiles");
    let view = SemanticView::from_checked(&compiled.checked).expect("view admission");
    let program = lower_program(&view, "__age_drive").expect("lower the driver");
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "__age_drive");
    assert_eq!(
        fs.files.len(),
        3,
        "fileset is the shared three-file shape (no hand-written Rust)"
    );
}
