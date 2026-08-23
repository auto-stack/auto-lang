# Widgets Gallery — Auto 实现的 UI widgets gallery

Auto 实现的组件文档画廊（`.at` 源码 → 生成 Vue + shadcn-vue），覆盖全部
shadcn widgets：每个组件一个页面，含示例预览（`preview-card`）、安装命令
（`codeblock`）和属性文档（`table`）。支持 **vue / vm / rust** 三端的思路，
当前 vue 端完全可用。

> 前身是 `examples/gallery`（改名而来），并吸收了原 `examples/ui/024-widgets-gallery`
> 的修复验证内容（024 目录已于 2026-08 清理删除）。

## 布局（`source/front/app.at`）

- **Header** — sticky，含移动端汉堡菜单 + 搜索
- **Sidebar**（桌面端）— 分组导航：Overview / **Layout（Plan 412）** / Form /
  Display / Feedback / Navigation / Overlay
- **内容区** — `outlet`，每个路由是一个组件文档页
- **移动端** — 底部导航栏 + drawer

## Widgets / 路由（62）

accordion, alert, alertdialog, aspectratio, avatar, badge, breadcrumb, button,
calendar, card, carousel, checkbox, collapsible, combobox, command, contextmenu,
datatable, datepicker, dialog, drawer, dropdownmenu, form, grid, hovercard,
input, label, menubar, navigationmenu, navlink, pagination, popover, progress,
radiogroup, scrollarea, select, separator, sheet, sidebar, skeleton, slider,
sonner, switch, table, tabs, textarea, toast, toggle, togglegroup, tooltip +
`/` index；**Layout 分组（Plan 412，sky 色）**：row, col, center, flex,
alignment, spacing, sizing, scroll, position, responsive, grid（重写迁入）,
grid-span。

## Plan 408 §9 修复在这里验证

| 路由 | 修复 |
|------|------|
| `/grid` | #1 — `grid` 不再是保留字（可作路由名） |
| `/slider` | #3 — Slider `value` int → `number[]`（`:default-value`） |
| `/drawer` | #4 — Drawer 需要 `vaul-vue` 依赖 |
| `/toast` | #5 — toast 标签 → `ui/sonner` 脚手架（`<Toaster/>`） |
| `/navlink` | #6 — NavLink `href` → `router-link` `to` |
| `/pagination` | #7 — shadcn-vue 正确导出名（PaginationContent…） |

（修复 #2 — Rust 模式 `outlet`/`link` 占位 — 属 codegen 层，无 vue 可见面。）

## 运行

```bash
cd examples/widgets-gallery
auto gen        # 生成各后端代码
auto run        # 全流程：pnpm install + shadcn-vue add + vite dev server
```

`auto run` 会自动装齐所需 `ui/*` shadcn 组件、打上 Sonner lucide 图标名兼容
补丁，并在 **http://localhost:3024/** 起 dev server。

## 目录结构

```
widgets-gallery/
├── pac.at              # 工作区配置（workspace.front = ./source/front）
├── source/
│   └── front/
│       ├── app.at      # 路由 + 响应式布局
│       ├── components/ # Auto 语言层组件参考定义（不参与 vue 生成）
│       └── pages/      # 每个 widget 一个文档页（62 页，含 12 页 Layout 分组）
├── vue-ref/            # 手写 Vue 参考实现（旧 component-gallery 原型，仅对比参考）
└── gen/                # 生成产物（gitignore）
    └── front/vue/
```

## 已知边界

- command / combobox / toggle-group 3 族组件目前退化为占位 div
  （缺 WidgetSpec，非 Plan 408 §9 范围），页面与文档正常展示。
  carousel 已可正常渲染 slide 内容（Basic / Sizes 示例均可见）。
- 未路由的 blocks 页已移除（避免 chart 组件脚手架风险），后续单独补。
- **Layout 分组的降级矩阵**（flex-wrap / absolute 定位 / order / self-* /
  row-span / fixed / sticky 在 VM 的行为对照）见 `/position` 页内表格与
  `docs/plans/412-layout-gallery.md` §5。
