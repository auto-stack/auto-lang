# vue-ref — 手写 Vue 参考实现（旧 component-gallery 原型）

这是 **Auto 实现之前**的手写 Vue 参考版组件画廊，即最早的
`examples/component-gallery` 时代的原型（`ComponentDocPage` / `DemoSection` /
`InstallationBlock` 等文档组件都是手工写的）。

保留此目录仅用于**对比参考**：理解 Auto 语言 `.at` 源码 → Vue 生成结果，
与手写实现之间的差异。它不是生成产物，不参与 `auto gen` / `auto run` 流程。

## 与各版本的关系

| 目录/产物 | 说明 |
|-----------|------|
| `examples/component-gallery` | 最早版本（手写 vue 原型），已在 2026-06-25 改名 `gallery`，仓库中已不存在 |
| `examples/gallery` | 中间版（.at 源码 + 手写 vue 参考），今天改名 `widgets-gallery` |
| `examples/widgets-gallery/` | **当前** Auto 实现版：`.at` 源码（`source/front/`）+ 自动生成的 `gen/front/vue` |
| `examples/widgets-gallery/vue-ref/` | 本目录 = 被保留下来的手写 vue 参考 |
| `website/public/ui/gallery/` | 老版本（component-gallery）的**静态构建产物**，即 https://auto-lang.cn/ui/gallery/#/ 展示的内容（`index.html` 标题仍为 `component-gallery`，2026-05-19 构建） |

## 本目录内容

- `src/components/ComponentDocPage.vue`、`DemoSection.vue`、`InstallationBlock.vue` — 手写的文档页/示例/安装说明组件
- `src/pages/*.vue` — 手写的组件文档页（每组件一页）
- `src/prismjs.d.ts` — Prism.js 类型 shim

> 提示：新画廊页面的 `preview-card` / `codeblock` 交互（预览/代码切换、复制按钮）
> 现在由编译器从 `.at` 自动生成，手写版里的 `DemoSection`/`InstallationBlock`
> 是这些能力的"参考原型"。
