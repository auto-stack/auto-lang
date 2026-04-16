# 001-helloworld — Static Text Display

The simplest possible AURA widget. Displays centered text on a white background.

## Concepts

- **View tree** — The `view` block defines the widget's visual structure as a nested tree of elements
- **Text widget** — `text` renders a string; use a literal string like `"Hello, World!"`
- **Col layout** — `col` arranges children vertically in a column
- **Class styling** — The `class` property applies CSS/Tailwind-style utility classes for layout and appearance

## Source

See `front/app.at`:

```auto
widget App {
    view {
        col {
            text "Hello, World!"
            class: "w-full h-full justify-center items-center bg-white"
        }
    }
}
```

## How to Run

```bash
cd examples/ui/001-helloworld
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `vue/` — Vue 3 + shadcn-vue
- `jet/` — Jetpack Compose (Kotlin)
- `ark/` — ArkTS (HarmonyOS)
- `rust/` — Rust GPUI

## Concepts Taught

- View tree structure with nested elements
- `text` widget for displaying static strings
- `col` container for vertical layout
- `class` property for styling with Tailwind utility classes
