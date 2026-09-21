---
plan_id: PLAN-677
status: reviewed                     # drafting → executing → execution_done → reviewed → archived
feature_name: vue-track-parity-generator-fixes
author: [zcode]
created_at: 2026-09-21
updated_at: 2026-09-21

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 vue 轨视图 menubar 族 lowering 契约, SD-02 vue 轨 for 透明包装与 natives 三层发射契约]
touched_goals: []

affects: [docs/specs/auto-lang/ui/design/app-generation.md, docs/specs/auto-lang/ui/overview.md]
current_step: 8
total_steps: 8
---

# [PLAN-677] vue 轨跨平台一致性五连修（生成器侧）

> completion_kind: delivered。合并收据：PLAN-677:r2 prepared 4b1f4916a（规范增量落 app-generation.md）→ 重基 4b1f4916a→36d75036f（range-diff 等价，旧新映射在案）→ landed ff-only master@36d75036f → ledger_refreshed P677-1（reviews 纯增量 11 行）→ archived 本提交 → cleaned 待记。

> 来源：消费方 auto-edit（M1 编辑器）vue/vm 双轨对照实测（2026-09-21，
> 工具链 v0.4.2-1652，PLAN-671 补件链退役后首轮真实使用反馈）。
> **用户裁定（2026-09-21）**：vue/vm 跨平台一致性问题**禁走生成物补件路线**
> （regen 后即丢），必须修 auto-lang 生成器本体。本计划全部修复面落在
> 本仓两个 crate：`crates/auto-lang`（ui_gen/vue.rs 发射层）与
> `crates/auto-man`（vue 工程模板/natives 层）。消费方仓零代码改动。

## 变更摘要

五项 vue 轨发射缺口修复 + 一项延伸一致性件（code_editor 内建桥最小面），
全部为生成器/模板侧改动：

1. **for 循环透明包装**（高度贯通）：单 `if` 体循环改发
   `<template v-for :key>`（不产盒），多语句体循环维持 div 包装不变。
2. **CodeEditor 深色主题 + foldGutter**：工程模板补 CodeMirror 深色
   gutter/语法高亮主题（走 shadcn CSS 变量）+ 折叠 gutter 与快捷键。
3. **视图声明式 menubar 族 lowering**：`menubar-menu/trigger/content/
   item/separator/checkbox-item` 映射 shadcn Menubar 组件树（镜像既有
   actions 占位 lowering 与 vm 侧 convert_menubar 语义）。
4. **natives 三层发射**：`file_basename` 与 `console_*` 族从抛错桩改为
   真 JS 实现（内存 console buffer）；其余维持 fail-fast 桩。
5. **code_editor 内建桥最小面**：`src/lib/editorBridge.ts` 注册表 +
   CodeEditor 以 editor-key 注册 + natives 路由，覆盖消费方实际行使的
   `set_text/text/cursor_*/selection_len/fold_*/undo/redo/select_all`
   （cut/copy/paste 尽力而为）。

## 目标

消费方 auto-edit vue 轨达到与 vm 轨一致的五点可用性（用户实测问题清单）：

| # | 用户问题 | 根因（生成物实勘） | 修复面 |
|---|---------|-------------------|--------|
| 1 | 编辑器高度内容化，不贯通 | `App.vue:296` for-lowering 发裸 `<div v-for :key>` 包装，斩断 flex 链（CodeEditor 自身 flex-1/h-full 够不着弹性列） | ui_gen/vue.rs for-lowering |
| 2 | 行号浅色主题 | CodeEditor.vue 模板（auto-man 内联串）无深色主题，CodeMirror 默认浅色 gutter/高亮 | auto-man/src/vue.rs 模板 |
| 3 | 缺 gutter 与折叠组件 | 模板无 foldGutter；菜单 ActFold 调 `code_editor_fold_toggle/fold_hidden_count` 为抛错桩 | auto-man 模板 + 内建桥 |
| 4 | menubar 无下拉 | 视图声明式 menubar 族无 vue lowering → 通用 div 退化（`App.vue:195-232`）；actions 占位路径已有 lowering 未被此源形态走到；shadcn `ui/menubar` 全家桶已在生成工程内 | ui_gen/vue.rs |
| 5 | 树点击文件打不开 | `natives.ts` 把 `file_basename` 发为抛错桩：TreeSelect 在 `tabs.push({title: file_basename(id)})` 处即抛（useEditorStore.ts:290-313）；`console_log/console_lines` 同为桩（console 面板恒空） | auto-man ensure_natives_layer |
| 5' | （延伸）切 tab/新建/折叠菜单 vue 轨控制台抛错 | SyncCursor（editor_store.at:143-147）调 `code_editor_cursor_line/col/selection_len`、ActNew 根 handler（app.at:247-252）调 `code_editor_set_text`、ActFold 调 `fold_toggle/fold_hidden_count`——全为桩，每次切 tab/新建即抛 | 内建桥（本计划第 5 项） |

