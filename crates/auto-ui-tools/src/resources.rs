//! UI asset manifest (PLAN-815 T-02, M815-A scope).
//!
//! Registration only: owner/source/license/relocation for every UI asset
//! group that M815-B will migrate into this package. The production embed
//! sources stay in `auto-man` until the one-shot migration (T-04) — this
//! table neither duplicates nor moves them. Tests pin the manifest against
//! the actual current sources so drift is caught before the migration.

/// License of an asset group.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum License {
    /// Authored in this repository.
    InHouse,
    /// Upstream MIT; snapshot provenance recorded next to the source.
    Mit { upstream: &'static str },
}

/// One asset group in the migration plan.
#[derive(Debug, Clone, PartialEq)]
pub struct AssetGroup {
    /// Stable group id.
    pub id: &'static str,
    /// Current production location (repo-relative, embed source of truth
    /// until T-04 performs the one-shot migration).
    pub current_source: &'static str,
    /// External build inputs referenced from outside the source dir
    /// (repo-relative).
    pub external_inputs: &'static [&'static str],
    /// Destination inside this package after migration (T-04).
    pub target: &'static str,
    pub license: License,
    /// How the source enters builds today.
    pub embed: EmbedKind,
}

/// How a group is compiled into the tool today.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EmbedKind {
    /// `rust-embed` over the source directory.
    RustEmbed,
    /// `include_str!` of individual files (drawlist renderer inputs).
    IncludeStr,
}

/// The frozen asset manifest (T-01 §2.4). `wm`'s external inputs live in
/// `packages/drawlist-renderer` and must move with the group.
pub const ASSET_GROUPS: &[AssetGroup] = &[
    AssetGroup {
        id: "gallery",
        current_source: "crates/auto-man/assets/gallery",
        external_inputs: &[],
        target: "crates/auto-ui-tools/assets/gallery",
        license: License::InHouse,
        embed: EmbedKind::RustEmbed,
    },
    AssetGroup {
        id: "wm",
        current_source: "crates/auto-man/assets/wm",
        external_inputs: &["packages/drawlist-renderer/src"],
        target: "crates/auto-ui-tools/assets/wm",
        license: License::InHouse,
        embed: EmbedKind::RustEmbed,
    },
    AssetGroup {
        id: "shadcn-ui",
        current_source: "crates/auto-man/assets/shadcn-ui",
        external_inputs: &[],
        target: "crates/auto-ui-tools/assets/shadcn-ui",
        license: License::Mit {
            upstream: "https://shadcn-vue.com registry snapshot 2026-08-27 (see SNAPSHOT.md; PLAN-571 button cva + sonner icon patches)",
        },
        embed: EmbedKind::RustEmbed,
    },
];

/// Look up a group by id.
pub fn group(id: &str) -> Option<&'static AssetGroup> {
    ASSET_GROUPS.iter().find(|g| g.id == id)
}
