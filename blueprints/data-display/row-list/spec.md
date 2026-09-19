+++
kind = "data-display"
name = "row-list"
palette = ["text", "badge", "separator"]
extension_points = ["rows", "row", "empty"]
variants = ["default"]

+++

# Intent

数据行列表**骨架**：列表容器 + 行布局（label 主文 + badge 右挂）+ 行间
分隔线 + **行 slot**。收敛的是"数据拉取/过滤 → 行列表渲染"的公共形状；
行内容与数据编排归消费方。

判定依据（design 30 §6 通道①）：jade 面板行族 4 件（backlinks /
outgoing_links / outline / unlinked_references，PLAN-074 下沉后均为
"watch 拉取 → 行列表渲染"形状）+ 076 检索/导航族 3 件同形（设计形状登记）
——7 使用位，远超 ≥2 门槛。

# What this blueprint absorbs (per-app variation)

空态文案（`empty_text`）/ 行密度与 token（消费方配方层）/ 行内富内容
（高亮/图标/操作钮——fork 行 slot，见 gotchas#2）/ 数据编排（watch 拉取、
过滤、排序——消费方 watch 块，074-sink-mode 步骤 2 同款）。

# Assembly guidance

- 消费方传 `rows: []Row`（row schema 全键书写：`{ label: str, badge: str }`，
  缺键访问 = VM 硬错——filetree gotcha#3 同型纪律）；`badge` 空串即不渲染
  右挂徽标。
- 行 slot 的行派生 fn 归消费方自备（.at 无跨文件导入——gotchas#2）。
- VM 轨消费走 root 投影断言 / 消费方内联行（gotchas#3）。

# References

- `default` — 三行样本 + 分隔线 + 空态分支的最小骨架（reference/default.at）。

# Gotchas

See gotchas.md。
