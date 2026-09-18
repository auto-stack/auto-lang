# 046-tabs-variants — Tabs 形态（PLAN-641）

Tabs 组件 `variant` 形态双端示例：`default`（按钮托盘，现状形态）与
`enclosed`（连通形态——激活 tab 与内容面板共享背景无缝，非激活 tab 扁平
仅背景色差）。

## Concepts

- **variant 形态词表** — `variant: "default" | "enclosed"`，挂 tabs 根节点；
  Chrome（顶部圆角）/ IDE（直角）观感差异走圆角 token（`rounded-t-*` class），
  不扩词表
- **受控选中** — `value: .state`（值串匹配 trigger `value`）或 `defaultvalue`
  定初始选中；点击经 `onselect`/trigger `onclick` 回调（首参=选中索引）
- **复合组件折叠（VM 轨）** — `tabs` + `tabs-list`/`tabs-trigger`/
  `tabs-content` 折叠为有状态 Tabs（此前 fallback 渲染为 Column）

## Source

See `front/app.at`.

## How to Run

```bash
cd examples/ui/046-tabs-variants
auto run              # Vue 模式（registry 组件，variant 下发 provide/inject）
auto run -r vm        # VM 模式（iced 连通渲染臂 + 复合标签折叠）
```

## 对照观感

- Section 1 `default`：按钮托盘（零回归形态）
- Section 2 `enclosed` 直角：IDE 编辑器 tab 观感
- Section 3 `enclosed` + `rounded-t-lg`：Chrome 浏览器 tab 观感
