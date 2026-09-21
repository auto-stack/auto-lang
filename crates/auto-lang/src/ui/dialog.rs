//! PLAN-681: native open/save file dialogs — a2r host of the VM builtins
//! `dialog_open` / `dialog_save` (natives 2927/2928, `auto.dialog.open/save`).
//!
//! The ui_gen builtin-call mapping table emits `auto_lang::ui::dialog::…`
//! for those names in generated apps, mirroring the VM shims
//! (`vm/native.rs::shim_dialog_open` / `shim_dialog_save`): same filter
//! parsing (`,`/`;`/space/tab separated, leading dots trimmed), same
//! default-file-name seeding, same `""`-on-cancel convention.
//!
//! Boundary (PLAN-681 §9 F1): the VM shims additionally parent the dialog
//! to the host window (`attach(main_hwnd)`); a2r generated apps call this
//! host directly and the dialog is unparented in v1 — modal parenting for
//! the a2r shell is a separate wiring concern, not a semantic fork.

/// `dialog_open(filter) -> String` — pick one file; `""` on cancel.
pub fn open(filter: &str) -> String {
    let exts: Vec<String> = filter
        .split([',', ';', ' ', '\t'])
        .map(|s| s.trim().trim_start_matches('.').to_string())
        .filter(|s| !s.is_empty())
        .collect();
    let mut dialog = rfd::FileDialog::new();
    if !exts.is_empty() {
        dialog = dialog.add_filter("Files", &exts);
    }
    dialog
        .pick_file()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}

/// `dialog_save(default_name) -> String` — pick a save target; `""` on cancel.
pub fn save(default_name: &str) -> String {
    let mut dialog = rfd::FileDialog::new();
    if !default_name.is_empty() {
        dialog = dialog.set_file_name(default_name);
    }
    dialog
        .save_file()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_default()
}
