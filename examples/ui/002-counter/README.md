# 002-counter — Interactive Counter

An increment/decrement/reset counter with three buttons demonstrating the Elm Architecture (model/view/update) pattern in AURA.

## Concepts
- **Model** — The `model` block holds the widget's mutable state (`var count int = 0`)
- **Msg enum** — `msg Msg { Inc, Dec, Reset }` declares the messages (events) the widget can receive
- **Event handlers** — `onclick: .Inc` binds a button click to a message; the `on` block maps messages to state updates
- **Button widget** — `button` renders a clickable element with a label and event binding
- **F-string interpolation** — `` `Counter: ${.count}` `` embeds model state in displayed text

## Source

```auto
widget Counter {
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

## Generated Output

### Vue 3

*(Placeholder: coming soon)*

### Jetpack Compose

*(Placeholder: coming soon)*

### ArkTS (HarmonyOS)

*(Placeholder: coming soon)*

### GPUI (Rust)

*(Placeholder: coming soon)*

### Tauri

*(Placeholder: coming soon)*

### VSCode WebView

*(Placeholder: coming soon)*

## Platform Notes

- The `on` block maps to platform-specific event dispatchers: Vue reactive refs, Compose `remember` + `mutableStateOf`, ArkTS `@State`, GPUI `Model`
- `.count` references the model field; the leading dot is shorthand for `self.count`
- All three buttons share the same `row` parent for horizontal layout
- Pattern matching in `on` uses `->` arrows (AutoLang convention, not `=>`)
