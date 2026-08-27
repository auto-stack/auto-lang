# 006-hero-section — Landing Page Hero Section

A full-width landing page hero with headline, subtitle, and CTA button over a theme-aware background.

**Plan 458: 首个"主题敏感"示例** —— 默认 Dark（渐变 + 白字），右上角 **Theme
Settings** 面板可在运行中切换 Light/Dark 与 5 个主题主色（accent）。

## Concepts

- Text hierarchy (headline + subtitle)
- Button with click handler
- Theme-aware styling (`style: if .dark_mode { ... } else { ... }`)
- Gradient background styling (dark) / light surface (light)
- Theme settings panel: `dark_mode` / `accent_color` 状态变量（双端运行时契约）
- Accent swatches (indigo / coral / ocean / sage / amber)

## Source

See `src/front/app.at`（节选）:

```auto
widget App {
    msg { GetStarted, ToggleSettings, SetTheme(str), SetAccent(str) }

    model {
        var dark_mode bool = true      // 双端主题钩子：变量名即契约
        var accent_color str = "indigo" // 双端主色钩子
        var settings_open bool = false
    }

    view {
        col {
            // 右上角 Theme Settings 触发按钮 + 弹出面板（Theme/Accent）
            // ... 面板内部用 style: if 切换选中态 ...
            style: if .dark_mode {
                "w-full h-full min-h-screen bg-gradient-to-b from-blue-500 to-purple-600 text-white"
            } else {
                "w-full h-full min-h-screen bg-gray-50 text-gray-900"
            }
        }
    }

    on {
        .GetStarted -> { print("Getting started!") }
        .ToggleSettings -> { .settings_open = !.settings_open }
        .SetTheme(t) -> { /* t == "light" → .dark_mode = false ... */ }
        .SetAccent(name) -> { .accent_color = name }
    }
}
```

## How to Run

```bash
cd examples/ui/006-hero-section
auto run                    # 默认 Dark（pac.at theme: "dark"）
auto run --theme light      # CLI 覆盖：浅色启动
auto run --theme light --accent ocean   # 同时指定主色
auto run --render vm        # VM/Iced 原生窗口（同样吃 --theme/--accent）
```

启动后点右上角 **⚙ Theme** 展开面板：Light/Dark 切换主题，5 个色点切换主色
（CTA 按钮颜色随之变化）。CLI/pac.at 只是启动默认值，应用内切换即视即生效。

## Concepts Taught

- Text hierarchy: two `text` elements render as headline and subtitle based on order
- Button with `onclick` handler bound to a message
- Theme-sensitive colors: text/background always paired per theme branch
- `dark_mode` / `accent_color` root state vars as the dual-backend runtime theme contract
- Message handling in the `on` block with `print()`
