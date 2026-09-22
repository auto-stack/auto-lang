---
plan_id: PLAN-684
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: 018-book-reader-sidebar-ux
author: [agent]
created_at: 2026-09-22
updated_at: 2026-09-22
plan_revision: 8
current_step: 15
total_steps: 15

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [widgets/viewport-boundary.md §3 demo 壳层 chrome 禁令, auto-lang/ui/overview.md 示例壳层与后端读盘契约节]
touched_goals: [GOAL-007, GOAL-010]

affects: [auto-lang/ui, widgets/viewport-boundary]
---

# [PLAN-684] gallery vue 轨示例修复与 UI/UX 重设计（018 + 022）

> 注：计划编号勘定——`new-plan.sh` 首配 682 与在途 `682-mcp-snapshot-determinism-and-vue-generator-fixes`
> （worktree `lang-682`，status reviewed/next merge）及主检出 `683-rq-remote-renderer` 冲突；
> 本计划改号 **684**，worktree = `D:/autostack/.wt/lang-684/auto-lang`，branch `plan-684-dev`。
>
> **revision 2（2026-09-22 用户走查追加）**：022-kanban 主题配色（深/浅均不适）+
> 应用框布局（未占满视口、无独立 app 框）并入本计划滚动修复。018 部分见既有
> T-01..T-07 / AC-01..08。
>
> **revision 3（2026-09-22 用户走查追加）**：024-charts 右侧图表区空白
> （裸名 `line-chart` 等发射 `<div :data=…>` 空壳，P601-T11 / P555-D4 同源）。
> 根因 = vue `tag_is_builtin` 把 schema `package_origin` 元素当 builtin，
> 挡住 known_sub_widgets 折叠桥接。修生成器 + 024 props 契约形态。
>
> **revision 4（2026-09-22 用户走查追加）**：024 Play 流式在 gallery 内嵌无效。
> 根因 = Vue 臂组件 Init 只在 onMounted 跑一次，父级 data 重播种后几何不重算
> （ADR-19 仅 VM 轨每帧补发 Init）。修 = 带 props 组件补 `watch(props)` 重放 Init。

> **revision 5（2026-09-22 用户走查追加）**：025-sys-monitor 五项——①窗口未适应
> ②选中横幅占核心（改右键结束）③列表松散/浮点长尾 ④三页美化 ⑤去用户页。
> 列拖拽记债（本批未做）。

> **revision 5（2026-09-22）**：025-sys-monitor——窗口适配、去选中横幅（右键结束）、
> 指标 1 位小数、紧凑列表、去用户页。列拖拽记债。提交 `5ea9d42a9`。

> **revision 6（2026-09-22）**：027-file-manager——①UI 未占满视口 ②Vue 轨无 FS
> （PLAN-680 Q3）目录空。方案：`h-full min-h-screen` 布局；Vue 轨演示虚拟目录，
> VM 轨真 `fs.read_dir`。

> **revision 7（2026-09-22）**：027 真·本地目录（PLAN-680 Q3 L2，对齐 020）——
> back.api `list_dir`；Vue 轨改走 HTTP 后端读盘。演示目录仅后备。

## 0. 变更摘要

**批次 A（018-book-reader，rev1）**：修复 gallery 内嵌时 Sidebar 溢出视口卡
（根因 = `fixed inset-y-0`），并按 Apple Books / Kindle 重做 Library / Detail /
Reading / Settings。

**批次 B（022-kanban，rev2）**：列底色硬编码 `bg-gray-100` / `bg-orange-50` /
`bg-green-50` 在深色模式下与前景色冲突（浅色列底 + 浅色标题 = 不可读），浅色模式
下卡片/列对比也不统一；布局 `items-start` + 固定 `w-80` 导致看板不占满视口、
既非 `window:"fit"` 内容窗（calculator）也非 `h-screen` 整页（music-player）。
重做为**独立 app 框**（`window:"1120x740"` + 全高看板）+ 语义 token 配色
（Linear/Trello 式列卡，状态用小色点标识）。

## 1. 目标

### Goals
- **G1（缺陷）**：018-book-reader 在 gallery `AppViewport` 嵌入态（1024×720 / 768×1024 /
  375px）内，侧栏与主内容均限制在视口卡内部；侧栏高度 = 容器高度，不再向上/向下穿出。
- **G2（UI/UX）**：四页信息架构与视觉对齐常见阅读 App：
  - Library：书架封面网格 + 「Continue Reading」推荐条 + 新建书对话框
  - Book Detail：封面/元信息 + 可滚动章节目录（当前章高亮）
  - Reading：沉浸式正文（排版受设置控制）+ TOC 抽屉 + 底部进度/翻章
  - Settings：字号 / 行距 / 主题说明，且**字号真实作用于阅读页**
- **G3（双轨）**：Vue 轨（`auto run` / gallery 内嵌）与 VM 轨（`auto run -r vm`）行为一致，
  无固定定位逃逸、无错误横幅。
- **G4（惯用法沉淀）**：嵌入安全壳层写入 Spec（demo 根禁用 viewport-fixed 铬件；
  用 flex/relative + 容器高度），供后续 demo 复用。
- **G5（022 配色）**：022-kanban 深/浅主题下均可读、与 AutoUI 语义 token 一致；
  列/卡不再使用原始 `gray/orange/green-50` 底色；状态身份用色点/徽标（双主题安全）。
- **G6（022 应用框）**：独立 app 框——`pac.at window:"1120x740"`（对齐 014-weather
  固定窗先例，非 calculator `fit`、非 music-player 整页）；布局占满视口/窗口
  （`h-full min-h-screen` + 三列 `flex-1 items-stretch` + 卡片区滚动）。
- **G7（022 主题契约）**：根 widget 声明 `dark_mode` / `accent_color`（Plan 458
  魔法变量），提供切换钮；双主题截图均符合 G5。
- **G8（024 图表可见）**：024-charts 右侧图表区渲染真实 Line/Bar/Area/Donut
  （不再空 div）；裸名 chart tag 折叠为包组件 SFC（清偿 P601-T11 图表半句）。
- **G10（025 系统监视器）**：全高独立框；去选中横幅（右键结束）；数值 1 位小数；
  紧凑列表；去用户页。列拖拽另债。

### Non-goals
- 不改 gallery `AppViewport.vue` 深覆盖策略本身（P675-D1 长期项另案）；本计划以
  **demo 侧修布局**为主（P675-D1 修复候选「按 demo 形态分派」中的 demo 面）。
- 不引入 theme-toggle schema 元素（P675-D2 既有债）；主题说明文案保留。
- 不做 EPUB/真实书籍导入；保持现有 3 本种子书 + CRUD 数据面。
- 不改 `back.api` / `book_store` / `board_store` 数据契约（仅 UI 消费）。
- 不迁移 widgets-gallery / 其它 demo；022 不做真实拖拽库替换（保留 HTML5 DnD）。
- 022 不做多看板/泳道/标签体系（Trello 全功能）；保持三列 CRUD 教学面。

## 2. 架构方案

### 2.1 溢出根因（已勘定）

| 证据 | 位置 | 说明 |
|---|---|---|
| `fixed inset-y-0 z-40` | `examples/ui/018-book-reader/src/front/app.at:24` | 侧栏 col 定位到**浏览器视口**，非 demo 容器；gallery 嵌入时穿出视口卡 |
| `min-h-screen` 根 | `app.at:20`、`reading.at:49` | AppViewport 有 `:deep(.min-h-screen){min-height:100%}` 覆盖，但 **fixed 未覆盖** |
| 已知债 | `docs/plans/KNOWN-DEBT-AND-RISKS.md` **P675-D1** ② | 「demo 侧栏漂移出视口卡（生成 SFC 的 sidebar 定位落在嵌入容器外的宿主坐标）」——本计划清偿 ② |
| 嵌入容器 | `auto-os/ui-gallery/src/gallery/AppViewport.vue:163-180` | `demo-mount-root` 仅 deep 覆盖 `.h-screen/.min-h-screen/.w-screen` |

### 2.2 嵌入安全壳层（F-1，G1）

**原则**：demo 根 chrome 不用 viewport-fixed（`fixed`/`inset-0`/`inset-y-0`/`sticky` 仅限
容器内滚动条），全部用 **flex 行 + 侧栏 `shrink-0` + 主区 `flex-1 overflow-auto`**，
高度锚定父容器（`h-full` / `h-screen` 语义，AppViewport deep 覆盖后 = 容器高）。

```
row.h-full.min-h-screen.bg-background          ← 容器高度（deep→100%）
├─ col.w-60.shrink-0.h-full                     ← 相对布局，不再 fixed
│  ├─ 品牌头
│  └─ sidebar_menu（flex-1 + overflow）
└─ col.flex-1.min-w-0.h-full
   ├─ 移动顶栏（md:hidden）
   └─ outlet（路由页自管内部滚动）
```

对话框遮罩（bookshelf `fixed inset-0`）在嵌入态同样会逃逸 → 改为 **容器内 absolute**
或 dialog 组件族（若 schema 已有）；验收以「遮罩不超出视口卡」为准。

### 2.3 阅读 App UI/UX（G2，风格锚）

**风格锚**：Apple Books 书架网格 + Kindle 沉浸阅读栏（dark-first，沿用 pac `theme: dark` +
indigo primary）。信息密度：书架可扫视、阅读页极简。

| 页面 | 现状问题 | 目标形态 |
|---|---|---|
| Library | 大片空黑、封面只有 📖、删除按钮 hover 难发现 | 顶部「Continue Reading」横幅（进度>0 且 <100 的第一本）；封面卡加高（aspect 封面感）、进度条、hover 浮层操作；空态引导 |
| Book Detail | 章节 4 列网格过密、无当前章感 | 左封面/元信息，右**编号章节列表**（当前章高亮 + 进度勾），主 CTA = Continue/Start |
| Reading | 左树 TOC 常驻占宽、字号设置不生效 | 沉浸正文（衬线感 leading-relaxed）；TOC 改**抽屉/侧滑**（桌面可开合，移动覆盖）；顶栏只留返回+章名+目录钮；底栏进度；**font_size/line_height 真实应用** |
| Settings | 字号写了 localStorage 但阅读页不读 | 字号（S/M/L）+ 行距（紧凑/舒适）+ 主题说明；storage key `br-font-size` / `br-line-height`；阅读页读取并映射 class |

### 2.4 排版偏好接线（功能缺口）

