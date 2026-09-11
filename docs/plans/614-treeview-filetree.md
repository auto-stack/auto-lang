---
plan_id: PLAN-614
status: executing              # execution_done →(review R1 blocked)→ executing;P614-C1 决策后恢复
feature_name: TreeView + FileTree 组件(official 包数据轨)与 WidgetGallery 落地
author: [zcode]
created_at: 2026-09-11
updated_at: 2026-09-11
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/tree-components.md: tree 组件族契约 v1——TreeView(通用受控树:数据轨 nodes schema/expanded/selected/on_toggle/on_select 受控契约、flatten-to-rows 渲染、pl-N 阶梯缩进)+FileTree(文件系统树预设:fs 形态 schema、kind/ext 图标映射、自包含展开态)双节"
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉一致(Vue/VM 双端 parity 锁定)——tree 组件族双端同源:同一 .at 组件源经 use-fn 转译(vue)/import_aliases(VM)双轨渲染,受控契约规避 VM P320 单态串扰"

affects: [docs/specs/auto-lang/ui, auto-os/widgets-gallery]
current_step: 7
total_steps: 9
---

# [PLAN-614] TreeView + FileTree 组件(official 包数据轨)与 WidgetGallery 落地

## 变更摘要

为 AutoUI 新增两个树组件并以 chart/diagram 家族同构的**official 包纯 Auto 组件**形态交付:

- **TreeView**——任意形状树状结构的通用受控组件(参考 Ant Design `Tree` / MUI
  `RichTreeView` 的数据轨 + 受控契约):`nodes` 嵌套 record 列表 + `expanded`/`selected`
  受控状态 + `on_toggle`/`on_select` 回调契约(PLAN-037 T3 通道)。
- **FileTree**——文件系统树展示,TreeView 的自包含预设(参考 Ant Design `Tree` 的
  `directory` prop):fs 形态数据(`{name, kind, children}`)自动映射目录/文件图标
  (folder/folder-open/file-text/image/file),内部持有展开态,开箱即用。

渲染采用 **flatten-to-rows** 策略:纯 Auto 工具函数(`tree_util.at`)把嵌套树按展开
态拍平为行记录列表,组件 `computed` 调用之(Plan 522 双端转译路径:vue 侧转译进 SFC /
VM 侧 import_aliases),view 用 `for` 循环发射行——**零递归 widget、零引擎改动、
零动态 Tailwind class**(缩进用 pl-0..pl-32 固定阶梯字面量,depth>8 钳制)。

落地载体 = widgets-gallery(PLAN-590 后现址 `auto-os/widgets-gallery/`):新增
`components/tree_util.at`、`components/treeview.at`、`components/filetree.at` +
`pages/treeview.at`、`pages/filetree.at` + app.at 路由/侧边栏 + index.at 组件卡与计数
(61→63)。auto-lang 侧沉淀 `docs/specs/auto-lang/ui/design/tree-components.md` 契约。

## 目标

1. **G1 通用树**:TreeView 能展示任意形状(任意深度/任意分叉/混合叶与枝)的树,
   展开收起、单选高亮、可选节点图标,双端(vue 浏览器 / vm iced 窗口)行为一致。
2. **G2 文件树**:FileTree 以文件系统语义(目录/文件图标、目录展开态 folder-open)
   展示树状数据,零状态配置开箱即用。
3. **G3 画廊落地**:WidgetGallery 新增 `/treeview` 与 `/filetree` 两页,导航可
   达、首页组件卡可索引、计数徽章更新,双端截图对拍通过。
4. **G4 契约沉淀**:tree 组件族契约 v1 入 `docs/specs/auto-lang/ui/design/`
   (diagram-components.md 同款体例),供后续 tree 族扩展(检查框多选/懒加载等)锚定。

### 非目标(v1 明确不做)

- 虚拟滚动/大节点量优化(>1k 节点);拖拽重排(DnD);复选框多选(checkedKeys 面);
  懒加载异步子节点;节点右键菜单;真实文件系统读取(vue 端无 fs,双端一致性优先;
  VM 端 `auto.fs.walk` 接入归 Phase 2);a2r/ark/jet 后端发射;多选 selection。
