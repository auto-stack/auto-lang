//! PLAN-979 T-03：**同一 AGE 源的双后端腿**（A2R→真实 rustc→运行）。
//!
//! 源同时含领域声明（`node`/`view`/`text`）与普通 Auto 代码（helper + `fn main()
//! int`）：AGE 前门经 T-02 接面产出受检描述；A2R 腿直接消费同一 `CheckedModule`
//! （`SemanticView::from_checked`）→ Rust fileset → 真实 rustc → 运行 → 父侧断言。
//! 不手写 Rust、不以 emit 成功冒充运行、不复制语义（值来自共享 verify/lowering）。

use ace_a2x_core::SemanticView;
use ace_a2x_rust::{assemble_record_text_file_set, lower_program};
use ace_core::source::age_h0::{compile_age_h0, LegacyWidgetAlias};

use crate::rt_common::{assert_value, out_root, rustc};

const RUNTIME_TEXT_CAP: &str = "ac.runtime.text.v1";

/// 同一份 AGE 源：领域文本叶 + 普通 helper + i32 `main`（A2R 入口契约）。
const SOURCE: &str = r#"fn counter_text(n int) str {
    if n > 2 {
        return "many"
    }
    return "few"
}
node Counter {
    view {
        text counter_text(2) {}
    }
}
fn main() int {
    let t = counter_text(3)
    return t.len_bytes()
}
"#;

/// AGE 源 → A2R 真实编译运行；返回 (stdout, stderr) 与受检描述的文本值。
fn run_age_chain(name: &str, src: &str) -> ((String, String), String) {
    let caps = [RUNTIME_TEXT_CAP.to_string()];
    let compiled =
        compile_age_h0(src, name, LegacyWidgetAlias::Reject, &caps).unwrap_or_else(|ds| {
            panic!(
                "AGE frontend diagnostics for {name}: {}",
                ds.iter()
                    .map(|d| format!("{}: {}", d.code, d.message))
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        });
    let leaf = &compiled.description.nodes[0].view.texts[0];

    // A2R 腿：同一 CheckedModule → SemanticView → Rust provider → fileset。
    let view = SemanticView::from_checked(&compiled.checked)
        .unwrap_or_else(|r| panic!("view admission for {name}: {r}"));
    let program =
        lower_program(&view, "main").unwrap_or_else(|r| panic!("lowering for {name}: {r}"));
    let fs = assemble_record_text_file_set(&view, &program, &compiled.input_hash, "main");

    let dir = out_root().join(format!("P979_AGE_{name}")).join("debug");
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
    let err = String::from_utf8_lossy(run.stderr.as_ref().expect("stderr")).to_string();
    ((out, err), leaf.fn_name.clone())
}

#[test]
fn p979_age_a2r_same_source_runs_through_real_rustc() {
    let ((out, _err), hidden_fn) = run_age_chain("COUNTER", SOURCE);
    // 普通 Auto helper 的算术结果经 A2R 真实运行：len("many") = 4。
    assert_value("COUNTER", &out, 4);
    // 领域描述存在：文本叶的单一声明角色由 Schema 决定，隐藏求值函数名按包前缀生成。
    assert_eq!(hidden_fn, "__age_h0_text_0");
}

#[test]
fn p979_age_a2r_avoids_artifacts_for_rejected_source_and_recovers() {
    // 坏源：共享检查拒绝 → 不产出 fileset（这里直接断言前门拒绝并给出原 span）。
    let bad = SOURCE.replace("text counter_text(2) {}", "text 1 + 2 {}");
    let caps = [RUNTIME_TEXT_CAP.to_string()];
    let ds = compile_age_h0(&bad, "bad.at", LegacyWidgetAlias::Reject, &caps)
        .expect_err("non-text leaf rejected before any artifact");
    assert_eq!(ds[0].code, "source.age.text-type");
    let at = bad.find("1 + 2").expect("expr offset");
    assert_eq!(ds[0].span.start, at);
    // 修复后同入口恢复正常：同一源照常运行。
    let ((out, _), _) = run_age_chain("COUNTER_FIXED", SOURCE);
    assert_value("COUNTER_FIXED", &out, 4);
}

/// 双后端矩阵：同一源在 A2R 侧真实运行的值（AV 侧见
/// `crates/av/tests/cases/plan979_age_h0_cpu.rs` 的同名矩阵）。
const MATRIX: [(&str, &str, i32); 3] = [
    (
        "LITERAL",
        "node Hello {\n    view {\n        text \"Hello\" {}\n    }\n}\nfn main() int {\n    return \"Hello\".len_bytes()\n}\n",
        5,
    ),
    (
        "HELPER",
        "fn counter_text(n int) str {\n    if n > 2 {\n        return \"many\"\n    }\n    return \"few\"\n}\nnode Counter {\n    view {\n        text counter_text(2) {}\n    }\n}\nfn main() int {\n    let t = counter_text(3)\n    return t.len_bytes()\n}\n",
        4,
    ),
    (
        "ARITH",
        "fn twice(n int) int {\n    return n + n\n}\nnode N {\n    view {\n        text \"ab\" {}\n    }\n}\nfn main() int {\n    return twice(3) + \"ab\".len_bytes()\n}\n",
        8,
    ),
];

#[test]
fn p979_age_a2r_parity_matrix_runs_each_source_through_real_rustc() {
    for (name, src, expected) in MATRIX {
        let ((out, _err), hidden) = run_age_chain(&format!("MATRIX_{name}"), src);
        assert_value(&format!("MATRIX_{name}"), &out, expected);
        assert_eq!(hidden, "__age_h0_text_0", "{name}: single hidden evaluator");
    }
}
