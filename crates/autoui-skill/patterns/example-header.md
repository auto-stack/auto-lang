# Pattern: Standard Top Title Bar & Settings Dropdown (标准顶部标题栏与主题设置组件模式)

## 适用场景 (Use Case)

为 AutoUI 示例（从 `008-pricing-table`、`009-article-feed` 起及后续所有示例）提供标准化的**顶部标题栏 (Top Title Bar)** 与 **右上角设置下拉面板 (Settings Dropdown Panel)**，支持运行期实时切换：
1. **深浅主题 (Theme: Light / Dark)**
2. **主题色板 (Accent Colors: indigo / coral / ocean / sage / amber)**

---

## 1. 契约定义与数据模型 (Contract & State Model)

每个采用本模式的组件/应用需在 `model` 与 `msg` 中声明如下标准变量与消息：

```auto
widget App {
    msg {
        ToggleSettings,
        SetTheme(str),
        SetAccent(str),
        // ... 业务自定义消息
    }

    model {
        var dark_mode bool = true
        var accent_color str = "indigo"
        var settings_open bool = false
        // ... 业务自定义状态
    }
```

---

## 2. 视图模板 (View Template)

### A. 顶部标题栏 (Top Title Bar)

```auto
// ── Top Title Bar ──
row {
    row {
        span "📰" {  // 示例专属图标 (Emoji / Icon)
            style: "text-xl"
        }
        span "Article Feed" {  // 示例名称
            style: if .dark_mode {
                "text-base font-bold text-zinc-100"
            } else {
                "text-base font-bold text-gray-900"
            }
        }
        span "009" {  // 示例三位编号 Badge
            style: if .dark_mode {
                "text-xs px-2 py-0.5 rounded-full bg-zinc-800 text-zinc-400 font-medium"
            } else {
                "text-xs px-2 py-0.5 rounded-full bg-gray-200 text-gray-600 font-medium"
            }
        }
        style: "gap-2.5 items-center"
    }

    spacer

    // Settings Trigger Button
    button "⚙ Settings" {
        variant: if .settings_open { "default" } else { "outline" }
        onclick: .ToggleSettings
        style: "px-3.5 py-1.5 text-xs font-semibold rounded-lg transition-colors cursor-pointer"
    }
    style: if .dark_mode {
        "w-full px-8 py-3.5 items-center border-b border-zinc-800 bg-zinc-900/60"
    } else {
        "w-full px-8 py-3.5 items-center border-b border-gray-200 bg-white/80"
    }
}
```

### B. 设置下拉面板 (Settings Dropdown Panel)

