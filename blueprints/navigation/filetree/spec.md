+++
kind = "navigation"
name = "filetree"
palette = []
extension_points = ["nodes", "default_expanded", "selection", "toggle"]
variants = ["default"]

[dataSource]
nodes = "[]Node"

+++

# Intent

文件系统树展示——TreeView 语义的 fs 预设（Ant Design Tree directory 形态）：
fs 形态数据（id=路径 node schema）自动映射目录/文件图标（folder/folder-open
按展开态；file 按扩展名），内部持有展开/选中态，零配置开箱即用。

> **palette = [] 注记（PLAN-643 merge 对账修正）**：原声明 `["icon", "text"]`
> 中 `icon` 虽为 schema `builtin_widget`（VM 原生渲染），但不在 vue 轨
> `WidgetRegistry` 注册表内，违反本契约"palette 必须在 AURA registry"规则
> （`palette_drift` 实红，被 070 实勘的 CARGO_MANIFEST_DIR 扫描根限制掩盖）。
> 组合形态 reference 恢复（PLAN-645）后包面仍由 TreeIcon（包内支撑件，
> registry 不扫描）组合，palette 维持置空；`icon` 词汇面缺口（schema
> builtin_widget ∉ vue registry）挂 KNOWN-DEBT 由 vocabulary-face 归属
> 计划统一裁定。

# What this blueprint absorbs

- 行派生（flatten_tree 纯函数：guides 缩进/chevron/图标映射/选中态 class）
- 展开/选中受控状态机（toggle_id）
- 有界图标调色板渲染（TreeIcon：目录族 + home/wrench/layout-grid/terminal/
  settings/eye/book-open/table/zap；调色板外名字渲染空）

# Assembly guidance

- 消费方传 `nodes`（node schema 全键必填：id/label/children/kind/icon/
  is_leaf/badge——VM Obj 字面量形状锁定，缺键访问=硬错）
- `default_expanded` 可选（缺省全收起）
- 组合形态导入：`use bps.navigation.filetree.reference.default: FileTree`
- **支撑件直用**（内联树行/自定义组合的消费方，如 VM 轨受 MCP 快照约束的
  场景）：`use bps.navigation.filetree.tree_util: flatten_tree, toggle_id` +
  `use bps.navigation.filetree.tree_icon: TreeIcon`

# References

- `default` — FileTree 组合形态（自持展开/选中态）。跨文件依赖经包内支撑件
  bare 导入（`use tree_util:` / `use tree_icon:`，父目录解析）；vue 轨 bps
  扫描发射器按 plan522 式 helper 转译把被引 fn 闭包内联进 SFC（PLAN-645
  恢复 DEBTS 070 第二行销号；回归样本 `examples/capability-tests/047-bp-compose`）。

# 包内支撑件（格式扩展，PLAN-070）

包根的 `tree_util.at` / `tree_icon.at` 是 reference 之外的**包内支撑件**：
registry 不扫描、不占 variant 位；供消费方按符号跨包直用。此扩展服务于
单文件单 widget 纪律（tree_icon.at 头注：auto run 增量路径多-widget 预存 bug）
——组件族无法合并进单一 reference 文件时的标准形态。

# VM 轨消费注意

vm 组件子树对 MCP 快照不可见（041 README「vm 组件边界」②）且组件行 press
静默崩溃（P614-C1/P618-D4 在册）——VM 轨 smoke 需要驱动树行的消费方
（jade-garden desktop 先例）应在根视图内联树行、只直用支撑件
（flatten_tree/TreeIcon），组合形态留给 vue 轨/无此约束的消费方。
