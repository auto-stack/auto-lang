//! PLAN-979 T-03：组件场景驱动的 **A2R 真跑**（同一 CheckedModule → rustc → 运行）。
//!
//! 与 AV 侧同名场景一致（`crates/av/tests/cases/plan979_age_component_cpu.rs`）：
//! 双实例、定向事件、Tick、helper 参与、typed view；A2R 侧以 `__age_drive` 为 i32
//! 入口真实编译运行并断言同一编码值（256^i 单射：instance0 个位、instance1 256 位）。

use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_program};
use ace_core::source::age_component::{compile_component_scenario, ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;

use crate::rt_common::{out_root, rustc};

const COUNTER: &str = "node Counter {\n    model {\n        count int = 0\n    }\n    on Inc {\n        count = count + 1\n    }\n    on Tick {\n        count = count + 1\n    }\n    view {\n        text \"c\" {}\n    }\n}\n";

/// helper 参与 init/handler，view 读状态（与 AV 侧 HELPER_COUNTER 同源）。
const HELPER_COUNTER: &str = "fn seed() int {\n    return 0\n}\nfn bump(n int) int {\n    return n + 1\n}\nfn label(n int) str {\n    if n > 2 {\n        return \"big\"\n    }\n    return \"small\"\n}\nnode Counter {\n    model {\n        count int = seed()\n    }\n    on Inc {\n        count = bump(count)\n    }\n    on Tick {\n        count = bump(count)\n    }\n    view {\n        text label(count) {}\n    }\n}\n";

/// 完整 A2R 链：编译（共享前门+唯一 verify）→ fileset → **真实 rustc** → 运行 stdout。
fn run_scenario(name: &str, src: &str, scenario: &ComponentScenario) -> String {
    let caps = ["ac.runtime.text.v1".to_string()];
    let compiled = compile_component_scenario(
        src,
        "counter.at",
        LegacyWidgetAlias::Reject,
        &caps,
        scenario,
    )
    .unwrap_or_else(|ds| {
        panic!(
            "component frontend diagnostics for {name}: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    });
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission for {name}: {r}"));
    let program =
        lower_program(&view, "__age_drive").unwrap_or_else(|r| panic!("lowering for {name}: {r}"));
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "__age_drive");

    let dir = out_root()
        .join(format!("P979_COMPONENT_{name}"))
        .join("debug");
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    for (path, content) in &fs.files {
        std::fs::write(dir.join(path), content).unwrap();
    }
    std::fs::write(dir.join("manifest.json"), &fs.manifest).unwrap();

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
        "rustc failed for {name}: {}",
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
    assert_eq!(
        run.kind,
        auto_tool_execution::ExitKind::Success,
        "wrapper exit for {name}"
    );
    String::from_utf8_lossy(run.stdout.as_ref().expect("stdout")).to_string()
}

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
    let out = run_scenario("PLAIN", COUNTER, &scenario);
    // instance0 = 2（个位）、instance1 = 2（256 位）→ 2 + 512。
    assert_eq!(out, "value:514\n", "A2R run matches the AV observation");
}

#[test]
fn p979_component_helpers_run_on_a2r_like_the_av_leg() {
    // 与 AV 侧 `p979_component_helpers_participate_and_view_reads_state` 同一场景：
    // instance0 三次 Inc → 3（个位）、instance1 一次 Inc → 1（256 位）→ 259。
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
            ScenarioOp::Send {
                instance: 1,
                msg: "Inc".to_string(),
            },
        ],
    };
    let out = run_scenario("HELPER", HELPER_COUNTER, &scenario);
    assert_eq!(out, "value:259\n", "A2R runs the helper scenario");
}

#[test]
fn p979_component_n_tick_matrix_runs_on_a2r() {
    // 与 AV 侧 tick 矩阵同源：instance0 四次、instance1 两次 → 4 + 2*256 = 516。
    let scenario = ComponentScenario {
        instances: 2,
        ops: vec![
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 1 },
            ScenarioOp::Tick { instance: 1 },
        ],
    };
    let out = run_scenario("TICKS", HELPER_COUNTER, &scenario);
    assert_eq!(out, "value:516\n");
}
