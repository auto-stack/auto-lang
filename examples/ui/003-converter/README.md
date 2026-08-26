# 003-converter — Temperature Converter

A bidirectional Celsius/Fahrenheit converter. Editing either field updates the model state and re-computes the other field — using inline lambda handlers (Plan 448), no `msg`/`on` round trip needed.

Inspired by 7GUIs Task #2 (Temperature Converter).

## Concepts

- **Input widget** — `input` renders an editable text field with `value` binding (folds to two-way `v-model`) and `placeholder`
- **Inline lambda handlers** — `oninput: () => {.fahrenheit = …}` binds typing directly to a state update; the compiler mints an anonymous event and synthesizes the handler
- **Model state** — `double` fields hold the current Celsius and Fahrenheit values
- **Design tokens** — `text-primary`, `bg-card`, `text-muted-foreground` style via the shadcn theme tokens instead of hardcoded colors
- **Layout spacing** — `gap-4` on the `row` keeps the two fields apart; the card container (`border rounded-2xl shadow-sm`) frames the scene

## Source

See `front/app.at`:

```auto
widget App {
    model {
        var celsius double = 0.0
        var fahrenheit double = 32.0
    }

    view {
        center {
            col {
                style: "w-full max-w-md p-8 bg-card border rounded-2xl shadow-sm mx-4"

                text "Temperature Converter" {
                    style: "text-2xl font-bold text-primary text-center mb-6"
                }

                row {
                    style: "gap-4"

                    col {
                        style: "flex-1 gap-1.5"
                        text "Celsius (°C)" {
                            style: "text-sm font-medium text-muted-foreground"
                        }
                        input (value: .celsius) {
                            oninput: () => {.fahrenheit = math.round((.celsius * 9.0 / 5.0 + 32.0) * 100.0) / 100.0}
                            placeholder: "0"
                        }
                    }

                    col {
                        style: "flex-1 gap-1.5"
                        text "Fahrenheit (°F)" {
                            style: "text-sm font-medium text-muted-foreground"
                        }
                        input (value: .fahrenheit) {
                            oninput: () => {.celsius = math.round((.fahrenheit - 32.0) * 5.0 / 9.0 * 100.0) / 100.0}
                            placeholder: "32"
                        }
                    }
                }

                text "°F = °C × 9/5 + 32" {
                    style: "text-xs text-muted-foreground text-center mt-6"
                }
            }
        }
    }
}
```

If an event later needs a payload or its logic is shared across several
bindings, declare it explicitly instead:

```auto
msg { CelsiusChanged, FahrenheitChanged }
on {
    .CelsiusChanged -> { .fahrenheit = math.round((.celsius * 9.0 / 5.0 + 32.0) * 100.0) / 100.0 }
}
// view binds: input (value: .celsius) { oninput: .CelsiusChanged }
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

- `input` widget with `value` binding to model fields (two-way via `v-model`)
- `oninput` inline lambda for responding to text field changes
- `placeholder` property for input hint text
- `row`/`col` nesting with `gap-*` for side-by-side spaced layout
- Theming with design tokens (`text-primary`, `bg-card`, `text-muted-foreground`)