- FileTree 受控透传(on_toggle 直通到页)——v1 自包含;若引擎链式路由验证顺畅可
  平滑升级,契约文档预留。

## 架构方案

### 载体决策:official 包纯 Auto 组件(不动引擎)

Plan 484 裁定"裸名由 official 包 Auto 组件承接"后,chart 族(437/484/498/499)与
diagram 族(502)均以 `widgets-gallery/src/front/components/*.at` 纯 Auto 组件交付,
双端同源零 Rust 改动。tree 族同构跟进——**本计划零 auto-lang 渲染器代码改动**,
auto-lang 侧只动规范文档与测试基线(golden 重基线)。

### 渲染策略:flatten-to-rows + computed(无递归)

任意深度树的标准高效做法(VS Code / GTK TreeView 同思路):把"可见行序列"拍平。

```
tree_util.flatten_tree(nodes, expanded)
  → [ {id, label, depth, has_kids, open, icon, badge, pad, ...}, ... ]
```

- 拍平在**纯 Auto 函数**(显式栈迭代 DFS)中完成,组件 `computed { rows => flatten_tree(.nodes, .expanded) }`
  消费——Plan 522 已实证"computed 调用 use 导入模块 fn 双端同源"(calendar 016
  build_month_grid 先例;chart_geom.at 的 `use chart_geom: dc, ds` 实证包内 util
  模块可被组件文件 use 导入)。
- 递归 widget 自引用(TreeNode 渲染 TreeNode)无先例且引擎未证,规避。
- 响应性:vue 侧 computed 依赖 props(reactive)自动重算;VM 侧每次 view 重建时
  重解 computed——受控状态变化 → 页面 dirty → 重建 → rows 刷新,无 Init 陈旧性
  问题(flow_diagram 的 Init 几何是 props 静态所以可 Init 一次;tree 的 rows 依赖
  受控态,必须 computed 形态)。

### 受控契约(规避 VM P320 单态串扰的关键)

VM 单态架构(P320)下同名子组件字段经根状态跨组件共享(chart 族以字段更名解耦,
flow-diagram 页注记双卡共享限制)。本设计从根上免疫:

- **TreeView 零 model 字段**——全部输入经 props(nodes/expanded/selected),rows 是
  computed 派生。同页多实例无串扰(vue/VM 行为一致)。
- 状态归**页面**(TreeViewPage/FileTreePage,页面 widget 名全仓唯一)。
- **FileTree** 自包含展开态(model 字段 `ftExpanded`/`ftSelected`,名字族专属),
  组件内绑定自身 msg 变体给 TreeView 的 on_*(单级 callback 路由,SettingsPanel
  `on_close: .Close` 同款机制一层),不依赖链式透传。

### 缩进与图标(双端可渲染性约束下的选型)

- **缩进**:行 record 携带 `pad` 字符串,取固定阶梯 `["", "pl-4", "pl-8", ..., "pl-32"]`
  按 depth 索引(depth>8 钳制 pl-32)。字面量随 tree_util.at 转译进 vue SFC 源 →
  Tailwind JIT 可扫描;VM 侧 class.rs `pl-{N}` 任意 N 已支持(class.rs:799,
  parse_size_value 通用数值臂)。**不用** `pl-[${d*16}px]` 动态拼串(vue JIT 扫不到,
  flow_diagram 的 mouse-area 动态 style 是 VM 侧定位层先例,树行缩进不复制该路径)。
- **图标**:`icon (name: ...)` 原语。VM 侧 lucide_svg 表已覆盖 `chevron-right`/
  `chevron-down`(renderer.rs:5448/5457)/`folder`/`folder-open`/`file`/`file-text`/
  `image`;vue 侧 lucide-vue-next 全量。展开指示 = chevron-down,收起 = chevron-right,
  叶节点 = 等宽占位 div(与 chevron 列对齐)。

### 交互语义(Ant Design Tree 对齐)

- **chevron 点击** → toggle(仅目录/有子节点者);**行点击** → select(单选,唯一
  selected id,"" = 无选中)。行点击不折叠目录(与 VS Code 不同,与 Ant 一致,可访问
  性更稳)。
- 回调经 PLAN-037 T3 callback-contract 通道:子组件声明 `msg { Toggle(str), Select(str) }`
  + `on_toggle: msg` / `on_select: msg` prop;VM 侧 dispatch_parent_route 单参对齐
  (PLAN-576 G4"1 形参塞载荷");vue 侧 defineEmits + `@toggle`/`@select`。
- payload 通道双端往返是本计划唯一未证链路(T-02 first-light 优先验证);若 VM 侧
  缺口,回退预案:TreeView 增加零参模式 demo(页面态 + 行内 onclick 绑定页面 msg
  构造 `onclick: .TvToggle(r.id)` 的页内渲染臂)——回退不改公共契约面。

## 需求分析与背景调查

### 用户授权范围

- 诉求(2026-09-11 对话):AutoUI 需要两个新组件——TreeView(任意形状树状结构)
  与 FileTree(文件系统树,TreeView 特例);参考其他 UI 框架的类似组件;设计
  AutoUI 自己的这俩组件;最终展示在 WidgetGallery 中。
- 仓库/动作范围:auto-os(widgets-gallery 源码 + auto-os 计划文档仓规约)+
  auto-lang(规范文档、golden 基线、计划文件)。无预算/时长约束。

### 管线调查结论(代码证据)

| 关注点 | 结论 | 证据 |
|---|---|---|
| 画廊现址 | PLAN-590 后物理迁 `auto-os/widgets-gallery/`,框架测试经 `resolve_os_top_dir` 解析序定位(env AUTO_OS_ROOT → 兄弟 → 主检出) | `crates/auto-lang/src/tests/gallery_pages_compile_tests.rs:22` |
| official 包机制 | `use { package: official from "./components" }`,包内组件互相引用(datepicker→Popover/Calendar)、包内 util 模块可被 use 导入 | `components/package.at`、`donut_chart.at:24`(use chart_geom) |
| computed 调模块 fn 双端 | Plan 522 实证:vue SFC 按需转译 / VM import_aliases 同名解析 | `examples/ui/016-calendar/src/front/calendar_util.at` 头注 |
| 受控回调契约 | `on_xxx: msg` prop + 匹配 Pascal msg 变体 → emit 声明;VM parent-route 单参对齐 | `ui_gen/vue.rs:2338`(PLAN-037 T3)、`ui/dynamic.rs:1197`(576 G4) |
| VM 单态共享 | 同名子组件字段经根状态共享;chart 族字段更名解耦 | line_chart.at 头注(P320)、`ui/dynamic.rs:703` |
| Icon 双端 | VM lucide_svg 表覆盖 chevron-right/down、folder/folder-open/file/file-text/image;vue 走 lucide-vue-next | `ui/iced/renderer.rs:5448` 等 |
| pl-N 缩进 | VM class.rs `pl-` 前缀任意数值;vue JIT 需字面量(阶梯方案满足) | `ui/style/class.rs:799`、`parse_size_value` |
| 页面布线 | 路由(app.at routes)+ 侧边栏(Display 组)+ 首页 component-card + 计数徽章("v1.0 — 61 Widgets") | `app.at`、`pages/index.at:18` |
| 测试闸门 | `gallery_pages_compile_tests.rs`(lib 级,入 cargo t,自动收页)/`gallery_golden.rs`(tests/ 集成,手动重基线) | 两文件头注 |

### 其他框架参考(设计输入)

| 框架 | 组件 | 借鉴点 |
|---|---|---|
| Ant Design(React) | `<Tree treeData>` + `directory` | 数据轨 record 契约(key/title/children/icon/isLeaf);controlled expandedKeys/selectedKeys + onExpand/onSelect;FileTree=directory 预设思路 |
| MUI x Tree View | `<RichTreeView items>` | `{id, label, children}` id 寻址;controlled expandedItems/selectedItems 单选默认 |
| Element Plus | `<el-tree data>` | 文件树图标开箱(show-icon);default-expanded-keys 初始展开面 |
| VS Code / GTK | TreeView | flatten-visible-rows 渲染模型(本设计拍平策略的依据) |
| shadcn-vue | (无官方树) | 社区方案均为 radix 嵌套组装——证明 shadcn 生态缺数据轨树,AutoUI 以数据轨补位;视觉对齐 gallery 现有 shadcn 风格 |

