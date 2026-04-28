# 013-todo — Todo List with Checkbox and Delete

A simplified todo list with add input, toggle done (checkbox), and delete functionality. Shows active item count.

## Concepts

- **List CRUD** — add, toggle, and delete todo items using individual model vars (no array literals in model yet)
- **Checkbox widget** — `checkbox { checked: .todo1_done, onclick: .Toggle1 }` for toggle state
- **Input widget** — `input { placeholder, value, oninput }` for adding new todos
- **Conditional logic** — `if .todo1_done` to adjust active count on toggle
- **Boolean state** — `bool` type for checked/unchecked state

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { Toggle1, Delete1, InputChanged }

    model {
        var input str = ""
        var todo1_text str = "Hello"
        var todo1_done bool = false
        var todo2_text str = "World"
        var todo2_done bool = true
        var active_count int = 1
    }

    view {
        center {
            col {
                row {
                    input { placeholder: "Add todo", value: .input, oninput: .InputChanged }
                }
                row {
                    checkbox { checked: .todo1_done, onclick: .Toggle1 }
                    text .todo1_text
                    button "x" { onclick: .Delete1 }
                }
                row {
                    checkbox { checked: .todo2_done }
                    text .todo2_text
                }
                text `Active: ${.active_count}`
            }
        }
    }

    on {
        .Toggle1 -> {
            .todo1_done = .todo1_done == false
            if .todo1_done { .active_count = .active_count - 1 } else { .active_count = .active_count + 1 }
        }
        .Delete1 -> {
            .todo1_text = .todo2_text
            .todo1_done = .todo2_done
            .todo2_text = ""
        }
    }
}
```

## How to Run

```bash
cd examples/ui/013-todo
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Notes

Due to parser limitations (no object/array literals in model), each todo item is stored as separate `var todoX_text` / `var todoX_done` pairs. Full TodoMVC compliance with filtering and inline editing requires future parser enhancements for typed msg variants and control flow in on handlers.

## Inspiration

TodoMVC (todomvc.com).
