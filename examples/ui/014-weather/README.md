# 014-weather — Weather Dashboard（PLAN-660）

天气仪表盘：**横屏（网页/桌面）优先**，竖屏布局可切换。`pac.at` 声明固定
初窗 `window: "960x680"`——**不用** `window: "fit"`（Plan 512：fit +
`lg:`/`max-w` 系在 iced 量测不稳，可能把窗量成极小、UI「只剩一张卡」）；
宽度全用固定 Tailwind 刻度（Hero `w-64`、外壳 `w-96` 系），整 app 套一层
card 框（`app_frame`）。数据为内置演示样本。

## Layout modes

| 模式 | 用途 | 说明 |
|---|---|---|
| `landscape`（默认） | 网页 / 桌面 | 左 Hero + 右「预报 Tab + 指标」 |
| `portrait` | 未来手机版 | 移动密度卡片；预报同样 Tab 化 |

预报区：**24小时 / 5日** Tab 互斥；五日卡与小时卡同构（最高/最低两行，
今天高亮）。

## Concepts

- **窗口尺寸** — 固定 `960x680`（r7；fit 弃用缘由见 pac.at 注记）
- **主题契约** — `dark_mode` / `accent_color`（Plan 458）
- **滚动** — AutoUI `scroll` → Vue `ScrollArea`；scroll 直包 `row`，
  勿外套 `container`（max-w-7xl 注入会致视口塌陷）
- **Hero** — view 内按 condition 的 if/else class（勿绑到 `style:`）
- **城市** — 10 城 pill；**toggle hover** 选中态必须自带 `hover:bg-primary/90`

## How to Run

```bash
cd examples/ui/014-weather
auto build
auto run                 # 960x680 窗 + 横屏
auto run --theme light
auto run -r vm
# 冒烟：C:\Python314\python.exe tests/vm_smoke.py
```
