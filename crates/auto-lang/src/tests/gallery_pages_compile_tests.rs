//! P499-6 清偿:widgets-gallery 全页可编译冒烟(lib 级,入 cargo t/tf 门禁)。
//!
//! 盲区成因:gallery_golden / docs_gen 都是 tests/ 集成目标,不在日常
//! 门禁(cargo t/tf)运行集内;kitchen-sink.at 解析错(Plan 497 提交,
//! "Expected term, got RBrace" ×20)因此溜进 master,vue serve 编译该页
//! 失败只打 Warning 跳页 → router 引用悬空 → widgets-gallery 整站 500。
//! (解析器后续修复已顺带治愈该文件;本测试钉住防线,杜绝复发。)
//!
//! 冒烟口径与 vue serve 完全一致(cmd_vue.rs Phase 3):逐页调
//! `ui_build_shadcn_with_widgets`,要求 Ok 且产出非空 SFC。

use std::fs;
use std::path::{Path, PathBuf};

fn front_pages_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../examples/widgets-gallery/src/front/pages")
}

fn collect_pages(dir: &Path, out: &mut Vec<PathBuf>) {
    let rd = match fs::read_dir(dir) {
        Ok(rd) => rd,
        Err(_) => return,
    };
    for e in rd.flatten() {
        let p = e.path();
        if p.is_dir() {
            collect_pages(&p, out);
        } else if p.extension().map_or(false, |x| x == "at") {
            out.push(p);
        }
    }
}

/// 每一页都必须编译成功且产出非空 SFC——任何一页失败,vue serve 就会
/// 跳页 + router 悬空(整站 500)。附文件名便于定位。
#[test]
fn widgets_gallery_all_front_pages_compile() {
    let dir = front_pages_dir();
    let mut pages = Vec::new();
    collect_pages(&dir, &mut pages);
    assert!(
        !pages.is_empty(),
        "pages dir not found or empty: {} — 路径漂移需同步修本测试",
        dir.display()
    );

    let mut failures = Vec::new();
    for p in &pages {
        let rel = p.strip_prefix(dir.parent().unwrap().parent().unwrap())
            .unwrap_or(p)
            .display()
            .to_string();
        match crate::ui_build_shadcn_with_widgets(p.to_str().unwrap(), None) {
            Ok((vue_code, widgets)) => {
                if vue_code.trim().is_empty() || widgets.is_empty() {
                    failures.push(format!("{rel}: 无 widget 声明/空 SFC(actions-only 页?)"));
                }
            }
            Err(e) => failures.push(format!("{rel}: {e}")),
        }
    }
    assert!(
        failures.is_empty(),
        "widgets-gallery 页编译失败(vue serve 将跳页致 router 悬空 500):\n  {}",
        failures.join("\n  ")
    );
}