- `settings.at` 已 `storage.set("br-font-size", s)`，`reading.at` **从未读取** → 接线：
  reading `.Init` 读 `storage.get("br-font-size"|"br-line-height")`，映射为正文 style/class。
- 字号映射：small=`text-sm` / medium=`text-base` / large=`text-lg`；
  行距：compact=`leading-snug` / comfy=`leading-relaxed`。

## 3. 技术栈

- 源语言：AutoUI `.at`（`examples/ui/018-book-reader/src/front/**`）
- 组件族：sidebar_*（Plan 562）、TreeView（reading 章节，可改为纯列表以减依赖）、
  progress、button/input/text/h1、style-if 条件类
- 双端：Vue codegen + VM/iced；gallery 内嵌经 AppViewport
- 存储：`storage.get/set`（阅读偏好）；后端 `back.api`（书/章/进度，不改契约）
- 验证：`cargo check -p auto-lang`、`cargo t`（改动面）、`auto run` / `auto run -r vm`、
  gallery 走查截图、必要时 Playwright（`tests/smoke.spec.ts`）

## 4. 需求分析与背景调查

### 4.1 授权记录

- **2026-09-22 用户会话授权**：「如截图，book-reader 的左侧 Sidebar 顶出了窗口的限制…
  请检查它的代码并修复。另外，这个界面安排不太合理，你看看借鉴一下常见的阅读 app，
  重新设计一下 UI/UX。请用一个新的计划 /auto-plan:new 来新建计划，并用 /auto-plan:work
  技能来实施它。」
- 范围：`examples/ui/018-book-reader`（UI/UX + 布局缺陷）；允许 worktree 实施、
  双轨/gallery 验证。无预算上限声明（批内小项规模）。
- 非授权范围见 Non-goals。

### 4.2 问题截图解读

- Sidebar（AutoRead / Library / Settings）从视口卡**左侧穿出**，高度=宿主窗口高，
  压住 gallery 自己的「018-b…」标题栏与底部源码 tabs——与 P675-D1 ② 描述一致。
- 主内容 Library 卡片区大片空白；书卡信息弱（图标封面、进度纯文字）。
- 实时内嵌视口尺寸控件（桌面 1024×720 等）可见 → 复现路径 = gallery Vue 轨选中 018。

### 4.3 相关契约 / 既有债

| 来源 | 要点 | 本计划处置 |
|---|---|---|
| P675-D1 ② | 018 侧栏漂移出视口卡 | **清偿**（G1） |
| P675-D1 ① 022-kanban 空白 | min-h-screen 族 | 不在本计划（018 只改自己的 min-h 策略） |
| `widgets/viewport-boundary.md` | 视口边界契约 | 引用；若需补 demo 铬件禁令 → SD-01 |
| Plan 562 sidebar 惯用法 | aside/provider 需 `display:flex + min-h-0` | 重设计时沿用 |
| PLAN-675 T-06 | theme-toggle 已撤 | 不恢复；Settings 文案改掉「sidebar 顶部」误导 |

### 4.4 阅读 App 对标要点（启发式，非像素克隆）

- **Library 主角**：书架是首页，少 chrome；「继续读」降低再入成本。
- **阅读页减法**：进入阅读后导航退到 TOC 抽屉/手势，正文最大化。
- **排版可控**：字号/行距/主题是阅读 App 刚需，设置必须生效。
- **章节可见性**：TOC 标编号、标进度，而不是抽象树根「Chapters」。

## 5. 详细设计

### 5.1 F-1 嵌入安全壳层（`app.at`）

1. 根 `row`：`min-h-screen` 保留（deep→100%），补 `h-full`，去掉对 fixed 的依赖。
2. 侧栏 `col`：
   - **删** `fixed inset-y-0 z-40`
   - **改** `w-60 shrink-0 h-full border-r border-border bg-card hidden md:flex flex-col`
   - 内部 sidebar_provider 沿用 `w-auto min-h-0 flex flex-1 flex-col`
3. 主区 `col`：**删** `md:ml-60`（不再为 fixed 让位）；改 `flex-1 min-w-0 h-full flex flex-col`
4. 移动顶栏保留（`md:hidden`）。
5. reading 页根同步：去掉纯 viewport 假设；TOC 列不 `fixed`。

### 5.2 F-2 Library（`bookshelf.at`）

- 顶区：标题 + 统计 + Add Book（保留）。
- **Continue Reading 条**（若存在 `0 < progress < 100` 的书）：横幅卡 = 封面色块 +
  书名/作者 + 进度% + CTA「Continue」→ `OpenBook` 或直达当前章（沿用 detail 的
  progress→chapter 换算，抽公共 fn 或 page 内复制最小逻辑）。
- **书卡网格**：封面块加高（`h-52` 或 aspect）、渐变封面 + 首字母/书名缩写（弱化 📖）、
  底部细进度条、书名/作者、进度徽标；hover 显 Remove（保留）并轻微 shadow。
- 对话框：遮罩从 `fixed inset-0` 改为 **absolute inset-0**（相对页面滚动容器）或
  `fixed` + gallery deep 覆盖不可用时的降级——**优先 absolute + 页面 col `relative`**，
  保证嵌入态遮罩不穿出。
- 空态保留并强化 CTA。

### 5.3 F-3 Book Detail（`book_detail.at`）

