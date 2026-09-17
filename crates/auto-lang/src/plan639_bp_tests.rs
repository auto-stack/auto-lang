//! PLAN-639 T-04: 跨包 blueprint `.at` 解析——VM/a2ts 双轨消费门（AC-03）。
//!
//! 语料 = `examples/capability-tests/046-bp-import`：pac.at 声明
//! `dep bps { path: "../../../blueprints" }`，app.at 经
//! `use bps.form.login.reference.minimal: LoginForm` 导入共享 bp 参考实现。
//! 断言点 = bp 渲染结构进入双轨产物：
//! - VM 轨：`build_dynamic_component` 编译宿主 + `view_template()` 结构含
//!   bp 子树（Sign in 按钮 / email 输入框 / 记住我 checkbox）；
//! - vue 轨：`generate_component_from_file` 产出的 SFC 含 bp 组件发射面。
//!
//! 零新解析设施——完全复用 PLAN-635 的声明门控 + `resolve_module_path`
//! dotted 候选（裁定见 docs/plans/attachments/639-rename-manifest.md §3）。

use std::path::PathBuf;

/// Locate the 046-bp-import fixture's app.at (repo-root relative, works from
/// the main checkout and from group worktrees).
fn fixture_app_at() -> Option<PathBuf> {
    let repo_root = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)?
        .to_path_buf();
    let app = repo_root
        .join("examples/capability-tests/046-bp-import/src/front/app.at");
    app.is_file().then_some(app)
}

/// Collect every tag + literal text in the template (structure snapshot walk).
fn collect_snapshot(node: &crate::aura::AuraNode, tags: &mut Vec<String>, texts: &mut Vec<String>) {
    use crate::ast::Expr;
    match node {
        crate::aura::AuraNode::Element { tag, props, children, .. } => {
            tags.push(tag.clone());
            for v in props.values() {
                if let crate::aura::AuraPropValue::Expr(Expr::Str(s)) = v {
                    texts.push(s.to_string());
                }
            }
            for c in children {
                collect_snapshot(c, tags, texts);
            }
        }
        crate::aura::AuraNode::Text(crate::aura::AuraTextContent::Literal(s)) => {
            texts.push(s.to_string());
        }
        crate::aura::AuraNode::ForLoop { body, .. } => {
            for c in body {
                collect_snapshot(c, tags, texts);
            }
        }
        crate::aura::AuraNode::Conditional { then_body, else_body, .. } => {
            for c in then_body {
                collect_snapshot(c, tags, texts);
            }
            if let Some(else_body) = else_body {
                for c in else_body {
                    collect_snapshot(c, tags, texts);
                }
            }
        }
        crate::aura::AuraNode::Link { children, .. } => {
            for c in children {
                collect_snapshot(c, tags, texts);
            }
        }
        _ => {}
    }
}

/// Runtime View 收集器：取文本面（Text/Button/Input/容器）。
fn collect_view_texts(view: &crate::ui::view::View<crate::ui::interpreter::DynamicMessage>, out: &mut Vec<String>) {
    use crate::ui::view::View;
    match view {
        View::Text { content, .. } => out.push(content.clone()),
        View::Button { label, content, .. } => {
            out.push(label.clone());
            if let Some(c) = content {
                collect_view_texts(c, out);
            }
        }
        View::Input { placeholder, value, .. } => {
            out.push(placeholder.clone());
            out.push(value.clone());
        }
        View::Row { children, .. } | View::Column { children, .. } => {
            for c in children {
                collect_view_texts(c, out);
            }
        }
        View::AnchorSlot { child, .. } => collect_view_texts(child, out),
        _ => {}
    }
}

/// AC-03 VM 轨：跨包 use 导入的 bp 子树进入运行时渲染结构。
///
/// 判定逻辑：若解析失败，`LoginForm {}` 是未知 tag，渲染折叠为字面
/// `<LoginForm />` 占位文本（aura_view_builder registry 兜底）；解析成功
/// 时 registry 命中 render_child_widget，bp 的 Sign in/Email/placeholder
/// 结构真实进入 View 树。
#[test]
fn t04_vm_track_renders_imported_bp_structure() {
    let Some(app_at) = fixture_app_at() else {
        eprintln!("[SKIP] 046-bp-import fixture not found");
        return;
    };
    let src = std::fs::read_to_string(&app_at).unwrap();
    let path_str = app_at.to_string_lossy().to_string();

    let dc = crate::build_dynamic_component(&src, Some(&path_str))
        .expect("bp-import host must compile with cross-package use import");

    let (view, _debug_map, _probe) = dc.view_with_debug();
    let mut texts = Vec::new();
    collect_view_texts(&view, &mut texts);

    assert!(
        !texts.iter().any(|t| t.contains("<LoginForm")),
        "bp must resolve to a real child widget, not the unknown-tag placeholder; texts: {texts:?}"
    );
    for expected in ["Sign in", "you@example.com", "Remember me", "Email"] {
        assert!(
            texts.iter().any(|t| t.contains(expected)),
            "VM runtime view must contain bp text {expected:?}; snapshot texts: {texts:?}"
        );
    }
}

/// AC-03 vue 轨：a2ts 生成面引用 bp 组件（import + 模板使用）。
///
/// 单文件 generate 的形态与 `auto build` 全量构建一致：宿主 SFC 经
/// import 引用 bp 组件工件（LoginForm.vue 由全量构建按 widget 发射），
/// 组件本体发射面由全量构建验证（见计划证据 `auto build` 记录）。
#[test]
fn t04_vue_track_emits_imported_bp() {
    let Some(app_at) = fixture_app_at() else {
        eprintln!("[SKIP] 046-bp-import fixture not found");
        return;
    };
    let result = crate::ui_gen::generate_component_from_file(
        &app_at,
        crate::ui_gen::ComponentGenOptions::default(),
    )
    .expect("bp-import host must generate SFC with cross-package use import");
    crate::drain_store_extra_files();

    assert!(
        result.vue_code.contains("import LoginForm from '@/components/LoginForm.vue'"),
        "vue SFC must import the imported bp component; code:
{}",
        result.vue_code
    );
    assert!(
        result.vue_code.contains("<LoginForm"),
        "vue SFC template must instantiate the bp component; code:
{}",
        result.vue_code
    );
}
