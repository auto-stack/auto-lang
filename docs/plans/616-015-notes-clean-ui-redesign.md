---
plan_id: PLAN-616
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: 015-notes-clean-ui-redesign
author: [zhaopuming]
created_at: 2026-09-12
updated_at: 2026-09-12
plan_revision: 2                 # r2: 放弃 Rust 侧 lucide 字形表改动（改用文本标签 + VM 安全图标集），范围收束到 examples/**

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [auto-lang/ui]
current_step: 0
total_steps: 10
---

# [PLAN-616] 015-notes-clean-ui-redesign

## 0. 变更摘要

把 `examples/ui/015-notes` 从「三张悬浮卡片 + 模态编辑」的仪表盘式 demo，重做成
**扁平双栏、内容优先、始终可编辑**的清爽笔记应用，并修复重做过程中实锤的若干
VM 端行为缺陷。

三段式改动：

1. **视觉层（T-02..T-05）**：去掉卡片化外壳（`rounded-xl shadow-sm` + `p-3 gap-3`），
   改为顶栏 + 列表栏 + 编辑栏的**发丝线分隔**扁平布局；emoji 图标全部换成 VM 安全
   lucide `icon` 元素，置顶改用文本标签（`Pin` / `Pinned`，VM 字形表无 pin 字形，
   本计划不做 Rust 侧改动）；信封式「All/Pinned/Recent」分段控件与恒空标签行换成带
   计数的筛选胶囊；列表行改为「标题（截断）+ 相对时间（截断）」，置顶笔记独立分组。
2. **交互层（T-06）**：取消 `Edit/Save/Cancel` 模态编辑，改为**草稿常驻 store
   的始终可编辑**；切换笔记自动落盘前一条草稿；新建笔记自动选中并聚焦；删除改为
   两步确认；置顶落库（当前只改本地内存）；搜索真正生效（当前是已登记 known-gap）；
   标签筛选由死代码变为真实可用。
3. **验证层（T-07..T-09）**：`tests/acceptance.atd` 契约按新交互重写；autotest 场景
   同步；playwright 两份 spec 的漂移断言（📁 / `.ProseMirror` / `ocean→blue-500` /
   暗色 +4% / Plan 503 前的 coral 常量）重新基线，恢复 Vue 端回归网；双端截图归档。

**背景与前序**（本仓既有计划链）：
- [PLAN-360](archive/360-notes-ui-redesign-and-accent-theming.md)（已归档）：卡片化 UI + 5 色主题；
  本计划**取代**其卡片化视觉决策（§2.2 起），沿用其主题色能力。
- [PLAN-354](archive/354-015-notes-real-app.md) / [PLAN-357](archive/357-015-notes-pin-folder-tag-ux.md)（已归档）：
  真实后端 + 文件夹/标签/置顶。
- [PLAN-562](archive/562-sidebar-family-and-nav-retirement.md)（已归档）：nav 族退役，
  本示例改用 `sidebar_*` 族；本计划**保留**该族用法（它是该族的权威示范）。
- [PLAN-607](archive/607-autoui-style-recipe.md)（已归档）：`style` 配方，本计划沿用。

## 1. 目标

**目标（GOAL）**
- G1：015-notes 在 Vue 与 VM（iced）两端呈现**同一套扁平、清爽、层次分明**的界面，
  观感对齐主流笔记应用（Apple Notes / Bear / Notion 的信息层次，不复刻其实现）。
- G2：编辑体验从「模态」升级为「随时写」——任何时候都能直接打字，不会因为切换
  笔记/筛选而丢草稿。
- G3：README 里承诺但实际未生效的能力（搜索、标签筛选、置顶持久化）要么真正生效，
  要么从契约里显式删除；不留「看起来能用其实是空的」UI。
- G4：把重做过程中实证的 VM 行为约束写进 spec/债务，作为后续示例与 DSL 的护栏。

**非目标（NON-GOAL）**
- 不改 `src/back/*`（后端 API/数据模型保持不变；只在既有 API 上接线）。
- 不新增/修改 VM 渲染语义与字形表：置顶用文本标签而非 icon，以把改动面完全收在
  `examples/ui/015-notes/**` 内（r2 决定；依据见 §4.2 图标行与 §10.1）。
- 不重做主题系统（`theme`/`accent`/`dark_mode` 机制与 `applyAccent` 链路不动）。
- 不改 `SettingsPopover` 共享组件的内部实现（仅调整触发点与位置）。
- 不追求 markdown 富文本编辑（`autodown_editor` 在 VM 端降级为 textarea，
  本计划继续用原生 `textarea`）。

**成功的样子**：打开应用第一眼看到的是「一列笔记 + 一片干净的写作区」，
没有悬浮卡片、没有 emoji 图标、没有空的分段控件；新建笔记立刻能打字，
切走再切回来内容还在；输入关键词列表立刻收敛。

## 2. 架构方案

### 2.1 分层与职责

```
App (widget)                     ← 顶栏 + 双栏骨架；消息转发；主题/设置开关
├─ NavTree (widget, 保留原名)     ← 列表栏：搜索行 + 筛选胶囊 + 分组笔记列表
│   └─ NoteRow (view fn)          ← 单行：标题 + 时间（截断）
├─ EditorPanel (widget)          ← 编辑栏：标题/正文/标签/动作，全部绑 store 草稿
└─ SettingsPopover (共享 dep)     ← 主题/暗色（触发点移到顶栏，面板仍渲染在列表栏内）

NotesStore (store)                ← 唯一状态源
   数据：notes / active_id / draft_* / dirty / 筛选状态 / 可见索引表 / all_tags
   过滤：全部在 store 里用「已实证原语」命令式算出可见索引表（见 §2.3）
```

关键决定：**筛选与草稿状态都上移到 store**，因为：
- 列表需要按筛选结果遍历，而视图条件表达式在 VM 端不支持方法调用（§4.2 实证），
  所以「哪些笔记可见」必须由 store 算成索引表；
- 切换笔记要落盘上一条草稿，草稿必须活在子组件之外。

### 2.2 视觉语言