## 详细设计

### 数据契约

**TreeView node record**(数据轨输入,Value::Obj 形态——VM 字段解析器仅解 Obj,
calendar_util 注记约束):

```
node = { id: str, label: str = id, children: List = [], icon: str = "",
         is_leaf: bool = false, badge: str = "", kind: str = "" }
```

- `children` 空 = 叶;`is_leaf: true` 可显式强制叶(空目录场景)。
- `kind` 仅 directory 模式消费:`"dir"`(或 children 非空)→ 目录图标族。
- `icon` 显式指定优先于 directory 自动映射。

**row record**(flatten_tree 输出,组件内私有):

```
row = { id: str, label: str, depth: int, has_kids: bool, open: bool,
        icon: str, badge: str, pad: str }
```

**FileTree node record**(fs 形态输入):

```
fsnode = { name: str, kind: str = "file", children: List = [] }
```

### 组件签名

```auto
// components/treeview.at
widget TreeView (nodes: List, expanded: List = [], selected: str = "",
                 on_toggle: msg, on_select: msg, directory: bool = false) {
    msg { Toggle(str), Select(str) }
    // 零 model;rows/每行图标解析全 computed
    computed {
        rows => flatten_tree(.nodes, .expanded, .directory)
    }
    view {
        col (style: "w-full text-sm") {
            for r in .rows {
                row (style: 行样式, onclick: .Select(r.id)) {
                    text (style: r.pad) {}            // 占位收 padding 阶梯
                    [chevron 图标按钮 onclick: .Toggle(r.id) | 叶占位 div]
                    [icon (name: r.icon) 若非空]
                    text r.label
                    [badge text 若非空]
                }
            }
        }
    }
}

// components/filetree.at
widget FileTree (nodes: List, default_expanded: List = []) {
    msg { Toggle(str), Select(str), Init }
    model { ftExpanded List = [], ftSelected str = "" }
    on { .Init -> { .ftExpanded = .default_expanded }
         .Toggle(id) -> { .ftExpanded = toggle_id(.ftExpanded, id) }
         .Select(id) -> { .ftSelected = id } }
    computed { ftNodes => map_fs_nodes(.nodes, "") }
    view { TreeView (nodes: .ftNodes, expanded: .ftExpanded, selected: .ftSelected,
                     on_toggle: .Toggle, on_select: .Select, directory: true,
                     style: "w-full") {} }
}
```

(行样式/选中态/hover 态为字面量 class,对齐 gallery 现有 shadcn 视觉语言
`rounded-md bg-accent` 选中、`hover:bg-accent/50` 悬停;具体以实现时与现页
视觉对拍微调。)

### tree_util.at 函数清单(纯 Auto,pub fn)

| fn | 签名 | 职责 |
|---|---|---|
| `flatten_tree` | `(nodes: List, expanded: List, directory: bool) -> List` | 显式栈迭代 DFS;跳过未展开子树;产 row 记录(depth/pad 阶梯 min(depth,8);directory 模式解析 icon:dir→folder/folder-open(open 态),ext→file-text/.md|.txt|.log|.json 族,image→.png|.jpg|.gif|.webp|.svg,其余→file) |
| `toggle_id` | `(list: List, id: str) -> List` | 返回切换 id 在列成员资格的新 List(页/组件 handler 复用) |
| `map_fs_nodes` | `(nodes: List, prefix: str) -> List` | fs 形态→TreeView node(id=路径 join,label=name,kind 透传,children 递归——迭代实现) |
| `collect_ids` | `(nodes: List, dirs_only: bool) -> List` | 全部(或仅目录)id 收集——Expand All/Collapse All demo 用 |

### 页面设计(pages/)