- 左：封面（加大）+ 标题/作者 + 章数/字数/进度 + 主 CTA。
- 右：`Chapters` 改为**有序列表式 TOC**（`01 · Title`），当前章（由 progress 换算）
  用 primary 描边/背景；点击 → `ReadChapter(n)`。
- 保留 Back。

### 5.4 F-4 Reading（`reading.at`）

- 壳：顶栏（返回 / 章名 / TOC 开关）+ 正文区 + 底栏（Prev / 进度 / Next）。
- TOC：`show_toc` model；桌面默认关，点 TOC 开侧滑面板（`absolute`/`w-64` 贴容器左，
  **非 fixed**）；列表用章号+章名（可弃 TreeView 降复杂度，**决策见 T-03**）。
- 正文：`max-w-2xl` 阅读栏 + 字号/行距 class；`.Init` 读 storage。
- 去掉 reading 根上的多余 `min-h-screen` 嵌套冲突（与壳层一份高度职责）。

### 5.5 F-5 Settings（`settings.at`）

- 字号三档（沿用）+ **新增行距两档**；写 `br-font-size` / `br-line-height`。
- 主题文案改为「跟随应用主题；阅读页为深/浅色底」，去掉「sidebar 顶部 toggle」失效指引。
- About 微调文案（无 ThemeToggle 宣称）。

### 5.7 022-kanban（rev2）

**问题勘定**：

| 症状 | 根因 | 证据 |
|---|---|---|
| 深色下列标题几乎不可见、列底刺眼 | 列底硬编码 `bg-gray-100`/`bg-orange-50`/`bg-green-50`（浅色专用），前景 `text-foreground` 随主题变浅色 | `board.at:44,76,108`；用户截图 |
| 浅色下卡片/列层次乱 | 卡片 `bg-card` 叠在浅列底上，对比不足；无统一边框/阴影层级 | 同上 |
| 窗口未占满、无独立 app 框 | ① `pac.at` 无 `window:`（默认非 fit 非固定）；② 列 `items-start` + 固定 `w-80` 不拉高度；③ 根仅 `min-h-screen` 无 `h-full` 填充链 | `pac.at`；`app.at:14`；`board.at:41` |

**目标形态（风格锚：Linear / Trello board × AutoUI token）**：

```
window: "1120x740"                    ← 独立 app 框（014-weather 先例）
col.w-full.h-full.min-h-screen        ← 占满嵌入视口/窗口
├─ header.h-14.shrink-0.bg-card       ← 应用铬件：标题 + 卡数 + 主题切换
└─ board.flex-1.min-h-0
   ├─ toolbar（Add card）
   └─ row.flex-1.items-stretch.gap-3
      ├─ col.flex-1.bg-muted/50       ← To Do · 色点 border
      ├─ col.flex-1.bg-muted/50       ← In Progress · 色点 primary
      └─ col.flex-1.bg-muted/50       ← Done · 色点 emerald
         └─ cards.flex-1.overflow-y-auto
```

- **配色**：全部语义 token（`bg-background/card/muted`、`text-foreground/muted`、
  `border-border`、`bg-primary`）；列状态仅用 8px 色点（`bg-border` / `bg-primary` /
  `bg-emerald-500`）+ 标题 `text-foreground`——双主题安全。
- **卡片**：`bg-card border border-border rounded-lg`，hover 提亮；移列/删除幽灵按钮。
- **主题契约**：根 `var dark_mode bool = true`、`var accent_color str = "indigo"`
  （Plan 458，变量名勿改）+ `ToggleDarkMode`。

- 封面色/进度换算保持 `book_store` / api 字段不动。
- 样式 token：`bg-background/bg-card/text-foreground/text-muted-foreground/border-border/
  bg-primary/...`，不引入新色板。
- 语义元素优先：button/input/text/progress/row/col/div；TreeView 仅在保留章节树时用。

### 5.8 024-charts 图表空白（rev3）

**问题勘定**（用户截图：左侧控制面板正常，右侧仅空圆角框）：

| 症状 | 根因 | 证据 |
|---|---|---|
| 右侧图表区空白 | 裸名 `line-chart` 等发射 `<div :data=…>` 空壳，非 `<LineChart>` | 生成 `App.vue:266`；KNOWN-DEBT P601-T11 / P555-D4 |
| 折叠桥接被跳过 | `tag_is_builtin("line-chart")` 因 schema `package_origin` 返回 true，跳过 known_sub_widgets 折叠 | `vue.rs`；与 `is_builtin_fold` 不同口径 |
| props 通道不完整 | 024 将 `data:`/`fields:` 写在 body 内，契约形态应在括号内 | `line_chart.at` Usage |

**修复**：① `tag_is_builtin` 对 PackageOrigin 返回 false（对齐 PLAN-643）；
② 024 四图表改括号 props 形态。

### 5.9 024 Play 流式无效（rev4）

**问题**：点 Play 后图表不动（轴仍 Jan–Jun）。

**根因**：Vue 臂组件 `.Init` 只发 `onMounted` 一次；父级 `monthly` 流式重播种后
`data` prop 变了但几何不重算。ADR-19「Init 每渲染帧重放」仅 VM 轨落地
（`chart-components.md` / `architecture.md` ADR-19）。

