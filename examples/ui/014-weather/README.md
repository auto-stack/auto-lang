# 014-weather — Weather Dashboard（PLAN-660 r2）

天气仪表盘 demo：**横屏（网页/桌面）优先**，竖屏（手机版）布局已保留可切换。
数据为内置演示样本，不接真实气象 API。

## Layout modes

| 模式 | 用途 | 说明 |
|---|---|---|
| `landscape`（默认） | 网页 / 桌面 | 左 Hero + 右指标/5日，底部 24h 横滑；`max-w-6xl` 宽容器 |
| `portrait` | 未来手机版 | r1 的移动密度卡片栈；页内 `scroll (axis: "y")` |

头部「横屏 / 竖屏」按钮切换 `layout_mode`；未来手机可按视口自动选择或做响应式合并。

## Concepts

- **主题契约** — `dark_mode` / `accent_color`（Plan 458，变量名勿改）。☀️/🌙 切换。
- **滚动** — 一律 AutoUI `scroll`（别名 `scrollable`；Vue → shadcn `ScrollArea`，
  VM → iced scrollable）。城市 pill 与 24h 用 `axis: "x"` + 定高 `container`，
  竖屏页用 `axis: "y"`。**不要**写 `overflow-x-auto` 或把 `scroll-pane` 当
  plain div（`scroll-pane` 在 Vue 臂会落到 overflow 类，不是 ScrollArea）。
- **条件 Hero** — 在 **view 里用 if/else class**，不要 `style: .some_class_var`
  （生成器会绑到 `:style`，Tailwind 渐变失效）。
- **城市** — 10 城 pill（不用 `select`，`selectitem` VM 为 none）。
- **预报** — 24h 横滑卡（「现在」高亮）+ 5 日温度区间条。

## Source

```
examples/ui/014-weather/
  pac.at                     # dep stylekit；不写死 theme
  src/front/types.at
  src/front/weather_data.at  # 10 城 mock
  src/front/app.at           # layout_mode + landscape/portrait 双布局
  README.md
```

## How to Run

```bash
cd examples/ui/014-weather
auto build
auto run                      # 默认深色 + 横屏
auto run --theme light
auto run -r vm                # VM 臂抽查（pill + 主题 + scroll）
```

Category A：仅示例资产，不跑 `cargo t` / `docs_gen`。

## 验收要点（r2）

- 横屏：Hero 与指标并排，24h 用 ScrollArea 横滑（无系统灰条）
- 竖屏：可切换预览，内容纵向 ScrollArea
- 切换城市 / 主题 / 刷新可用；语义 token
