//! 跨仓解析序路径定位（Stage B 系，auto-os 侧资产）。
//!
//! Stage B（auto-os Design 01）把桌面域资产（apps/画廊/shell 等）迁入
//! auto-os 伞形仓后，框架仓的测试语料锚、docs 管线与 CLI 落点都需要跨仓
//! 定位这些资产。解析序沿仓际约定（env 覆盖 → 兄弟检出 → 主检出兜底），
//! 与 `ui::app_registry::resolve_os_manifest_root`（P-3）同族；本模块
//! **不挂 feature 门**——`ui_gen`（docs 管线）与 CLI 等无 `ui` feature 的
//! 构建形态也要消费。

use std::path::{Path, PathBuf};

/// Stage B P-5：解析序定位 auto-os 顶层随迁资产目录（画廊两件等 590 批
/// 迁入 auto-os 顶层/`apps/` 的资产）。解析序：`AUTO_OS_ROOT` env（设置即
/// 权威）→ 兄弟 `parent/auto-os` → 主检出兜底 `D:/autostack/auto-os`；
/// 首个含 `<name>` 子目录的候选胜；全缺 → None（solo 检出静默不炸——
/// 测试/docs 管线语料锚据此 SKIP，见 gallery_pages_compile_tests /
/// schema_drift / docs_gen / gallery_golden 等消费方）。
///
/// `parent` 语义同 [`crate::ui::app_registry::resolve_os_manifest_root`]：
/// 其 `auto-os` 子目录为兄弟候选的基目录（仓根的父目录）。
pub fn resolve_os_top_dir(parent: &Path, name: &str) -> Option<PathBuf> {
    if let Some(root) = std::env::var_os("AUTO_OS_ROOT") {
        let p = PathBuf::from(root).join(name);
        return p.is_dir().then_some(p);
    }
    [parent.join("auto-os"), PathBuf::from("D:/autostack/auto-os")]
        .into_iter()
        .map(|root| root.join(name))
        .find(|p| p.is_dir())
}
