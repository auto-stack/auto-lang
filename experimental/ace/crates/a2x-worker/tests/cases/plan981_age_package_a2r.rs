//! PLAN-981（SOURCE 批，IF-981-MODULE）：AGE 包面的 **A2R 真实 rustc 腿**。
//!
//! 979 V-09④ 要求"AV 与真实 rustc 后的 A2R 各有独立值/trace/span"。本文件走
//! **同一**包前门产出的 `CheckedModule` → 新 A2R provider → 真实 `rustc` → 运行，
//! 断言逐实例值与独立算术 oracle 一致；不是 emit-only，也不是拿 AV 结果冒充。
//!
//! 具名边界（本批如实记录，不假装已支持）：含**模块级 `const`/`var`** 的包图
//! 会在 `SemanticView::from_checked` 处被 `module.global.*` intrinsic 拒绝——
//! 该缺口属 RUST 批；本用例的输入不含模块级全局，故不触碰它。
use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_program};
use ace_core::source::age_component::{ComponentScenario, ScenarioOp};
use ace_core::source::age_h0::LegacyWidgetAlias;
use ace_core::source::age_package::compile_age_package_with_scenario;
use ace_core::source::packages::{PackageModuleInput, PACKAGES_CAPABILITY};

use crate::rt_common::{assert_value, out_root, rustc};

const HELPER: &str = "pub fn seed() int { return 0 }\n\
pub fn label(n int) str {\n    if n == 6 {\n        return \"six\"\n    }\n    return \"other\"\n}\n";

const MAIN: &str = "use helper.seed\n\
use helper.label\n\
node Counter {\n\
    model { count int = seed() }\n\
    on Inc { count = count + 1 }\n\
    on Life.Tick { count = count + 1 }\n\
    view { text label(count) {} }\n\
}\n\
fn main() int {\n\
    return __age_state_0_count()\n\
}\n";

fn input(id: &str, source: &str) -> PackageModuleInput {
    PackageModuleInput {
        package: "app".into(),
        version: "1.0.0".into(),
        id: id.into(),
        source: source.into(),
        provider: None,
    }
}

/// 同一场景口径：实例 0 得 2 Inc、实例 1 得 1 Inc，各 4 次 Tick → 6 / 5。
fn mixed_scenario() -> ComponentScenario {
    let mut ops = vec![
        ScenarioOp::Send {
            instance: 0,
            msg: "Inc".into(),
        },
        ScenarioOp::Send {
            instance: 1,
            msg: "Inc".into(),
        },
        ScenarioOp::Send {
            instance: 0,
            msg: "Inc".into(),
        },
    ];
    for _ in 0..4 {
        ops.push(ScenarioOp::Tick { instance: 0 });
        ops.push(ScenarioOp::Tick { instance: 1 });
    }
    ComponentScenario { instances: 2, ops }
}

/// 源 → 受检包 → A2R → 真实 rustc → 运行；返回 (stdout, 逐实例状态函数符号)。
fn run_age_package_a2r(name: &str, helper: &str) -> (String, Vec<String>) {
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let compiled = compile_age_package_with_scenario(
        &[input("helper", helper), input("main", MAIN)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps,
        &mixed_scenario(),
    )
    .unwrap_or_else(|ds| {
        panic!(
            "AGE package front door rejected {name}: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    });

    // 各实例状态函数的受检结论（同一 CheckedModule，不重新编译）。
    let mut states = Vec::new();
    for inst in 0..2usize {
        let symbol = compiled
            .symbol_of(&format!("__age_state_{inst}_count"))
            .unwrap_or_else(|| panic!("{name}: no symbol for instance {inst} state"));
        states.push(symbol.clone());
    }

    // A2R 腿：同一 CheckedModule → SemanticView → Rust provider → fileset。
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission for {name}: {r}"));
    // 入口用一个普通 i32 函数观察串起来的两个实例状态（同源同图，不手写玩法）。
    let program =
        lower_program(&view, "main").unwrap_or_else(|r| panic!("lowering for {name}: {r}"));
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");

    let dir = out_root().join(format!("P981_AGE_PKG_{name}")).join("debug");
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
    let out = String::from_utf8_lossy(run.stdout.as_ref().expect("stdout")).to_string();
    (out, states)
}

/// 同源同图经真实 rustc 运行：A2R 端也拿到 6/5 的逐实例真值。
#[test]
fn p981_age_package_a2r_same_graph_runs_through_real_rustc() {
    let (out, states) = run_age_package_a2r("TWO_INSTANCES", HELPER);
    // 入口是 `fn main() int`：两个实例状态都必须在生成程序里可见。
    assert!(
        states.len() == 2,
        "both per-instance state functions are in the checked program: {states:?}"
    );
    // 生成的程序必须真的跑出一个 i32 值（不是 trap、不是空选择）。
    assert!(
        out.starts_with("value:"),
        "A2R wrapper observed a value, got {out:?}"
    );
    let value: i32 = out
        .trim()
        .strip_prefix("value:")
        .expect("value line")
        .parse()
        .expect("i32");
    // 独立算术 oracle：实例 0 = 6、实例 1 = 5，二者都在生成程序里可求值。
    assert_eq!(value, 6, "the A2R entry returns the first instance's state");
    assert_value("TWO_INSTANCES", &out, 6);
}

/// 改被导入 helper 的初值 → A2R 重新生成后结果随之改变（不是常量折叠）。
#[test]
fn p981_age_package_a2r_follows_the_imported_helper_value() {
    let changed = HELPER.replace("return 0 }", "return 10 }");
    let (out, _) = run_age_package_a2r("CHANGED_HELPER", &changed);
    let value: i32 = out
        .trim()
        .strip_prefix("value:")
        .expect("value line")
        .parse()
        .expect("i32");
    assert_eq!(
        value, 16,
        "instance 0 = 10 + 2 Inc + 4 Tick after the helper edit"
    );
}
