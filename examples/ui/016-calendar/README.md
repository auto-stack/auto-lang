# 016-calendar — Month-View Calendar

A month-view calendar grid with 7 columns, date cells, event highlights, and previous/next month navigation.

## Concepts

- **Grid layout** — `grid { cols: 7 }` creates a 7-column calendar grid
- **Grid items** — `grid-item { text "1" { ... } }` for individual date cells
- **Date arithmetic** — month navigation with `PrevMonth` / `NextMonth` messages
- **Event indicators** — highlighted dates with `bg-blue-100 rounded-full` styling
- **Header navigation** — prev/next buttons flanking month/year display

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { PrevMonth, NextMonth }

    model {
        var month str = "April"
        var year_display str = "2026"
        var d1 str = "Su"
        var d2 str = "Mo"
        // ... day headers
    }

    view {
        center {
            col {
                row {
                    button "<" { onclick: .PrevMonth }
                    col {
                        text .month { class: "text-2xl font-bold" }
                        text .year_display { class: "text-sm text-gray-500" }
                    }
                    button ">" { onclick: .NextMonth }
                }
                row {
                    text .d1 { class: "text-xs font-semibold text-gray-400 ..." }
                    // ... 7 day headers
                }
                grid {
                    grid-item { text "1" { class: "..." } }
                    grid-item { text "5" { class: "... bg-blue-100 rounded-full" } }
                    // ... 35 date cells
                    cols: 7
                    gap: 0
                }
                class: "w-full max-w-md p-6 bg-white rounded-2xl shadow-lg"
            }
        }
    }

    on {
        .PrevMonth -> { .month = "March" }
        .NextMonth -> { .month = "May" }
    }
}
```

## How to Run

```bash
cd examples/ui/016-calendar
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI
