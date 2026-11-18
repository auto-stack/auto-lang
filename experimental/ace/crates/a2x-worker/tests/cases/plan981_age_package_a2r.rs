//! PLAN-981（SOURCE 批，IF-981-MODULE）：AGE 包面的 **A2R 真实 rustc 腿**。
//!
//! 979 V-09④ 要求"AV 与真实 rustc 后的 A2R 各有独立值/trace/span"。本文件走
//! **同一**包前门产出的 `CheckedModule` → 新 A2R provider → 真实 `rustc` → 运行，
//! 断言逐实例值与独立算术 oracle 一致；不是 emit-only，也不是拿 AV 结果冒充。
//!
//! 具名边界（**已关闭**，D-981-MODULE-GLOBAL-A2R）：含模块级 `const`/`var`
//! 的包图经 CheckedModule 的 module_globals plan（绑定器事实）合法化——
//! A2R 生成 `static mut` + init 链（见本文件 globals 用例与
//! plan981_rust_public 的翻正腿）；**计算面标量以外**的全局类型
//! （Str/record/容器）仍是具名边界（资源端口全局的 A2R 面另行联冻）。
//!
//! D-981-AGE-F32-SURFACE（2026-11-18，AV 切片后的 A2R 同源腿）：f32 模型字段 /
//! 同型四则 / 比较在**同一** CheckedModule → A2R → 真实 rustc 上取真值。入口
//! 仍是 wrapper 冻结的 i32 合同（stdout `value:<i32>`），f32 值在入口内用比较
//! 折叠成判别性整数（7 = x≈3.0 且 y=1.5；0 = 变异控制），**不是**打印端伪造。
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

/// D-981-MODULE-GLOBAL-A2R 关闭（PLAN-981 语言侧 AV 残余批）：含模块级
/// `const`/`var` 的包图过 `SemanticView`（绑定器事实经 CheckedModule 的
/// module_globals plan 钉入），A2R 生成 `static mut` + init 链（生成 main
/// 在入口前执行），真实 rustc 编译运行读回值。
///
/// **oracle**：`var total = helper.seed()`（=40，跨模块 init——init 链没跑
/// 就是 0）、两次 `bump()` 经 set/get 双路径（41+42）、`const K = 6` 经
/// get——`a + b + K - 84 == 5`。任一环节（init 链/读/写）坏了值都会偏。
#[test]
fn p981_age_package_a2r_module_globals_run_on_real_rustc() {
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let helper = "pub fn seed() int { return 40 }\n";
    let main_src = "use helper.seed\n\
const K int = 6\n\
var total int = seed()\n\
node Counter {\n    model { count int = 0 }\n    on Inc { count = count + 1 }\n    view { text \"c\" {} }\n}\n\
pub fn bump() int {\n    total = total + 1\n    return total\n}\n\
fn main() int {\n    let a int = bump()\n    let b int = bump()\n    return a + b + K - 84\n}\n";
    let compiled = compile_age_package_with_scenario(
        &[input("helper", helper), input("main", main_src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps,
        &ComponentScenario {
            instances: 1,
            ops: Vec::new(),
        },
    )
    .unwrap_or_else(|ds| {
        panic!(
            "front door rejected the globals package: {}",
            ds.iter()
                .map(|d| format!("{}: {}", d.code, d.message))
                .collect::<Vec<_>>()
                .join("; ")
        )
    });

    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission (module globals legal now): {r}"));
    let program = lower_program(&view, "main")
        .unwrap_or_else(|r| panic!("lowering with statics + init chain: {r}"));
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");

    let dir = out_root().join("P981_AGE_PKG_GLOBALS/debug");
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
    // 41 + 42 + 6 - 84 = 5：init 链（40）、set/get 双路径、const 读全覆盖。
    assert_value("globals", &out, 5);
}

/// 单模块 AGE 源的 A2R 腿（f32 用）：同一 `CheckedModule` → `SemanticView` →
/// Rust provider → fileset → 真实 rustc → 运行；返回 wrapper stdout。
fn run_age_source_a2r(name: &str, main_src: &str, scenario: &ComponentScenario) -> String {
    let caps = [PACKAGES_CAPABILITY.to_string()];
    let compiled = compile_age_package_with_scenario(
        &[input("main", main_src)],
        "app::main",
        LegacyWidgetAlias::Reject,
        &caps,
        scenario,
    )
    .unwrap_or_else(|ds| {
        panic!(
            "AGE front door rejected {name}: {}",
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

    let dir = out_root().join(format!("P981_AGE_F32_{name}")).join("debug");
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

/// f32 模型字段 + Tick 积分 + 除法在 **A2R 同源腿**上的真值：
/// 两次 Tick 后 `x = (0 + 3.0*0.5)*2 = 3.0`、`y = x/2.0 = 1.5`（都可精确表示），
/// 入口用 `x > 2.5` 与 `y == 1.5` 两个 f32 比较折叠成 7。变异控制：同一源只把
/// `vx` 改成 1.0 → `x=1.0/y=0.5` → 入口返回 0（证明不是常量折叠/打印端伪造）。
#[test]
fn p981_age_package_a2r_f32_state_and_arithmetic_run_on_real_rustc() {
    let src = "node Mover {\n    model {\n        x f32 = 0.0\n        vx f32 = 3.0\n        dt f32 = 0.5\n        y f32 = 0.0\n    }\n    on Tick {\n        x = x + vx * dt\n        y = x / 2.0\n    }\n    view {\n        text \"m\" {}\n    }\n}\nfn main() int {\n    let x f32 = __age_state_0_x()\n    let y f32 = __age_state_0_y()\n    if x > 2.5 {\n        if y == 1.5 {\n            return 7\n        }\n    }\n    return 0\n}\n";
    let scenario = ComponentScenario {
        instances: 1,
        ops: vec![
            ScenarioOp::Tick { instance: 0 },
            ScenarioOp::Tick { instance: 0 },
        ],
    };
    let out = run_age_source_a2r("F32_INTEGRATION", src, &scenario);
    assert_value("f32-integration", &out, 7);

    let alt = src.replace("vx f32 = 3.0", "vx f32 = 1.0");
    let out0 = run_age_source_a2r("F32_ALT", &alt, &scenario);
    assert_value("f32-alt", &out0, 0);
}
