//! PLAN-803 V01 — an EXTERNAL consumer using only `auto-man-assets`.
//!
//! Everything here must hold without auto-lang/auto-man/auto-gen/auto-ui/
//! auto-vm/auto-val/auto-ast anywhere in the build (enforced structurally
//! by this crate's manifest and by `verify-deps.sh`). It proves an external,
//! language-free consumer can enumerate and read ALL three resource groups
//! and drive the pure scaffold path planning: per-family projections,
//! multi-`name` replacement, nested directories, unknown families, Unicode
//! package names, and unrendered `${name}` bytes.
//!
//! Build-mode note (F803-R1-03): default debug builds read the crate's
//! source assets directory dynamically (rust-embed, no `debug-embed`);
//! release builds embed the bytes. The boundary + missing-resource control
//! is pinned by `mode-boundary-probe.py` next to this fixture.

use auto_man_assets::{plan_template_family, GhsAssets, IarAssets, TemplateAssets};

/// Frozen template enumeration (order = rust-embed walkdir sort_by_file_name,
/// identical mechanism & layout as the pre-extraction `Templates::iter()`).
const TEMPLATE_KEYS: [&str; 13] = [
    "app/name.at",
    "app/pac.at",
    "capp/name.c",
    "capp/pac.at",
    "clib/name.c",
    "clib/name.h",
    "clib/pac.at",
    "lib/name.at",
    "lib/pac.at",
    "libdemo/apps/name_demo/name_demo.c",
    "libdemo/libs/name/name.c",
    "libdemo/libs/name/name.h",
    "libdemo/pac.at",
];

const GHS_KEYS: [&str; 3] = ["default.gpj", "program.gpj", "subproject.gpj"];
const IAR_KEYS: [&str; 4] = ["iar.ewd", "iar.ewp", "iar.ewt", "iar.eww"];

#[test]
fn external_consumer_enumerates_all_three_groups() {
    let templates: Vec<String> = TemplateAssets::iter().map(|k| k.to_string()).collect();
    assert_eq!(templates, TEMPLATE_KEYS, "template keys/order drifted");

    let ghs: Vec<String> = GhsAssets::iter().map(|k| k.to_string()).collect();
    assert_eq!(ghs, GHS_KEYS);

    let iar: Vec<String> = IarAssets::iter().map(|k| k.to_string()).collect();
    assert_eq!(iar, IAR_KEYS);

    // every key is gettable with non-empty bytes
    for key in TEMPLATE_KEYS {
        let f = TemplateAssets::get(key).expect(key);
        assert!(!f.data.is_empty(), "{key} empty");
    }
    for key in GHS_KEYS {
        assert!(
            GhsAssets::get(key).expect(key).data.len() > 100,
            "{key} tiny"
        );
    }
    for key in IAR_KEYS {
        assert!(
            IarAssets::get(key).expect(key).data.len() > 100,
            "{key} tiny"
        );
    }
}

#[test]
fn external_consumer_plans_family_paths() {
    // full projection for one family: order, source keys, output rel paths
    let plan = plan_template_family("libdemo", "mylib");
    let projection: Vec<(&str, &str)> = plan
        .iter()
        .map(|p| (p.source_key.as_str(), p.output_rel.as_str()))
        .collect();
    assert_eq!(
        projection,
        [
            (
                "libdemo/apps/name_demo/name_demo.c",
                "apps/mylib_demo/mylib_demo.c"
            ),
            ("libdemo/libs/name/name.c", "libs/mylib/mylib.c"),
            ("libdemo/libs/name/name.h", "libs/mylib/mylib.h"),
            ("libdemo/pac.at", "pac.at"),
        ]
    );

    // top-level family: `name.at`→`mylib.at`, `pac.at` untouched
    let app = plan_template_family("app", "mylib");
    assert_eq!(
        app.iter()
            .map(|p| p.output_rel.as_str())
            .collect::<Vec<_>>(),
        ["mylib.at", "pac.at"]
    );

    // planned bytes are the raw embedded template — `${name}` NOT rendered
    let name_at = &app[0];
    assert_eq!(name_at.source_key, "app/name.at");
    assert_eq!(
        String::from_utf8_lossy(&name_at.bytes),
        "print(\"Hello, ${name}!\")\n"
    );
}

#[test]
fn external_consumer_unknown_family_and_unicode_name() {
    assert!(plan_template_family("no-such-family", "x").is_empty());

    let uni = plan_template_family("clib", "组件");
    assert_eq!(
        uni.iter()
            .map(|p| p.output_rel.as_str())
            .collect::<Vec<_>>(),
        ["组件.c", "组件.h", "pac.at"]
    );
}

/// Zero **output** IO contract: planning/enum never creates or writes files.
/// (F803-R1-03 build-mode boundary: in default debug builds rust-embed
/// dynamically reads the crate's source assets directory; release builds
/// embed the bytes. Pinned by mode-boundary-probe.py next to this fixture.)
#[test]
fn external_consumer_planning_has_no_output_io() {
    let dir =
        std::env::temp_dir().join(format!("man_assets_consumer_iofree_{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let before: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let _ = plan_template_family("libdemo", "iofree");
    let _ = TemplateAssets::get("app/pac.at");
    let _ = GhsAssets::get("default.gpj");
    let _ = IarAssets::get("iar.eww");
    let after: Vec<_> = std::fs::read_dir(&dir)
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    assert_eq!(before, after, "planning/enum must not create anything");
    let _ = std::fs::remove_dir_all(&dir);
}
