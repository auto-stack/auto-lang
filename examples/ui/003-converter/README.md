# 003-converter — Temperature Converter

A bidirectional Celsius/Fahrenheit converter. Editing either field updates the model state.

Inspired by 7GUIs Task #2 (Temperature Converter).

## Concepts

- **Input widget** — `input` renders an editable text field with `value` binding and `oninput` event
- **Model state** — String fields hold the current Celsius and Fahrenheit values
- **Event binding** — `oninput` fires on every keystroke, updating the model via message handlers

## Source

See `front/app.at`:

```auto
widget App {
    msg Msg { CelsiusChanged, FahrenheitChanged }

    model {
        var celsius str = "0"
        var fahrenheit str = "32"
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

            class: "p-6 max-w-md mx-auto gap-4"
        }
    }

    on {
        .CelsiusChanged -> {
            .celsius = .celsius
        }
        .FahrenheitChanged -> {
            .fahrenheit = .fahrenheit
        }
    }
}
```

## How to Run

```bash
cd examples/ui/003-converter
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `vue/` — Vue 3 + shadcn-vue
- `jet/` — Jetpack Compose (Kotlin)
- `ark/` — ArkTS (HarmonyOS)
- `rust/` — Rust GPUI

## Concepts Taught

- `input` widget with `value` binding to model fields
- `oninput` event for responding to text field changes
- `placeholder` property for input hint text
- `row` and `col` nesting for side-by-side input layout
- Model state with `var` for mutable string values
