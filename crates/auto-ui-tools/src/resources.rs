//! UI asset manifest (PLAN-815 T-02 registration; T-04 one-shot migration
//! completed).
//!
//! The production embed sources for every UI asset group now live in this
//! package (single source; the historic auto-man copies are gone). The
//! manifest keeps owner/source/license/relocation provenance for review and
//! delivery audits; tests pin the manifest against the actual sources so
//! drift is caught.

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
    /// Production location (repo-relative, embed source of truth — the
    /// T-04 one-shot migration moved these into this package).
    pub current_source: &'static str,
    /// External build inputs referenced from outside the source dir
    /// (repo-relative).
    pub external_inputs: &'static [&'static str],
    /// Historic pre-T-04 location (auto-man) — kept for provenance; must
    /// stay empty after the one-shot migration.
    pub migrated_from: &'static str,
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

/// The frozen asset manifest (T-01 §2.4; relocation executed in T-04).
/// `wm`'s external inputs live in this package's `drawlist-renderer/`
/// (moved together with the group from the repo-root `packages/` location).
pub const ASSET_GROUPS: &[AssetGroup] = &[
    AssetGroup {
        id: "gallery",
        current_source: "crates/auto-ui-tools/assets/gallery",
        external_inputs: &[],
        migrated_from: "crates/auto-man/assets/gallery",
        license: License::InHouse,
        embed: EmbedKind::RustEmbed,
    },
    AssetGroup {
        id: "wm",
        current_source: "crates/auto-ui-tools/assets/wm",
        external_inputs: &["crates/auto-ui-tools/drawlist-renderer/src"],
        migrated_from: "crates/auto-man/assets/wm",
        license: License::InHouse,
        embed: EmbedKind::RustEmbed,
    },
    AssetGroup {
        id: "shadcn-ui",
        current_source: "crates/auto-ui-tools/assets/shadcn-ui",
        external_inputs: &[],
        migrated_from: "crates/auto-man/assets/shadcn-ui",
        license: License::Mit {
            upstream: "https://shadcn-vue.com registry snapshot 2026-08-27 (see SNAPSHOT.md; PLAN-571 button cva + sonner icon patches; LICENSE travels with the group since T-04)",
        },
        embed: EmbedKind::RustEmbed,
    },
];

/// Look up a group by id.
pub fn group(id: &str) -> Option<&'static AssetGroup> {
    ASSET_GROUPS.iter().find(|g| g.id == id)
}