### 非目标

- auto-edit 仓任何代码/生成物改动（消费方验证用冷再生成，产物不入仓提交）。
- vm/iced 轨任何行为变化（五项修复面全部是 vue 发射路径；vm 侧仅作对照
  回归探针）。
- code_editor 桥的 cut/copy/paste 完保（浏览器剪贴板权限模型限制，
  尽力而为 + 登记降级）； rope/delta 编辑器内核（PLAN-673 领地）。
- RQ/a2r 面（PLAN-674 领地）；`auto run -r vue` 官方入口的运行时
  （vm 宿主内建无 vue 运行时的架构限制不变）。
- dialog_open/dialog_save/Env.get/Process.exit 等 vm-only 内建的 vue
  实现（维持桩 + 消费方 README 登记制）。

## 架构方案

四个发射层落点 + 一个运行时桥，全部在生成期凝固（无消费方补件）：

```
crates/auto-lang/src/ui_gen/vue.rs      ← ① for 透明包装 ③ menubar 族 lowering
crates/auto-man/src/vue.rs              ← ② CodeEditor 模板 ④ natives 三层
crates/auto-man/src/vue.rs (模板集)      ← ⑤ editorBridge.ts 新模板 + CodeEditor 注册
```

- **① for 透明包装**：fallback v-for wrapper（注释见 vue.rs L107-121，
  发射点 ~L8049；既有测试 `test_plan008_loop_wrapper_hoists_child_key`）
  现无条件发 `<div v-for="..." :key="...">`。改为：**循环体为单个 if
  语句时发 `<template v-for="..." :key="...">`**（Vue3 允许 key 上
  template；template 不产盒 → 子内容直接成为父 flex 容器的 item，
  高度链贯通）。多语句体循环维持 div 包装（多子迭代在块格式化上下文
  里的既有布局语义不变，防全局回归）。key 提升（branch-first text
  hoisting）语义平移到 template 的 :key。
- **② CodeEditor 深色主题 + 折叠**：`generate_code_editor()`（auto-man
  vue.rs ~L700-780 内联模板串）extensions computed 内追加：
  - 深色主题：`EditorView.theme` 走工程既有 shadcn CSS 变量
    （`--background/--foreground/--muted/--border/--accent`，html.dark
    级联，CodeMirror 注入样式可消费 CSS var）+ `syntaxHighlighting(
    HighlightStyle.define(...))` 深色调色板（comment/string/keyword/
    number/operator/类型五族，与 iced 端 syntect AutoLang 观感对齐）。
  - 折叠：`foldGutter()` + `foldKeymap` + `syntaxHighlighting` 同源
    （@codemirror/language，已是依赖零新增）。
- **③ menubar 族 lowering**：视图组件树中 `menubar*` kind 现走通用
  unknown-kind div 退化。新增 kind→shadcn 组件映射（组件已在脚手架
  `components/ui/menubar/` 全量在架）：menubar-menu→MenubarMenu、
  menubar-trigger→MenubarTrigger（子文本）、menubar-content→
  MenubarContent、menubar-item→MenubarItem（title→文本、icon→lucide
  导入 kebab_to_pascal 既有 helper、shortcut→MenubarShortcut、
  `enabled: expr`→`:disabled="!(expr)"`、onclick→@click 契约事件）、
  menubar-separator→MenubarSeparator、menubar-checkbox-item→
  MenubarCheckboxItem（`checked: expr`→`:checked`）。项渲染结构镜像
  既有 `generate_actions_menubar_html`（vue.rs L6249）；语义锚 vm 侧
  `convert_menubar_component`（aura_view_builder.rs L7486）。