**修复**：带 props 的组件 Init 发 `__autoReplayInit` + `onMounted` +
`watch(() => props, …, { deep: true })`（ADR-19 Vue 对等；Init 纯派生幂等契约）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/widgets/viewport-boundary.md` | before：视口边界主要约束 h-screen/w-screen；after：增补「demo 壳层 chrome 禁用 viewport-fixed（fixed/inset-y-0/inset-0 仅限容器内 absolute 遮罩）；侧栏/顶栏用 flex+容器高」 | P675-D1 根因通用化，防再发 | AC-01, AC-02 |
| SD-02 | modify | `docs/specs/auto-lang/ui/overview.md`（示例/画廊节或 018 相关） | before：018 仅有实现描述；after：补「阅读示例 UI/UX 契约（嵌入安全壳、TOC 抽屉、阅读偏好 storage 键）」一小节 | 018 作为 Tier-4 阅读范例的可复用约定 | AC-03..AC-06 |
| SD-03 | modify | `docs/plans/KNOWN-DEBT-AND-RISKS.md` P675-D1 | before：② 开放；after：② 标注清偿（PLAN-682） | 债闭环 | AC-01 |

## 6. 测试设计

| 层 | 内容 | 命令/方法 | 期望 |
|---|---|---|---|
| 编译 | 018 语料无错 | `cargo check -p auto-lang`；涉及 codegen 断言则 `cargo t gallery` / 相关模块 | 0 error |
| 语料/回归 | 若 018 进 gallery golden/compile 测试 | `cargo t 018` 或 `cargo tv`（视 diff 面） | 绿；无新增红 |
| Vue 单跑 | 布局+交互 | `cd examples/ui/018-book-reader && auto run` + Playwright/手操 | 侧栏在窗内；四页可走通 |
| VM 单跑 | 对称 | `auto run -r vm` | 结构树/路由/偏好可操作 |
| Gallery 嵌入 | **主验收** | ui-gallery `auto run` 选 018，1024×720 / 768×1024 / 375 三档截图 | 侧栏不穿出视口卡；Continue/TOC/字号可见生效 |
| 对话框 | Add Book 遮罩 | 嵌入态点 Add | 遮罩不超出视口卡 |

门禁（Category B，仅语料/UI 样例则 A/B 边界）：diff 触及 `crates/` 才跑对应单测；
本计划默认 **不改 crates/**，以语料 + 实机走查为主。若为 AppViewport deep 覆盖加规则
（Non-goal 倾向不加），则升 Category B。

## 7. 验收标准

- **AC-01** gallery 内嵌 018：侧栏完整落在视口卡内（截图对照：无「AutoRead」条穿出
  卡顶/卡左）；三档尺寸均满足。验证：走查截图 ×3。
- **AC-02** standalone `auto run` 窗口内侧栏同布局（非嵌入回归）。验证：截图/手测。
- **AC-03** Library：Continue Reading 条在有进行中书籍时可见且可点击进入；书卡显示
  进度条与标题作者。验证：走查截图 + 点击路径到 detail。
- **AC-04** Book Detail：章节列表带编号，当前章高亮（有进度时），点击进入阅读。
  验证：走查截图。
- **AC-05** Reading：TOC 可开合且不逃出容器；正文应用 Settings 字号/行距；Prev/Next
  与进度条工作。验证：改设置后刷新阅读页截图对比 + 翻章手测。
- **AC-06** Settings：字号/行距三档/两档可选并持久化；文案不再指向不存在的 theme-toggle。
  验证：设置截图 + 重进读 storage。
- **AC-07** Vue/VM 双轨无错误横幅；Add 对话框遮罩限于容器内。验证：双轨走查。
- **AC-08** P675-D1 ② 债行更新为已清偿并链到本计划。验证：KNOWN-DEBT diff。
- **AC-09（rev2）** 022 在 gallery 嵌入 1024×720 内看板占满视口卡（无大片上下空白、
  列高等于内容区高）；三列可滚动。验证：截图。
- **AC-10（rev2）** 022 深色模式下列标题/卡片可读（不再浅底浅字）；浅色模式下列/卡
  层次清晰。验证：双主题截图 ×2。
- **AC-11（rev2）** 022 不再使用 `bg-gray-100` / `bg-orange-50` / `bg-green-50` 等
  原始色板底色；状态以色点/语义文本区分。验证：源码 grep + 截图。
- **AC-12（rev2）** `pac.at` 声明 `window: "1120x740"`（独立 app 框，非 fit 非整页）；
  根具备 `dark_mode`/`accent_color` 主题契约。验证：pac.at/app.at 源检 + 切换截图。
- **AC-13（rev3）** 024 gallery 嵌入右侧图表区可见折线/柱/面积/环（切换类型）；
  生成物含 `<LineChart` 等组件引用而非空 `<div :data=…>`。验证：截图 + SFC grep。
- **AC-14（rev3）** `tag_is_builtin` 对 `package_origin` 返回 false，裸名 chart
  走 known_sub_widgets 折叠；`cargo t test_charts_gallery_compiles` 绿。验证：单测。
- **AC-15（rev4）** gallery 内嵌 024 点 Play 后 x 轴标签从 Jan… 变为 t1/t2…，
  曲线随滑窗移动；生成物含 `watch(() => props` 重放 Init。验证：前后截图 + SFC grep。
- **AC-16（rev5）** 025-sys-monitor：全高独立框（嵌入 1024×720 无大片空白）；
  去选中横幅（右键结束进程）；指标 1 位小数；紧凑列表；用户页退役。验证：走查截图
  （提交 5ea9d42a9/52f3b159f/3f87dbaed）。
- **AC-17（rev6 前置）** 026-database：fit gallery 视口（105b6fcd9）+ 紧凑表格 +
  单行标题居中（301ae0a42/597504f74）。验证：走查截图。
- **AC-18（rev7）** 027-file-manager vue 轨真盘：gallery 内嵌经 back-proxy VM 会话
  读真实主目录（面包屑 C:\›Users›zhaop、66 项、盘符 C/D/E/G、双击进 Documents 29 项、
  名称排序 asc↔desc 翻转、平板 768 档无溢出）；standalone `auto run`（rust 骨架后端
  fs_home()=""、list ok:false）落 6 项演示目录后备；VM 轨真盘不回归（VM 窗口实拍：
  真实条目+排序/日期正常，共享 build_crumbs/sort_files 提取无恙）。验证：
  evidence/p682/{gallery_027_embed,nav,tablet}.png + vue_standalone_demo_fallback.png +
  vm_standalone_027.png + probe_027_*.mjs。

## 8. 执行步骤

> 工作树（Plan 529）：`D:/autostack/.wt/lang-684/auto-lang -b plan-684-dev`；
> 计划簿记（勾选/frontmatter）在主检出；代码只在 worktree。

1. **T-01 嵌入安全壳层**
   - 文件：`examples/ui/018-book-reader/src/front/app.at`
   - 操作：按 §5.1 去掉 sidebar `fixed inset-y-0 z-40` 与主区 `md:ml-60`；侧栏改
     `shrink-0 h-full`；主区 `flex-1 min-w-0 h-full flex flex-col`；根补 `h-full`。
   - 验证：`auto run` gallery 内嵌截图侧栏不穿出；standalone 不塌。
   - 关联：AC-01, AC-02, SD-01

2. **T-02 Library 重设计**
   - 文件：`examples/ui/018-book-reader/src/front/pages/bookshelf.at`
   - 操作：Continue Reading 条；书卡封面/进度/操作层；Add 对话框遮罩改容器内
     absolute + 父级 relative。
   - 验证：截图；Add 对话框遮罩不穿出。
   - 关联：AC-03, AC-07

3. **T-03 Reading 重设计 + 排版接线**
   - 文件：`examples/ui/018-book-reader/src/front/pages/reading.at`
   - 操作：TOC 抽屉（决策：优先纯列表降依赖；若必须树则保留 TreeView 并限容器高）；
     字号/行距从 storage 映射到正文；底栏/顶栏精简。
   - 验证：开合 TOC 截图；改 Settings 后正文 class 变化；翻章。
   - 关联：AC-05, AC-07

4. **T-04 Book Detail 重设计**
   - 文件：`examples/ui/018-book-reader/src/front/pages/book_detail.at`
   - 操作：§5.3 布局与 TOC 列表化、当前章高亮。
   - 验证：有进度书的 detail 截图。
   - 关联：AC-04

5. **T-05 Settings 接线与文案**
   - 文件：`examples/ui/018-book-reader/src/front/pages/settings.at`
   - 操作：行距选项；storage 键；去掉失效 theme-toggle 指引。
   - 验证：设置页截图 + 重进保持。
   - 关联：AC-06

6. **T-06 双轨 + Gallery 走查**
   - 操作：Vue/VM 单跑 + gallery 三档尺寸截图；对照 AC-01..07。
   - 验证：截图入册 `docs/plans/evidence/p682/`。
   - 关联：AC-01..AC-07

7. **T-07 规范增量与债册**
   - 文件：`docs/specs/widgets/viewport-boundary.md`、`docs/specs/auto-lang/ui/overview.md`、
     `docs/plans/KNOWN-DEBT-AND-RISKS.md`
   - 操作：SD-01..03 落地。
   - 验证：债行 P675-D1 ② 已清偿。
   - 关联：AC-08, SD-01..03

8. **T-08 022-kanban 配色与应用框**（rev2）
   - 文件：`examples/ui/022-kanban/{pac.at,src/front/app.at,src/front/pages/board.at}`
   - 操作：§5.7——`window:"1120x740"`；根 `h-full min-h-screen` 全高壳；三列
     `flex-1 items-stretch` + 卡片区 `overflow-y-auto`；列底 `bg-muted/50` 等语义
     token，去掉 `bg-gray-100/orange-50/green-50`；状态色点；`dark_mode`/`accent_color`
     主题契约 + 切换钮。
   - 验证：`auto build` 绿；深/浅双主题截图列底与前景可读、看板占满视口。
   - 关联：AC-09..AC-12, G5..G7

9. **T-09 022 gallery 嵌入走查**（rev2）
   - 操作：`AUTO_GALLERY_APPS=<clone>/examples/ui` 启 gallery，选 022，三档尺寸截图。
   - 验证：无溢出、无花屏配色；窗口形态可见（独立 app 框）。
   - 关联：AC-09, AC-10

10. **T-10 024-charts 图表可见**（rev3）
    - 文件：`crates/auto-lang/src/ui_gen/vue.rs`（`tag_is_builtin`）、
      `examples/ui/024-charts/src/front/app.at`（props 契约形态）
    - 操作：§5.8——package_origin 不挡折叠；024 props 全进括号。
    - 验证：`cargo t test_charts_gallery_compiles` 绿；gallery 024 截图右侧有图。
    - 关联：AC-13, AC-14, G8

11. **T-11 024 gallery 走查 + 计划收口**（rev3）
    - 操作：gallery 选 024 切换四类图截图；更新债册 P601-T11。
    - 验证：截图入册；P601-T11 图表半句标注清偿。
    - 关联：AC-13

12. **T-12 ADR-19 Vue Init 重放**（rev4）
    - 文件：`crates/auto-lang/src/ui_gen/vue.rs`
    - 操作：带 props 组件 `.Init` 发 `__autoReplayInit` + `onMounted` +
      `watch(() => props, …, { deep: true })`；import 补 `watch`。
    - 验证：024 内嵌 Play 后轴标签变 t1…；SFC 含 watch 重放。
    - 关联：AC-15, G9

13. **T-13 025-sys-monitor 重设计**（rev5）
    - 提交：`5ea9d42a9`（紧凑 UI + 1 位小数）→ `52f3b159f`（自适应速率单位 + 粘滞
      排序 + perf 崩溃修 + 详情页退役）→ `3f87dbaed`（ApplySort on refresh——排序
      方向不再每 tick 翻转）。
    - 验证：gallery 走查（全高框/右键结束/紧凑列表）。列拖拽记债（rev5 注记，
      未入 KNOWN-DEBT——观察项）。
    - 关联：AC-16, G10

14. **T-14 026-database 视口适配**（rev6 前置批）
    - 提交：`105b6fcd9`（fit gallery viewport + 紧凑表）→ `301ae0a42`（单行标题 +
      紧凑分页）→ `597504f74`（标题垂直居中）。
    - 验证：gallery 走查截图。
    - 关联：AC-17

15. **T-15 027-file-manager 真·本地目录**（rev6 演示 FS `0c8ab50b7`+`0a264e36e` →
    rev7 真盘读路径）
    - 文件：`examples/ui/027-file-manager/{pac.at,src/back/api.at,
      src/front/app.at,src/front/components/fs_util.at}` +
      `crates/auto-lang/src/ui_gen/ts_adapter.rs`（VM_ONLY_OBJECT_NATIVES 补
      file/process 小写模块名——027 vue-tsc 历史首绿）+ `crates/auto-man/src/vue.rs`
      （gallery 缓存失配扫描 worker 显式 16MB 栈——2MB 缺省栈深语料解析整进程崩）。
    - 形态：thin 契约纪律（#[api] 端点单行委托 impl_*；gallery back-proxy VM 会话
      真盘 / standalone rust 后端落模板 Default → 演示目录仅后备）；路径/面包屑/
      排序纯函数双轨同源（VM inline 块迁移共用）。写操作 vue 轨守卫（P684-D1 债）。
    - 验证：AC-18 三路径（gallery 真盘 / standalone vue 后备 / VM 不回归）；
      `auto build` 绿；`cargo t vm_only` 2/2 + `cargo t gallery` 12/12。
    - 关联：AC-18, PLAN-680 Q3 L2

## 9. 复审记录

- [2026-09-22] `stage: new` | revision: 1 | outcome: **pass** | next: **work**
  - 范围/授权齐备（用户明示新建计划并实施）；根因证据：app.at:24 `fixed inset-y-0` +
    P675-D1 ② + AppViewport deep 覆盖缺口；任务 T-01..T-07 覆盖 AC-01..08 与 SD-01..03。
  - 变更任务/验收 ID：无（初版）。
- [2026-09-22] `stage: revise` | revision: 3 | outcome: **pass** | next: **work**
  - 用户追加 024-charts 右侧空白（截图确证）；根因 = `tag_is_builtin` 未排除
    PackageOrigin（P601-T11 图表半句）。新增 G8、AC-13/14、T-10/T-11、§5.8。
  - 018/022 进度：代码均在 `plan-684-dev`（a746b76c1 / 129b979de / 8184fb142）；
    gallery 走查截图在案。
- [2026-09-22] `stage: work` | plan_id: PLAN-684 | plan_revision: 8 | outcome:
  **pass** | code_commit: plan-684-dev `6044fb1b9`（T-15 rev7 + T-07 补做；前序
  a746b76c1..0a264e36e 十四提交在案） | task_ids: T-01..T-15 | evidence:
  AC-01..18 全映射——018/022/024 走查截图（8e4ba1eab）+ 025/026 走查（T-13/14
  提交列）+ 027 三路径（AC-18：gallery 真盘 66 项/导航/排序/平板 + standalone
  vue 骨架后备 + VM 实拍不回归；evidence/p682/ 新 8 件）；门禁：auto build 绿
  （vue-tsc 首绿经 ts_adapter 桩表补全）、cargo t vm_only 2/2、gallery 12/12；
  T-07 中断漏做已补（SD-01..03 + P675-D1 ①② demo 面清偿 + P684-D1/D2 记债）
  | blockers: 无 | next: review
- [2026-09-22] `stage: review` | plan_id: PLAN-684 | plan_revision: 8 | outcome:
  **pass** | reviewed_commit: `d433e3f29`（复审随批含测试修复；实现面
  a746b76c1..6044fb1b9 十六提交） | base_commit: `ce77f58fe` |
  dependency_revisions: 无跨仓依赖（auto-os ui-gallery 消费面经
  AUTO_GALLERY_APPS 走查验证，无代码改动） | spec_inputs:
  docs/specs/widgets/viewport-boundary.md + docs/specs/auto-lang/ui/overview.md
  （SD-01/02 落地核读）+ KNOWN-DEBT P675-D1/P670-D1 | acceptance_results:
  AC-01..08 pass（018：走查 8e4ba1eab + 代码抽查 storage 接线
  reading.at:158/settings.at:120-139、债行更新 6044fb1b9）；AC-09..12 pass
  （022：走查 + pac.at:12 window:"1120x740" + app.at:14-15 主题契约 + 原始
  色板 grep 0 命中）；AC-13..15 pass（024：走查前后对照 + SFC watch 重放 +
  tf 内 test_charts_gallery_compiles 绿）；AC-16 pass（025 三提交走查）；
  AC-17 pass（026 三提交走查）；AC-18 pass（027 三路径本会话实证：gallery
  真盘 66 项/导航 29 项/排序翻转/平板档 + standalone vue 骨架后备
  fs_home()=""/list ok:false + VM 实拍不回归；probe_027_*.mjs 可重放） |
  findings: F-1 T-07（SD-01..03）被中断会话漏做——本会话补齐（6044fb1b9）；
  F-2 plan408 断言滞后于 T-12 ADR-19 发射形态（本分支引入，tf 快败根因）——
  已修（d433e3f29，断言改 async __autoReplayInit）；F-3 auto-man 测试面
  PLAN-681 generate_api_rs 三参化遗留 11 处 E0061（master 测试目标编译即坏，
  非本分支引入）——随批修复构建；余 3 红全预存（shell_pack=在册已知 +
  plan593 css golden/merged_api_client=681/672 基线漂移类，另案） |
  evidence: tf 3722/3722 全绿（--no-fail-fast）+ auto-man 324/327（3 预存
  上列）+ cargo t vm_only 2/2 + gallery 12/12 + 027 auto build 绿（vue-tsc
  首绿）+ evidence/p682/ 25 PNG + 6 探针脚本 | next: merge
- [2026-09-22] `stage: merge` | plan_id: PLAN-684 | plan_revision: 8 | outcome:
  **pass** | delivery_commit: `e6b408c52`（master tip，ff-only） | 五 checkpoint：
  **prepared**=账本三件套随交付提交（ui/plans.md 表行+specs.json P684-1/P684-2 designs/reviews 两节外科插入+INDEX 再生零差异=项目级索引无新项目）；**landed**=rebase 双轮（1311fee29→87c2e50a4 他方会话并行前移；KNOWN-DEBT EOF P683/P684 并集+specs.json/plans.md 条目级 JSON 并集×2 轮），range-diff 全等（`=` 逐提交；哈希映射
  a746b76c1→71bf8e64a / 319716236→0cebf7742 / 3ae4c42f8→371fbbd54 / 8e4ba1eab→38ee2d27f / 5ea9d42a9→250d8e879 / 0c8ab50b7→999476b55 / 0a264e36e→18912bfc3 / 528edf399→6664f4716 / 6044fb1b9→b4b186eaf / d433e3f29→fed6f00eb / 8910d8086→37ac4b3a1→e6b408c52），origin push force-with-lease+主检出 ff-only；**ledger_refreshed**=specs.json 落库含 P684-1/P684-2（readback：master 树 grep P684-1 命中）；**archived**=本行（git mv archive/ + status archived）；**cleaned**=随后行。
  - 落地终态门禁补注：rebase 后全量 tf（-E 'not(test(/ffi_dual/))'，ffi_dual_018_dep_fields
    =run_with_capture 双重解释器路径 P574 文档化 Windows 挂起类、纯 master 预存）：
    5428 run/5418 pass/10 红——全预存（musk p053/p054 ×6=在册 musk×6 待认领、
    projector_counter=P678 在册、p508_g2=陈旧 exe 假红类、a2vue desktop 金样+
    native_gate coverage=纯 master 复跑同败实证=682/683 期漂移另案）；增量面
    （vm_only/gallery/plan408）全绿+rebase 前基线 tf 3722/3722 全绿。
  - worktree 注记：lang-684 实为独立 clone（origin=主仓本地路径，非注册 worktree
    ——前会话创建形态），guard 后整目录删除（分支经 push 已并入主仓再删）。
  - 同会话复审局限注记：本复审在实施会话内进行；判据全部自仓内耐久工件
    （提交 diff/测试输出/走查截图/可重放探针）重建，不依赖执行者口头总结。
  - 会话中断接续注记：前一 agent 中断于 rev7 中途（api.at 对象访问误用
    fs.read_dir 字符串数组形态 + pac 已改未提交）；本会话重写为 thin 契约形态、
    补 vue 轨接线/守卫/纯函数迁移与两处 crates 修复（桩表 + 扫描栈）后收口。

## 10. 待澄清事项

- （默认决策，实施中可翻）TOC 用纯列表而非 TreeView——层级仅 book→chapters，树过重；
  若用户要求保留树形再改回。
- （默认决策）Add 对话框遮罩用容器内 absolute，不用 dialog 组件族——避免扩 schema 面。
- 阅读页是否在嵌入态隐藏壳层侧栏（更沉浸）：**默认保留壳层侧栏**（gallery 演示仍需
  可导航）；仅 reading 内部 TOC 抽屉化。
