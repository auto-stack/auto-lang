# 024-widget-gallery — Comprehensive Component Showcase

A shadcn/ui-style component gallery with a sidebar listing all widgets and a main area showing each widget demo with live preview.

## Concepts

- **Sidebar navigation** — `col` with widget names (Button, Input, Toggle, Select, etc.) for navigation
- **Conditional content switching** — `if .current_tab == "button"` / `"input"` / ... shows different widget demos
- **All widgets in one app** — demonstrates button variants, input fields, textarea, toggle, select, progress, table, badge, card, avatar, tabs, and divider
- **Widget demos** — each section shows the widget with descriptive text and interactive examples
- **Button variants** — primary, secondary, danger, and outline button styles
- **Toggle widget** — `toggle { value: .toggle_on, onchange: .ToggleNotif }` for on/off switching
- **Table display** — manual row-based table with header and data rows
- **Avatar sizes** — multiple `avatar` elements at different sizes (`w-8`, `w-10`, `w-12`, `w-16`)

## Source

See `src/front/app.at`:

```auto
widget App {
    model {
        var current_tab str = "button"
        var btn_text str = "Click me"
        var input_val str = ""
        var toggle_on bool = false
        var dark_mode bool = false
        var selected str = "Option 1"
        var progress_val int = 65
        // ... table data, badge labels, card content
    }

    view {
        row {
            // Sidebar
            col {
                h3 "Widgets"
                col {
                    span "Button"
                    span "Input"
                    span "Textarea"
                    span "Toggle"
                    span "Select"
                    span "Progress"
                    span "Table"
                    span "Badge"
                    span "Card"
                    span "Avatar"
                    span "Tabs"
                    span "Divider"
                }
                class: "w-48 bg-gray-50 p-4 border-r h-full"
            }

            // Content area
            col {
                if .current_tab == "button" {
                    col {
                        h2 "Button"
                        row {
                            button .btn_text
                            button "Primary" { class: "bg-blue-500 text-white rounded-lg" }
                            button "Danger" { class: "bg-red-500 text-white rounded-lg" }
                        }
                    }
                }
                if .current_tab == "toggle" {
                    col {
                        h2 "Toggle"
                        toggle { value: .toggle_on, onchange: .ToggleNotif }
                    }
                }
                // ... 10+ more widget sections
            }
        }
    }

    on {
        .ToggleNotif -> { .toggle_on = !.toggle_on }
        .ToggleDark -> { .dark_mode = !.dark_mode }
    }
}
```

## How to Run

```bash
cd examples/ui/024-widget-gallery
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Notes

The sidebar navigation is currently visual-only — clicking widget names does not switch tabs. Full interactivity requires `msg` variants with parameters or a routing system, which are planned future enhancements.

## Inspiration

shadcn/ui docs, PrimeReact component gallery.
