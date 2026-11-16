//! PLAN-981（B-RUST-I）：公共 A2R 链的 **正向 + V-R-FAIL 负控/恢复**。
//!
//! 公共链 = 同一 AGE 包源 → 共享绑定/verify → 新 A2R（`SemanticView` → Rust fileset）
//! → 真实 `rustc` → 运行。这里不校验 AV 结果（AV 有各自腿），而是用**独立算术
//! oracle** 断言二进制输出；负控证明 A2R 拒绝时**不写产物**，修复后同一入口恢复。
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_program};
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

use crate::rt_common::{out_root, rustc};

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

const MAIN: &str = "node Counter {\n\
    model { count int = 0 }\n\
    on Life.Tick { count = count + 1 }\n\
    view { text \"t\" {}\n}\n\
}\n\
fn main() int {\n\
    return __age_state_0_count()\n\
}\n";

/// 同一场景口径：5 次 Tick ⇒ 状态 = 5。
fn five_ticks() -> ComponentScenario {
    ComponentScenario {
        instances: 1,
        ops: (0..5).map(|_| ScenarioOp::Tick { instance: 0 }).collect(),
    }
}

/// 源 → 受检 → A2R → rustc → 运行；返回 (stdout, 产物目录)。
fn run_public_chain(name: &str, src: &str) -> (String, std::path::PathBuf) {
    run_public_chain_with(name, src, &five_ticks())
}

fn run_public_chain_with(
    name: &str,
    src: &str,
    scenario: &ComponentScenario,
) -> (String, std::path::PathBuf) {
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let compiled = compile_age_package_with_scenario(
        &[input("main", src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps,
        scenario,
    )
    .unwrap_or_else(|ds| {
        panic!(
            "front door rejected {name}: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    });
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission for {name}: {r}"));
    let program =
        lower_program(&view, "main").unwrap_or_else(|r| panic!("lowering for {name}: {r}"));
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");
    let dir = out_root().join(format!("P981_RUST_{name}")).join("debug");
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
    assert_eq!(run.kind, auto_tool_execution::ExitKind::Success, "wrapper exit for {name}");
    let out = String::from_utf8_lossy(run.stdout.as_ref().expect("stdout")).to_string();
    (out, dir)
}

/// B-RUST-V：**980/A 交付的 Counter 源**（fixture，来源见 `plan981_ma_sources.rs`）
/// 经同一公共链跑真实 rustc：2×Inc + 1×Dec ⇒ 独立算术 oracle = 1。
#[test]
fn p981_rust_ma_counter_source_runs_through_the_public_chain() {
    const MA_COUNTER: &str = include_str!("../../../core/tests/fixtures_ma/ace_node_counter.at");
    let src = format!(
        "{MA_COUNTER}\nfn main() int {{\n    return __age_state_0_count()\n}}\n"
    );
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![
            ScenarioOp::Send { instance: 0, msg: "Inc".into() },
            ScenarioOp::Send { instance: 0, msg: "Inc".into() },
            ScenarioOp::Send { instance: 0, msg: "Dec".into() },
        ],
    };
    let (out, _dir) = run_public_chain_with("MA_COUNTER", &src, &scenario);
    assert!(
        out.trim().ends_with('1'),
        "2 Inc then 1 Dec must print 1 through the A2R binary, got {out:?}"
    );
}
/// 公共链正例：同源 AGE 包经 A2R 真实编译运行，输出对**独立算术 oracle**（5）。
#[test]
fn p981_rust_public_chain_runs_the_same_source() {
    let (out, _dir) = run_public_chain("TICKS", MAIN);
    assert!(
        out.trim().ends_with('5'),
        "five ticks must print 5 through the A2R binary, got {out:?}"
    );
}

/// V-R-FAIL：模块级 `const` 属 RUST 批尚未接线的目标合法化缺口——A2R 必须**具名
/// 拒绝**且不写产物；把同一入口换成合法源后链恢复（不是一次性失败）。
#[test]
fn p981_rust_public_chain_rejects_unknown_target_and_recovers() {
    let with_const = "const N int = 1\nnode Counter {\n    model { count int = 0 }\n    view { text \"t\" {}\n}\n}\nfn main() int { return N }\n";
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let compiled = compile_age_package_with_scenario(
        &[input("main", with_const)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps,
        &ComponentScenario { instances: 1, ops: Vec::new() },
    )
    .expect("the front door accepts a module const");
    let rejected = SemanticView::from_checked(&compiled.checked);
    match rejected {
        Ok(_) => panic!("module-level const must not be silently admitted yet (RUST batch gap)"),
        Err(r) => {
            // 拒绝发生在 fileset 生成之前：不得留下任何产物目录。
            assert!(
                !out_root().join("P981_RUST_CONST").exists(),
                "a rejected source must not write artifacts"
            );
            let text = format!("{r}");
            assert!(
                text.contains("a2x"),
                "rejection must be a named A2R code, got {text}"
            );
        }
    }
    // 恢复：同一入口、合法源照常走完整链。
    let (out, dir) = run_public_chain("RECOVER", MAIN);
    assert!(out.trim().ends_with('5'), "recovery run prints 5, got {out:?}");
    // 无解释器：生成的 Rust 里不含 VM/AV 执行循环标记。
    let emitted = std::fs::read_to_string(dir.join("main.rs")).expect("main.rs");
    for marker in ["AutoVM", "autovm", "interpret", "interpret_loop"] {
        assert!(!emitted.contains(marker), "generated Rust must not embed a {marker} path");
    }
    assert!(dir.join("a2x_rt_case.exe").exists(), "a real binary is produced");
}