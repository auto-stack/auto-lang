//! Plan 370 D1-D7: REAL 015-notes headless behavior tests (VM mode).
//!
//! These drive the actual 015-notes App (parsed from examples/ui/015-notes)
//! via `DynamicComponent::on_with_input`, exactly as a button click would.
//! They cover the business behaviors the acceptance.atd D1-D7 contracts
//! describe — note selection, creation, view tabs, tag filter, pin toggle,
//! dark mode — against the real store-composable handler chain
//! (`store.X()` → merged root state).
//!
//! This complements `desktop_behavior.rs` (which uses inline Counter widgets
//! to test DynamicComponent mechanics) and `plan370_store_vm_tests` (which
//! tests the data/render layer). Here we exercise the real handler chain.
//!
//! Parameterized handlers use the iced renderer's payload encoding:
//!   `{event}\u{1F}{type}\u{1F}{value}`  (type ∈ i/u/b/f/d/s)
//! e.g. SelectNote(3) → "SelectNote\u{1F}i\u{1F}3".

#[cfg(test)]
mod plan370_015_behavior_tests {
    use crate::plan370_test_support::build_015_component;
    use crate::ui::dynamic::DynamicComponent;
    use auto_val::Value;

    /// Payload-encoded event helper: `{event}\u{1F}i\u{1F}{n}` for an int arg.
    fn with_int(event: &str, n: i32) -> String {
        format!("{}\u{1F}i\u{1F}{}", event, n)
    }

    /// Payload-encoded event helper: `{event}\u{1F}s\u{1F}{s}` for a str arg.
    fn with_str(event: &str, s: &str) -> String {
        format!("{}\u{1F}s\u{1F}{}", event, s)
    }

    /// Read a scalar state field as a string (for assertions), panicking on
    /// error with a clear message.
    fn state_str(dc: &DynamicComponent, field: &str) -> String {
        match dc.read_state(field) {
            Ok(v) => match v {
                Value::Int(i) => i.to_string(),
                Value::Bool(b) => b.to_string(),
                Value::Str(s) => s.as_str().to_string(),
                other => format!("{:?}", other),
            },
            Err(e) => panic!("read_state('{}') failed: {}", field, e),
        }
    }

    /// Number of notes (derefs the VmRef list). Returns 0 on error.
    fn notes_count(dc: &DynamicComponent) -> usize {
        dc.read_state_as_vec("notes").map(|v| v.len()).unwrap_or(0)
    }

    /// Read a field of notes[idx] (e.g. "pinned", "title"). The element is a
    /// heap-id Int; materialize_obj_ref turns it into a Value::Obj.
    fn note_field(dc: &DynamicComponent, idx: usize, field: &str) -> Value {
        let notes = dc
            .read_state_as_vec("notes")
            .unwrap_or_else(|e| panic!("read notes: {}", e));
        let elem = notes
            .get(idx)
            .unwrap_or_else(|| panic!("notes[{}] out of range (len {})", idx, notes.len()));
        let obj = dc.bridge().materialize_obj_ref(elem);
        match obj {
            Value::Obj(o) => o
                .get(field)
                .unwrap_or_else(|| panic!("note field '{}' missing", field)),
            other => panic!("notes[{}] not an Obj: {:?}", idx, other),
        }
    }