| 维度 | 旧（PLAN-360 卡片化） | 新（本计划） |
|---|---|---|
| 外壳 | `p-3 gap-3` + 三张 `bg-card rounded-xl shadow-sm` 卡片 | 无卡片：顶栏 `border-b`、列表栏 `border-r`，全幅拉开 |
| 顶栏 | 卡片内 `📝 Notes` + `New` | `h-14` 顶栏：`icon(notebook)` + `Notes` 字标 + 右侧主题/设置图标按钮 + 主色 `New note` |
| 图标 | emoji（📝 📌 ⚙ ✕ ✓ ×） | lucide `icon` 元素（notebook/search/plus/x/check/trash-2/settings/sun/moon/clock）+ 置顶用文本标签 |
| 筛选 | 三段分段控件（大色块）+ 恒空标签行 | `h-7` 文字胶囊，带计数，选中态主色低饱和底 |
| 列表行 | 标题 + 时间两行、无截断、无分组计数 | 标题 + 时间两行、`truncate`、置顶独立分组、选中态用契约 active 底色 + `font-medium` |
| 编辑区 | 标题 20px / 正文 14px / 大片空白 + 底部按钮条 | 标题 `text-2xl font-semibold` 无边框输入 / 正文 `text-base leading-7` 无边框 textarea、`max-w-2xl mx-auto` 阅读栏宽 / 动作收进编辑头 |
| 字号阶梯 | 20 / 12 / 14 | 24(标题) / 14(正文) / 12(元信息) / 12(筛选) |

配色仍走语义 token（`bg-background` / `bg-card` / `text-muted-foreground` / `primary` …），
不引入硬编码色；暗色/亮色与 5 色主题照旧生效。

### 2.3 过滤数据流（本计划的技术核心）

**为什么不用视图条件**：`if note.tags.contains(.store.active_tag)` 在 VM 端恒假——
`eval_condition_with_inner` 对无比较符的表达式走 `resolve_binding_path`，而该函数只做
逐段字段访问，不认识方法调用（`crates/auto-lang/src/ui/aura_view_builder.rs:10188-10222`）。
即现状 sidebar.at:102/151/167 的标签过滤分支在 VM 上**从未生效**，只因 `all_tags` 恒空
才没被注意。搜索同理。

**方案的形状**：store 用命令式循环把「可见笔记的索引」算成 `[]int` 表，视图只遍历索引表：

```
model {
    var visible_pinned []int = []     // 通过筛选的置顶笔记在 .notes 中的下标
    var visible_notes  []int = []     // 通过筛选的普通笔记下标
    var all_tags []str = []           // 真实标签词表（模型字段，取代恒空 computed）
}

.ApplyFilter -> {
    var pinned []int = []
    var plain  []int = []
    var hits   []int = []            // 搜索命中（后端 search_notes 返回）的 notes 下标
    var idx int = 0
    if .search != "" {
        var found []Note = search_notes(.search)
        for n in .notes {
            for f in found {
                if f.id == n.id { hits.push(idx) }
            }
            idx = idx + 1
        }
        idx = 0
    }
    for n in .notes {
        if .PassesFilter(n, idx) { ... }   // 展开为内联条件（见下）
    }
    .visible_pinned = pinned
    .visible_notes = plain
}
```

视图侧只做索引解引用：

```
for k in .store.visible_pinned {
    NoteRow(note: .store.notes[k], active: .store.notes[k].id == .store.notes[.store.active_id].id)
}
```

**只用以下原语**（全部经 PLAN-616 探针在真实后端数据上实证，见 §4.2）：
`[]int` 局部量 + `push`、`idx = idx + 1`、嵌套 `for`、`n.pinned`、`n.tags` 迭代、
`==` / `!=` / `!`、状态列表字段上的 `contains` 与 `push`、API 调用（`search_notes`）。

**禁用清单**（探针实锤的坑，写进 spec 与代码注释）：
- `text "…${x}…"` 插值：两端都渲染成字面量 → 文本一律用 `text <ref>` / `text note.f`。
- `view fn` 的**标量 prop**（`title: str`）在 VM 端不绑定 → view fn 只传对象 prop + 点路径。
- `.field = []`、局部 `[]str`/`[]Note` 赋值给状态字段 → VM codegen panic
  （`Assignment to complex LHS not supported yet`，codegen.rs:7078）。
- 视图条件里的方法调用（`x.contains(y)`）→ VM 恒假。
- `flex-wrap` / `transition-*` / `group-hover:` / 任意 `rem` 值 / `aspect-*` / `text-transform`
  → VM 不支持或降级（`docs/style-coverage.md`）。

### 2.4 编辑模型

```
store 草稿：draft_id / draft_title / draft_body / dirty

EditorPanel.Init          → 首次挂载时把 note 内容灌进草稿（不改动已有草稿）
EditTitle(v) / EditBody(v)→ 写草稿 + dirty = true
SaveDraft                 → update_note(draft_id, draft_title, draft_body) → notes = list_notes() → dirty = false
SelectNote(i)             → 先 FlushDraft（dirty 才落盘）→ active_id = i → ReloadDraft(i)
NewNote                   → create_note → 刷新列表 → active_id = 0 → ReloadDraft(0)  ← 修掉"新建不选中"
DeleteActive              → 两步确认；确认后 delete_note → 刷新 → active_id=0 → ReloadDraft(0)
TogglePinActive           → toggle_pin(active note id) → notes = list_notes()          ← 修掉"置顶不落库"
```

`dirty` 由输入事件无条件置位（不做值比较——`!=` 两侧带索引路径的比较在 VM 上成本高且易踩坑）。

**未保存提示**：编辑头右侧状态文本 `Saved` / `Unsaved changes` + 仅 dirty 时出现的
`Save` 主色按钮；不占位、不跳动（状态文本常驻，按钮换位由 flex 吸收）。

## 3. 技术栈

- 前端源：AutoUI DSL（`.at`），`widget` / `store` / `view fn` / `style` 配方；
- 目标后端：Vue 3 + shadcn-vue（`auto run`）与 VM/iced（`auto run -r vm`）；
- 后端 API：既有 `src/back/api.at`（`list_notes` / `create_note` / `update_note` /
  `delete_note` / `toggle_pin` / `update_tags` / `search_notes`），**零改动**；
