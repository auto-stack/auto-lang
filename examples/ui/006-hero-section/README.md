# 006-hero-section — Landing Page Hero Section

A full-width landing page hero with headline, subtitle, and CTA button over a gradient background.

## Concepts

- Text hierarchy (headline + subtitle)
- Button with click handler
- Gradient background styling
- Centering with flexbox (justify-center, items-center)

## Source

See `front/app.at`:

```auto
widget App {
    msg Msg { GetStarted }

    view {
        col {
            text "Build Beautiful Apps"
            text "Write once, run anywhere with AutoLang"
            button "Get Started" { onclick: .GetStarted }
            class: "w-full h-full justify-center items-center bg-gradient-to-b from-blue-500 to-purple-600 text-white p-8 gap-4"
        }
    }

    on {
        .GetStarted -> { print("Getting started!") }
    }
}
```

## How to Run

```bash
cd examples/ui/006-hero-section
auto gen              # Generate code for all backends (vue, jet, ark, rust)
auto run              # Run dev server
```

After `auto gen`, generated projects appear in:
- `vue/` — Vue 3 + shadcn-vue
- `jet/` — Jetpack Compose (Kotlin)
- `ark/` — ArkTS (HarmonyOS)
- `rust/` — Rust GPUI

## Concepts Taught

- Text hierarchy: two `text` elements render as headline and subtitle based on order
- Button with `onclick` handler bound to a message
- Gradient background via `bg-gradient-to-b from-blue-500 to-purple-600`
- Full-page centering using `justify-center items-center` on the root column
- Message handling in the `on` block with `print()`
