//! PLAN-457: Bundled shadcn-vue ui component snapshots.
//!
//! `assets/shadcn-ui/<component>/<file>` mirrors what the shadcn-vue CLI
//! writes under a project's `src/components/ui/<component>/`. On cold start
//! ([`materialize`]) copies each requested component's files into the
//! generated project **write-if-missing**, so the `pnpm dlx
//! shadcn-vue@latest add` round trip (registry fetch + CLI download +
//! internal reinstall) is skipped entirely for bundled components.
//!
//! Components absent from the bundle are reported in
//! [`MaterializeReport::missing`] and fall back to the CLI path
//! (`VueProject::install_shadcn_components`, which runs *after*
//! `npm install`). The bundle is a source-only snapshot: dependency
//! requirements stay declarative through `OPTIONAL_DEPS` /
//! `VueDependencyUsage` (Plan 442 P0-1 style), never patched into
//! package.json after the fact.

use rust_embed::Embed;
use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use crate::AutoResult;

#[derive(Embed)]
#[folder = "assets/shadcn-ui"]
pub struct ShadcnUiAssets;

/// Component names present in the bundle, sorted.
pub fn bundled_components() -> Vec<String> {
    let mut names: HashSet<String> = HashSet::new();
    for item in ShadcnUiAssets::iter() {
        if let Some((name, _)) = item.as_ref().split_once('/') {
            names.insert(name.to_string());
        }
    }
    let mut sorted: Vec<String> = names.into_iter().collect();
    sorted.sort();
    sorted
}

/// Whether a component has a bundled snapshot.
pub fn is_bundled(component: &str) -> bool {
    let prefix = format!("{}/", component);
    ShadcnUiAssets::iter().any(|p| p.as_ref().starts_with(&prefix))
}

/// PLAN-706 D3: scan a bundled component's sources for `@/components/ui/<name>`
/// imports and return those names (bundled only). Used to expand materialize
/// with a bounded transitive closure — e.g. `form` → `label` (FormLabel.vue).
fn bundled_ui_imports(component: &str) -> Vec<String> {
    let prefix = format!("{}/", component);
    let mut found = HashSet::new();
    for path in ShadcnUiAssets::iter() {
        let p = path.as_ref();
        if !p.starts_with(&prefix) {
            continue;
        }
        let Some(data) = ShadcnUiAssets::get(p) else {
            continue;
        };
        let Ok(text) = std::str::from_utf8(data.data.as_ref()) else {
            continue;
        };
        // `@/components/ui/label` / `@/components/ui/sidebar/index`
        let mut rest = text;
        while let Some(idx) = rest.find("@/components/ui/") {
            rest = &rest[idx + "@/components/ui/".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || *c == '-')
                .collect();
            if !name.is_empty() && is_bundled(&name) {
                found.insert(name);
            }
        }
    }
    let mut v: Vec<String> = found.into_iter().collect();
    v.sort();
    v
}

/// Expand `requested` with bundled transitive `@/components/ui/*` deps
/// (≤2 hops) so scaffolds like form→label land together.
fn expand_transitive(components: &[String]) -> Vec<String> {
    let mut out: Vec<String> = Vec::new();
    let mut seen = HashSet::new();
    let mut frontier: Vec<String> = components.to_vec();
    for _ in 0..2 {
        let mut next = Vec::new();
        for c in frontier {
            if !seen.insert(c.clone()) {
                continue;
            }
            out.push(c.clone());
            for dep in bundled_ui_imports(&c) {
                if !seen.contains(&dep) {
                    next.push(dep);
                }
            }
        }
        frontier = next;
        if frontier.is_empty() {
            break;
        }
    }
    out
}

fn component_dest(output_dir: &Path, component: &str) -> PathBuf {
    output_dir
        .join("src")
        .join("components")
        .join("ui")
        .join(component)
}

/// Outcome of [`materialize`] for logging / tests.
#[derive(Debug, Default, PartialEq)]
pub struct MaterializeReport {
    /// Files freshly copied into the project.
    pub written: usize,
    /// Files already on disk (write-if-missing — user edits preserved).
    pub skipped_existing: usize,
    /// Requested components without a bundled snapshot (CLI fallback).
    pub missing: Vec<String>,
}