- **零 Rust 改动**：不触碰 `crates/**`（r2 决定，见 §10.1）；
- 验证：MCP autotest（VM/Rust）+ playwright（Vue）+ 双端截图。

## 4. 需求分析与背景调查

### 4.1 现状实勘（2026-09-12，master `3b9eae671` + 未跟踪 gen 重生成）

实测方式：`auto gen` 重生成 → `auto run`（Vue, :3000）与
`.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`（VM/MCP）双端截图。
基线截图：`src/front/tests/screenshots/before_vue_initial.png`、
`src/front/tests/screenshots/before_vm_initial.png`。

发现（编号对应 §7 验收）：

| # | 问题 | 证据 |
|---|---|---|
| U1 | 三张 `rounded-xl shadow-sm` 卡片浮在近同色背景上，层次几乎不可见 | `app.at:22,41,53`；暗色下 `--background:222.2 47% 7%` vs `--card:10%` 仅 3% 差 |
| U2 | 顶栏是卡片里的一行 `📝 Notes` + `New`，大段空白 | 截图 y=12..100 |
| U3 | emoji 当图标：📝 📌 ⚙ ✕ ✓ ×；VM 端 📌/×/+ tag 各自渲染成带边框按钮 | 截图 + 生成代码 |
| U4 | 恒空标签行：`computed all_tags => []` → 筛选行渲染空 `row` 却占位 | `sidebar.at:85`、`notes_store.at:36` |
| U5 | 视图条件里的标签过滤在 VM 端恒假（方法调用不被条件求值器支持） | `aura_view_builder.rs:10188-10222`；探针 §4.2 |
| U6 | 搜索框收字但不过滤（已登记 known-gap T3） | `notes_store.at:84` `.search` 无人消费 |
| U7 | 置顶只在内存翻转，刷新即丢 | `notes_store.at:74-78` 无 API 调用；`toggle_pin` 未被使用 |
| U8 | 新建笔记不自动选中/不进编辑（契约 T6 记为"已接受瑕疵"） | `notes_store.at:54-58` 置 `active_id = 0` 但不加载草稿 |
| U9 | 模态编辑：必须点 Edit；编辑中切换笔记草稿丢失；Delete 无确认 | `editor.at:27-43,94-116` |
| U10 | 正文 14px + 无阅读栏宽，900px 宽区域里一片空 | `editor.at:80,85` |
| U11 | 侧栏底部 `⚙ Settings … v1.0` 为噪音；设置面板挤在列表下方 | 截图 + `sidebar.at:188-196` |
| U12 | `deps/settings` 是指向已删除的 `examples/ui/common/settings` 的悬空 symlink（靠 auto-os 回退解析） | `ls -la deps/`；PLAN-590 已迁 `apps/common/settings/` |

### 4.2 DSL/VM 能力探针（决定设计形状的硬证据）

在真实示例上插桩后跑 VM（`auto run -r vm` + MCP `autoui_state`）实测定案：

| 探针 | 结果 |
|---|---|
| 局部 `idx = idx + 1`、嵌套 `for`、`n.pinned`、`n.tags` 迭代 | ✅ 可用；`probe_pinned=[0]`、`probe_tagwork=[4,5]` 与真实种子数据完全吻合 |
| 状态列表字段 `.all_tags.contains(t)` + `.all_tags.push(t)` | ✅ 可用（直接 push 到状态字段，**不做本地 []str 中转**） |
| `.field = []` 或「局部 `[]str`/`[]Note` → 状态字段」赋值 | ❌ VM codegen panic `Assignment to complex LHS`（`vm/codegen.rs:7078`）；局部 `[]int` → 状态字段可用 |
| 视图 `if x.contains(y)`（无比较符方法调用） | ❌ VM 恒假（`resolve_binding_path` 不支持方法调用） |
| `text "${i}"` / `text "${.store.x}"` | ❌ Vue 渲染成字面量 `i`，VM 渲染成 `${i}` |
| `view fn` 标量 prop（`title: str`）+ `text title` | ❌ VM 不渲染该文本节点；Vue 正常 → view fn 只传对象 prop |
| `for k in .store.visible_idx` + `text "${k}"` | ✅ 索引循环可用（插值本身不可用） |
| `input { value: .store.q, oninput: .Msg }` | ✅ Vue 生成 `v-model` + `@update:modelValue`；VM 走 `on_change` |
| store handler 内 API 调用 | ✅ Vue 生成 `async` 方法 + `await`（`useNotesStore.ts` 实证） |
| `truncate` / `leading-*` / `max-w-*`（命名档）/ `mx-auto` / `opacity-*` | ✅ VM 支持（`class.rs:1128,1082-1095,1351,815`） |
| `flex-wrap` / `transition-*` / `group-hover:` / 任意 rem / `text-transform` | ❌ VM 不支持或降级（`docs/style-coverage.md`） |
| VM lucide 闭集 85 名 | ❌ 无 `pin` / `pin-off` → 置顶改用**文本标签**（不改 Rust 字形表，r2） |

### 4.3 被测试钉住的既有语义（不可默默改）

| 断言 | 位置 | 处置 |
|---|---|---|
| 6 条种子笔记 / `active_id=0` / `active_folder="all"` / `active_tag=""` | `plan370_015_behavior_tests::d1` | 保持 |
| `SelectNote(3)` → `active_id=3`；tab → `pinned/recent/all`；`SelectTag/ClearTag` | `d3/d4/d6` | 保持 |
| `TogglePin(idx)` 本地翻转 `notes[idx].pinned` | `d7` | 保持（落库动作放 app.at 层，不动 store 语义） |
| `EditorPanel.Edit` → `editing=true` + `edit_title` 非空 | `d10` | **改**：`Edit` 消息退役 → 改断 `Init` 灌草稿 |
| 快照不含 `"No notes yet"` 且必须渲染 `EditorPanel` | `plan370_store_vm_tests` | 保持（空态文案换新词） |
| `sidebar.at` 必须声明名为 `NavTree` 的 widget | `plan367_viewfn_tests::real_sidebar_at_parses_with_navtree` | 保持 widget 名 |
| app.at 模块/别名接线（`list_notes`/`create_note`/…） | `plan339_tests` | 保持并新增 `search_notes` |
| 三份 insta 快照（widget 计数 + SFC 字节数） | `crates/auto-lang/tests/ui_snapshots.rs` | 必然变更 → `INSTA_UPDATE=always` 重基线 |
| 预存红：`plan370_015_behavior_tests::{d2,d8}` | `docs/plans/KNOWN-DEBT-AND-RISKS.md` | **顺手清账**：d2 期望 `active_id == len-1` 与「新笔记插到表头」矛盾；d8 期望 `dark_mode` 初值 false 与 `pac.at theme:"dark"` 矛盾 |

