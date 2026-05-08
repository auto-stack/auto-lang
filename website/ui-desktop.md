# Desktop UI

<Badge type="warning" text="Coming soon" />

Auto's desktop UI backend is under active development.

## Target Backends

| Backend | Framework | Status |
|---------|-----------|--------|
| Tauri | Rust + WebView | 🚧 In progress |
| Winit | Raw windowing | 🚧 In progress |
| LVGL | Embedded C | 🚧 In progress |

## How It Works

The same Auto `view` block that generates Vue for the web will also generate platform-native desktop code:

- **Tauri**: compiles views to Vue + Rust backend
- **Winit**: compiles views to raw winit event loops
- **LVGL**: compiles views to C structs and event handlers

[← Back to UI](/ui)
