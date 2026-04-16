# 008-pricing-table — Three-Tier Pricing with Toggle

Three pricing tiers (Basic, Premium, Enterprise) with a monthly/yearly toggle switch. The Premium tier is highlighted as "Most Popular". Each tier shows the price (adjusted by toggle state) and a feature list.

## Concepts
- Switch/toggle widget (monthly vs yearly)
- Conditional content (price changes with toggle state)
- List rendering (features per plan)
- Card variants (highlighted "popular" tier)

## Source

```auto
widget PricingTable {
    msg Msg { ToggleYearly }

    model {
        var is_yearly bool = false
        var plans = [
            { name: "Basic", monthly: 9, yearly: 90, features: ["5 Projects", "1GB Storage", "Email Support"] }
            { name: "Premium", monthly: 29, yearly: 290, popular: true, features: ["Unlimited Projects", "10GB Storage", "Priority Support", "API Access"] }
            { name: "Enterprise", monthly: 99, yearly: 990, features: ["Unlimited Projects", "Unlimited Storage", "24/7 Support", "API Access", "Custom Domain"] }
        ]
    }

    view {
        col {
            text "Pricing Plans"
            text "Choose the plan that fits your needs"
            switch { value: .is_yearly, label: "Yearly", onchange: .ToggleYearly }
            row {
                for plan in .plans {
                    col {
                        if plan.popular {
                            text "Most Popular"
                        }
                        text plan.name
                        if .is_yearly {
                            text f"$${plan.yearly}/yr"
                        } else {
                            text f"$${plan.monthly}/mo"
                        }
                        col {
                            for feature in plan.features {
                                text f"- $feature"
                            }
                        }
                        button "Choose Plan"
                        class: "bg-white rounded-lg shadow p-6 flex-1"
                    }
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

## Generated Output

### Vue 3
<!-- Placeholder: coming soon -->

### Jetpack Compose
<!-- Placeholder: coming soon -->

### ArkTS (HarmonyOS)
<!-- Placeholder: coming soon -->

### GPUI (Rust)
<!-- Placeholder: coming soon -->

### Tauri
<!-- Placeholder: coming soon -->

### VSCode WebView
<!-- Placeholder: coming soon -->

## Platform Notes
- The `switch` widget maps to a toggle input: `<input type="checkbox">` in Vue with switch styling, `Switch` composable in Jetpack Compose, `Toggle` in ArkTS, and a custom toggle in GPUI.
- Nested `for` loops (plans then features) demonstrate multi-level list rendering. Each generator handles this with nested iteration constructs.
- The `plan.popular` field is only present on the Premium plan. Accessing a missing field on other plans evaluates to falsy, so the "Most Popular" badge only appears on the Premium card.
- The `f"$${plan.yearly}/yr"` f-string uses `${...}` for expression interpolation. The double `$$` is not an escape -- the literal `$` before `{` is plain text, and `${plan.yearly}` is the interpolation.
- The `.is_yearly` toggle state affects all three cards simultaneously through reactive binding.
