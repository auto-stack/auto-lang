---
plan_id: PLAN-614
status: drafting               # drafting → executing → execution_done → reviewed → archived
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
current_step: 0
total_steps: 8
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

- **T-01 建组** [auto-lang master + auto-os main]
  master 提交 `.next-id`+本计划骨架;`git worktree add D:/autostack/.wt/lang-614/auto-lang -b plan-614-dev`;
  `git -C D:/autostack/auto-os worktree add D:/autostack/.wt/lang-614/auto-os -b plan-614-dev`。
  验证:两 worktree `git status` clean,分支名正确。
- **T-02 first-light:tree_util + TreeView 骨架 + 最小页** [auto-os worktree]
  `components/tree_util.at`(flatten_tree/toggle_id 先行)、`components/treeview.at`
  (受控全签名,行渲染简化版)、`pages/treeview.at`(Basic 单卡)、app.at 路由+侧边栏
  两项。验证:`AUTO_OS_ROOT=... cargo t gallery_pages`(过)后,worktree 画廊
  `auto run` 与 `auto run -r vm` 双端:chevron 往返 + 行选中 + payload 回调
  (on_toggle/on_select)双端生效——payload 链路不通则启动回退预案(§架构方案)。
- **T-03 TreeView 组件与页面完整化** [auto-os worktree]
  directory 模式 + 图标解析入 flatten_tree;collect_ids;页面四卡+属性表+任意形状
  数据(AC-04)。验证:双端目检四卡;`cargo t gallery_pages`。
- **T-04 FileTree 组件与页面** [auto-os worktree]
  `components/filetree.at`(map_fs_nodes+自包含态)、`pages/filetree.at`(两卡+属性
  表)、路由。验证:双端目录图标切换/默认展开/选中;`cargo t gallery_pages`。
- **T-05 gallery 布线收口** [auto-os worktree]
  index.at 两张 component-card + 计数 63;侧边栏图标核对(VM lucide 表);README
  计数核对(如有)。验证:首页卡可跳转(双端);`cargo t gallery_pages`。
- **T-06 规范沉淀** [auto-lang worktree]
  `docs/specs/auto-lang/ui/design/tree-components.md` 新建 + overview.md tree 段 +
  plans.md 614 行 + INDEX 重生 + `.autoos/specs.json` upsert(merge 技能收口前 provisional)。
  验证:`python scripts/spec-index.py` 干净退出。
- **T-07 golden 重基线 + 门禁** [auto-lang worktree]
  `GALLERY_GOLDEN_UPDATE=1 cargo test -p auto-lang --test gallery_golden` → 人工
  复核 fixtures diff → 裸跑二次稳定;`cargo check -p auto-lang`;`cargo t`。
- **T-08 双端全量验证 + 复审准备** [双 worktree]
  autoui-verifier 截图矩阵(两页 × {初始,展开,选中} × 双端)+ 交互脚本;证据存
  `pages/../tests/screenshots/` 同款目录;回填计划勾选与证据行;status → execution_done。
- **T-09 独立复审(/auto-plan:review)**:AC-01..07 逐条对码核验;遗漏/绕开扫描入
  KNOWN-DEBT-AND-RISKS.md;健康检查(无警告/无 debug 残留)。

## 复审记录

- 2026-09-11 stage:new——rev1 起草交付 work。背景调查完成(管线/机制/图标/样式
  双端覆盖证据齐);唯一未证链路(payload 回调双端往返)已在 T-02 设 first-light
  验证 + 回退预案。outcome: pass(待用户确认后入 work)。

## 待澄清事项

1. **FileTree 真实文件系统接入**(Phase 2 候选):v1 静态数据(vue 端无 fs 原语,
   双端一致性优先);VM 端 `auto.fs.walk`(native_catalog 2860)接入是否立项,待
   用户后续指示——不阻塞本计划。
2. tree 族 Phase 2 面预登记(非本计划范围):复选框多选(checkedKeys)、懒加载、
   DnD、虚拟滚动——契约文档非目标节收录。
