# Gotchas — navigation/filetree

## 1. VM 轨不要把树行藏进组件子树（jade desktop 067 实证）

**wrong**: VM 应用把 FileTree 组合形态挂进组件，靠 MCP press 组件行驱动 smoke。

**why**: vm 组件子树对 MCP 快照不可见（041 README「vm 组件边界」②）；组件行
press 在 VM 轨静默崩溃（P614-C1、P618-D4 家族在册）。

**right**: VM 轨消费方在根视图内联树行（`for r in flatten_tree(...)` 派生循
环），直用包内支撑件 `tree_util`/`tree_icon`；组合形态 FileTree 留给 vue 轨。

## 2. store 驱动的 vue FileTree 不是本 blueprint 的变体（jade web 谱系）

**wrong**: 把 jade web 的 store 驱动 FileTree（composable use 块 + dyn lucide +
FileTreeNode 递归 + ext prompt 流）当作本 bp 的第二 variant 收编。

**why**: 与本 bp 是**同名不同物**——数据流（store 直绑 vs props 受控）、运行时
惯用（vue 专属 vs 双轨兼容）、功能面（头部 prompt 操作 vs 纯树行）全不同；
硬收编违反契约 Q6 双形态同语义。

**right**: web 谱系保持独立组件身份；若未来要收编，须先做 bp v2 设计
（header slot + dataSource store 注入 + 递归/派生双形态裁定）。

## 3. node schema 缺键 = VM 硬错

**wrong**: 构造 node 字面量省略 badge/icon 等键（JS 习惯省缺省字段）。

**why**: VM Obj 字面量为形状锁定类型实例，缺键字段访问=硬错（P614 实证）。

**right**: 全键书写：`{ id: str, label: str, children: List, kind: str, icon: str, is_leaf: bool, badge: str }`；icon 留空串让 directory 模式自动映射。

## 4. 调色板外图标名渲染为空（非报错）

**wrong**: `TreeIcon (name: "rocket")` 期待兜底图标。

**why**: TreeIcon 是**有界**调色板（分支闭集），plan 418 前的 vue 轨对动态名
只发射占位；契约即"调色板外渲染空"。

**right**: 扩图标 = 改包内 tree_icon.at 增分支（走变体提升评审），消费方不
绕过调色板自造 icon 分支。

## 5. for-in 遍历函数参数列表零次迭代

**wrong**: 在支撑件函数里用 `for x in list` 遍历**参数**列表。

**why**: VM 轨 P614 实证：for-in 对参数列表迭代零次（对状态路径正常）。

**right**: 一律 while + 索引（本包 tree_util.at 全体纪律）。
