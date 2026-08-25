# 008-pricing-table — Three-Tier Pricing Table

Three pricing tiers (Basic, Premium, Enterprise) with a monthly/yearly toggle switch.

## Concepts

- Switch widget for toggling state
- Conditional display with boolean model field
- Card layout in a horizontal row
- Message handling to flip a boolean

## Source

See `front/app.at`:

```auto
widget App {
    msg { ToggleYearly }

    model {
        var is_yearly bool = false
        var plan1_name str = "Basic"
        var plan1_price str = "$9/mo"
        var plan2_name str = "Premium"
        var plan2_price str = "$29/mo"
        var plan3_name str = "Enterprise"
        var plan3_price str = "$99/mo"
    }

    view {
        col {
            text "Pricing Plans"
            text "Choose the plan that fits your needs"
            switch { value: .is_yearly, label: "Yearly", onchange: .ToggleYearly }
            row {
                col {
                    text .plan1_name
                    text .plan1_price
                    button "Choose Plan"
                    class: "bg-white rounded-lg shadow p-6 flex-1"
                }
                col {
                    text .plan2_name
                    text .plan2_price
                    text "Most Popular"
                    button "Choose Plan"
                    class: "bg-white rounded-lg shadow p-6 flex-1"
                }
                col {
                    text .plan3_name
                    text .plan3_price
                    button "Choose Plan"
                    class: "bg-white rounded-lg shadow p-6 flex-1"
                }
                class: "gap-6"
            }
            class: "w-full p-8 gap-6 items-center"
        }
    }

    on {
        .ToggleYearly -> { .is_yearly = !.is_yearly }
    }
}
```

## How to Run

```bash
cd examples/ui/008-pricing-table
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `vue/` — Vue 3 + shadcn-vue
- `jet/` — Jetpack Compose (Kotlin)
- `ark/` — ArkTS (HarmonyOS)
- `rust/` — Rust GPUI

## Concepts Taught

- Switch widget: `switch` with `value`, `label`, and `onchange` properties
- Message handling: `ToggleYearly` message flips the `is_yearly` boolean with `!`
- Card layout: three `col` cards in a `row` with `flex-1` for equal width
- Static badge: "Most Popular" text on the Premium card
- Centered layout: `items-center` on the parent column
