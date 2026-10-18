// Plan 673 T-04 / PLAN-807 — the rope moved to the `auto-text-core` crate
// (single source: algorithm, paging, digests, snapshots). This module is a
// thin re-export preserving the historical in-crate path
// (`crate::ui::code_editor::core::rope::*`) with UNCHANGED type identity —
// `rope::Rope` here IS `auto_text_core::Rope`. The editor kernel and the
// diff engine consume the same single implementation through this facade.
pub use auto_text_core::rope::{PageConfig, PageState, PageStore, Rope, RopeSnapshot};
