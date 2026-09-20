+++
kind = "layout"
name = "sandwich"
palette = ["text", "separator", "icon"]
extension_points = ["toolbar", "sidebar", "content", "statusbar"]
variants = ["default", "full"]

+++

# Intent

桌面应用**上中下三层壳**骨架：顶 toolbar 定高行 / 中弹性主区（default 变体
含 sidebar 左栏）/ 底 statusbar 定高行。普通 app 填四个结构性出口即得双臂
语义正确的满窗壳——不再手写 `h-screen/h-8 shrink-0/flex-1
items-stretch/overflow-*` 组合。

物化两笔引擎教训为可复用资产（PLAN-665 G2）：655 StretchLine 定高行语义
（041 回归 df90a448b 修复）与 663 视口契约（全屏外壳惯例 h-screen，
docs/specs/widgets/viewport-boundary.md）。渲染器高度语义再变动时，本 bp
的 bp-gate 单元先报警。

判定依据（契约 Q5 准入五条）：#1 业界共识锚——VS Code / Eclipse
workbench、PatternFly Page（header+main+footer）；#2/#4 骨架 bp 例外
（PLAN-075 和解注记：布局骨架+slot 为契约本体）；#3 palette 最小集
`["text","separator","icon"]` ⊆ WidgetRegistry；#5 本 spec 六问可答。
官方集缺位实证：有 `layout/status-bar`（底栏骨架）与
`navigation/sidebar-shell`（web 页框架），无桌面应用竖向壳。

# What this blueprint absorbs (per-app variation)

toolbar/sidebar/statusbar 的实际内容（四口全是结构性出口）；sidebar 宽度
（w-56 钉骨架内，见 gotchas#4）；content 内部分栏（消费方自行
`row flex-1 items-stretch`）；menubar/actions 契约（应用侧既有 `actions{}`
机制，不进骨架）。

# Assembly guidance

- 消费方经 `use bps.layout.sandwich.reference.default: SandwichShell` 导入
  （L1 通道），调用位填 slot：`slot(name: "toolbar") {...}` /
  `slot(name: "sidebar") {...}` / `slot(name: "statusbar") {...}`，裸子节点
  落默认出口（content）。
- 无侧栏形态用 `full` 变体（`...reference.full: SandwichShellFull`）。
- 骨架根容器 `h-full w-full` 填消费方容器（041 donor 同款）——满窗由
  消费方根决定（vue 脚手架 `html,body,#app{height:100%}` 链天然支持；
  全屏 demo 形态 h-screen 见 viewport-boundary 契约，嵌套固定高单元用
  h-full 同语义）。
- statusbar 建议直接嵌 `layout/status-bar`（bp 组合优于重复）；content 内
  分栏用 `row flex-1 items-stretch`（4xx 分栏样板）。
- VM 轨消费断言：本仓 snapshot v2(rendered) 已可见子件子树（PLAN-665
  T-00 实证），slot 填充文本可直接 needle；root 投影字段仍可叠加。

# References

- `default` — 含 sidebar 出口（w-56 左栏）的三层壳（reference/default.at，
  widget `SandwichShell`）。
- `full` — 无侧栏变体：主区仅 content 弹性列（reference/full.at，widget
  `SandwichShellFull`——SidebarShellCompact 先例：双变体同页消费时 VM
  registry 按名注册不互踩）。

# Gotchas

See gotchas.md。