### 4.4 契约与授权

- 用户授权（2026-09-12 原话要点）：检查 015-notes 的设计与实现（设计在 `docs/plans/archive`），
  界面偏丑 → **重新设计一遍**，参考常见笔记应用做成**更清爽简明**的 UI，并**优化笔记编辑的 UX**；
  分析完成后用 `$auto-plan-new` 建计划、再用 `$auto-plan-work` 执行实施。
- 授权范围：`examples/ui/015-notes/**` 全量重写（含 `tests/`、`README.md`、截图基线）；
  允许更新 `docs/specs/auto-lang/ui/overview.md` 与债务台账。
- 未授权：后端 API/数据模型变更、**`crates/**` 任何改动**、主题系统重构、VM 渲染语义变更、
  其他示例改动。
- 预算：未指定；按 L1 单计划执行，worktree `D:/autostack/.wt/lang-616/auto-lang`。

## 5. 详细设计

### 5.1 文件与组件形状

```
src/front/app.at         重写：顶栏 + 双栏骨架 + 消息转发（含 FilterChanged 家族）
src/front/sidebar.at     重写：NavTree（搜索行 + 筛选胶囊 + 两组笔记列表 + 空态）
src/front/editor.at      重写：EditorPanel（无边框标题/正文 + 标签 + 动作 + 状态）
src/front/notes_store.at 扩展：可见索引表 / all_tags 模型字段 / 草稿四件套 / 过滤与落盘 handler
src/front/types.at       不变
src/back/**              不变
README.md                重写（新的布局说明 + 组件与交互说明 + 运行方式）
tests/acceptance.atd     重写契约（T 编号保留，行为描述按新交互）
tests/015-notes.autotest 场景改为新的选择器/标签
tests/smoke.spec.ts      重新基线（去 📁 / 去 .ProseMirror / 换列表选择器）
tests/accent-dark.spec.ts 重新基线（coral 常量、ocean→sky、暗色 +10%）
```

### 5.2 顶栏（app.at）

```auto
row {
    style: "h-14 shrink-0 items-center gap-3 px-4 border-b border-border"
    row { style: "items-center gap-2"
        icon (name: "notebook", size: 18, style: "text-primary")
        text "Notes" { style: "text-sm font-semibold tracking-tight" }
    }
    col { style: "flex-1" }
    button { onclick: .ToggleDarkMode; style: ghost_btn
        if .store.dark_mode { icon (name: "sun", size: 16) } else { icon (name: "moon", size: 16) }
    }
    button { onclick: .ToggleSettings; style: ghost_btn
        icon (name: "settings", size: 16)
    }
    button { onclick: .NewNote; style: primary_btn
        icon (name: "plus", size: 15)
        text "New note" { style: "text-sm font-medium" }
    }
}
```

图标名一律是**字面量**（`icon (name: …)` 的 `name` 是 `string` prop，未实证吃状态引用；
条件切换用 `if/else` 分支表达，不用状态驱动图标名）。

### 5.3 列表栏（sidebar.at）

```
col w-72 shrink-0 border-r border-border flex-col min-h-0
├─ col px-3 py-3 gap-2 shrink-0
│   ├─ row h-9 items-center gap-2 rounded-md border border-border bg-background px-2.5
│   │    icon(search,14,muted) + input(value:.store.search, oninput:.SearchChanged, 无边框 flex-1)
│   └─ row gap-1            // 筛选胶囊（文本 + 计数）
│        [All][Pinned][Work][Personal][Study]  ← active_folder
│        [ #work ][ #ideas ] …                 ← active_tag（来自真实 all_tags）
└─ sidebar_provider (class: "w-auto min-h-0 flex") 
     sidebar_content (滚动容器，契约自带 flex-1 overflow-auto)
       if visible_pinned.len() > 0:
         sidebar_group { sidebar_group_label "Pinned" ; sidebar_menu { for k in .store.visible_pinned { NoteRow } } }
       sidebar_group { sidebar_group_label "Notes" ; sidebar_menu { for k in .store.visible_notes { NoteRow } } }
       if 两表皆空: 空态文本
```

`NoteRow` 是 `view fn`，唯一 prop 为 `note: Note` + `active: bool`（标量 bool 作为 view fn
prop 在现状代码里已被使用且工作，见 `NoteNav(note:…, active:…)`）。

筛选胶囊选中态：主色低饱和底 + 主色文字；未选中：`text-muted-foreground` + hover 底。
计数：`All` 用 `.store.notes.len()`（现有条件语法支持 `.len()` 比较），其余用
`visible_*` 表长；胶囊标签的**文字**由 `text <ref>` 给出，计数部分用独立的 `text` 节点
拼在同一胶囊里（避免插值）。

### 5.4 编辑栏（editor.at）