- **treeview.at(TreeViewPage)**:h1+描述+Installation codeblock+preview-card×4
  (Basic 纯 chevron / With Icons 节点 icon 字段 / Selection 选中态回显
  (`text f"Selected: ${.tvSelC}"`)/ Expand-Collapse All 按钮 + collect_ids);
  Properties 表(nodes/expanded/selected/on_toggle/on_select/directory 六行)。
  页面态:`tvNodes`(静态嵌套数据,含 >8 深度一支验证 pad 钳制)、`tvExpanded`、
  `tvSelected`、`tvSelC`(Selection 卡独立选中);handler `TvToggle(str)`/`TvSelect(str)`
  /`TvSelC(str)`/`TvExpandAll`/`TvCollapseAll`(全 one-liner,toggle_id/collect_ids)。
- **filetree.at(FileTreePage)**:h1+描述+Installation+preview-card×2
  (Basic——静态示例 fs 树(以画廊自身 src/front 结构为素材,default_expanded
  = ["src","front"])/ Internal Selection 说明卡);Properties 表(nodes/
  default_expanded + fsnode schema 注记)。

### gallery 布线(auto-os/widgets-gallery/src/front)

1. `app.at`:routes 增 `"/treeview" -> use treeview`、`"/filetree" -> use filetree`;
   Display 组 sidebar_menu 增两项目(图标 `list-todo` / `folder`,VM lucide 表已覆盖)。
2. `pages/index.at`:Display 类目 grid 增两张 component-card
   (`to: "/treeview", name: "TreeView", desc: "Hierarchical tree data"` /
   `to: "/filetree", name: "FileTree", desc: "File system tree"`);徽章
   `text "v1.0 — 61 Widgets"` → `63 Widgets`。
