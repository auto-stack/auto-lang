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
pub fn materialize(output_dir: &Path, components: &[String]) -> AutoResult<MaterializeReport> {
    let mut report = MaterializeReport::default();
    for comp in components {
        let prefix = format!("{}/", comp);
        let files: Vec<String> = ShadcnUiAssets::iter()
            .map(|p| p.to_string())
            .filter(|p| p.starts_with(&prefix))
            .collect();
        if files.is_empty() {
            report.missing.push(comp.clone());
            continue;
        }

        let dest = component_dest(output_dir, comp);
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
            fs::create_dir_all(&dest)
                .map_err(|e| format!("mkdir {}: {e}", dest.display()))?;
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

    #[test]
    fn materialize_is_write_if_missing_and_idempotent() {
        let dir = std::env::temp_dir().join(format!(
            "automan-shadcn-test-idem-{}",
            std::process::id()
        ));
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