- **④ natives 三层发射**：`ensure_natives_layer`（auto-man vue.rs
  L1642-1795）现单一 names[] 全发抛错桩。拆三层：
  - **R 层（真实现）**：`file_basename`（`p.split(/[\\/]/).pop()||p`，
    语义对齐 vm 内建，work 期实勘 vm 实现核对分隔符/空串行为）；
    `console_log/console_lines/console_clear`（模块级内存 buffer，
    语义对齐 vm：log 追加行、lines 取全量、clear 清空——vm 实现为准）。
  - **B 层（桥路由）**：第 5 项 code_editor 桥覆盖的名字 → 注册为
    bridge 调用函数。
  - **S 层（fail-fast 桩）**：其余维持现状（671 §10-1 裁定不变）。
  幂等机制沿用既有 preserve 名单与 main.ts import 守卫。
- **⑤ editorBridge**：auto-man 新增 `src/lib/editorBridge.ts` 模板：
  `Map<string, EditorView>` 注册表 + `register/unregister/withView`
  + 逐内建 API（setText/text/cursorLine/cursorCol/selectionLen/
  foldToggle/foldHiddenCount/undo/redo/selectAll/cut/copy/paste）。
  CodeEditor 模板增 `editorKey` prop，`on_ready` 注册、卸载注销；
  ui_gen/vue.rs 的 code_editor 发射把 DSL `key:` 实参透传为
  `:editor-key`。natives B 层名字 → `globalThis` 函数调用 bridge
  （含 vm 语义换算：vm cursor 返回 0-based（store +1 消费）、fold
  行号 1-based（ActFold 传 2）、fold_hidden_count = 当前折叠隐藏行数）。
  同 key 多实例（tab 复用）后注册者胜（与 vm 单活动编辑器语义一致）。

## 需求分析与背景调查

- **授权**：用户 2026-09-21 明确指令——五问题综合成一个计划并一起修；
  修复面=Auto 代码/生成器（非消费方补件）；流程=auto-plan 范式。
  授权仓：本仓（auto-lang）实现 + auto-edit 仓只读作消费方验证锚。
- **消费方实证**（2026-09-21 实勘，auto-edit 主检出
  `specs/auto-edit`，工具链 1652）：
  - `gen/front/vue/src/App.vue:296` 裸 v-for div 包装（问题 1）；
  - `gen/front/vue/src/components/CodeEditor.vue` 全文无深色主题/fold
    （问题 2/3）；
  - `gen/front/vue/src/App.vue:195-232` menubar 子树全裸 div（问题 4）；
  - `gen/front/vue/src/lib/natives.ts` names[] 含
    file_basename/console_log/console_lines（问题 5）；
  - `gen/front/vue/src/stores/useEditorStore.ts:290-313` TreeSelect
    抛点链条（问题 5）；同文件 TabActivate→SyncCursor 链（问题 5'）。
- **上游在途协调**：PLAN-672（ui-gallery-vue-fix-batch，N5 back_proxy
  面）与 PLAN-675（routes-in-embed，current_step 5/7）在途——与本计划
  文件面不相交（672 在 auto-man api/代理侧、675 在 embed 路由侧；
  本计划在 ui_gen/vue.rs 发射层 + auto-man 前端模板层）。672 若也在
  auto-man/src/vue.rs 有改动，merge 顺序以先落者为准、后落者 rebase
  （登记进执行步骤 T-07 前置检查）。
- **规格现状**：ui/design/app-generation.md 为 vue 工程生成契约文档；
  menubar 族 vue lowering 契约、for 包装策略、natives 分层均未见条款
  ——本计划以 SD-01/SD-02 提出增补（复审定稿）。

## 详细设计

### 规范增量

| delta_id | add/modify/retire | docs/specs/... 目标 | before/after 规则 | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | auto-lang/ui/design/app-generation.md | before: 视图声明式 menubar 族 vue 侧无条款（实践=通用 div 退化）；after: menubar 族必须 lower 到 shadcn Menubar 组件树（项渲染=icon+title+MenubarShortcut、enabled→:disabled 取反、checked→MenubarCheckboxItem、onclick→契约事件），与 vm convert_menubar 语义同源 | 双轨一致性（用户裁定：一致性修复只认生成器） | AC-04 |
| SD-02 | add | auto-lang/ui/design/app-generation.md | before: for 一律发 div 包装、natives 单层抛错桩；after: 单 if 体循环发 template v-for 透明包装（多语句体维持 div）；natives 按 R（真实现：file_basename/console 族）/B（editorBridge 路由：code_editor 桥面）/S（fail-fast 桩）三层发射 | 高度链贯通 + vue 轨内建诚实化 + 消费方主流程零抛错 | AC-01/02/03/05/06 |

### 关键设计决策

- **单 if 体才透明化**：多语句循环体的 div 包装承载块级布局语义
  （多子迭代互不并排），全局 contents 化会改变既有消费方布局——
  收窄到「单 if 体」族（auto-edit 编辑器循环即此形态）， blast
  radius 最小化；gallery 消费方构建作回归门（T-07）。
- **主题走 CSS 变量而非硬编码色**：生成工程的 shadcn token 级联
  （html.dark）是唯一真源，CodeMirror 注入样式消费同一变量，避免
  第二套调色板漂移；高亮五族用 HighlightStyle 深色定值（ syntect
  AutoLang 观感对齐,不定死变量——token 色无对应 shadcn 变量）。
- **桥注册以 DSL key 为幂**：store 侧全部经 `active_key` 路由
  （editor_store.at 既有语义），桥 registry 同键对齐；后注册胜
  = 与 vm「单活动编辑器」一致。
- **语义换算集中桥内**：0-based cursor、1-based fold 行号、隐藏行
  计数,全部在 editorBridge.ts 内换算成 CodeMirror API,模板与
  natives 层不做第二次换算（单点换算防两端语义漂移）。

## 测试设计

- **单测（随改动入库）**：
  - ui_gen/vue.rs：单 if 体循环→template v-for 快照断言（含 key
    提升平移）；多语句体→维持 div 包装（负例）；menubar 族 lowering
    快照（四菜单一勾选项一分隔线一 enabled 项，断言组件名/属性/
    lucide 导入/MenubarShortcut）；code_editor 发射含 :editor-key。
  - auto-man/src/vue.rs：CodeEditor 模板断言（foldGutter/主题 CSS
    变量/editorKey prop/注册卸载）；ensure_natives_layer 三层断言
    （R 层真实现体、B 层 bridge 调用、S 层抛错桩；幂等二写不变）。
- **作用域门禁（AGENTS.md Change-Scoped Gate）**：改动 crates/ 下
  Rust 源 → 按 AGENTS.md 作用域跑 `cargo tt`（ui_gen + auto-man
  vue 作用域）；merge 前全量 `cargo t` 一次。
- **消费方验证（auto-edit 主检出，验证产物不入仓）**：worktree
  auto.exe → 冷删 `gen/front/vue` → `build --gen-only -r vue` →
  二跑幂等（diff 零漂移）→ `regen_vue.py --build`（构建绿）→
  vite dev + `auto run --server vm` 双终端 → 浏览器实测五点 +
  控制台零错 → vm 轨 `auto run -r vm` 轻探针（menubar 存在、树开
  文件、编辑器折叠——不跑全矩阵,vm 发射路径零改动仅对照）。
- **gallery 回归**：examples/ui-gallery（或现役 gallery 消费方）
  vue 构建绿（for 包装/menubar lowering 改动的既有消费方回归面）。

## 验收标准

- **AC-01（高度贯通）**：冷再生成后,auto-edit vue 轨打开文件时编辑器
  填满 tab 条以下至状态栏的全部剩余高度（不随内容行数塌缩）。
  验证：浏览器截图 + `App.vue` 中编辑器循环为 `<template v-for>`。
- **AC-02（深色行号）**：gutter 背景与编辑区同为深色、行号数字为浅色
  高对比；语法高亮五族在深色底可读。验证：浏览器截图。
- **AC-03（折叠可用）**：gutter 出现折叠箭头,点击可折/展 fn 块;菜单
  「视图→折叠切换」触发后隐藏行数语义与 vm 一致（ActFold 折第 2 行
  fn 块）。验证：浏览器点击 + 控制台无错。
- **AC-04（menubar 下拉）**：四个菜单可点开下拉,项含图标/标题/快捷键
  右对齐;点击菜单项触发对应动作（新建建 tab、切换 Console 开面板、
  关于弹窗）;「保存」在无 tab 时禁用。验证：浏览器实操 + DOM 快照。
- **AC-05（树开文件）**：点击树中文件名（含 README.md）打开新 tab 并
  显示内容;控制台无 vmOnly 抛错;console 面板（Ctrl+J）有 open 日志行。
  验证：浏览器实操 + 后端 access log 出现 read_text 200。
- **AC-06（主流程零抛错）**：新建 tab/切换 tab/折叠菜单三流程浏览器
  控制台无任何 `[auto-gen] VM-only` 抛错（SyncCursor/ActNew/ActFold
  走桥）。验证：浏览器控制台全程监听。
