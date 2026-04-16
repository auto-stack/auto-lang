# 001-helloworld — Static Text Display

The simplest possible AURA widget. Displays centered text on a white background.

## Concepts
- **View tree** — The `view` block defines the widget's visual structure as a nested tree of elements
- **Text widget** — `text` renders a string; use a literal string like `"Hello, World!"`
- **Col layout** — `col` arranges children vertically in a column
- **Class styling** — The `class` property applies CSS/Tailwind-style utility classes for layout and appearance

## Source

```auto
widget App {
    view {
        col {
            text "Hello, World!"
            class: "w-full h-full justify-center items-center bg-white"
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

- This example has no interactivity, so all platforms render identically
- The `class` property maps to Tailwind CSS (Vue/Tauri/VSCode), Compose modifiers (Jetpack), ArkTS attributes, or GPUI style objects
- `w-full h-full justify-center items-center` centers the text both horizontally and vertically within the full available space
