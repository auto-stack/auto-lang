# 007-stats-board — Dashboard Metrics with Progress Bars

Four metric cards showing revenue, active users, orders, and growth rate. Each card has a title, value, and progress bar indicating completion percentage.

## Concepts
- Progress widget (value-based fill bar)
- Repeated card patterns (for loop over model data)
- Data display (key metrics)
- Responsive grid layout (row with flex-1 cards)

## Source

```auto
widget StatsBoard {
    model {
        var metrics = [
            { title: "Revenue", value: "$256K", progress: 75 }
            { title: "Users", value: "1,453", progress: 60 }
            { title: "Orders", value: "83M", progress: 90 }
            { title: "Growth", value: "+12%", progress: 45 }
        ]
    }

    view {
        col {
            text "Dashboard Overview"
            row {
                for m in .metrics {
                    col {
                        text m.title
                        text m.value
                        progress { value: m.progress }
                        class: "bg-white rounded-lg shadow p-4 flex-1"
                    }
                }
                class: "gap-4"
            }
            class: "w-full p-8 gap-6"
        }
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
- The `progress` widget maps to a native progress bar on each platform: `<progress>` in HTML, `LinearProgressIndicator` in Jetpack Compose, `Progress` in ArkTS, and a custom rendered bar in GPUI.
- The `for m in .metrics` loop iterates over the model array, creating one card per metric. On platforms without reactive list rendering (GPUI), the generator unrolls the loop at compile time.
- `flex-1` ensures all cards share equal width in the row. On platforms without flexbox (Jetpack Compose), this maps to `Modifier.weight(1f)`.
- Object literal access (`m.title`, `m.value`, `m.progress`) uses dot notation consistent across all generators.
