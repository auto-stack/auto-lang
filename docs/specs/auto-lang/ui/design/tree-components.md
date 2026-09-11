# tree 组件族契约 v1（TreeView / FileTree，裸名）

> 来源：plan-614（2026-09-11 执行）；姊妹篇
> [chart-components.md](chart-components.md)（plan-437/484/498/499）、
> [diagram-components.md](diagram-components.md)（plan-502）。
> 载体：`widgets-gallery/src/front/components/{tree_util,treeview,filetree,tree_icon}.at`
> + 页面 `pages/{treeview,filetree}.at`——画廊现址 `auto-os/widgets-gallery/`
> （PLAN-590 后；框架测试经 `resolve_os_top_dir` 解析序定位）。
> 形态：official 包纯 Auto 组件（Plan 484 裁定），双端同源零引擎渲染器改动。

## 范围

v1 = **TreeView**（通用受控树）+ **FileTree**（文件系统树自包含预设）+
TreeIcon（有界图标调色板）。复选框多选/懒加载/DnD/虚拟滚动/真实文件系统
读取（VM `auto.fs.walk` 接入）归 Phase 2，未落地不入本契约。

## 数据轨 schema（node record）

```
node = { id: str, label: str, children: List, kind: str, icon: str,
         is_leaf: bool, badge: str }
```

- **字段必填全键书写**——VM Obj 字面量为形状锁定类型实例，缺键字段访问
  = 硬错（`Field 'is_leaf' not found on type instance StateObjectLit`，
  P614 实证）。可选语义字段（kind/icon/badge）以 `""`/`false` 显式占位。
- `children` 空 = 叶；`is_leaf: true` 可强制叶（空目录场景）。
- `kind`: `"dir"` / `"file"` / `""`，仅 `directory: true` 模式消费。
- `icon` 显式指定（lucide 名，限 TreeIcon 调色板）优先于 directory 自动映射。
- FileTree 约定 `id` = 路径（`"src/components"`，fs walk 语义）。

## row record（flatten_tree 输出，组件私有）

```
row = { id, label, depth, has_kids, open, leaf, icon, badge, pad, chev, state }
```

缩进/图标/chevron 方向/选中态 class 全部派生进 row，组件 view 零逻辑。

## 组件契约

### TreeView（受控）

```
TreeView (nodes: List, expanded: List = [], selected: str = "",
          on_toggle: msg, on_select: msg, directory: bool = false)
msg { Toggle(str), Select(str) }   // 纯 emit 子件,零 model
```

- **受控**：`expanded`/`selected` 归页面；`on_toggle(id)` / `on_select(id)`
  回调经 PLAN-037 T3 通道，payload = 首实参（VM 声明式 C2①：HandlerNotFound
  臂父路由照派载荷=首实参；vue：msg 变体即 emit 名）。
- **交互语义（Ant Design Tree 对齐）**：chevron 点击 → toggle；行（标签区）
  点击 → select（单选，`""` = 无选中）；行点击不折叠目录。
- **rows 全 computed 派生**（`flatten_tree(.nodes, .expanded, .directory,
  .selected)`），零 model 字段。同页多实例在 vue 轨天然隔离；VM 轨见
  「P320 单实例约束」。

### FileTree（自包含预设）

```
FileTree (nodes: List, default_expanded: List = [])
msg { Init, Toggle(str), Select(str) }
model { ftExpanded, ftSelected }   // 内部持有展开/选中态
```

- `default_expanded` 经 Init 播种（PLAN-536 T3 挂载语义，仅一次）。
- `directory: true` 语义内置（folder/folder-open 按展开态、ext→file 图标）。
- v1 **自渲染行**（与 TreeView 同款 row 结构）而非组合 TreeView——P320
  单实例约束下 TreeView 须全画廊唯一，FileTree 独立实现（行视图小面积
  重复为接受代价）。

### TreeIcon（有界调色板）

vue 轨 `icon` 元素只发射**字面量名**（动态名落 Circle 占位，P601-T11 同族
缺口），故按 `name` 分支发射真 lucide 组件。调色板 = folder/folder-open/
file/file-text/image + home/wrench/layout-grid/terminal/settings/eye/
book-open；调色板外名字双端渲染为空。chevron 在组件内直接双字面量分支。

## 渲染：flatten-to-rows（拍平策略）

`flatten_tree(nodes, expanded, directory, selected)` 显式栈迭代 DFS（无递归
——chart/diagram 族同款纪律），跳过未展开子树，产可见行序列。组件
`computed` 消费（Plan 522 use-fn 双端路径：vue 转译进 SFC / VM
import_aliases）。缩进 = `pl-4` 阶梯字面量数组（depth>8 钳制 pl-32）；
字面量随 use-fn 转译进 SFC 源 → Tailwind JIT 可扫描，VM class.rs 任意数值。

行命中区：**chevron 命中区与标签命中区为兄弟节点**（嵌套可点击在 vue 事件
冒泡下双触发）；动态 class 串必须走 `class:` prop（`style:` 的 f-string 被
发到 `:style`，Tailwind 类失效）。

## VM 轨实证约束（P614，组件侧纪律）

1. **for-in 对函数参数列表迭代零次**（状态路径正常）——纯函数模块的列表
   遍历一律 `while + 索引`（索引/len/字面量比较实证正常）。见 tree_util.at
   头注。引擎修复前，语料新增纯 fn 遵循同款纪律。
2. **Obj 字面量形状锁定**：缺键字段访问硬错（见 schema 节）。
3. **同名子组件全画廊唯一实例**（P320 单态，全页常驻挂载放大跨页播种
   共享；flow-diagram 页双卡限制同族）。TreeView/FileTree 页面均单实例。
4. **view 端 f-string 内方法调用不解析**（`.len()` 落空；纯字段插值正常）
   ——派生量在 handler 内预计算入 model（如 tvCount）。
5. **computed 调 use-fn 的模块名解析**：子目录模块须点分路径
   （`use components.tree_util: ...`，`pages/` 下解析序 = 本目录 → 父目录）。

## 已知缺陷

- **P614-C1（阻塞 VM 端行点击验证，待引擎立项）**：MCP `autoui_action
  press` 命中子组件行（onclick 带循环变量 payload）时 VM 进程静默死亡
  （无 panic 日志，疑 debug-id 跨帧解引用/计算派生载荷生命周期；vue 轨
  同交互正常，真实鼠标点击影响面未测）。复现：`auto run -r vm` 打开
  /treeview 页 → snapshot → press 任意树行 vnode id → 连接重置。按钮级
  交互（Expand all/Collapse all）与视觉呈现不受影响。

## 双端验证基线（plan-614 落地时）

- vue：chevron 往返、行选中回显、Expand all（11 dirs，含 level-9/bottom.txt
  的钳制缩进）、FileTree 默认展开/badge——Playwright 实证。
- VM：树行渲染、深链钳制、folder/file 图标、badge 右对齐、Expand all
  （11 dirs echo）/Collapse all（0 dirs）——MCP snapshot + 截图实证；
  行点击见 P614-C1。
