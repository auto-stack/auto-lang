//! Asset-manifest pinning (PLAN-815 T-02 registration → T-04 one-shot
//! migration completed): production embed sources live in this package,
//! the historic auto-man copies must stay empty, licenses travel with the
//! groups. Runs only in the workspace (manifest-dir relative scan); the
//! fixture consumer gate compiles the lib without running these.

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
    let sources: Vec<&str> = ASSET_GROUPS.iter().map(|g| g.current_source).collect();
    let mut unique = sources.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(sources.len(), unique.len(), "distinct production sources");
    for g in ASSET_GROUPS {
        assert!(matches!(g.license, License::InHouse | License::Mit { .. }));
    }
    assert!(matches!(
        group("shadcn-ui").unwrap().license,
        License::Mit { upstream } if upstream.contains("shadcn-vue")
    ));
}

#[test]
fn production_sources_exist_and_are_non_empty() {
    let root = repo_root();
    for g in ASSET_GROUPS {
        let source = root.join(g.current_source);
        assert!(
            source.is_dir(),
            "{} source missing: {}",
            g.id,
            source.display()
        );
        let files = count_files(&source);
        assert!(files > 0, "{} source is empty", g.id);
    }
}

#[test]
fn known_source_anchors_are_present() {
    let root = repo_root();
    // gallery: single viewport host file
    assert!(root
        .join("crates/auto-ui-tools/assets/gallery/AppViewport.vue")
        .is_file());
    // wm: window manager hosts + remote renderer external input (moved with
    // the group in T-04 — packages/drawlist-renderer no longer exists)
    assert!(root
        .join("crates/auto-ui-tools/assets/wm/VirtualWindow.vue")
        .is_file());
    assert!(root
        .join("crates/auto-ui-tools/drawlist-renderer/src")
        .is_dir());
    let wm = group("wm").unwrap();
    assert!(wm
        .external_inputs
        .contains(&"crates/auto-ui-tools/drawlist-renderer/src"));
    // shadcn: snapshot provenance + upstream MIT license travel with the group
    assert!(root
        .join("crates/auto-ui-tools/assets/shadcn-ui/SNAPSHOT.md")
        .is_file());
    assert!(root
        .join("crates/auto-ui-tools/assets/shadcn-ui/LICENSE")
        .is_file());
    // file-count anchors (coarse; catches wholesale source removal)
    let shadcn = count_files(&root.join("crates/auto-ui-tools/assets/shadcn-ui"));
    assert!(
        shadcn > 300,
        "shadcn snapshot unexpectedly small: {shadcn} files"
    );
}

#[test]
fn historic_auto_man_copies_stay_removed() {
    // T-04 one-shot migration discipline: the pre-T-04 production sources in
    // auto-man (and the repo-root drawlist package) must stay empty — no
    // parallel embed sources may reappear.
    let root = repo_root();
    for g in ASSET_GROUPS {
        let old = root.join(g.migrated_from);
        assert!(
            !old.exists(),
            "historic copy reappeared for {}: {}",
            g.id,
            old.display()
        );
    }
    assert!(!root.join("packages/drawlist-renderer").exists());
}

/// T-07 子批2 资源审计钉扎：manifest 登记的生产源就是三个 embed 结构
/// 实际消费的文件夹——每组 embed 面非空且锚文件在面内；shadcn 组的 MIT
/// LICENSE + SNAPSHOT.md 随组进入 embed（许可随发行面旅行——按组件
/// materialize 只拷组件文件，组级许可文件不进用户工程，留在二进制与
/// 仓内组源，见 attachments/815/t07-acceptance/batch2/resource-audit.md）。
#[test]
fn embed_consumption_matches_registered_sources() {
    use auto_ui_tools::{gallery_assets, vue_shadcn, wm_assets};

    // gallery → materialize <output>/src/gallery/ (RustEmbed over the
    // registered source dir; anchor file present in the embed surface)
    let gallery = gallery_assets::bundled_files();
    assert!(!gallery.is_empty(), "gallery embed surface is non-empty");
    assert!(
        gallery
            .iter()
            .any(|f| f.replace('\\', "/") == "AppViewport.vue"),
        "gallery anchor missing from embed surface: {gallery:?}"
    );

    // wm → materialize <output>/src/wm/ (+ drawlist renderer inputs, which
    // the manifest records as external_inputs of the group)
    let wm = wm_assets::bundled_files();
    assert!(!wm.is_empty(), "wm embed surface is non-empty");
    assert!(
        wm.iter()
            .any(|f| f.replace('\\', "/") == "VirtualWindow.vue"),
        "wm anchor missing from embed surface: {wm:?}"
    );

    // shadcn-ui → per-component materialize into src/components/ui/<name>/
    let components = vue_shadcn::bundled_components();
    assert!(
        components.len() >= 10,
        "shadcn snapshot unexpectedly thin: {components:?}"
    );
    // The MIT license + snapshot provenance travel with the group into the
    // embed surface (they live at the group root, outside <component>/ dirs,
    // so per-component materialize does not copy them into user projects).
    assert!(
        vue_shadcn::ShadcnUiAssets::get("LICENSE").is_some(),
        "MIT LICENSE is part of the shadcn embed surface"
    );
    assert!(
        vue_shadcn::ShadcnUiAssets::get("SNAPSHOT.md").is_some(),
        "snapshot provenance is part of the shadcn embed surface"
    );
}