/// Copy bundled component sources into the generated project.
///
/// Write-if-missing: an existing file is never overwritten (the file may be
/// user-patched or come from a previous CLI add), mirroring how
/// `copy_public_assets` treats already-copied trees.
///
/// PLAN-706: the request set is expanded with bundled transitive
/// `@/components/ui/*` deps (bounded, 2 hops) before copy.
pub fn materialize(output_dir: &Path, components: &[String]) -> AutoResult<MaterializeReport> {
    let mut report = MaterializeReport::default();
    for comp in expand_transitive(components) {
        let prefix = format!("{}/", comp);
        let files: Vec<String> = ShadcnUiAssets::iter()
            .map(|p| p.to_string())
            .filter(|p| p.starts_with(&prefix))
            .collect();
        if files.is_empty() {
            report.missing.push(comp.clone());
            continue;
        }

        let dest = component_dest(output_dir, &comp);
        for embedded_path in files {
            let file_name = embedded_path
                .rsplit('/')
                .next()
                .ok_or_else(|| format!("bad bundle path: {embedded_path}"))?;
            let target = dest.join(file_name);
            if target.exists() {
                report.skipped_existing += 1;
                continue;
            }
            fs::create_dir_all(&dest).map_err(|e| format!("mkdir {}: {e}", dest.display()))?;
            let data = ShadcnUiAssets::get(&embedded_path)
                .ok_or_else(|| format!("bundle miss: {embedded_path}"))?;
            fs::write(&target, data.data.as_ref())
                .map_err(|e| format!("write {}: {e}", target.display()))?;
            report.written += 1;
        }
    }
    Ok(report)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn button_is_bundled_with_expected_files() {
        assert!(is_bundled("button"));
        let index = ShadcnUiAssets::iter()
            .find(|p| p.as_ref() == "button/index.ts")
            .expect("button/index.ts in bundle");
        let data = ShadcnUiAssets::get(index.as_ref()).unwrap();
        let text = String::from_utf8(data.data.as_ref().to_vec()).unwrap();
        assert!(text.contains("export"), "index.ts should re-export");
    }

    #[test]
    fn bundled_names_are_sorted_and_unique() {
        let names = bundled_components();
        let mut dedup = names.clone();
        dedup.dedup();
        assert_eq!(names, dedup);
        let mut sorted = names.clone();
        sorted.sort();
        assert_eq!(names, sorted);
    }

    #[test]
    fn unknown_component_reports_missing() {
        let dir = std::env::temp_dir().join(format!(
            "automan-shadcn-test-missing-{}",
            std::process::id()
        ));
        let report = materialize(&dir, &["no-such-component".to_string()]).unwrap();
        assert_eq!(report.written, 0);
        assert!(report.missing.contains(&"no-such-component".to_string()));
        let _ = fs::remove_dir_all(dir);
    }

    /// PLAN-706 D3: form scaffold's FormLabel imports @/components/ui/label —
    /// materialize must expand the transitive closure or Form pages 500.
    #[test]
    fn materialize_form_pulls_label_transitively() {
        let dir = std::env::temp_dir().join(format!(
            "automan-shadcn-test-form-label-{}",
            std::process::id()
        ));
        let _ = fs::remove_dir_all(&dir);
        let report = materialize(&dir, &["form".to_string()]).unwrap();
        assert!(
            dir.join("src/components/ui/label/index.ts").exists(),
            "form → label 传递依赖应落地 (written={}, missing={:?})",
            report.written,
            report.missing
        );
        assert!(
            dir.join("src/components/ui/form/index.ts").exists(),
            "form 本体应落地"
        );
        let _ = fs::remove_dir_all(dir);
    }

    #[test]
    fn materialize_is_write_if_missing_and_idempotent() {
        let dir =
            std::env::temp_dir().join(format!("automan-shadcn-test-idem-{}", std::process::id()));
        let _ = fs::remove_dir_all(&dir);

        let report1 = materialize(&dir, &["button".to_string()]).unwrap();
        assert!(report1.written > 0);
        assert_eq!(report1.skipped_existing, 0);
        let report2 = materialize(&dir, &["button".to_string()]).unwrap();
        assert_eq!(report2.written, 0);
        assert_eq!(report2.skipped_existing, report1.written);

        // User edits survive a re-materialize.
        let edited = dir.join("src/components/ui/button/Button.vue");
        fs::write(&edited, "// local patch").unwrap();
        let report3 = materialize(&dir, &["button".to_string()]).unwrap();
        assert_eq!(report3.written, 0);
        assert_eq!(fs::read_to_string(&edited).unwrap(), "// local patch");

        let _ = fs::remove_dir_all(dir);
    }
}

// ── PLAN-571: 烘焙资产 button cva 与 Rust 侧 variants.rs 互锁锚 ─────
#[cfg(test)]
mod plan571_button_asset_interlock_tests {
    /// 烘焙快照 button/index.ts 的 cva 与 ui::style::variants 单源表一致性：
    /// default = UA 等价中性基线（muted 填充+发丝描边）；primary/submit 键在册
    /// （主题色填充）。改任一侧须同步（ui_gen/vue.rs 同款互锁测试）。
    #[test]
    fn button_asset_cva_matches_plan571_variant_table() {
        let ts = crate::vue_shadcn::ShadcnUiAssets::get("button/index.ts")
            .expect("button/index.ts 在烘焙包内")
            .data;
        let ts = std::str::from_utf8(&ts).expect("utf8");
        for (key, needle) in [
            ("default", "bg-muted border border-border text-foreground hover:bg-muted/70"),
            ("primary", "bg-primary text-primary-foreground hover:bg-primary/90"),
            ("submit", "bg-primary text-primary-foreground hover:bg-primary/90"),
        ] {
            let line = ts
                .lines()
                .find(|l| l.trim_start().starts_with(&format!("{}:", key)))
                .unwrap_or_else(|| panic!("cva 缺 {} 键", key));
            assert!(line.contains(needle), "cva[{}] = {} 缺 {:?}", key, line.trim(), needle);
        }
    }
}