3. 双页均 `use { package: official from "../components" }`(flow-diagram 页同款)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-lang/ui/design/tree-components.md` | 无 tree 族契约 → tree-components.md v1(TreeView 数据轨/受控契约/flatten-to-rows/pad 阶梯/directory 模式 + FileTree fs 映射/自包含态 + 非目标面) | chart/diagram 家族契约体例延续;后续 tree 族扩展锚点 | AC-01/02/03/04 |
| SD-02 | modify | `docs/specs/auto-lang/ui/overview.md` | 无 tree 段 → 组件族清单增 tree 段(双端同源受控树,契约链接 SD-01) | overview 是族级索引 | AC-07 |
| SD-03 | modify | `docs/specs/auto-lang/ui/plans.md` + `docs/specs/INDEX.md` | 无 614 行 → 增 PLAN-614 行/INDEX 重生 | 台账纪律 | AC-07 |

(auto-os 侧不维护 specs(规范单源在 auto-lang);widgets-gallery README 若述及
组件计数则同步——实现时核对。)

## 测试设计

| 闸门 | 内容 | 时机 |
|---|---|---|
| `cargo check -p auto-lang` | auto-lang 零代码改动,快检守恒 | T-06 后 |
| `cargo t gallery_pages` | 全页可编译冒烟(自动收新页;在 auto-lang worktree 跑,`AUTO_OS_ROOT=D:/autostack/.wt/lang-614/auto-os` 指向工作副本) | T-03/T-04/T-05 各一次 |
| `GALLERY_GOLDEN_UPDATE=1 cargo test -p auto-lang --test gallery_golden` 后裸跑对比 | 新页/新组件 SFC golden 重基线(diff 人工复核:仅增页/组件,无既有页漂移) | T-07 |
| autoui-verifier 双端 | `auto run`(vue)与 `auto run -r vm`(VM)于 auto-os worktree 画廊:两页截图矩阵(初始/展开后/选中后)+ 点击交互(chevron 往返/行选中/Expand All);`scripts/test_vue_playwright.mjs`/`test_vm_mcp.py` | T-02(first-light)与 T-08(全量) |
| 双端对拍口径 | 同状态截图目检一致(布局/图标/缩进/选中态);非像素对齐(画廊既有口径) | T-08 |

VM 大改触发面:本计划**不改 VM/编译器**(`cargo tv` 不触发);aavm 零触发。

## 验收标准

- **AC-01 契约落地**:components/{tree_util,treeview,filetree}.at 存在且签名/记录
  schema 与 §详细设计一致(目检 + gallery_pages 编译过 = 契约可编译)。
- **AC-02 vue 端交互**:`auto run` 下 /treeview 页:chevron 点击展开/收起往返;
  行点击选中高亮(唯一);缩进随深度递增;图标(显式 icon + directory 自动映射)
  正确;Expand/Collapse All 生效;/filetree 页:目录 folder/folder-open 切换、
  默认展开生效、文件 ext 图标正确(Playwright 截图为证)。
- **AC-03 VM 端一致**:`auto run -r vm` 下同 AC-02 场景全部复现,与 vue 截图同状态
  目检一致(MCP 截图为证)。
- **AC-04 任意形状**:treeview 页演示数据含:深分支(>8 深度)、单子节点链、
  混合叶/枝、空 label 边缘——全部正常渲染(pad 钳制不破版)。
- **AC-05 gallery 布线**:两页经侧边栏 Display 组与首页 component-card 可达;
  徽章计数 63;`cargo t gallery_pages` 全绿。
- **AC-06 golden 基线**:golden 重基线后二次裸跑稳定;diff 仅含新增
  treeview/filetree 页与组件(无既有页漂移)。
- **AC-07 规范沉淀**:SD-01..03 落地(specs.json upsert + INDEX 重生 +
  spec-index.py 校验过)。

## 执行步骤

> 跨仓计划(Plan 529 分组平铺):`.wt/lang-614/{auto-lang, auto-os}` 双 worktree;
> 计划簿记(本文件勾选/frontmatter)在 auto-lang 主检出;代码在 worktree。

- **T-01 建组** [x] [✅ 已完成] master a3d53cbfc(骨架+next-id);worktree 组 `.wt/lang-614/{auto-lang@plan-614-dev, auto-os@plan-614-dev(99b094c)}`+auto-down detached 依赖 worktree(跨仓 path 依赖 `../auto-down` 组内解析所需)。
- **T-02 first-light:tree_util + TreeView 骨架 + 最小页** [x] [✅ 已完成] 四文件+布线一次成形;`cargo t gallery_pages` 过(17s);vue 实机选中回显(`selected: src`)+chevron 往返(level-2 现身)实证——payload 声明式链路 vue 端通。first-light 暴露并修复:①flatten_tree 局部 `rows` 撞消费组件 computed 名被误插 `.value`(改名 outRows);②动态 f-string `style:` 落 `:style` Tailwind 失效(改 `class:` prop);③vue icon 动态名 Circle 占位(TreeIcon 分支化);④`use tree_util` 从 pages/ 解析不到(改点分 `use components.tree_util:`)。
- **T-03 TreeView 完整化** [x] [✅ 已完成] directory 模式图标/ext_icon/collect_ids 入 tree_util;页面收敛为单实例综合卡(P320 多实例实证限制,原四卡方案归档于本文件 git 历史);vue Expand all=11 dirs(含 level-9/bottom.txt 钳制缩进,截图 p614_vue_treeview_expandall.png)。
- **T-04 FileTree 组件与页面** [x] [✅ 已完成] filetree.at 自包含(Init 播种 default_expanded+toggle_id)——实现从组合 TreeView 收敛为自渲染行(P320 TreeView 全画廊单实例约束,组合会在跨页常驻挂载下播种串扰);map_fs_nodes 剪除(fs 形态由页面按 id=路径书写);VM 端截图 p614_vm_filetree_select.png(folder-open/嵌套缩进/badge 全对)。
- **T-05 gallery 布线收口** [x] [✅ 已完成] app.at 路由两行+Display 侧边栏两项目(list-todo/folder 图标)+index.at 两张 component-card+计数 61→63+Display count 8→10;vue 实机侧边栏/首页卡可达。
- **T-06 规范沉淀** [x] [✅ 已完成] tree-components.md v1(契约+VM 轨五条纪律+P614-C1 缺陷节+双端验证基线)+overview.md tree 组件族段+plans.md 614 行+INDEX 重生(26 projects);specs.json upsert 留 merge 技能(活账本在主检出)。
- **T-07 golden 重基线 + 门禁** [x] [✅ 已完成] golden 重基线 88 文件(含 auto-os main 既有漂移——master 基线对本已红,PLAN-008 等画廊演进未重基线;新增 tree 三组件两页);裸跑双绿;收集器补跳过纯 util 模块(无 `widget ` 声明,package.at 同意图内容化);`cargo t --no-fail-fast` 失败集=master 预存红(21 共同,含 lucide manifest/plan055)±跨仓环境敏感 ffi/osconfig 摆动,零 tree/gallery 相关;p614 VM 探针测试绿(while+索引 has_id/flatten/computed 三路)。
- **T-08 双端全量验证** [ ] [⚠ R1 重开——AC-03 行点击场景被 P614-C1 阻塞] vue 全交互 ✓(复审独立复验通过);VM 视觉/按钮级 ✓(复审复验:树行/图标/展开态渲染完整);未决:MCP press 子组件行崩溃(3/3)+OS 级合成点击注入不可达(DPI/坐标限制)→ 真实鼠标行为需人工点击判定。VM 视觉证据:p614_vm_treeview_expandall.png/p614_vm_filetree_select.png/p614_review_vm_before_click.png。
- **T-09 独立复审(/auto-plan:review)**:AC-01..07 逐条对码核验;遗漏/绕开扫描入
  KNOWN-DEBT-AND-RISKS.md;健康检查(无警告/无 debug 残留)。

## 复审记录

- 2026-09-11 stage:new——rev1 起草交付 work。背景调查完成(管线/机制/图标/样式
  双端覆盖证据齐);唯一未证链路(payload 回调双端往返)已在 T-02 设 first-light
  验证 + 回退预案。outcome: pass(待用户确认后入 work)。
- 2026-09-11 补充(用户反馈修复,auto-os plan-614-dev cf3a901):vue 版主内容面板无滚动条——main 缺 `display:flex`(block),高度链在 main→SidebarProvider 断裂,SidebarContent(overflow-auto) 永不触发,内容被 overflow-hidden 裁剪;shell 形态自 Plan 562 sidebar 族迁移引入,**预存缺陷非 tree 改动回归**;补 `flex flex-col` 修复(浏览器实测 scrollTop 500/原生滚动条 15px;VM 容器类无高度语义不受影响)。gallery 其它页同享此修复。
- 2026-09-11 stage:work | plan_id: PLAN-614 | plan_revision: 1 | outcome: pass(含阻塞挂账) |
  code_commit: auto-lang plan-614-dev 7ae85051b + auto-os plan-614-dev 7e0d8a8(master a3d53cbfc 基) |
  task_ids: T-01..T-08 完成,T-09 归 review |
  evidence: gallery_pages 编译冒烟绿(17s)/p614 VM 探针绿/golden 重基线双绿(含 auto-os 既有漂移,复核理由在 T-07)/cargo t 失败集=master 预存红±跨仓环境摆动零相关/vue 全交互+VM 视觉按钮级截图三张 |
  blockers: P614-C1=VM 轨 MCP press 子组件行(带循环变量 payload 的 onclick)进程静默崩溃(无 panic,复现步骤入契约缺陷节)——阻塞 VM 端行点击场景验收,组件侧已穷尽(FileTree 自带 handler 亦崩→非纯 emit 臂特有),需引擎侧 debugger 立项;真实鼠标点击影响面未测 |
  next: review(/auto-plan:review 独立复审;merge 时补 specs.json upsert)
- 2026-09-11 补充(review 期间):cf3a901 滚动修复发生在 T-07 golden 基线之后→app.at 哈希漂移二次重基线(diff 仅 app.at +14B,裸跑绿;worktree 929680ac5);工作阶段 vue 端截图补档 p614_vue_treeview_expandall.png。

- 2026-09-11 stage:review | plan_id: PLAN-614 | plan_revision: 1 | outcome: blocked | reviewed_commit: auto-lang 929680ac5 + auto-os cf3a901 | base_commit: auto-lang a3d53cbfc / auto-os 99b094c | dependency_revisions: auto-down detached 634f608(仅构建解析) | spec_inputs: docs/specs/auto-lang/ui/design/tree-components.md @929680ac5(评审通过,描述与现状一致;P614-C1 缺陷节+五条 VM 纪律为持久决策) |
  acceptance_results: AC-01 pass(组件/util/schema 与设计一致,gallery_pages 在 tf 内绿)/AC-02 pass(独立浏览器复验:toggle 往返 echo 2↔1 dirs、选中回显 selected: src、滚动 scrollTop 300+15px 滚动条、FileTree 行/badge/图标 token 统计与展开态吻合(pages 收起))/AC-03 **partial**(渲染/深链/图标/按钮级 fresh 复现;行点击=MCP press 崩溃 3/3 无法按计划验证方法执行;OS 级合成点击注入不可达=DPI 虚拟化+z-order 工具限制非结论)/AC-04 pass(混合叶枝+显式 icon 优先+深链,vue level-9/bottom 与 VM 本轮截图)/AC-05 pass(首页双卡/63 徽章/Display(10)/侧边栏项)/AC-06 pass(cf3a901 后二次重基线 diff 仅 app.at +14B,裸跑绿)/AC-07 pass(规范四件+INDEX 重生+spec-index 干净) |
  findings: R-614-1(blocker,engine)=P614-C1 MCP press 子组件行崩溃 3/3 可复现,AC-03 行点击不可验证,真实鼠标影响面未定——需决策:人工点击判定/引擎立项/修订 AC-03;R-614-2(debt,engine)=for-in 参数列表零次迭代(while+索引纪律已入契约);R-614-3(debt,auto-man)=增量路径多-widget .at 同码写盘(ToggleItem.vue=ToggleGroup 内容实证,本计划以独立文件绕开);R-614-4(debt,vue.rs P601-T11 同族)=icon 动态名 Circle 占位+TreeIcon class 带 -icon 后缀 token(无害记录);R-614-5(observation)=master 的 golden 基线对 auto-os main 本已红(PLAN-008 等既有漂移),本次两次重基线已含复核说明;R-614-6(cosmetic)=组件通道对纯 util .at 打 No-widget Warning(预期可静默) |
  evidence: cargo tf 3516/3516 绿(review 档,含 gallery_pages);golden 二次重基线+裸跑绿(929680ac5);vue 独立复验数据(toggle/选中/滚动/图标 token/FileTree 行集);VM 复验截图 p614_review_vm_before_click.png(树完整渲染:folder-open/file-text/book-open/chevron 双态/badge v2·12/echo 2 dirs)+p614_review_vm_hover_check.png(OS 级点击未命中记录);R-614-1 复现=treeview/filetree 页 press 树行 vnode id |
  next: blocked——待决策(1)用户物理点击 VM 窗口 src 行人工判定真实鼠标路径(窗口已开,选中应见高亮+echo);(2)P614-C1 引擎立项;(3)修订 AC-03 口径。决策后恢复相应阶段 |

## 待澄清事项

1. **P614-C1(阻塞项,需决策)**:VM 轨 MCP press 子组件行崩溃(详见复审记录
   blocker 与 tree-components.md 缺陷节)。建议独立引擎计划立项(debugger 定位
   debug-id 跨帧生命周期/计算派生载荷);真实鼠标点击是否受累待人工实测。
2. **work 期间实证的引擎/管线缺陷清单**(组件侧已绕开,是否立项修引擎待指示):
   ①for-in 对函数参数列表零次迭代(状态路径正常);②auto run 增量路径多-widget
   .at 同码写盘(ToggleItem.vue=ToggleGroup 内容实证);③vue 轨 icon 动态名
   Circle 占位(P601-T11 同族并档);④view f-string 内方法调用不解析。
3. **FileTree 真实文件系统接入**(Phase 2 候选):v1 静态数据(vue 端无 fs 原语,
   双端一致性优先);VM 端 `auto.fs.walk`(native_catalog 2860)接入是否立项,待
   用户后续指示——不阻塞本计划。
4. tree 族 Phase 2 面预登记(非本计划范围):复选框多选(checkedKeys)、懒加载、
   DnD、虚拟滚动——契约文档非目标节收录。
