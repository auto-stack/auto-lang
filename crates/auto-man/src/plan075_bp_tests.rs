//! PLAN-075: G-6 未修面收口定向测（074-sink-mode G-6：components//bps 与 dep
//! 通道的同文件模块 fn 同病）。夹具 = `examples/capability-tests/048-bp-module-fn`
//! （pac.at 本地库形态 dep + reference 携带文件顶层 `fn count_badge`）。
//!
//! 断言面 = `VueProject::from_workspace` 的 dep 重生成臂：reference 的同文件
//! 模块 fn 必须随 use 池一并重挂（`.with_module_fns`），SFC 自包含——否则
//! computed 调用点有 emission 无定义（vue-tsc TS2304，与 074 jade
//! outline_panel 兄弟通道红同型）。作用域跑法：
//! `cargo nextest run -p auto-man plan075`（auto-man 不在日常 `cargo t` 档）。

use std::path::{Path, PathBuf};

/// 048 夹具定位（主检出与组 worktree 通用——repo-root 相对，plan645 同形）。
fn fixture_root() -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)?
        .to_path_buf();
    let p = root.join("examples/capability-tests/048-bp-module-fn");
    p.join("pac.at").is_file().then_some(p)
}

/// 把 048 夹具复制到临时沙箱再跑 from_workspace——`Pac::resolve()` 会物化
/// `deps/bplocal` junction（mklink /J），junction 只允许出现在仓库外沙箱
/// （Plan 529 红线；`auto build` 类验证一律沙箱形态，075 R-B 裁定）。
/// `label` 参与目录名——并行测试同毫秒启动不得撞沙箱路径（error 32 实证）。
fn sandbox_copy(fixture: &Path, label: &str) -> PathBuf {
    let sandbox = std::env::temp_dir().join(format!(
        "plan075-g6-{label}-{}",
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fn cp_dir(src: &Path, dst: &Path) {
        std::fs::create_dir_all(dst).unwrap();
        for entry in std::fs::read_dir(src).unwrap().flatten() {
            let from = entry.path();
            let to = dst.join(entry.file_name());
            if from.is_dir() {
                cp_dir(&from, &to);
            } else {
                std::fs::copy(&from, &to).unwrap();
            }
        }
    }
    cp_dir(fixture, &sandbox);
    sandbox
}

/// 沙箱清理：先摘 `deps/` 下的 junction（remove_dir 只删链接本身，不穿透），
/// 再整树删除——顺序颠倒会让 remove_dir_all 穿透链接删到夹具源。
fn sandbox_remove(sandbox: &Path) {
    let deps = sandbox.join("deps");
    if let Ok(entries) = std::fs::read_dir(&deps) {
        for entry in entries.flatten() {
            let p = entry.path();
            let _ = std::fs::remove_dir(&p);
        }
    }
    let _ = std::fs::remove_dir_all(sandbox);
}

/// 正断言：dep 通道 reference 的同文件模块 fn 内联进 SFC（G-6 收口面）。
#[test]
fn plan075_dep_channel_same_file_module_fns_inlined_into_sfc() {
    let Some(fixture) = fixture_root() else {
        eprintln!("[SKIP] 048-bp-module-fn fixture not found");
        return;
    };
    let sandbox = sandbox_copy(&fixture, "pos");
    let project = crate::vue::VueProject::from_workspace(&sandbox)
        .expect("048 sandbox project must generate");

    let demo = project
        .components
        .iter()
        .find(|(_, _, _, widget_name)| widget_name == "ModuleFnDemo")
        .expect("dep reference widget ModuleFnDemo must be generated");
    let (_, _, code, _) = demo;
    assert!(
        code.contains("function count_badge"),
        "dep-channel SFC must inline the same-file module fn (G-6); code:\n{code}"
    );
    // 调用点与定义同在（computed 发射 + fn 定义），SFC 自包含。
    assert!(
        code.contains("count_badge("),
        "computed call site must reference the inlined fn; code:\n{code}"
    );

    sandbox_remove(&sandbox);
}

/// 负断言（夹具内自证）：`function count_badge` 定义在全项目 SFC 中有且
/// 仅有一处（ModuleFnDemo 的 dep 臂产物）——消费方 app.at 与其他组件不得
/// 携带定义，排除"定义本来就在别处"的伪证。
#[test]
fn plan075_helper_defined_only_in_bp_component() {
    let Some(fixture) = fixture_root() else {
        eprintln!("[SKIP] 048-bp-module-fn fixture not found");
        return;
    };
    let sandbox = sandbox_copy(&fixture, "neg");
    let project = crate::vue::VueProject::from_workspace(&sandbox)
        .expect("048 sandbox project must generate");

    let definers: Vec<&str> = project
        .components
        .iter()
        .filter(|(_, _, code, _)| code.contains("function count_badge"))
        .map(|(_, _, _, widget_name)| widget_name.as_str())
        .collect();
    assert_eq!(
        definers,
        vec!["ModuleFnDemo"],
        "helper definition must exist only in the bp component SFC; definers: {definers:?}"
    );

    sandbox_remove(&sandbox);
}