- **AC-07（生成确定性）**：冷删 gen → 一跑生成 → 记录快照 → 二跑生成
  diff 为空（幂等由生成器保证,无补件层）。验证：git status/diff 干净
  （gen 目录 gitignore 时用文件哈希对照）。
- **AC-08（vm 轨零回归）**：vm 轨启动、menubar 呈现、树开文件、折叠
  菜单行为与基线一致（发射面改动全在 vue 路径,此项为保险探针）。
  验证：`auto run -r vm` + MCP 快照/手工冒烟。
- **AC-09（本仓门禁）**：新增单测全绿 + `cargo tt` 作用域门禁绿 +
- **AC-10（选中色 r2）**：编辑器内选中文字的背景为半透明微亮层（非亮白块），与背景同令牌源。验证：浏览器选中文字截图。
  gallery vue 构建绿。验证：cargo 输出 + gallery 构建 exit 0。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T-01 for 透明包装**[✅ 已完成 85bd9068e：template v-for 单-if 体透明化,测试 test_plan677_single_if_loop_wraps_in_template_not_div+plan008 回归绿]（AC-01）：`crates/auto-lang/src/ui_gen/vue.rs`
  for-lowering——识别循环体为单 if 语句时发 `<template v-for :key>`。
  含既有 test_plan008 族平移 + 新快照测试。
- **T-02 CodeEditor 主题+折叠**[✅ 已完成 dbd939541：CSS 变量主题+foldGutter/foldKeymap+旧版自有脚手架覆写路径,测试两件绿]（AC-02/03 UI 面）：
  `crates/auto-man/src/vue.rs` generate_code_editor 模板——深色主题
  （CSS 变量 + HighlightStyle）+ foldGutter/foldKeymap。模板断言测试。
- **T-03 menubar 族 lowering**[✅ 已完成 25d3ff30c：registry 双形态别名+MenuBarCheckboxItem spec+generate_menubar_view_node 发射器,plan630 既有测试绿,失败集与 master 比对零回归]（AC-04）：`crates/auto-lang/src/ui_gen/
  vue.rs` 视图组件发射增 menubar* kind 映射,复用/镜像
  generate_actions_menubar_html 的项渲染；lucide 导入走既有 helper。
  快照测试。
- **T-04 natives 三层**[✅ 已完成 T-04 提交：R 层 file_basename/console_* 真实现（语义镜像 native.rs/ui_console.rs）,S 层桩不变,plan677_natives_r_tier_real_impls 绿]（AC-05）：`crates/auto-man/src/vue.rs`
  ensure_natives_layer——R 层（file_basename/console 三件真实现）、
  S 层保持;分层结构为 T-05 的 B 层预留挂点。单测。
- **T-05 editorBridge**[✅ 已完成 4a09aa7b9：bridge 模板+CodeEditor editorKey 注册+natives B 层 13 内建路由+:editor-key 动态绑定,测试四件绿,失败集与 master 比对零回归]（AC-03 菜单折叠/AC-06）：auto-man 新增
  editorBridge.ts 模板 + CodeEditor editorKey 注册 + ensure_natives_layer
  B 层路由；auto-lang ui_gen/vue.rs code_editor 发射透传 DSL key。
  单测（桥语义换算：0-based cursor/1-based fold/隐藏行计数）。
- **T-06 消费方冷验证**[✅ 已完成：冷删 gen→worktree exe 再生成→vue-tsc+vite build 绿→浏览器实测 README 打开/menubar 下拉(文件菜单展开+新建动作)/深色 gutter/编辑器贯通/控制台零错；vm 轨探针起窗正常]（AC-01..07）：worktree auto.exe 构建 →
  auto-edit 冷双跑再生成 + pnpm build → 双终端起服 → 浏览器五点
  实测（截图留档 docs/plans/evidence/）+ vm 轻探针（AC-08）。
- **T-07 门禁收口**[✅ 已完成 2a69327eb：cargo tt 4063/4063 全绿(金样 desktop_surface_asset 经 bless 同步 T-01 透明包装)；ui_gen::vue 与 auto-man 失败集与 master 逐一比对零回归；稳态二跑生成零漂移]（AC-09）：`cargo tt` 作用域 + 全量 cargo t 一跑 +
- **T-08 编辑器选中色 r2**[✅ 已完成 5d2e2a464：selectionBackground 前景令牌 0.14 透明层+focused 全链选择器压 baseTheme,模板测试绿,消费方浏览器实测 computed=rgba(248,250,252,0.14) 且截图微亮层确认]（AC-10）：CodeEditor 模板主题块补 `.cm-selectionBackground`（hsl(var(--foreground) / 0.14)——比背景 稍亮且透明,双主题自适应）+ 光标色；用户 r2 实测反馈：选中框太白。
  gallery vue 构建;核对 PLAN-672 在 auto-man/src/vue.rs 的在途冲突
  （先落者为准）。