```auto
// ── Settings Dropdown Panel (When Open) ──
if .settings_open {
    row {
        spacer
        col {
            // Theme Section
            col {
                span "THEME" {
                    style: if .dark_mode {
                        "text-xs font-semibold text-zinc-400 uppercase tracking-wide"
                    } else {
                        "text-xs font-semibold text-gray-500 uppercase tracking-wide"
                    }
                }
                row {
                    button "☀️ Light" {
                        onclick: .SetTheme("light")
                        style: if .dark_mode {
                            "flex-1 px-3 py-1.5 text-xs rounded-lg bg-zinc-800 text-zinc-300 border border-zinc-700 hover:bg-zinc-700 transition-colors"
                        } else {
                            "flex-1 px-3 py-1.5 text-xs rounded-lg bg-primary text-primary-foreground font-semibold shadow-sm"
                        }
                    }
                    button "🌙 Dark" {
                        onclick: .SetTheme("dark")
                        style: if .dark_mode {
                            "flex-1 px-3 py-1.5 text-xs rounded-lg bg-primary text-primary-foreground font-semibold shadow-sm"
                        } else {
                            "flex-1 px-3 py-1.5 text-xs rounded-lg bg-gray-100 text-gray-700 border border-gray-200 hover:bg-gray-200 transition-colors"
                        }
                    }
                    style: "w-full gap-2 mt-1"
                }
                style: "w-full gap-1"
            }

            // Accent Section (5 Palettes)
            col {
                span "ACCENT COLOR" {
                    style: if .dark_mode {
                        "text-xs font-semibold text-zinc-400 uppercase tracking-wide mt-1.5"
                    } else {
                        "text-xs font-semibold text-gray-500 uppercase tracking-wide mt-1.5"
                    }
                }
                row {
                    button "" {
                        variant: "ghost"
                        onclick: .SetAccent("indigo")
                        style: if .accent_color == "indigo" {
                            if .dark_mode {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-indigo-500 border-2 border-white ring-2 ring-white ring-offset-2 ring-offset-zinc-900"
                            } else {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-indigo-500 border-2 border-zinc-900 ring-2 ring-zinc-900 ring-offset-2 ring-offset-white"
                            }
                        } else {
                            "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-indigo-500 opacity-80 hover:opacity-100"
                        }
                    }
                    button "" {
                        variant: "ghost"
                        onclick: .SetAccent("coral")
                        style: if .accent_color == "coral" {
                            if .dark_mode {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-rose-500 border-2 border-white ring-2 ring-white ring-offset-2 ring-offset-zinc-900"
                            } else {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-rose-500 border-2 border-zinc-900 ring-2 ring-zinc-900 ring-offset-2 ring-offset-white"
                            }
                        } else {
                            "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-rose-500 opacity-80 hover:opacity-100"
                        }
                    }
                    button "" {
                        variant: "ghost"
                        onclick: .SetAccent("ocean")
                        style: if .accent_color == "ocean" {
                            if .dark_mode {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-blue-500 border-2 border-white ring-2 ring-white ring-offset-2 ring-offset-zinc-900"
                            } else {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-blue-500 border-2 border-zinc-900 ring-2 ring-zinc-900 ring-offset-2 ring-offset-white"
                            }
                        } else {
                            "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-blue-500 opacity-80 hover:opacity-100"
                        }
                    }
                    button "" {
                        variant: "ghost"
                        onclick: .SetAccent("sage")
                        style: if .accent_color == "sage" {
                            if .dark_mode {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-emerald-500 border-2 border-white ring-2 ring-white ring-offset-2 ring-offset-zinc-900"
                            } else {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-emerald-500 border-2 border-zinc-900 ring-2 ring-zinc-900 ring-offset-2 ring-offset-white"
                            }
                        } else {
                            "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-emerald-500 opacity-80 hover:opacity-100"
                        }
                    }
                    button "" {
                        variant: "ghost"
                        onclick: .SetAccent("amber")
                        style: if .accent_color == "amber" {
                            if .dark_mode {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-amber-500 border-2 border-white ring-2 ring-white ring-offset-2 ring-offset-zinc-900"
                            } else {
                                "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-amber-500 border-2 border-zinc-900 ring-2 ring-zinc-900 ring-offset-2 ring-offset-white"
                            }
                        } else {
                            "w-6 h-6 p-0 rounded-full min-w-0 aspect-square shrink-0 bg-amber-500 opacity-80 hover:opacity-100"
                        }
                    }
                    style: "w-full gap-3 mt-1 items-center"
                }
                style: "w-full gap-1"
            }

            style: if .dark_mode {
                "w-64 p-4 rounded-xl bg-zinc-900 text-zinc-100 border border-zinc-800 shadow-2xl gap-3 mr-8 -mb-40 z-50"
            } else {
                "w-64 p-4 rounded-xl bg-white text-gray-900 border border-gray-200 shadow-2xl gap-3 mr-8 -mb-40 z-50"
            }
        }
        style: "w-full"
    }
}
```

---

## 3. 事件处理契约 (Event Handlers)

```auto
on {
    .ToggleSettings -> { .settings_open = !.settings_open }
    .SetTheme(t) -> {
        if t == "light" {
            .dark_mode = false
        }
        if t == "dark" {
            .dark_mode = true
        }
    }
    .SetAccent(c) -> { .accent_color = c }
}
```

---

## 4. `pac.at` 默认配置规范

在 `pac.at` 中显式指定默认暗色主题与预设主色：

```yaml
theme: "dark"
accent: "indigo"
```
