//! Plan 549: UI Gallery runtime assets (`assets/gallery/*` → `<output_dir>/src/gallery/`).

use rust_embed::Embed;
use std::collections::HashSet;
use std::fs;
use std::path::Path;

use colored::Colorize;

use crate::AutoResult;

#[derive(Embed)]
#[folder = "assets/gallery"]
pub struct GalleryAssets;

/// Bundled gallery asset file names, sorted.
pub fn bundled_files() -> Vec<String> {
    let mut names: HashSet<String> = GalleryAssets::iter().map(|p| p.as_ref().to_string()).collect();
    let mut sorted: Vec<String> = names.drain().collect();
    sorted.sort();
    sorted
}

/// Materialize the Gallery runtime assets into `<output_dir>/src/gallery/`.
pub fn materialize(output_dir: &Path) -> AutoResult<()> {
    let dst_dir = output_dir.join("src").join("gallery");
    fs::create_dir_all(&dst_dir)?;
    for name in bundled_files() {
        let Some(file) = GalleryAssets::get(&name) else { continue };
        let dst = dst_dir.join(&name);
        fs::write(&dst, file.data)?;
        println!("  {} Gallery runtime: {}", "✓".bright_green(), dst.display());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Plan 672: Vue 臂视口安全居中契约（对齐 VM 臂 m-auto 语义）——
    /// demo-mount-root 必须是 flex 列容器且直接子项 margin:auto：
    /// 小于视口双向居中、溢出时 auto 归零不裁顶、满幅 demo 零变化。
    /// 防回归：模板被重排/重写时居中语义不得静默丢失。
    #[test]
    fn app_viewport_template_has_safe_centering() {
        let file = GalleryAssets::get("AppViewport.vue").expect("embedded AppViewport.vue");
        let text = std::str::from_utf8(&file.data).expect("utf8");
        assert!(
            text.contains("demo-mount-root ash-scroll w-full h-full flex flex-col"),
            "demo-mount-root must stay a flex column container carrying the \
             ash-scroll AutoUI scrollbar class:\n{text}"
        );
        assert!(
            text.contains(".demo-mount-root > :deep(*)") && text.contains("margin: auto;"),
            "direct children must keep margin:auto safe centering:\n{text}"
        );
    }
}