    // ── D1: Init loads seed data ────────────────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d1_init_loads_seed_notes() {
        let dc = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        // db.at seeds 6 notes (Welcome..Sprint Planning).
        assert_eq!(notes_count(&dc), 6, "Init should load 6 seed notes");
        assert_eq!(state_str(&dc, "active_id"), "0", "initial active_id");
        assert_eq!(state_str(&dc, "active_folder"), "all", "initial active_folder");
        assert_eq!(state_str(&dc, "active_tag"), "", "initial active_tag empty");
    }

    // ── D2: NewNote appends + re-points active_id ───────────────────────────
    // Exercises the store method call chain: App.NewNote → store.NewNote()
    // → create_note() + notes = list_notes() + active_id = len-1.

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d2_new_note_appends() {
        let mut dc = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        let before = notes_count(&dc);
        dc.on_with_input("NewNote", None);
        let after = notes_count(&dc);
        assert_eq!(
            after,
            before + 1,
            "NewNote should append one note ({} -> {})",
            before,
            after
        );
        // store.NewNote sets active_id = notes.len() - 1 (the new note).
        assert_eq!(
            state_str(&dc, "active_id"),
            (after - 1).to_string(),
            "active_id should point at the new note"
        );
    }

    // ── D3: SelectNote(i) sets active_id (parameterized handler) ────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d3_select_note_sets_active_id() {
        let mut dc = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        assert_eq!(state_str(&dc, "active_id"), "0", "precondition: active_id 0");
        dc.on_with_input(&with_int("SelectNote", 3), None);
        assert_eq!(
            state_str(&dc, "active_id"),
            "3",
            "SelectNote(3) should set active_id=3"
        );
    }

    // ── D4: View tabs switch active_folder ──────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d4_view_tabs_switch_folder() {
        let mut dc = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        assert_eq!(state_str(&dc, "active_folder"), "all");
        dc.on_with_input("SelectPinned", None);
        assert_eq!(state_str(&dc, "active_folder"), "pinned");
        dc.on_with_input("SelectRecent", None);
        assert_eq!(state_str(&dc, "active_folder"), "recent");
        dc.on_with_input("SelectAll", None);
        assert_eq!(state_str(&dc, "active_folder"), "all");
    }

    // ── D6: Tag filter sets/clears active_tag (parameterized handler) ───────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d6_tag_filter_sets_and_clears() {
        let mut dc = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        assert_eq!(state_str(&dc, "active_tag"), "");
        dc.on_with_input(&with_str("SelectTag", "work"), None);
        assert_eq!(state_str(&dc, "active_tag"), "work", "SelectTag(work)");
        dc.on_with_input("ClearTag", None);
        assert_eq!(state_str(&dc, "active_tag"), "", "ClearTag");
    }

    // ── D7: TogglePin flips the active note's pinned flag ───────────────────
    // App handler is `.TogglePin -> { store.TogglePin(store.active_id) }`
    // (NOTE: parameterless despite `TogglePin(int)` in the msg decl — it uses
    // store.active_id, which defaults to 0 = "Welcome", pinned:true).
    // store.TogglePin(idx): if idx < notes.len() { notes[idx].pinned = !... }

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d7_toggle_pin_flips_flag() {
        let mut dc = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        // active_id=0 → notes[0] is "Welcome", pinned:true in the seed data.
        assert_eq!(state_str(&dc, "active_id"), "0", "precondition: active_id 0");
        let before = match note_field(&dc, 0, "pinned") {
            Value::Bool(b) => b,
            other => panic!("notes[0].pinned not a bool: {:?}", other),
        };
        dc.on_with_input("TogglePin", None);
        let after = match note_field(&dc, 0, "pinned") {
            Value::Bool(b) => b,
            other => panic!("notes[0].pinned not a bool after toggle: {:?}", other),
        };
        assert_ne!(
            before, after,
            "TogglePin should flip notes[active_id].pinned ({} -> {})",
            before, after
        );
    }

    // ── D8: ToggleDarkMode flips dark_mode ──────────────────────────────────

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d8_toggle_dark_mode() {
        let mut dc = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        assert_eq!(state_str(&dc, "dark_mode"), "false", "initial dark_mode");
        dc.on_with_input("ToggleDarkMode", None);
        assert_eq!(state_str(&dc, "dark_mode"), "true", "after first toggle");
        dc.on_with_input("ToggleDarkMode", None);
        assert_eq!(state_str(&dc, "dark_mode"), "false", "after second toggle");
    }

    // ── D9: SetAccent changes accent_color ──────────────────────────────────
    // NOTE: SetAccent is declared on the NavTree child widget (sidebar.at), not
    // App. NavTree currently renders as FALLBACK in VM mode (sidebar.at's
    // `view fn` parse issue), so handler_NavTree_SetAccent isn't synthesized.
    // We verify the store method chain directly: on_with_input_for("NotesStore",
    // "SetAccent", ...) routes to handler_NotesStore_SetAccent on root state.
    // This proves the store.SetAccent(name) method call works end-to-end.

    #[cfg(feature = "ui-interpreter")]
    #[test]
    fn d9_set_accent_color() {
        let mut dc = match build_015_component() {
            Some(c) => c,
            None => {
                eprintln!("plan370: SKIPPED — app.at not found");
                return;
            }
        };
        assert_eq!(state_str(&dc, "accent_color"), "indigo", "initial accent");
        // SetAccent lives on NavTree; drive the store directly to exercise the
        // store.SetAccent(name) method-call chain (the D-GAP-4 codegen path).
        dc.on_with_input_for("NotesStore", &with_str("SetAccent", "coral"), None);
        assert_eq!(
            state_str(&dc, "accent_color"),
            "coral",
            "store.SetAccent(coral) should update accent_color"
        );
    }
}
