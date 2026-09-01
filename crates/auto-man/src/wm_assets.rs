//! Plan 465: 桌面宿主 WM 运行时资产（`assets/wm/*` → 生成项目 `src/wm/`）。
//!
//! 手写模板资产（同 shadcn-ui 捆绑惯例）：store.ts（WmStore）、layout.ts
//! （463 layout.rs 的 TS 直译，I6 对拍共享表）、keyboard.ts（R12 桌面热键）、
//! VirtualWindow.vue / Taskbar.vue（schema/aura.at `vue:` 映射的 DOM 叶实现）。
//! `generate_desktop_host` 每次 run 全量覆写（资产是生成物，勿手改产物）。
//!
//! Plan 516: 追加 `RemoteWindow.vue` / `remote.ts`（远程窗 wm 叶与会话
//! 切片）+ 远程渲染器运行时（`packages/drawlist-renderer/src` 的编译期
//! 拷贝 → `src/wm/remote-renderer/`）。渲染器包零改动；拷贝与包源的
//! 漂移由 wm-test 测试位（resolveId 直指包源测 wm 资产）双向钉住。

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
    materialize_renderer(output_dir)
}

/// 远程渲染器运行时（Plan 516）：`packages/drawlist-renderer/src`（508
/// 交付，vitest 20/20 锁）的编译期拷贝。生成项目 wm 资产以相对路径
/// `./remote-renderer/index.ts` 消费——与测试位同一导入面。
const REMOTE_RENDERER_FILES: &[(&str, &str)] = &[
    ("codec.ts", include_str!("../../../packages/drawlist-renderer/src/codec.ts")),
    ("connect.ts", include_str!("../../../packages/drawlist-renderer/src/connect.ts")),
    ("index.ts", include_str!("../../../packages/drawlist-renderer/src/index.ts")),
    ("messages.ts", include_str!("../../../packages/drawlist-renderer/src/messages.ts")),
    ("render.ts", include_str!("../../../packages/drawlist-renderer/src/render.ts")),
];

fn materialize_renderer(output_dir: &Path) -> AutoResult<()> {
    let dst_dir = output_dir.join("src").join("wm").join("remote-renderer");
    fs::create_dir_all(&dst_dir)?;
    for (name, code) in REMOTE_RENDERER_FILES {
        fs::write(dst_dir.join(name), code)?;
        println!("  {} Wm remote renderer: {}", "✓".bright_green(), name);
    }
    Ok(())
}
