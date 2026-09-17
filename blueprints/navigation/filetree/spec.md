+++
kind = "navigation"
name = "filetree"
palette = ["icon", "text"]
extension_points = ["nodes", "default_expanded", "selection", "toggle"]
variants = []

[dataSource]
nodes = "[]Node"

+++

# Intent

文件系统树展示——TreeView 语义的 fs 预设（Ant Design Tree directory 形态）：
fs 形态数据（id=路径 node schema）自动映射目录/文件图标（folder/folder-open
按展开态；file 按扩展名），内部持有展开/选中态，零配置开箱即用。

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

（暂无——组合形态 FileTree 暂缓，见下）

## 组合形态暂缓（PLAN-070 T-03 实证）

FileTree 组合 widget（自持展开/选中态，源码见 git af8c72a84 的
reference/default.at）在 vue 轨 bps 扫描下不可构建：扫描发射器不转译
`.at` 跨文件 fn 导入（`use bps...tree_util:`/bare 形态均实测
`Cannot find name 'flatten_tree'/'toggle_id'`，046-bp-import 构建断裂
复现），且单文件内联会撞单文件单 widget 纪律/制造 fn 双份漂移面。
待发射器补 plan522 式 fn 转译后回归（auto-lang DEBTS 070 第二行）。
本包当前交付 = 支撑件（tree_util/tree_icon）+ 契约 + gotchas——活消费面
（jade desktop/041）全部经支撑件符号导入，不受影响。

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
