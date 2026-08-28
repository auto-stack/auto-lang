//! Plan 465: 桌面宿主 WM 运行时资产（`assets/wm/*` → 生成项目 `src/wm/`）。
//!
//! 手写模板资产（同 shadcn-ui 捆绑惯例）：store.ts（WmStore）、layout.ts
//! （463 layout.rs 的 TS 直译，I6 对拍共享表）、keyboard.ts（R12 桌面热键）、
//! VirtualWindow.vue / Taskbar.vue（schema/aura.at `vue:` 映射的 DOM 叶实现）。
//! `generate_desktop_host` 每次 run 全量覆写（资产是生成物，勿手改产物）。

use rust_embed::Embed;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use colored::Colorize;

use crate::AutoResult;

#[derive(Embed)]
#[folder = "assets/wm"]
pub struct WmAssets;

/// Bundled wm asset file names, sorted.
pub fn bundled_files() -> Vec<String> {
    let mut names: HashSet<String> = WmAssets::iter().map(|p| p.as_ref().to_string()).collect();
    let mut sorted: Vec<String> = names.drain().collect();
    sorted.sort();
    sorted
}

/// Materialize the WM runtime into `<output_dir>/src/wm/` (overwrite —
/// the assets are owned by the generator, not the user).
pub fn materialize(output_dir: &Path) -> AutoResult<()> {
    let dst_dir = output_dir.join("src").join("wm");
    fs::create_dir_all(&dst_dir)?;
    for name in bundled_files() {
        let Some(file) = WmAssets::get(&name) else { continue };
        let dst = dst_dir.join(&name);
        fs::write(&dst, file.data)?;
        println!("  {} Wm runtime: {}", "✓".bright_green(), dst.display());
    }
    Ok(())
}
