# 006-hero-section — Landing Page Hero Block

A full-width landing page hero section with headline, subtitle, and CTA button over a gradient background.

## Concepts
- Rich text layout (headline + subtitle hierarchy)
- Button variants (CTA styling)
- Centering with flexbox (justify-center, items-center)
- Gradient background (bg-gradient-to-b)

## Source

```auto
widget HeroSection {
    view {
        col {
            text "Build Beautiful Apps"
            text "Write once, run anywhere with AutoLang"
            button "Get Started" { onclick: .GetStarted }
            class: "w-full h-full justify-center items-center bg-gradient-to-b from-blue-500 to-purple-600 text-white p-8"
        }
    }

    on {
        .GetStarted -> { print("Getting started!") }
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
- The gradient background uses Tailwind-style utility classes. On platforms without Tailwind (GPUI, Jetpack Compose), the generator maps `bg-gradient-to-b` to a native vertical gradient.
- The two `text` elements render with different visual weight: the first as a large headline (h1), the second as a subtitle (h2). Font sizing is derived from context order.
- The CTA button inherits `text-white` from the parent column. On some platforms the button may render with its own default text color unless explicitly styled.
