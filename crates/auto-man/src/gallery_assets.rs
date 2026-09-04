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
