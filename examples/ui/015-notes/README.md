# 015-notes — Split-Pane Note-Taking App

A split-pane layout with a searchable sidebar of notes and a main editor area showing note content.

## Concepts

- **Split pane layout** — `row` with a fixed-width sidebar (`w-64`) and a flexible main area (`flex-1`)
- **Search/filter** — input field with `oninput` binding for note search
- **Selection state** — highlighted note in sidebar using different text classes (`text-blue-600` for selected)
- **h2 heading** — `h2 "Notes"` for section headings
- **Nested columns** — multiple `col` blocks for sidebar items and content areas

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { NewNote, SearchChanged }

    model {
        var search str = ""
        var note_count str = "3 notes"
        var note1_title str = "Welcome"
        var note1_body str = "This is your notes app."
        var note1_time str = "Just now"
        var note2_title str = "Shopping List"
        var note2_body str = "Milk, Eggs, Bread"
        var note2_time str = "2 hours ago"
        var note3_title str = "Meeting Notes"
        var note3_body str = "Q3 roadmap discussion"
        var note3_time str = "Yesterday"
    }

    view {
        col {
            row {
                h2 "Notes" { class: "text-xl font-bold text-gray-800" }
                button "+ New Note" { onclick: .NewNote }
            }
            row {
                col {
                    input { placeholder: "Search...", oninput: .SearchChanged }
                    col { text .note1_title { class: "...text-blue-600" } }
                    col { text .note2_title { class: "...text-gray-700" } }
                    col { text .note3_title { class: "...text-gray-700" } }
                    class: "w-64 border-r"
                }
                col {
                    text .note1_title { class: "text-lg font-semibold p-6" }
                    text .note1_body { class: "text-gray-700 px-6" }
                    class: "flex-1"
                }
            }
        }
    }
}
```

## How to Run

```bash
cd examples/ui/015-notes
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI
