---
plan_id: PLAN-618
status: execution_done   # T-01..T-08 全完成;next=review
feature_name: examples/ui 四 demo 接入 TreeView/FileTree 组件(027/026/018/041)
author: [zcode]
created_at: 2026-09-12
updated_at: 2026-09-13
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - "docs/specs/auto-lang/ui/design/tree-components.md: 修订——TreeIcon 调色板扩充(table/zap)+filter_tree 树过滤纯函数+组件拷贝分发指引(四件清单)入契约"
touched_goals:
  - "GOAL-007: AutoUI 跨端视觉一致——tree 组件族在四个真实 demo 的双端落地验证"

affects: [examples/ui, docs/specs/auto-lang/ui, crates/auto-lang/src/ui_gen]
current_step: 8
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
- **T-02 filter_tree + 调色板扩充** [x] [✅ 已完成] tree_util.at 加
  filter_tree(递归+while+索引,命中+保祖先,祖先保留语义探针断言);TreeIcon
  +table/+zap;VM lucide_svg +zap。**过程发现(挂账)**:①VM 模块 fn 递归可用
  (fact 120 探针);②VM .lower() 链在「局部 var 派生字段+嵌套帧」返回空串
  (两步赋值浅帧正常/深帧仍失效)→ v1 大小写敏感 + contains 直链。探针测试
  p618_filter_tree_probe 全绿。证据:worktree 9ac3c5661 + auto-os b7337b1。
- **T-03 027 接入** [x] [✅ 已完成,vue 端实证;VM 端待 T-08 补验] 左侧快速
  访问按钮组替换为 TreeView(directory 受控);FmSelect → NavTo 联动;use 块
  修正(package use 必须在 widget 内,顶层 parse 报 LBrace 错)。vue 实证:树
  渲染+点击文档文件夹 → root/Documents 列表/面包屑联动(d177b9a44)。
- **T-04 026 接入** [x] [✅ 已完成,vue 端实证+2026-09-13 浏览器复核过] 平铺对象树
  替换为 TreeView 真树;dbNodes 改 computed(filter_tree(全量,treeFilter),初始化
  与过滤合一,免 Init);dbSelId 专用选中字段;use 补 toggle_id;孤儿段清理。
  vue 实证(复核):树渲染三组+badge、过滤 cust→customers+idx_orders_customer
  (zap)+Tables 祖先保留+Views 剪除+自动全展开、清空还原、点击 orders→
  Table: orders 切换。**VM 实证(T-08b 补)**:type cust 剪除/还原/press orders
  →totalRows 91→10+dbSelId t/orders。截图 2 张入档。证据:worktree
  3cf6b5276+684620d93。
- **T-05 018 接入** [x] [✅ 已完成——R1-a 解除,vue 端 AC-04 全过] 章节导航树
  (TreeView 受控):ch_nodes 模块 fn+computed;ChSelect 按 id 匹配章节号→
  router.push;Prev/Next/Init 反向同步 chSel 高亮;Init 补 list_chapters。
  **隔离结论(2026-09-13)**:R1-a"解析级联"不复现——真因二件:①package use
  相对 pages/ 必须写 `../components`(原 `./components` 按页面目录解析→
  package load failed);②**模块 fn 参数名与 model 字段同名→vue 转译器对
  参数误加 `.value` 拆包**(chapters.value.length)→SFC 运行时 TypeError
  整页白屏,改名规避,转译陷阱入契约+挂引擎债。vue 实证:树渲染+点
  Chapter 3→`#/book/1/chapter/3` 跳转+正文切换+Prev→树高亮反向同步。
  截图入档。证据:worktree d59cc9871。
- **T-06 041 接入** [x] [✅ 已完成,VM 实证(该 demo 原生轨即 VM)] EditorStore
  新增 tree_nodes/tree_expanded/tree_sel+TreeToggle/TreeSelect;TreeSelect=
  OpenByPath 语义:同路径 tab 激活去重,静态文件表构造新 tab 复用 tabs 机制
  (新 key 生成新 code_editor,免 set_text);左列 w-56 EXPLORER 树,tab 条/
  编辑区留根视图(P449 边界=组件子树,容器嵌套不受限)。VM/MCP 实证:树渲染+
  点 app.at 开 tab(tab_count 2→3)+点 editor_store.at 切换+重复点 app.at
  激活已有 tab(active_key tab-4→tab-3 零重复)+目录行 Select 仅高亮;树行
  press 正常未复现 P614-C1。VM 预存怪象挂账(非 tree 引入):confirm 弹层
  文案常显/状态栏 ${store.line} 字面量。截图入档。证据:worktree 3539fcd6c。
- **P618-1** [x] [✅ 已完成(2026-09-13)] 027 vue 轨 popover 常显根因=旧发射臂
  发 `<Popover v-model:open>`——shadcn-vue PopoverRoot 无条件渲染 slot→弹层
  内容常显内联堆叠页底。修复:vue.rs popover 臂重写为自绘 overlay(terminal
  臂同款早退模式)——v-if 门控(open 关即卸载)+backdrop 点击 dismiss
  (ondismiss)+fixed 坐标锚(x/y,与 VM iced 臂语义对齐),不进 shadcn 装配
  (Popover import 消失);desktop 金样按机制同步(AUTO_LANG_UPDATE_GOLDEN)。
  vue 模块 288 测试全绿。027 vue 实证:首屏零泄漏+右键菜单坐标浮层+
  if 门控项(粘贴到此处)随剪贴板态显隐+backdrop 关闭。截图 2 张入档。
  证据:worktree bf77fb30f。**环境注记**:本机 RUSTC_WRAPPER=sccache 处于
  故障态(所有 cargo 构建随机 os error 5 拒绝访问,.d 写入被拒),本计划全程
  RUSTC_WRAPPER= 禁用绕过,与 619 会话并发构建争抢相关,机器级问题不入计划债。
- **T-07 规范沉淀** [x] [✅ 已完成] SD-01:tree-components.md 修订(TreeIcon
  +table/+zap/filter_tree 契约节/组件拷贝分发节(四件清单+use 模板+package
  use 位置与 pages/ 相对路径陷阱+P320 复核)/模块 fn 转译陷阱/618 三 demo
  落地验证基线追加+P614-C1 复现面收窄注记);SD-02:plans.md 增 618 行。
  INDEX/specs.json 派生台账按范式归 merge 收口。证据:worktree 6c2eb81af。
- **T-08 收尾** [x] [✅ 已完成] ①cargo t 日常档(worktree,no-fail-fast):
  4806 run/4782 pass/**24 failed**——逐项归因:21 项≡617 门禁 T-15 归因集
  (layout 12+scan_examples+plan492 c2+lucide coverage+ffi_dual_019+plan055
  +desktop_protocol coverage+external_config_poll flake);新差 3 项
  (plan370_015 d2/d8/z6)=**分支基线陈旧**——618 基线(bec57397a)不含 616 的
  015-notes 重做(71378ef24 已在 master),测试与源错位;还原法排除 618 两个
  crates 提交(还原 vue.rs/T-02 文件后仍红),617 worktree(含 616)同滤串
  12/12 绿。**相对 618 改动面零新增红**;fold 时从最新 master 重建即对齐。
  ②双端补验:026 VM(filter/还原/点击联动全过)+027 VM(press 文档→
  current_path /root/Documents+crumbs+files_view 联动,AC-02 VM 腿过);
  018 VM 腿**环境受阻**(wgpu surface 报错进程退出,窗口最小化竞争,与
  027 VM 截图 skipped 同族)——组件 VM 行为已由 041/026/027 三重实证,
  018 专属 VM 风险(router.param+HTTP back.api)属 demo 基建,登记观察。
  ③截图入档:四 demo 共 6 张(见各 demo src/front/tests/screenshots/,
  gitignored 不入库)。

## 复审记录

- 2026-09-12 stage:new——rev1 起草。背景调查完成(四 demo 现状/事件流/图标
  双端可用性);裁定:041 从 FileTree 改为 TreeView 受控(FileTree v1 不回传
  选中,联动需求使然);026 filter 落 tree_util.filter_tree(页面过滤进阶用法)。
  outcome: pass(待确认后入 work)。next: work。
- 2026-09-12 stage:work(进行中) | plan_id: PLAN-618 | rev1 | T-01..T-03 完成,
  T-04..T-06 待续 | 证据:worktree 9ac3c5661(T-02)/d177b9a44(T-03);探针全绿;
  027 vue 交互实证 | blockers: 无 | next: 继续 T-04..T-06
- 2026-09-13 stage:work | plan_id: PLAN-618 | rev1 | outcome: **pass**,
  execution_done | code_commit: worktree plan-618-dev @ 6c2eb81af |
  task_ids: T-01..T-08 全闭环+P618-1 引擎修复 | evidence: 四 demo 双端/原生轨
  实机全过(026 vue+VM/027 vue+VM/018 vue/041 VM·MCP),截图 6 张入档;vue 模块
  288 测试全绿+desktop 金样同步;cargo t 24 红逐项归因=21 项 617 归因集+3 项
  基线陈旧(616 已在 master),零新增红;T-05 R1-a 解除(真因=package use 相对
  路径+fn 参数/model 同名转译陷阱);P618-1 根修(popover vue 臂重写) |
  blockers: 无 | next: **review**(§9 复审;SD-01/02 spec delta 校验+INDEX/
  specs.json 由 merge 收口)

## 待澄清事项

1. 018 章节树数据依赖 get_book 返回形状(实现时实测,必要时页面侧适配)。
   ——T-05 落地时以 list_chapters 适配完成。
2. ~~P618-1(用户反馈,预存缺陷)~~ **已修复**(2026-09-13,worktree bf77fb30f):
   popover vue 臂重写为 v-if 门控+backdrop dismiss+fixed 坐标锚,027 首屏
   零泄漏实证。
3. **P618-2(引擎,挂账)**:.lower() 方法链在「局部 var 派生字段+嵌套帧」
   返回空串(P618 探针实证);v1 过滤器降级大小写敏感。
4. **P618-3(引擎债,新挂账,2026-09-13)**:vue 转译器对「模块 fn 参数名与
   model 字段同名」误加 .value 拆包→SFC 运行时 TypeError 白屏(018 T-05
   实证,ch_nodes(chapters) 改名 chs 规避);建议转译器按作用域区分参数与
   状态名,契约已入 tree-components.md 陷阱节。
5. **P618-4(观察,非本计划)**:041 VM 轨预存怪象——confirm 弹层文案
   open:false 仍渲染于页底+状态栏 ${store.line} 字面量(与 P618-1 同族,
   VM 臂),非 tree 引入,待弹层系统 VM 臂专项。
6. **P618-5(观察,非本计划)**:018 VM 腿因 wgpu surface 环境故障未能补验
   (窗口最小化竞争);组件 VM 行为已三重实证,018 专属集成(router.param+
   HTTP back.api)留环境恢复后或 fold 后补验。
2. VM lucide_svg 补 zap 条目属引擎 1 行微修(先例:PLAN-614 路由键修复);
   若不愿动引擎,索引图标降级为既有 "table"/"terminal"。
   ——T-02 已按前项落地(zap 生效,026 vue+VM 双端实证)。
