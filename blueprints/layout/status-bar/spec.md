+++
kind = "layout"
name = "status-bar"
palette = ["text", "separator"]
extension_points = ["left", "center", "right"]
variants = ["default"]

+++

# Intent

底部状态栏**骨架**：高度（h-8）/顶部分隔线（border-t）/三区水平布局
（左-中-右）+ **内容 slot**。骨架收敛的是布局与节奏；各区内容是消费方
的 slot 变体，不进骨架。

判定依据（design 30 §6 通道③"同名不同物"）：jade 三版 status_bar 互为
同名不同物——web 版 ext-composable 耦合（store 直绑+光标位）、desktop 版
值 props（041 形态适配）、041-auto-edit 静态适配——数据流/运行时惯用/
功能面互异，不硬抽全量 bp，抽**骨架 + 内容 slot**。

# What this blueprint absorbs (per-app variation)

三区内容（光标位/保存态/统计/通知）/ 字号与 token 密度（jade 11px+zinc
面属消费方配方层）/ 图标位（未来有需要再议变体提升）。

# Assembly guidance

- 消费方传 `left` / `center` / `right` 文本（值 props 形态，desktop 版
  同款），或 fork 骨架在 `// EDIT:` 区填自有组件（web 富内容形态）。
- 内容 slot 的行派生逻辑归消费方自备（.at 无跨文件导入——见 gotchas#2）。
- VM 轨消费走 root 投影断言（子件子树对 MCP 快照不可见——gotchas#3）。

# References

- `default` — 三区文本 + 分隔线的最小骨架（reference/default.at）。

# Gotchas

See gotchas.md。
