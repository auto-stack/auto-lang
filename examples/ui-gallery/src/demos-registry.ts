// src/demos-registry.ts — bootstrap placeholder (Plan 549)
// Overwritten on `auto run` by auto-man generator from examples/ui.
import type { Component } from 'vue'

export interface DemoMeta {
  id: string
  title: string
  category: string
  icon: string
  description: string
  tags: string[]
  doc: string
  source: string
  pac: string
  loadable: boolean
  load?: () => Promise<{ default: Component }>
}

export const DEMOS: DemoMeta[] = [
  {
    id: "001-helloworld",
    title: "001 Hello World",
    category: "01-basic",
    icon: "sparkles",
    description: "最简静态文本示例",
    tags: ["Basic", "Text", "Layout"],
    doc: "# 001-helloworld\n\n最基础的 AutoUI 示例，展示静态文本渲染与页面居中对齐。\n\n## 概念\n- `center` 容器自动实现水平垂直居中。\n- `text` 原语输出文本内容。",
    source: "widget App {\n    view {\n        center {\n            text \"Hello, AutoUI!\"\n        }\n    }\n}",
    pac: "name: \"001-helloworld\"\nscene: \"ui\"\nrender: \"vue\"",
    loadable: true,
  },
  {
    id: "002-counter",
    title: "002 计数器 (Counter)",
    category: "01-basic",
    icon: "calculator",
    description: "Elm 架构加减计数器与内联 Lambda 事件处理",
    tags: ["Elm", "State", "Lambda", "Button"],
    doc: "# 002-counter — Interactive Counter\n\nAn increment/decrement/reset counter with three buttons demonstrating the Elm Architecture (model/view/update) pattern in AURA.\n\n## Concepts\n- **Model** — The `model` block holds the widget's mutable state (`var count int = 0`)\n- **Inline lambda handlers** — `onclick: () => {.count += 1}` binds a click directly to a state update\n- **Button widget** — `button` renders a clickable element with a label and event binding\n- **F-string interpolation** — `` `Counter: ${.count}` `` embeds model state in displayed text",
    source: "widget App {\n    model {\n        var count int = 0\n    }\n    view {\n        center {\n            text `Counter: ${.count}`\n            row {\n                button \"-\" { onclick: () => {.count -= 1} }\n                button \"Reset\" { onclick: () => {.count = 0} }\n                button \"+\" { onclick: () => {.count += 1} }\n            }\n        }\n    }\n}",
    pac: "name: \"002-counter\"\nscene: \"ui\"\nrender: \"vue\"",
    loadable: true,
  }
]

export function findDemo(id: string): DemoMeta | undefined {
  return DEMOS.find((d) => d.id === id)
}

export function getCategories(): string[] {
  return ['01-basic', '02-components', '03-apps', '04-systems']
}