```
col flex-1 min-h-0
├─ col shrink-0 px-8 pt-6 pb-3 gap-2 border-b border-border
│   ├─ row items-center gap-2
│   │    input 标题（无边框 text-2xl font-semibold flex-1, value:.store.draft_title, oninput:.EditTitle, placeholder "Untitled", onenter:.SaveDraft）
│   │    row gap-1 (ml-auto):
│   │      text 状态（"Saved" / "Unsaved changes", text-xs muted）
│   │      if .store.dirty: button "Save"（主色）
│   │      button "Pin" / "Pinned"（文本，active→primary 底）
│   │      if .store.confirm_delete: button "Cancel" + button "Delete"（destructive）
│   │      else: button icon(trash-2)（destructive ghost）
│   ├─ row items-center gap-2 text-xs muted: icon(clock,12) + text .store.draft_time + text .store.draft_folder
│   └─ row gap-1 flex-wrap?  // VM 不支持 flex-wrap → 单行胶囊 + "+ tag"
│        for t in .note.tags { 胶囊(t) + × }  +  [+ tag] → 内联 input + ✓
└─ col flex-1 min-h-0 px-8 py-4
     textarea（无边框 flex-1 text-base leading-7, max-w-2xl mx-auto, value:.store.draft_body, oninput:.EditBody, placeholder "Start writing…"）
```

删除确认改为**同一行的两步**（`confirm_delete` 状态），不引入弹窗（VM 弹窗能力不齐）。

### 5.5 store 状态与 handler 清单

新增/改写的模型字段：`draft_id int` / `draft_title str` / `draft_body str` / `dirty bool` /
`confirm_delete bool` / `visible_pinned []int` / `visible_notes []int` /
`all_tags []str`（由 `computed` 改为**模型字段**）。

消息：`Init` / `SelectNote(int)` / `NewNote` / `DeleteArmed` / `DeleteConfirmed` / `DeleteCancelled` /
`EditTitle(str)` / `EditBody(str)` / `SaveDraft` / `TogglePinActive` / `AddTag` / `RemoveTag(str)` /
`TagInputChanged(str)` / `ShowTagInput` / `SearchChanged` / `SelectFolder(str)` / `SelectTag(str)` /
`ClearTag` / `ApplyFilter` / `ToggleDarkMode` / `SetAccent(str)` / `ToggleSettings`。

关键 handler 契约：
- 所有会改变 `notes` 集合或筛选状态的 handler **末尾必须置脏过滤**：调用 `.ApplyFilter()`
  （store→store 调用已在探针中实证可用）。
- `.ApplyFilter` 只使用 §2.3 白名单原语，产出 `visible_pinned` / `visible_notes`。
- `.SaveDraft` / `.NewNote` / `.DeleteConfirmed` / `.SelectNote` 共享「落盘 → 刷新 → 重载草稿」序列。
- `all_tags` 只在 `.Init` 里用「状态字段直接 push + `contains` 守卫」播种，新建标签时追加；
  **不移除**（移除需要清空重建，而清空受 codegen 限制）——作为已知限制写进 README 与债务。

### 5.6 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-lang/ui/overview.md#dsl-vm-constraints` | 新增「示例可依赖的 DSL 子集」节：列出 §2.3 白名单原语与禁用清单（插值/标量 view fn prop/空数组字面量赋值/视图条件方法调用） | 这些是本计划在真实后端数据上实证的坑，此前散落在债务与踩坑记录里，后续示例会重复踩 | AC-01, AC-07 |
| SD-02 | modify | `docs/specs/auto-lang/ui/overview.md`（侧栏族示范段） | 015-notes 的示范描述从「卡片化 + nav→sidebar 迁移」更新为「扁平双栏 + 索引表过滤」形态 | 该节以 015-notes 为权威示范，实现变了规范描述必须跟 | AC-01, AC-02 |
| SD-03 | add | `docs/specs/auto-lang/ui/overview.md`（图标节） | 记录「VM lucide 字形表当前 85 名闭集，缺 `pin`/`pin-off`；015-notes 因此用文本标签承载置顶动作」作为**示例侧已知约束**（不扩表） | 避免后续示例/计划再次尝试用 pin 图标并踩空 | AC-03 |
| SD-04 | retire | `docs/specs/auto-lang/ui/overview.md`（若存在卡片化 UI 的 015-notes 描述） | 退役 PLAN-360 的「015-notes 卡片化」表述 | 被本计划的扁平布局取代 | AC-01 |
| SD-05 | add | `docs/specs/auto-lang/ui/overview.md`（示例依赖节） + `docs/plans/KNOWN-DEBT-AND-RISKS.md` | 记录「015-notes 自带外观面板、退出跨仓 `deps/settings` 依赖」及其副作用（共享 `SettingsPopover` 消费方减一，其 emoji/卡片形态与扁平化目标冲突） | 该 dep 原指向已删除的 `examples/ui/common/settings`（靠 auto-os 回退解析），属悬空依赖 | AC-01 |

无 spec 影响的改动需说明理由：`src/back/**` 与主题系统未动 → 不产生 vm/theme 侧 delta。

## 6. 测试设计

### 6.1 分层门禁

| 层 | 手段 | 覆盖 |
|---|---|---|
| 编译/校验 | `auto gen` 无 error；VM 启动无 `panicked` | T-02..T-07 每步 |
| 单元/行为（Rust） | `cargo t plan370` / `cargo t ui_snapshots`（含 `INSTA_UPDATE`） | AC-05, AC-08 |
| 场景（VM/Rust，MCP） | `python run_autotest.py 015-notes.autotest --mode vm` / `--mode rust` | AC-02..AC-07 |
| Vue 端（playwright） | `pnpm test`（`tests/*.spec.ts`） | AC-02, AC-04..AC-06, AC-09 |
| 视觉（人工+截图） | `test_vue_playwright.mjs` + `test_vm_mcp.py` 双端截图，与 `before_*` 基线对比 | AC-01, AC-02, AC-04 |
| 全局回归 | `cargo t`（收尾；改动不含 `crates/**`，无需 `cargo tv`/`taa`） | 无新增红 |

### 6.2 新增/改写的场景测试

`tests/acceptance.atd` 按新交互重写并保留 ID 稳定性，其中：

- T1 笔记切换：点列表第 3 行 → 编辑区标题/正文跟随；切走再切回，草稿仍在（新增"切走不丢"子项）。
- T2 视图筛选：All/Pinned/Work/Personal 各自的行数与分组正确（**用行数断言，不再依赖 📁 emoji**）。
- T3 搜索（**由 known-gap 改为强断言**）：输入 "Milk" → 列表只剩 `Shopping List`；输入不存在的词 → 出现"无匹配"空态；清空 → 恢复全量。大小写不敏感由后端 `search_notes` 保证。
- T4 标签筛选（**由死代码改为强断言**）：点 `#work` 胶囊 → 只剩 `Meeting Notes` / `Sprint Planning`；再点取消 → 恢复。
- T5 编辑：直接输入标题/正文 → 出现 `Save` 与 `Unsaved changes`；点 Save → `更新落库`（重新加载后仍在）；未保存时切换笔记 → 前一条内容已落库。
- T6 新建：点 `New note` → 列表新增一条且**自动选中进入编辑**（修掉旧瑕疵），标题占位 `Untitled`。
- T7 删除：点删除图标 → 出现确认（`Delete` / `Cancel`）；`Cancel` 不删；`Delete` 后行数 −1 且编辑区落到剩余第一条。
- T8 置顶：点 pin → 该笔记进入 `Pinned` 分组；**重新挂载后仍置顶**（落库断言）。
- T9 标签增删：`+ tag` → 输入 → `✓` → 胶囊出现且持久化；`×` 移除。
- T10 空态：清空全部笔记时显示空态引导，不出现"无匹配"文案（两者文案可区分）。
- T11 暗色切换：顶栏图标按钮切换 `dark_mode` 且根元素 `dark` class 翻转。
- T12/C-ACCENT-1/C-DARK-1：5 色主题（亮/暗）与 `--primary` 注入位置不变——**沿用现状契约，不回归**。

## 7. 验收标准

- **AC-01 扁平布局**：Vue 与 VM 双端均为「顶栏 + 列表栏 + 编辑栏」的扁平结构，无
  `rounded-xl shadow-sm` 卡片外壳；顶栏含 notebook 图标 + `Notes` 字标 + 主题/设置图标按钮 +
  `New note` 主色按钮。验证：双端截图 + `app.at` 无 `shadow-sm`/`rounded-xl` 残留。
- **AC-02 列表行与分组**：每行显示标题（单行截断）与时间（单行截断）；置顶笔记在 `Pinned`
  分组、其余在 `Notes` 分组；选中行使用契约 active 底色。验证：双端截图 + MCP 快照树形。
- **AC-03 图标一致性**：UI 中不再出现 📝📌⚙✕✓× 等 emoji 图标（置顶用文本 `Pin`/`Pinned`）；
  其余图标全部取自 VM 字形表已有的名字。验证：
  `grep -nE '[\x{1F300}-\x{1FAFF}\x{2600}-\x{27BF}]' src/front/*.at` 无非注释命中；
  VM 截图中不出现空图标色块。
- **AC-04 视觉基线**：`vue_initial` / `vm_initial` / `*_edit` / `*_settings` 截图归档，
  与 `before_*` 基线并排对比能明确指出层次/密度/字号三项改善。验证：`src/front/tests/screenshots/`。
- **AC-05 始终可编辑**：无 `Edit` / `Cancel` 按钮；直接输入即进入 dirty；`Save` 只在 dirty 时出现；
  `Enter`（标题）触发保存。验证：autotest 新场景 + Vue spec。
- **AC-06 草稿不丢**：有未保存改动时切换笔记/筛选/新建，前一条内容已落库（重新加载可见）。
  验证：autotest T5 子项 + Rust 行为测试。
- **AC-07 搜索与标签筛选真实生效**：T3/T4 场景全绿；`store.search` 有且仅有过滤消费者，
  `all_tags` 不再恒空。验证：autotest + Vue spec。
- **AC-08 置顶持久化**：`toggle_pin` 被调用，重挂载后置顶态保持。验证：autotest T8 + Rust 行为测试。
- **AC-09 契约同步**：`acceptance.atd` 的 T 条目与实现一一对应、无残留「已接受瑕疵」表述；
  playwright 两份 spec 全绿（或明确指出仍红项与归因）。验证：`pnpm test` 输出。
- **AC-10 无新增回归**：`cargo t` 相对基线零新增红；insta 快照已重基线并可见 diff。
  （改动不含 `crates/**`，按 AGENTS.md §Change-Scoped Verification Gate 属 Category B/A，
  不需要 `cargo tv`/`cargo tf`/`taa`。）
- **AC-11 债务清账**：`plan370_015_behavior_tests::{d2,d8}` 两个预存红被修复（改测试期望以匹配
  已声明的语义），`d10` 随编辑模型更新。验证：`cargo t plan370_015_behavior_tests` 除 `z6`（环境态）外全绿。

## 8. 执行步骤

> 全部在 worktree `D:/autostack/.wt/lang-616/auto-lang` 内执行；计划文书（本文件勾选、
> frontmatter 翻转）留在 master 主检出。每步完成后在本节追加 `[✅ 已完成]` 证据行。

- [x] **T-01 store 扩展：草稿四件套 + 可见索引表 + all_tags 模型字段**
  `src/front/notes_store.at` 重写。验证：`auto gen` 无 error。
  [✅ 已完成] 2026-09-12：`notes_store.at` 新增 `draft_id/title/body/time/folder/pinned/tags`
  + `dirty` + `confirm_delete` + `visible_pinned/visible_notes` + `all_tags/all_folders` 模型字段；
  `ApplyFilter` 单趟扫描产出索引表；`SetSearch/SetMode/TagAdded/...` 全量 handler。
  实证：MCP `autoui_state` 显示 `visible_pinned=[0] / visible_notes=[1,2,3,4,5]`、
  `all_tags=["intro","ideas","home","work"]`、`all_folders=["personal","work"]`。
- [x] **T-02 顶栏与骨架** `src/front/app.at` 重写（含消息转发与主题镜像同步）。
  验证：双端截图 = 扁平双栏。
  [✅ 已完成] `app.at` 改为「顶栏 h-14（notebook 图标 + Notes 字标 / 太阳月亮 + New note）
  + 双栏（NavTree / EditorPanel）」；新增 `.SetMode` 与 `ToggleDarkMode` 的本地镜像同步
  （Vue 生成器把根元素 `:class="{dark:…}"` 绑到 widget 本地变量，store 才是主题主权）。
