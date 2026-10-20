//! Asset-manifest pinning (PLAN-815 T-02): the registration table must
//! match the real production sources until the one-shot M815-B migration
//! (T-04) — sources exist and are non-empty, licenses recorded, targets
//! unique and not yet created. Runs only in the workspace (manifest-dir
//! relative scan); the fixture consumer gate compiles the lib without
//! running these.

use auto_ui_tools::resources::{group, License, ASSET_GROUPS};
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    // crates/auto-ui-tools -> repo root
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .unwrap()
        .to_path_buf()
}

fn count_files(dir: &Path) -> usize {
    let mut count = 0;
    let mut stack = vec![dir.to_path_buf()];
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).expect("readable dir") {
            let entry = entry.expect("entry");
            if entry.file_type().unwrap().is_dir() {
                stack.push(entry.path());
            } else {
                count += 1;
            }
        }
    }
    count
}

#[test]
fn manifest_groups_are_registered_once_with_licenses() {
    let ids: Vec<&str> = ASSET_GROUPS.iter().map(|g| g.id).collect();
    assert_eq!(ids, vec!["gallery", "wm", "shadcn-ui"], "frozen group set");
    let targets: Vec<&str> = ASSET_GROUPS.iter().map(|g| g.target).collect();
    let mut unique = targets.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(targets.len(), unique.len(), "distinct relocation targets");
    for group in ASSET_GROUPS {
        assert!(matches!(
            group.license,
            License::InHouse | License::Mit { .. }
        ));
    }
    assert!(matches!(
        group("shadcn-ui").unwrap().license,
        License::Mit { upstream } if upstream.contains("shadcn-vue")
    ));
}

#[test]
fn production_sources_exist_and_are_non_empty() {
    let root = repo_root();
    for group in ASSET_GROUPS {
        let source = root.join(group.current_source);
        assert!(
            source.is_dir(),
            "{} source missing: {}",
            group.id,
            source.display()
        );
        let files = count_files(&source);
        assert!(files > 0, "{} source is empty", group.id);
    }
}

#[test]
fn known_source_anchors_are_present() {
    let root = repo_root();
    // gallery: single viewport host file
    assert!(root
        .join("crates/auto-man/assets/gallery/AppViewport.vue")
        .is_file());
    // wm: window manager hosts + remote renderer external input
    assert!(root
        .join("crates/auto-man/assets/wm/VirtualWindow.vue")
        .is_file());
    assert!(root.join("packages/drawlist-renderer/src").is_dir());
    let wm = group("wm").unwrap();
    assert!(wm
        .external_inputs
        .contains(&"packages/drawlist-renderer/src"));
    // shadcn: snapshot provenance record travels with the group
    assert!(root
        .join("crates/auto-man/assets/shadcn-ui/SNAPSHOT.md")
        .is_file());
    // file-count anchors (coarse; catches wholesale source removal)
    let shadcn = count_files(&root.join("crates/auto-man/assets/shadcn-ui"));
    assert!(
        shadcn > 300,
        "shadcn snapshot unexpectedly small: {shadcn} files"
    );
}

#[test]
fn migration_targets_do_not_exist_yet() {
    // A-stage discipline: production embed sources stay in auto-man until
    // the T-04 one-shot migration — no parallel copies may appear early.
    let root = repo_root();
    for group in ASSET_GROUPS {
        let target = root.join(group.target);
        assert!(!target.exists(), "premature copy at {}", target.display());
    }
}
