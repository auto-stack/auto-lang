# 002-counter — Interactive Counter

An increment/decrement/reset counter with three buttons demonstrating the Elm Architecture (model/view/update) pattern in AURA.

## Concepts

- **Model** — The `model` block holds the widget's mutable state (`var count int = 0`)
- **Msg enum** — `msg Msg { Inc, Dec, Reset }` declares the messages (events) the widget can receive
- **Event handlers** — `onclick: .Inc` binds a button click to a message; the `on` block maps messages to state updates
- **Button widget** — `button` renders a clickable element with a label and event binding
- **F-string interpolation** — `` `Counter: ${.count}` `` embeds model state in displayed text

## Source

See `front/app.at`:

```auto
widget App {
    msg Msg { Inc, Dec, Reset }

    model {
        var count int = 0
    }

    view {
        col {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: .Dec }
                button "Reset" { onclick: .Reset }
                button "+" { onclick: .Inc }
            }
        }
    }

    on {
        .Inc -> { .count = .count + 1 }
        .Dec -> { .count = .count - 1 }
        .Reset -> { .count = 0 }
    }
}
```

## How to Run

```bash
cd examples/ui/002-counter
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `vue/` — Vue 3 + shadcn-vue
- `jet/` — Jetpack Compose (Kotlin)
- `ark/` — ArkTS (HarmonyOS)
- `rust/` — Rust GPUI

## Concepts Taught

- Elm Architecture: `model` block for state, `msg` enum for events, `on` block for updates
- `button` widget with `onclick` event binding
- `row` container for horizontal layout of buttons
- F-string interpolation with `${.count}` to display reactive model values
- Pattern matching in `on` block with `->` arrows
