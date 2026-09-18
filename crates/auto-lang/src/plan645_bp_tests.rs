//! PLAN-645: bps 扫描 fn 转译（DEBTS 070 第二行偿还）+ 组合形态回归样本。
//!
//! 语料 = `blueprints/navigation/filetree/reference/default.at`（组合形态
//! FileTree，bare 导入包内支撑件 tree_util/tree_icon）+ 消费方夹具
//! `examples/capability-tests/047-bp-compose`（pac.at dep bps + bps 限定
//! use 导入）。断言点 = plan522 式 helper 转译的 bp 面：
//! - 正：reference 的跨文件 fn 导入按依赖闭包内联进 SFC（flatten_tree →
//!   has_id/ext_icon 到不动点），SFC 自包含（vue-tsc TS2304 断裂面修复）；
//! - 负：未导入符号（filter_tree/collect_ids）不入 SFC（仅被引符号闭包，
//!   Q-1 默认裁定）；
//! - 消费方：bps 限定组合导入发射组件 import（046 L1 通道同形）。
//!
//! 纯 vue 轨（generate_component_from_file），零 ui-iced 依赖——与
//! plan639/640（VM+a2ts 双轨门控）谱系分工。

use std::path::PathBuf;

/// Repo-root relative path resolution (works from the main checkout and from
/// group worktrees — same shape as plan639_bp_tests::fixture_app_at).
fn repo_file(rel: &str) -> Option<PathBuf> {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)?
        .to_path_buf();
    let p = root.join(rel);
    p.is_file().then_some(p)
}

fn generate(path: &PathBuf) -> crate::ui_gen::GeneratedComponent {
    let result = crate::ui_gen::generate_component_from_file(
        path,
        crate::ui_gen::ComponentGenOptions::default(),
    )
    .expect("bp composition source must generate");
    crate::drain_store_extra_files();
    result
}

/// AC-01 正断言：bp reference 的跨文件 fn 导入内联进 SFC（SFC 自包含）。
#[test]
fn t02_bp_reference_cross_file_fns_inlined_into_sfc() {
    let Some(reference_at) =
        repo_file("blueprints/navigation/filetree/reference/default.at")
    else {
        eprintln!("[SKIP] filetree reference fixture not found");
        return;
    };
    let result = generate(&reference_at);

    // 直接被引符号（use tree_util: flatten_tree, toggle_id）。
    for fn_name in ["flatten_tree", "toggle_id"] {
        assert!(
            result.vue_code.contains(&format!("function {fn_name}")),
            "SFC must inline imported helper `{fn_name}`; code:\n{}",
            result.vue_code
        );
    }
    // 依赖闭包到不动点：flatten_tree → has_id / ext_icon。
    for fn_name in ["has_id", "ext_icon"] {
        assert!(
            result.vue_code.contains(&format!("function {fn_name}")),
            "SFC must inline transitive closure helper `{fn_name}`; code:\n{}",
            result.vue_code
        );
    }
    // 包内支撑件 widget 导入（use tree_icon: TreeIcon → 组件 import）。
    assert!(
        result
            .vue_code
            .contains("import TreeIcon from '@/components/TreeIcon.vue'"),
        "SFC must import the TreeIcon support widget; code:\n{}",
        result.vue_code
    );
}

/// AC-01 负断言：未导入符号不入 SFC（仅被引符号闭包——Q-1 默认裁定）。
#[test]
fn t02_unimported_helpers_not_pulled() {
    let Some(reference_at) =
        repo_file("blueprints/navigation/filetree/reference/default.at")
    else {
        eprintln!("[SKIP] filetree reference fixture not found");
        return;
    };
    let result = generate(&reference_at);
    // filter_tree / collect_ids 未被 reference 导入，也不在 flatten_tree/
    // toggle_id 的调用闭包内——不得整池倾倒。
    for fn_name in ["filter_tree", "collect_ids"] {
        assert!(
            !result.vue_code.contains(&format!("function {fn_name}")),
            "SFC must NOT inline unimported helper `{fn_name}` (closure-only discipline)"
        );
    }
}

/// AC-01 消费方断言：047 组合夹具（bps 限定 use）发射组件 import，
/// 与 046 L1 通道同形。
#[test]
fn t03_compose_fixture_consumer_imports_bp_component() {
    let Some(app_at) =
        repo_file("examples/capability-tests/047-bp-compose/src/front/app.at")
    else {
        eprintln!("[SKIP] 047-bp-compose fixture not found");
        return;
    };
    let result = generate(&app_at);
    assert!(
        result
            .vue_code
            .contains("import FileTree from '@/components/FileTree.vue'"),
        "consumer SFC must import the composed bp component; code:\n{}",
        result.vue_code
    );
    assert!(
        result.vue_code.contains("<FileTree"),
        "consumer template must render the bp component; code:\n{}",
        result.vue_code
    );
}
