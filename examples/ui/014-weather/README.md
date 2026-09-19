# 014-weather — Weather Dashboard（PLAN-660 r6）

天气仪表盘：**横屏（网页/桌面）优先**，竖屏布局可切换。`pac.at` 声明
`window: "fit"`（同 calculator），外壳无 `min-h-screen`，窗口随内容收缩；
整 app 套一层 card 框（`max-w-4xl mx-auto`）。数据为内置演示样本。

## Layout modes

| 模式 | 用途 | 说明 |
|---|---|---|
| `landscape`（默认） | 网页 / 桌面 | 左 Hero（高度对齐右栏）+ 右「预报 Tab + 指标」 |
| `portrait` | 未来手机版 | 移动密度卡片；预报同样 Tab 化 |

预报区：**24小时 / 5日** Tab 互斥；五日卡与小时卡同构（最高/最低两行）。

## Concepts

- **窗口 fit** — `window: "fit"` + 无 min-h-screen（fit 按内容量高）
- **主题契约** — `dark_mode` / `accent_color`（Plan 458）
- **滚动** — AutoUI `scroll` → Vue `ScrollArea`
- **Hero** — view 内按 condition 的 if/else class（勿绑到 `style:`）
- **城市** — 10 城 pill；**toggle hover** 选中态必须自带 `hover:bg-primary/90`

## How to Run

```bash
cd examples/ui/014-weather
auto build
auto run                 # fit 窗口 + 横屏
auto run --theme light
auto run -r vm
# 冒烟：C:\Python314\python.exe tests/vm_smoke.py
```
