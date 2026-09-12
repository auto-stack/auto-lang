---
plan_id: PLAN-618
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: examples/ui 四 demo 接入 TreeView/FileTree 组件(027/026/018/041)
author: [zcode]
created_at: 2026-09-12
updated_at: 2026-09-12
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/tree-components.md: 修订——TreeIcon 调色板扩充(table/zap)+filter_tree 树过滤纯函数+组件拷贝分发指引(四件清单)入契约"
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉一致——tree 组件族在四个真实 demo 的双端落地验证"

affects: [examples/ui, docs/specs/auto-lang/ui]
current_step: 0
total_steps: 8
---

# [PLAN-618] examples/ui 四 demo 接入 TreeView/FileTree 组件(027/026/018/041)

## 变更摘要

把 PLAN-614 交付的 tree 组件族(TreeView 受控树/FileTree 自包含预设/TreeIcon
调色板/tree_util 拍平与过滤工具)接入四个真实 demo,以真实场景检验并改善 UX:

- **027-file-manager**:新增左侧文件夹树面板(TreeView 受控 directory 模式),
  on_select 复用页面既有 `.NavTo(path)` 驱动右侧文件列表与面包屑。
- **026-database**:左侧"对象树"(现手写三组平铺按钮 ≈100 行重复模板)替换为
  真树(TreeView),接入 tree_util 新增 `filter_tree` 实现既有 treeFilter 搜索;
  节点 id 前缀编码类型(`t/`/`v/`/`i/`)供页面单 handler 分发。
- **018-book-reader**:阅读页新增章节导航树(TreeView,book→chapters),
  on_select → `router.push` 章节路由(既有 `/book/:id/chapter/:ch` 契约)。
- **041-auto-edit**:新增左侧文件浏览器面板,以 **TreeView 受控**形态接入
  (页面持 selected,on_select → 新增 OpenByPath 动作按路径打开编辑 tab,
  复用既有 tabs 机制)——FileTree v1 不回传选中,联动需求故用受控形态。

组件分发 = shadcn 式拷贝:tree 四件(tree_util/treeview/filetree/tree_icon.at
+package.at)复制进各 demo `src/front/components/`。

## 目标

1. 四个 demo 双端(vue/VM)接入树组件且交互正确(选中/展开/过滤/导航驱动)。
2. 026 的平铺按钮树被真树替换,代码量净减,treeFilter 功能保留且体验增强
   (过滤后自动全展开)。
3. tree_util 新增 `filter_tree` 纯函数(while+索引纪律)入契约;TreeIcon 调色板
   扩充 `table`/`zap`(VM lucide_svg 补 zap 条目——引擎 1 行微修)。
4. 拷贝分发清单成文(契约修订),四个 demo 的组件副本与画廊源保持同版本。

### 非目标

- 共享组件包/registry 机制(多 demo 拷贝去重)——单独立项。
- FileTree on_select 透传(自包含契约扩展)——Phase 2。
- 027 真实文件系统接入(auto.fs.walk)——Phase 2。
- 018 后端 chapter 元数据契约变更(树数据由页面从既有 get_book 结果构造)。

## 架构方案

- 组件分发:每 demo `src/front/components/` 放入 tree 四件拷贝 + package.at,
  页面 `use { package: official from "./components" }` +
  `use components.tree_util: ...`。
- 027/026/018 用 **TreeView 受控**(页面持有 expanded/selected,事件驱动页面
  状态);041 亦用 TreeView 受控(OpenByPath 桥接)。
- 026 过滤:`filter_tree(nodes, kw)` 保留命中节点与全部祖先,过滤结果以
  `collect_ids(result, false)` 全展开;kw="" 还原全量。
- 双端纪律沿用 tree-components.md(while+索引遍历/全键 schema/单实例/view
  f-string 禁方法调用)。

## 需求分析与背景调查

- 用户授权(2026-09-12):"027, 026, 018, 041 这4个demo都可以加上
  TreeView或FileTree组件来改善UI/UX,请分析并建立一个新的计划文件"。
- 027:`.NavTo(c.path)` 页面 handler 既有(app.at:244 面包屑在用);数据为静态
  `files_view`/`crumbs`(app.at:184/33),无真实 fs;左树数据按同世界静态构造。
- 026:对象树 = 手写三组平铺按钮(app.at:861-940+,Tables7/Views2/Indexes3,
  每对象一个 button + 重复选中态三元样式);`treeFilter`+`.TreeFilterChanged`
  既有(app.at:47/687);`.SelectTable/.SelectView/.SelectIndex` 三 handler
  保留,页面新增 `SelectObject(id)` 按前缀转发。
- 018:阅读页 `/book/:id/chapter/:ch` 路由驱动,prev/next 导航;章节列表经
  `use back.api: get_book`(pages/reading.at:7);书架 sidebar 为平铺菜单。
  树数据由页面把 get_book 结果映射为 nodes;on_select handler 调 router.push。
- 041:tab 化代码编辑器(store.tabs/active_key,app.at:107/136),ActOpen 走
  打开流程;文件浏览器面板为新增 UI,store 需新增"按路径打开"动作
  (静态演示文件表 path→内容/语言)。
- TreeIcon 调色板现状:folder/folder-open/file/file-text/image/home/wrench/
  layout-grid/terminal/settings/eye/book-open + chevron 双态;VM lucide_svg
  已有 table,无 zap(026 索引图标需补)。
