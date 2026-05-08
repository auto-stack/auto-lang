# 桌面端 UI

<Badge type="warning" text="即将推出" />

Auto 的桌面端 UI 后端正在积极开发中。

## 目标后端

| 后端 | 框架 | 状态 |
|------|------|------|
| Tauri | Rust + WebView | 🚧 开发中 |
| Winit | 原生窗口 | 🚧 开发中 |
| LVGL | 嵌入式 C | 🚧 开发中 |

## 工作原理

同一个 Auto `view` 块，既可以为 Web 生成 Vue，也可以生成桌面端原生代码：

- **Tauri**：编译视图为 Vue + Rust 后端
- **Winit**：编译视图为原生 winit 事件循环
- **LVGL**：编译视图为 C 结构体和事件处理器

[← 返回 UI](/zh/ui)
