# 002-counter — Interactive Counter

An increment/decrement/reset counter with three buttons demonstrating the Elm Architecture (model/view/update) pattern in AURA — in its simplest form, using inline lambda handlers (Plan 448).

## Concepts

- **Model** — The `model` block holds the widget's mutable state (`var count int = 0`)
- **Inline lambda handlers** — `onclick: () => {.count += 1}` binds a click directly to a state update; the compiler mints an anonymous event and synthesizes the handler, so simple callbacks need no `msg`/`on` round trip
- **Compound assignment** — `.count += 1` / `.count -= 1` work in handler bodies on every backend
- **Button widget** — `button` renders a clickable element with a label and event binding
- **F-string interpolation** — `` `Counter: ${.count}` `` embeds model state in displayed text

## Source

See `front/app.at`:

```auto
widget App {
    model {
        var count int = 0
    }

    view {
        center {
            text `Counter: ${.count}`
            row {
                button "-" { onclick: () => {.count -= 1} }
                button "Reset" { onclick: () => {.count = 0} }
                button "+" { onclick: () => {.count += 1} }
            }
        }
    }
}
```

When a widget outgrows the inline form (payload-carrying events, handlers
called from several places), declare the events explicitly instead:

```auto
msg { Inc, Dec, Reset }   // names are optional since Plan 448 A
on {
    .Inc -> { .count = .count + 1 }
}
// view binds: button "+" { onclick: .Inc }
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

- Elm Architecture: `model` block for state, inline lambdas or `msg`/`on` for events and updates
- `button` widget with `onclick` event binding
- `row` container for horizontal layout of buttons
- F-string interpolation with `${.count}` to display reactive model values