## 复审记录

- 2026-09-21 drafting 完成（/auto-plan:new）。授权依据：用户同日指令
  「综合成一个 vue 版改进计划,一起改进；要改就改 Auto 代码或生成器」。
  outcome: pass（范围内无阻塞决策）→ next: work。
- 2026-09-21 stage: work | plan_id: PLAN-677 | plan_revision: 2 |
  outcome: pass | code_commit: 85bd9068e..2a69327eb（worktree
  plan-677-dev，基线 c8bdffa32）| task_ids: T-01..T-07 全闭环 |
  evidence: cargo tt 4063/4063 全绿；cargo t --no-fail-fast 失败集
  21 项与 master 逐一比对完全一致（全预存红，零新增）；消费方
  auto-edit 冷双跑再生成稳态零漂移 + vue-tsc/vite build 绿 +
  浏览器实测（README 树点击开 tab、menubar 四菜单下拉+动作、
  编辑器高度贯通、深色 gutter、控制台零 vmOnly 抛错）；vm 轨
  探针起窗正常 | blockers: 无 | next: review。
- 2026-09-21 r2（revision 2）：T-08 选中色闭环（用户反馈「选中框太白」
  → 前景令牌 0.14 透明层 + focused 全链选择器压 CM baseTheme）。
  outcome: pass | code_commit: 5d2e2a464 | blockers: 外部在途——
  PLAN-676 物化的 bps gallery-shell 依赖生成 GalleryShell.vue 引用
  未物化的 ui/popover,vue-tsc TS2307 挡 pnpm build(与 677 零关系:
  master 工具链同现;dev 运行不受影响,因应用源码无引用,模块不入
  图) | unblock: 676 落其物化修复或移除该未用依赖 | next: review。

- 2026-09-21 stage: review | plan_id: PLAN-677 | plan_revision: 2 |
  outcome: pass | reviewed_commit: 5d2e2a464 (worktree plan-677-dev) |
  base_commit: c8bdffa32 | dependency_revisions: auto-down fba6563
  (sibling detached) | spec_inputs:
  docs/specs/auto-lang/ui/design/app-generation.md @ 5d2e2a464 |
  acceptance_results: AC-01..AC-10 全 pass——AC-01 高度(template v-for
  +shell 630/720 截图)/AC-02 深色 gutter(截图)/AC-03 foldGutter marker
  渲染+foldKeymap+桥 fold_toggle(合成事件点击在 IAB 不可信=工具链限制,
  非阻塞登记)/AC-04 menubar 下拉+动作(快照 expanded+新建触发)/AC-05
  README 打开(后端 read_text 200+tab+内容)/AC-06 主流程零 vmOnly 抛错
  (errs=[])/AC-07 稳态二跑零漂移/AC-08 vm 轨探针起窗/AC-09 门禁
  (cargo tf 3694/3694 全绿+cargo tt 4063/4063 全绿+cargo t 失败集
  与 master 一致 21 预存)/AC-10 选中色 rgba(248,250,252,0.14) |
  findings: 无阻塞;独立性限制=实现会话自审,结论由重跑门禁
  (tf/tt/t 三档)与消费方产物重建,不依赖执行摘要 |
  evidence: docs/plans/evidence/677/*.png(终态/选中色/工具链回退恢复
  三张)+各提交内测试名 | next: merge。
- 追加修复（执行期实勘发现,均在授权范围内）：①`json.to_value(api
  调用)` 恒等映射错误（wire JSON 串未 parse,渲染树毒化=tab 不上屏
  真根因）→ T-05b；②gen-only 管线 shell 不同步致模板修复不可传播
  → regenerate_source_files 挂 ensure_code_editor_component。

## 待澄清事项

- （无阻塞项）cut/copy/paste 桥在受限浏览器环境的降级口径在 T-05
  实现时定（execCommand → clipboard API → 登记降级,三级回退）。
- PLAN-672 若先改 auto-man/src/vue.rs,T-07 按 rebase 协调,不影响
  本计划契约。
