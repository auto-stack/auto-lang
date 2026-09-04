# AutoUI Gallery — 官方示例交互式画廊与教程中心

> **Plan 549**: AutoUI 官方应用示例集成画廊。

## 概述

AutoUI Gallery 是一个集“实时内嵌运行”、“代码逐行解析”、“Elm 架构教程”于一体的一站式体验平台：
- **左侧分类导航**：完整聚合 `examples/ui/` 目录下全部 30+ 个独立 Demo，支持按分类检索与模糊过滤。
- **右侧上方**：100% 真实运行的 AutoUI 实例视口，支持交互、重置状态与多设备响应式视口切换（Desktop / Tablet / Mobile）。
- **右侧下方**：基于 AutoDown 原生渲染的详细开发教程、Elm 状态管理理念与源码查看器。

## 快速运行

```bash
cd examples/ui-gallery
auto run
```

## 技术架构

- **宿主语言**：AutoUI (`scene: "ui"`)
- **生成目标**：Vue 3 + Vite + Tailwind CSS + shadcn-vue
- **应用内嵌机制**：构建期扫描编译各 Demo SFC，运行期基于 `AppViewport` 独立挂载与沙盒隔离。
- **文档引擎**：AutoDown 核心引擎 (`@autodown/engine`) 原生渲染各 Demo 目录下的 `README.md` / `tutorial.ad`。
