# Pattern: Store Composable

## Use case

Defining reactive state shared across multiple widgets via Vue composables
(Pinia-style stores).

## Standard .at structure

```auto
store NotesStore {
    model {
        var notes: Note[] = []
        var active_id int = 0
        var loading bool = false
        var error str = ""

        // Plan 360: accent theming
        var accent_color str = "indigo"
    }

    // Computed values (Plan 367 P2-2)
    computed active_note Note {
        if .notes.len() > 0 && .active_id < .notes.len() {
            return .notes[.active_id]
        }
        return Note{}
    }

    // API interactions
    use back.api: list_notes, create_note, update_note, delete_note

    on {
        .Init -> {
            .loading = true
            .notes = list_notes()
            .loading = false
        }
        .CreateNote(title str, body str) -> {
            create_note(title, body, "")
            .Init()  // refresh
        }
        .UpdateNote(id int, title str, body str) -> {
            update_note(id, title, body)
            .Init()
        }
        .DeleteNote(id int) -> {
            delete_note(id)
            .Init()
        }
        .SetActive(id int) -> {
            .active_id = id
        }

        // Plan 360: accent theming
        .SetAccent(name str) -> {
            .accent_color = name
        }
        .ToggleDarkMode -> {
            .dark_mode = !.dark_mode
        }
    }
}
```

## Key constraints

- Store name must match `use store: Name` in widgets (C2)
- `use back.api:` declarations connect store handlers to REST endpoints
- Computed fields (Plan 367) reduce duplication in widgets
- `active_id` pattern: stores the selected item index, used by list-detail layout
- If accent theming is used (Plan 360): declare `accent_color` + `SetAccent` + `ToggleDarkMode`

## How widgets consume the store

```auto
widget Sidebar {
    use store: NotesStore  // ← C2: must match store name

    view {
        for note in store.notes {
            button note.title { onclick: store.SetActive(i) }
        }
    }
}
```

## Pitfalls

- **P3**: `use store:` must use the `store` path. Alternative syntax
  (`use back.store:`) is not recognized (C2)
- Store handlers that call API functions: `use back.api:` must list
  all called functions
- Computed fields read like state vars but are derived — don't assign to them

## Validation checklist

- [ ] Store declared with `store Name { ... }` syntax
- [ ] Widgets reference via `use store: Name` (exact match)
- [ ] API functions declared in `use back.api:`
- [ ] No `store is not defined` in generated output (R002)
