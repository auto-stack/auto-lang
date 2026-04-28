# 022-kanban — Trello-Style Board

A Kanban board with three columns (To Do, In Progress, Done) where cards can be moved between columns.

## Concepts

- **Column layout** — three side-by-side columns using `row` with `flex-shrink-0` cards
- **Card movement** — `MoveToInProgress` / `MoveToDone` messages shift cards between columns by reassigning model vars
- **Count badges** — item counts displayed as `rounded-full` badges next to column headers
- **Column theming** — each column has distinct background colors (`bg-gray-50`, `bg-orange-50`, `bg-green-50`)
- **Action buttons** — `>` button on each card to move it to the next column

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { MoveToInProgress, MoveToDone, AddCard }

    model {
        var todo1 str = "Design landing page"
        var todo2 str = "Write API docs"
        var todo3 str = "Fix login bug"
        var todo4 str = "Setup CI/CD"
        var prog1 str = "Implement auth flow"
        var prog2 str = "Build dashboard"
        var done1 str = "Setup database"
        var done2 str = "Create user model"
        var done3 str = "Design logo"
        var todo_count str = "4"
        var prog_count str = "2"
        var done_count str = "3"
    }

    view {
        col {
            row {
                text "Project Board" { class: "text-xl font-bold" }
                button "+ Add Card" { onclick: .AddCard }
            }
            row {
                col {
                    row { text "To Do"; text .todo_count { class: "bg-gray-100 rounded-full" } }
                    col {
                        row { text .todo1; button ">" { onclick: .MoveToInProgress } }
                        row { text .todo2; button ">" { onclick: .MoveToInProgress } }
                        // ... more cards
                    }
                    class: "w-80 bg-gray-50 rounded-xl"
                }
                col {
                    row { text "In Progress"; text .prog_count { class: "bg-orange-50 rounded-full" } }
                    col { row { text .prog1; button ">" { onclick: .MoveToDone } } }
                    class: "w-80 bg-orange-50 rounded-xl"
                }
                col {
                    row { text "Done"; text .done_count { class: "bg-green-50 rounded-full" } }
                    col { text .done1; text .done2; text .done3 }
                    class: "w-80 bg-green-50 rounded-xl"
                }
            }
        }
    }

    on {
        .MoveToInProgress -> {
            .prog1 = .todo1
            .todo1 = .todo2
            .todo2 = .todo3
            .todo3 = .todo4
            .todo4 = ""
            .todo_count = "3"
            .prog_count = "3"
        }
    }
}
```

## How to Run

```bash
cd examples/ui/022-kanban
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Notes

Due to parser limitations, drag-and-drop reorder is not implemented. Cards move via explicit `>` button clicks. True drag reorder requires future drag-and-drop support in AURA.

## Inspiration

Trello, Linear.