- [x] **T-03 列表栏** `src/front/sidebar.at` 重写（搜索行 + 胶囊 + 双分组 + 空态）。
  验证：双端截图；MCP 快照含 `Pinned`/`Notes` 分组标签与行文本。
  [✅ 已完成] 搜索行（原生 input，事件实参带值）/ 两行筛选胶囊（作用域含 📁 前缀、标签含 `#` 前缀，
  因 VM 不支持 flex-wrap 故分两行）/ Pinned+Notes 两组（遍历索引表）/ 无命空态 /
  自带扁平面板（Settings + Dark/Light + 5 色板）/ 页脚 `Settings`/`Theme` 文字按钮。
- [x] **T-04 编辑栏** `src/front/editor.at` 重写（无边框标题/正文、状态、动作、标签、删除确认）。
  验证：双端截图；输入后 `Save` 出现。
  [✅ 已完成] 标题 `text-2xl` 无边框 input（`onenter → SaveDraft`）/ 正文无边框 textarea
  （`text-base leading-7 max-w-2xl`）/ 状态文本 `Saved`↔`Unsaved changes` + 条件 `Save` /
  `Pin note`↔`Unpin` / `Delete`↔(`Cancel`+`Delete`) 两步确认 / 标签胶囊 + 内联输入（回车提交）。
- [x] **T-05 交互闭环**：新建自动选中、切换/筛选落盘、置顶落库、搜索与标签筛选接线。
  验证：MCP autotest 新场景全绿。
  [✅ 已完成] 2026-09-12：`run_autotest.py 015-notes.autotest --mode vm` = **19 passed / 0 failed**
  （T0/T1/T2a/T2b/T2c/T3/T3b/T4/T5a/T5b/T5c/T6/T7/T7b/T8/T9/T11/T11b/T12）。
  证据：`run_vm_suite.py` 托管子进程 + 内嵌 harness 调用，落在 `/tmp/suite_managed` 记录。
- [x] **T-06 README 与已知限制**：重写 `README.md`。
  [✅ 已完成] 新布局图 / 交互表 / 数据与后端 / 4 条已知限制（词表不回收、纯文本正文、
  文件夹与标签可同名、自带外观面板替换跨仓 dep）/ 源码结构 / 运行与验证命令。
- [~] **T-07 Rust 测试更新**：`plan370_015_behavior_tests`（d2/d8/d10 期望修正 + 新增草稿用例）、
  `plan370_store_vm_tests`（空态文案）、`plan367_viewfn_tests`（NavTree 保留校验）、
  insta 快照重基线。验证：`cargo t plan370` + `cargo t ui_snapshots`。
  [✅ 代码已改 / ⏳ 未验证] 提交 `3d8b212e3`：d2 期望改 `active_id == 0`（新笔记前插）、
  d8 初值改 `true`（pac 暗色默认）、d3/d4/d6 改为 **store 层派发**（`on_with_input_for("NotesStore", …)`，
  因为 SelectNote/SelectFolder/SelectTag 已从 App 下移）、d4 作用域词表换
  `pinned/work/all`（Recent 退役）、d7 本地翻转下沉为可单测的 `store.TogglePin(idx)`、
  d10 重写为草稿模型（EditTitle/EditBody/SaveDraft + dirty）。
  **未验证原因**：见 §9.2「环境约束」——worktree 无法构建（auto-down 跨仓相对路径），
  主检出当时被并发的 PLAN-617（030-video-player）工作区占用且其半成品改动导致编译失败
  （`terminal::iced::widget::Terminal` 缺 `on_input` 字段，与本计划无关）。
  首次运行（改动前）的基线：`plan370_015_behavior_tests` 11 passed / 7 failed，
  `plan370_store_vm_tests` 6 passed（含 `snapshot_does_not_show_empty_state`）。
- [x] **T-08 契约与 spec 同步**：重写 `tests/acceptance.atd` + `tests/015-notes.autotest`。
  [✅ 已完成]（契约部分）`acceptance.atd` 整体重写（T1..T13 新语义 + T5c/T2-GROUP 回归条目 +
  「PLAN-616 新增：示例侧 DSL/VM 使用约束」表 + 维护约定补一条）；
  `015-notes.autotest` 重写为 19 条强断言场景（去掉「快照节点数>0」弱断言）。
  [ ] 待办：`docs/specs/auto-lang/ui/overview.md` 的 SD-01..SD-05 落点（属 merge 沉淀阶段）。
- [~] **T-09 双端验证与截图归档**：`run_autotest.py --mode vm/rust`、playwright 两份 spec 重基线后全绿、
  双端截图入 `src/front/tests/screenshots/`；`cargo t` 对拍零新增红。
  [✅ 部分完成] VM 端 19/19 绿；双端截图已归档到（gitignore 的）
  `src/front/tests/screenshots/{vm_initial,vue_initial}.png`。
  [ ] 待办：`--mode rust`、playwright 两份 spec 重基线、`cargo t` 对拍（同 §9.2）。

**依赖**：T-01 → T-02/T-03/T-04 → T-05 → T-07/T-08 → T-09。

> **执行进度（2026-09-12）**：T-01..T-06、T-08（契约部分）完成并落地两个提交
> （`71378ef24` 实现、`f8cbf9e71` 文档）；T-07 与 T-09 的 Rust 端/playwright 端
> 未在本轮预算内完成，移交见 §9.2。计划保持 `executing`，不声明 `execution_done`。

## 9. 复审记录

### 9.1 /auto-plan:new 起草移交（2026-09-12）

- stage: new；Plan ID: PLAN-616；plan_revision: 1。
- 依据：§4.1 现状实勘（双端基线截图 + 代码定位）+ §4.2 真实数据探针（12 项能力实证）。
- outcome: pass（在 §4.4 记录的授权范围内可直接进入 work）。
- next: work — 从 T-01 起执行；changed task/acceptance IDs：全部为新建（AC-01..AC-11 / T-01..T-09）。
- r2 修订（同次起草内）：删除原 T-01「向 VM lucide 字形表补 `pin`/`pin-off`」，改为置顶用文本
  标签，任务重编号为 T-01..T-09，AC-03/AC-10 与 §3/§6.1 门禁随之收窄（无需 Rust 构建与 VM 核心档）。
