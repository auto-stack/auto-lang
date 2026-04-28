# 011-calculator — Four-Function Calculator

Basic calculator with a display and a 5-row button grid (0-9, +, -, *, /, =, C, %, .).

## Concepts

- **Grid layout** — 4-column button grid arranged using nested `row` elements
- **Complex state machine** — chained operations with `display`, `prev_value`, `operator`, and `new_number` flags
- **Conditional logic in handlers** — `if` statements in `on` blocks to differentiate new-number vs append behavior
- **Class variants** — distinct button styles (`calc-btn-func`, `calc-btn-op`, `calc-btn-num`, `calc-btn-eq`)

## Source

See `src/front/app.at`:

```auto
widget App {
    msg Msg { Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9, Add, Sub, Mul, Div, Equals, Clear, Dot, Percent }

    model {
        var display str = "0"
        var prev_value double = 0.0
        var operator str = ""
        var new_number bool = true
    }

    view {
        center {
            col {
                // Display
                col {
                    text .display {
                        class: "text-5xl font-light text-gray-800 text-right w-full"
                    }
                    text .operator {
                        class: "text-sm text-gray-400 text-right w-full"
                    }
                    class: "w-full p-6 bg-gray-900 rounded-t-2xl"
                }

                // Button grid: 4 columns
                col {
                    row {
                        button "C" { onclick: .Clear, class: "calc-btn-func" }
                        button "%" { onclick: .Percent, class: "calc-btn-func" }
                        button "/" { onclick: .Div, class: "calc-btn-op" }
                        button "*" { onclick: .Mul, class: "calc-btn-op" }
                    }
                    row {
                        button "7" { onclick: .Digit7, class: "calc-btn-num" }
                        button "8" { onclick: .Digit8, class: "calc-btn-num" }
                        button "9" { onclick: .Digit9, class: "calc-btn-num" }
                        button "-" { onclick: .Sub, class: "calc-btn-op" }
                    }
                    row {
                        button "4" { onclick: .Digit4, class: "calc-btn-num" }
                        button "5" { onclick: .Digit5, class: "calc-btn-num" }
                        button "6" { onclick: .Digit6, class: "calc-btn-num" }
                        button "+" { onclick: .Add, class: "calc-btn-op" }
                    }
                    row {
                        button "1" { onclick: .Digit1, class: "calc-btn-num" }
                        button "2" { onclick: .Digit2, class: "calc-btn-num" }
                        button "3" { onclick: .Digit3, class: "calc-btn-num" }
                        button "=" { onclick: .Equals, class: "calc-btn-eq" }
                    }
                    row {
                        button "0" { onclick: .Digit0, class: "calc-btn-num calc-btn-wide" }
                        button "." { onclick: .Dot, class: "calc-btn-num" }
                    }
                    class: "w-full p-2 bg-gray-800 rounded-b-2xl gap-1"
                }
                class: "w-80 bg-gray-900 rounded-2xl overflow-hidden shadow-xl"
            }
        }
    }

    on {
        .Clear -> { .display = "0"; .prev_value = 0.0; .operator = ""; .new_number = true }
        .Digit1 -> { if .new_number { .display = "1" } else { .display = .display + "1" }; .new_number = false }
        // ... other digits follow same pattern
        .Add -> { .operator = "+"; .new_number = true }
        .Equals -> { .display = "0"; .operator = ""; .new_number = true }
    }
}
```

## How to Run

```bash
cd examples/ui/011-calculator
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `gen/vue/` — Vue 3 + shadcn-vue
- `gen/jet/` — Jetpack Compose (Kotlin)
- `gen/ark/` — ArkTS (HarmonyOS)
- `gen/rust/` — Rust GPUI

## Inspiration

Flutter Simplistic Calculator, iced calculator examples.
