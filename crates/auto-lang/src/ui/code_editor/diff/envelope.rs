// PLAN-703 供⑤ / PLAN-807 — the envelope ALGORITHM lives in
// `auto-text-core::diff::envelope`; this historical path re-exports it with
// unchanged function identity. The one UI-owned piece stays here: the
// editor-registry key→snapshot lookup (`code_editor_with`) feeding the pure
// core face — key lookup is UI state, the envelope is not.
pub use auto_text_core::diff::envelope::{
    diff_dirs_envelope, diff_files_envelope, diff_files_envelope_from_paths,
    diff_files_envelope_from_paths_window, diff_files_envelope_window,
};

use auto_text_core::diff::envelope::{diff_snapshots_envelope_snapshots, DiffSnapshotsPair};

use crate::ui::code_editor::core::rope::RopeSnapshot;

/// `diff_snapshots(key_a, key_b)` — direct buffer-registry read (editing
/// buffer comparison; zero full-text VM transit). Missing keys → err form.
/// PLAN-807: signature, judgement priority (key_a first) and byte output
/// preserved; the diff itself runs in `auto-text-core` over the resolved
/// snapshots.
pub fn diff_snapshots_envelope(key_a: &str, key_b: &str) -> String {
    let sa: Option<RopeSnapshot> =
        crate::ui::code_editor::code_editor_with(key_a, |core| core.doc_snapshot());
    let sb: Option<RopeSnapshot> =
        crate::ui::code_editor::code_editor_with(key_b, |core| core.doc_snapshot());
    let pair = match (sa, sb) {
        (Some(a), Some(b)) => DiffSnapshotsPair::Both(a, b),
        (None, _) => DiffSnapshotsPair::Missing(key_a.to_string()),
        (Some(_), None) => DiffSnapshotsPair::Missing(key_b.to_string()),
    };
    diff_snapshots_envelope_snapshots(pair)
}