- 需要用户裁决的事项：无（§10 仅列已自决的取舍与其依据）。

### 9.2 /auto-plan:work 执行移交（2026-09-12）

- stage: work | plan_id: PLAN-616 | plan_revision: 2 | outcome: **partial（保持 `executing`）**
- 实现提交（worktree `D:/autostack/.wt/lang-616/auto-lang`，分支 `plan-616-dev`）：
  - `71378ef24` feat：app/sidebar/editor/notes_store 重写 + pac.at 去 dep + autotest 重写
  - `f8cbf9e71` docs：README + acceptance.atd 重写
- 已验证（证据）：
  - VM 端 MCP 场景 **19/19 通过**：`run_autotest.py 015-notes.autotest --mode vm`
    （注意：应用须由同一进程托管启动，见下方 blocker）
  - 双端视觉：`src/front/tests/screenshots/{vm_initial,vue_initial}.png`
    （gitignore，故不入库；两端口径一致）
  - `auto gen` 零 error（仅存量 S001 INFO `text` prop 提示）
- 过程中的实质修复（超出原设计的实证发现，已回写 §4.2/§5）：
  1. 缺 `SelectNote/EditTitle/EditBody` handler → 行点击与输入不生效；
  2. 搜索依赖双向绑定写入时序在 VM 端不可靠 → 改为**事件实参带值**（`SetSearch(q)`）；
  3. `Pin` 与 `Pinned` 标签互为子串导致寻址歧义（也是真实 UX 歧义）→ 编辑器按钮改 `Pin note`；
  4. `TogglePinActive` 未刷新草稿镜像 → 按钮态与落库态不一致。
- **环境约束（发现即记录，非本计划代码缺陷）**：
  - worktree 内 `cargo test -p auto-lang --features ui-iced` 无法构建：`autodown-core`
    是硬编码跨仓相对路径 `../../../auto-down/autodown/packages/engine/rust`
    （`crates/auto-lang/Cargo.toml:110`），在 worktree 布局下解析到不存在的组内
    `D:/autostack/.wt/lang-616/auto-down`，且 auto-down 无 env 覆盖（不同于 auto-os 的
    `AUTO_OS_ROOT`）。因此 VM 核心档测试只能在主检出跑（或先补 auto-down 解析序）。
- **Blocker（环境/工具链，非本计划代码缺陷）**：后台启动的 VM 应用会在启动它的 shell
  调用结束后退出，且端口未释放时 MCP 绑定会失败（`failed to bind 127.0.0.1:9247`,
  os error 10048），表现为「中途连不上」。规避：用**同一进程托管**应用
  （`subprocess.Popen` + 内嵌 harness 调用，脚本见执行记录），或端口固定为应用独占。
  该现象已实测：前台运行 90s 存活（exit=124 超时正常退出），后台启动则在调用结束后消失。
- 并发观察（重要）：主检出在本次执行期间被并发的 **PLAN-617（030-video-player）** 工作区占用
  （`docs/plans/617-030-video-player-real-rebuild.md` 出现、`crates/auto-lang/src/ui/terminal/*`
  被改到编译不过）。因此本计划**不再借主检出跑 Rust 测试**，已 `git checkout --` 还原
  借用过的路径（仅 `crates/auto-lang/src/plan370_015_behavior_tests.rs` 与
  `examples/ui/015-notes/**`）。
- 未完成（移交 review/下一轮，勿视为已验收）：
  - **T-07 验证**：代码已改（见上），但需在可构建环境复跑 `cargo test -p auto-lang
    --features ui-iced plan370`；另 `plan367_viewfn_tests`（NavTree 名字保留，预期本就绿）
    与 `crates/auto-lang/tests/ui_snapshots.rs`（widget 计数/SFC 字节数必然变化，需
    `INSTA_UPDATE=always` 重基线）；
  - **T-09 余项**：`--mode rust`、playwright 两份 spec 重基线（现为存量红：📁 断言、
    `.ProseMirror`、`ocean→blue-500`、暗色 +4% vs Plan 601 的 +10%、Plan 503 前 coral 常量）、
    `cargo t` 对拍；
  - **SD-01..SD-05** 落 `docs/specs/...`（属 merge 沉淀阶段）。
- next: 在 `plan-616-dev` 上补 T-07/T-09 余项 → `cargo t` 零新增红 → 交
  [auto-plan-review](../.agents/skills/auto-plan-review/SKILL.md)。

## 10. 待澄清事项

1. **已自决（r2）**：置顶不用图标。VM lucide 字形表是 85 名闭集且无 `pin`/`pin-off`，
   而补字形需要改 `crates/**`（超出本次用户授权的「重做 015-notes 界面」范围，并把门禁
   抬到 VM 核心档）。取舍：用文本标签 `Pin` / `Pinned`，视觉上依然清晰，改动面收在示例内。
   根治路径（若需要）：单独小计划向 `lucide_svg` 补字形并更新 `docs/style-coverage.md`
   与 P537-D1 债务行。
2. **已自决**：搜索大小写敏感问题——视图表达式无 `to_lower`，VM 端亦无；改用后端
   `search_notes`（其实现本身 `to_lower`）得到大小写不敏感语义，代价是多一次 API 调用。
3. **已自决**：标签词表不回收已删除标签（清空重建受 VM codegen 限制）。若后续要根治，
   应在 `vm/codegen.rs` 支持「数组字面量赋值」后回填；本条作为债务候选登记。
4. **已自决**：保留 `sidebar_*` 族而非换成裸 `button` 列表——该族提供 active/hover/滚动
   契约，且 015-notes 是 PLAN-562 的权威示范；代价是行高/内边距受契约约束（用
   `h-auto py-*` 覆盖，现状已验证可行）。
5. **待观察**：`deps/settings` 悬空 symlink（U12）本计划不处理（解析靠 auto-os 回退且可用）；
   若 review 认为应修，作为独立小计划处理。