- 组件副本版本锚:auto-os main 797fbd8(tree guides 版)。

## 详细设计

### 各 demo 接入设计

- **027**:app.at 布局左列加 `col (style: "w-60 border-r ...")` 内嵌
  `TreeView (nodes: .fmNodes, expanded: .fmExpanded, selected: .fmSel,
  on_toggle: .FmToggle, on_select: .FmSelect, directory: true)`;`.FmSelect(id)`
  → 复用 NavTo 内部逻辑(置 crumbs/files_view);fmNodes 按 /root 世界构造
  (Documents/Pictures/Music/... 静态层级)。
- **026**:对象树区替换为 TreeView;`dbNodesFull`(连接→db→三组→对象,对象
  badge=行数)为 model 全量;treeFilter oninput → `.TreeFilterChanged` →
  `dbNodes = filter_tree(dbNodesFull, kw)` + `dbExpanded =
  collect_ids(dbNodes, false)`(kw="" 还原);on_select(id) 按前缀分发到
  SelectTable/SelectView/SelectIndex;选中由 `selected` 呈现。
- **018**:阅读页左列(既有布局加 w-56 树列)TreeView;chapters → nodes
  (label=章题,id=章节号);on_select(id) → `router.push` 该章;当前章经
  `selected` 高亮(页面在章切换 handler 同步 selected)。
- **041**:左列 TreeView(静态项目文件树);`on_select → .OpenByPath(path)`
  → store 新增动作:静态文件表查内容/语言,构造/激活 tab(复用 tabs 机制)。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/design/tree-components.md | TreeIcon 调色板 +table/+zap;新增 filter_tree 契约节(过滤保祖先、kw 空还原、while+索引纪律);新增"组件拷贝分发"节(四件清单+use 模板+P320 单实例提示) | 四 demo 落地的契约增量;分发指引防漏件 | AC-02/03/06 |
| SD-02 | modify | docs/specs/auto-lang/ui/plans.md | 增 PLAN-618 行 | 台账 | AC-07 |

## 测试设计

| 闸门 | 内容 |
|---|---|
| 每 demo | 双端实机(autoui-verifier:vue Playwright + VM MCP 截图/交互);examples/ui 不在 gallery_pages 覆盖内,以 demo 自身双端运行为准 |
| 引擎 | `cargo test -p auto-lang --lib child_emit`(回归);lucide zap 截图验证 |
| 收尾 | `cargo t`(日常档,失败集对齐 master 预存红) |

## 验收标准

- **AC-01 分发**:四 demo components/ 各含 tree 四件+package.at,use 声明
  正确;拷贝自 797fbd8 版本;四 demo 双端编译通过。
- **AC-02 027**:左树展示 /root 层级;点击树文件夹 → 右侧列表/面包屑导航到
  该目录(双端);chevron 展开/收起正常。
- **AC-03 026**:平铺按钮树删除;真树三组+badge 行数;treeFilter 输入过滤
  (命中+祖先保留,自动全展开,清空还原);点击对象 → 右侧数据区切换(双端)。
- **AC-04 018**:阅读页章节树可见;点击章节 → 路由跳转该章(双端);当前章
  高亮(selected)。
- **AC-05 041**:左文件树面板;点击文件 → 打开/切换对应编辑 tab(双端)。
- **AC-06 图标**:026 表/视图/索引图标(table/eye/zap)双端正确;VM lucide
  zap 条目生效。
- **AC-07 规范**:SD-01/02 落地,spec-index 干净。
- **AC-08 收尾**:四 demo 双端截图入档;cargo t 失败集=master 预存红。

## 执行步骤

- **T-01 worktree 建组**:本计划只动 examples/ui(auto-lang 仓)——裁定
  单仓:仅 auto-lang worktree `.wt/lang-618/auto-lang @ plan-618-dev`
  (骨架已提交 master c4433cde5)。
- **T-02 filter_tree + 调色板扩充**:tree_util.at 加 filter_tree(while+索引,
  命中+保祖先);TreeIcon +table/+zap;VM lucide_svg +zap(1 行);探针/单测。
  验证:child_emit/探针测试绿。
- **T-03 027 接入**:components 拷贝+左树面板+FmSelect→NavTo;双端验证。
- **T-04 026 接入**:替换平铺树+filter;双端验证(过滤/选中/数据区切换)。
- **T-05 018 接入**:阅读页章节树+router.push;双端验证。
- **T-06 041 接入**:OpenByPath store 动作+左文件树;双端验证。
- **T-07 规范沉淀**:SD-01/02 落地+INDEX 重生。
- **T-08 收尾**:cargo t 对齐预存红;四 demo 截图入档;execution_done。

## 复审记录

- 2026-09-12 stage:new——rev1 起草。背景调查完成(四 demo 现状/事件流/图标
  双端可用性);裁定:041 从 FileTree 改为 TreeView 受控(FileTree v1 不回传
  选中,联动需求使然);026 filter 落 tree_util.filter_tree(页面过滤进阶用法)。
  outcome: pass(待确认后入 work)。next: work。

## 待澄清事项

1. 018 章节树数据依赖 get_book 返回形状(实现时实测,必要时页面侧适配)。
2. VM lucide_svg 补 zap 条目属引擎 1 行微修(先例:PLAN-614 路由键修复);
   若不愿动引擎,索引图标降级为既有 "table"/"terminal"。
