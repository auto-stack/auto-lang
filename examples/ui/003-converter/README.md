# 003-converter — Temperature Converter

A bidirectional Celsius-to-Fahrenheit converter. Editing either field instantly updates the other, with validation error display.

Inspired by 7GUIs Task #2 (Temperature Converter).

## Concepts
- **Input widget** — `input` renders an editable text field with `value` binding and `oninput` event
- **Computed properties** — Derived values calculated in event handlers rather than stored separately
- **Bidirectional data flow** — Two inputs drive each other through message handlers
- **Validation** — Parse checking with error state in the model and conditional error display
- **Pattern matching on Option** — `is parsed { Some(c) -> ..., None -> ... }` handles parse success/failure

## Source

```auto
widget Converter {
    msg Msg {
        CelsiusChanged(str),
        FahrenheitChanged(str)
    }

    model {
        var celsius str = "0"
        var fahrenheit str = "32"
        var error str = ""
    }

    view {
        col {
            text "Temperature Converter"
            class: "text-2xl font-bold mb-4"

            row {
                col {
                    text "Celsius"
                    input (value: .celsius) {
                        oninput: .CelsiusChanged
                        placeholder: "Enter Celsius"
                    }
                }

                col {
                    text "Fahrenheit"
                    input (value: .fahrenheit) {
                        oninput: .FahrenheitChanged
                        placeholder: "Enter Fahrenheit"
                    }
                }
            }

            if .error != "" {
                text .error
                class: "text-red-500 text-sm mt-2"
            }

            class: "p-6 max-w-md mx-auto gap-4"
        }
    }

    on {
        .CelsiusChanged(val) -> {
            .error = ""
            .celsius = val
            let parsed = val.to_f64()
            is parsed {
                Some(c) -> {
                    .fahrenheit = (c * 9.0 / 5.0 + 32.0).to_string()
                }
                None -> {
                    .error = "Invalid Celsius value"
                }
            }
        }
        .FahrenheitChanged(val) -> {
            .error = ""
            .fahrenheit = val
            let parsed = val.to_f64()
            is parsed {
                Some(f) -> {
                    .celsius = ((f - 32.0) * 5.0 / 9.0).to_string()
                }
                None -> {
                    .error = "Invalid Fahrenheit value"
                }
            }
        }
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

- `to_f64()` returns `Option<f64>` — `Some` on success, `None` on invalid input
- The converter formula uses floating-point arithmetic: `C * 9/5 + 32` and `(F - 32) * 5/9`
- Error display is conditional: the red text only appears when `.error` is non-empty
- On native platforms (GPUI, Compose), `input` maps to the platform's text field component with two-way binding
- Initial values are `celsius = "0"` and `fahrenheit = "32"` (0 C = 32 F)
