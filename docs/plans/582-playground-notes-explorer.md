---
plan_id: PLAN-582
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: playground-notes-explorer
author: [zhaopuming]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [playground-vue, website, auto-playground]   # specs 路径
current_step: 0
total_steps: 16
---

# [PLAN-582] Notes Explorer 笔记站 + 宿主合一 + 电子书 Run 嵌入（Playground 设计 · Plan B）

> 前置依赖：[PLAN-581](581-playground-notes-foundation.md)（SnippetRunner/PlaygroundCard 组件 + notes manifest 管线）已合入。

## 变更摘要

按 [Playground 设计](../design/documents/playground-architecture.md) §6/§7/§8 完成 Playground 在线体验层的呈现与部署合一：

1. **Notes Explorer 笔记站**：`auto-playground-vue` 新增 `NotesExplorer` 组件（左分组树+搜索，右说明+PlaygroundCard+期望输出对照），vm-golden 笔记提供实际输出 vs `.expected.out` 行级对照高亮。
2. **宿主合一**：website `/playground`（EN/ZH）升级为 Notes Explorer；`website/public/playground/` 旧静态 SPA 退役为重定向页；`crates/auto-playground/frontend` 宿主切换为 Notes Explorer（IDE 全功能经切换入口保留）；后端 `/api/examples` 改读 manifest（单一事实源，回退目录扫描）。
3. **电子书 Run 嵌入**：VitePress 主题层对 ` ```auto ` 围栏后处理——右上角 ▶ Run 原地展开 SnippetRunner（autorun），可收起还原；含 `import` 的围栏锁提示不提供 Run；双语书零内容改动同等生效。

## 目标

- G1: `/playground` 成为 website 唯一 playground 入口，呈现全部 manifest 笔记（≥4 源分组），支持搜索/深链/键盘导航，EN/ZH 一致。
- G2: **静态优先**——无后端时全部笔记可浏览阅读，Run/转译位呈降级引导（启动后端命令 + 部署说明）。
- G3: vm-golden 笔记有"期望输出"tab：实际 stdout 与 expectedOutput 行级对照（✓ 一致 / ✗ 差异高亮）。
- G4: 书页 `auto` 围栏可一键运行（展开/收起），依赖型围栏锁提示。
- G5: 旧 `/playground/` 静态产物退役重定向；`build-playground.mjs` 的 website 同步分支删除。
- G6: 后端自服务 UI（`cargo run -p auto-playground` 打开的页面）与 website 同体验；IDE 全功能入口保留。
- G7: `/api/examples` 响应数据与 manifest 同源（响应 schema 不变，兼容 ExampleSelector/Full）。

## 架构方案

对应 [Playground 设计](../design/documents/playground-architecture.md)：

- §6 Notes Explorer：`NotesExplorer.vue`（布局壳）+ `useNotes.ts`（manifest 加载/索引/搜索）+ `ExpectedOutputPanel.vue`（对照）；视觉消费 VitePress CSS 变量，包内提供 fallback token（SPA 宿主无 VitePress 变量时）。
- §7 电子书：主题层 fence 后处理（`AutoFence.vue` 包裹 lang=auto 围栏），首版用围栏原文 + 启发式 `import` 检测，不接 manifest 索引。
- §8 部署合一：三个消费方同一组件族——VitePress `/playground`、后端 `frontend/`（宿主组件从 `AutoPlaygroundFull` 换 `NotesExplorer`，保留"IDE 模式"切换）、`/api/examples` 读 manifest。
- 单一事实源：`scripts/build-playground-notes.mjs` 增加第二输出 `crates/auto-playground/notes.json`（gitignore，构建期产物）；`examples.rs` 启动时存在即读、缺失回退现有目录扫描。

## 技术栈

- **组件**：Vue 3.5 + TypeScript（复用 581 的 SnippetRunner/PlaygroundCard/usePlayground）；lucide-vue-next。
- **构建/类型门禁**：vite 8 + `vue-tsc -b`（packages）；VitePress 1.6（website，主题层 `enhanceApp`/markdown 钩子）。
- **后端局部改动**：Rust（axum routes，serde_json 反序列化 manifest），门禁 `cargo check -p auto-playground`（不触 `crates/auto-lang`，禁跑 cargo t/tf/tv/tb）。
- **e2e**：Playwright 1.59（website 现有 `npm run test:e2e` 体系）。

## 需求分析与背景调查

（取材 [docs/specs/overview.md](../specs/overview.md) 与模块 specs）

- **[playground-vue](../specs/playground-vue/project.md)**（active）：581 后已有 SnippetRunner/PlaygroundCard 三层；本计划在其上加 NotesExplorer 与 ExpectedOutputPanel，`lang/` CodeMirror 支持复用。
- **[auto-playground](../specs/auto-playground/project.md)**（active）：`routes/examples.rs` 现扫 `examples/playground-demo/`；本计划改读 manifest（`Example` 响应结构不变）。`frontend/` 现渲染 `AutoPlaygroundFull`，换宿主 NotesExplorer。构建同步：`scripts/build-playground.mjs` 构建后同步 `frontend/dist` 与 `website/public/playground`——website 分支本期删除。
- **[website](../specs/website/project.md)**（active）：VitePress 站，`playground.md`（EN/ZH）现内嵌 `<AutoPlayground>`；主题在 `website/.vitepress/theme/`（现有组件 AIHero/UIGallery 等，无围栏后处理）；e2e 走 Playwright（`npm run test:e2e`）。books 为 `prepare-content.js` 从外仓 `../book`（autostack/book）物化的 gitignore 生成物。
- 历史：两入口并存源于 `95089c156`（全量 SPA 部署 website）+ `ebc43e7ac`（iframe 换内联组件但旧页未退役），Playground 设计 §1.1。

## 详细设计

### 1. useNotes composable

`src/composables/useNotes.ts`：`fetchNotes(base)`（默认 `/playground-data/notes.json`，SPA 宿主经参数改 base）→ `{ groups, flatNotes, byId, search(q) }`；搜索 = 标题+tags `includes` 简易匹配；返回加载/错误态。

### 2. NotesExplorer 组件

布局（Playground 设计 §6.1）：左栏分组树（折叠态 + 计数徽章 + 搜索框）；右栏笔记头（标题、来源路径 chip → GitHub 链接 `<repo>/blob/master/<sourcePath>`、来源类型徽章 `vm-golden|aavm-corpus|book|demo|parity`）、说明折叠区、PlaygroundCard（`noteId` 驱动）。深链 `#/notes/<noteId>`（hash 变化不触发 VitePress 路由）；键盘 ↑/↓ 切换、Ctrl+Enter 运行。CSS：优先 `var(--vp-c-border)` 等 VitePress 变量，包内 `:root` fallback token。

### 3. ExpectedOutputPanel

实际 stdout 与 `expectedOutput` 逐行比对（trim 行尾空白与末尾空行后精确比对）；三态：未运行（展示期望）、一致（✓ 绿徽章）、差异（✗ 红 + 行级 +/- 高亮）。挂在 PlaygroundCard 输出区 tab（仅 note.expectedOutput 非空时出现）。

### 4. 项目型笔记（kind=project）

PlaygroundCard 内文件 tab（entry 锁 `main.at`，多文件可切换编辑，运行走现有 run API 的 files 形态——`usePlaygroundFull` 已支持项目运行，复用其请求构造）；"在 IDE 中打开"按钮 → 同页切换 IDE 模式（SPA 宿主渲染 `AutoPlaygroundFull` 并 loadExample；VitePress 宿主无后端时禁用并提示）。

### 5. 后端 /api/examples 单一事实源（Playground 设计 §9-④ 落地）

- `scripts/build-playground-notes.mjs`：增加输出 `crates/auto-playground/notes.json`（同内容；两处均 gitignore）。
- `examples.rs`：启动时探测 `CARGO_MANIFEST_DIR/notes.json`，存在→解析 manifest 映射为现有 `Example { name, source, example_type, project_dir, files }` 响应（保持 schema 兼容）；不存在→现有目录扫描。选择"文件探测+回退"而非 include_str!（生成时序与体积），CI/本地 `prepare-content` 链保证生成。

### 6. 电子书围栏后处理

`website/.vitepress/theme/`：`AutoFence.vue`（Run 按钮 + 展开 SnippetRunner + 收起）；theme `index.ts` 增强 `enhanceApp`/markdown 配置将 lang=auto 围栏替换为该组件包裹（VitePress 自定义 container/fence 渲染钩子）。围栏原文即 `code` prop；含 `import` 行 → 锁图标 + tooltip"依赖多文件模块，请在 Playground 打开"（链接笔记站）。`.cn.md` 经 VitePress 同管线自动生效。

### 7. 无后端降级

`usePlayground` 增加 backend 探测（首个请求失败置 `backendDown`）；PlaygroundCard/SnippetRunner 动作位降级为引导卡（`cargo run -p auto-playground` + website 部署说明锚点）。浏览/阅读不受影响（manifest 是静态资产）。

## 测试设计

- **分级门禁**：触 `crates/auto-playground/src/routes/examples.rs`（Rust）→ Category B 局部：`cargo check -p auto-playground`（零警告）；该 crate 无测试面则不跑 cargo t（不触 `crates/auto-lang`，**禁跑 cargo t/tf/tv/tb**）。
- 组件：`cd packages/auto-playground-vue && npm run build`。
- website：`cd website && npm run build`（EN/ZH 全量）+ `npm run dev` 手动冒烟（树/搜索/深链/书页 Run/降级态）。
- e2e（Playwright，最小集）：`/playground` 渲染 ≥4 分组、笔记点击出卡片；books 页 Run 按钮存在；无后端降级态可见。后端相关断言仅在本地起服时执行（CI 跳过）。
- 后端：`cargo run -p auto-playground` + `curl -s http://127.0.0.1:<port>/api/examples | jq '.examples | length'` 与 manifest 计数一致（端口以 `main.rs` 实际配置为准）。

## 验收标准

1. `/playground`（EN/ZH）呈现 Notes Explorer：≥4 源分组树、计数徽章、搜索、深链 `#/notes/<id>`；旧 `/playground/`（带斜杠）访问重定向至 `/playground`。
2. 断开后端（或静态预览 `npm run preview`）：全部笔记可浏览，Run 位呈降级引导，无白屏/报错。
3. 任一 vm-golden 笔记：运行后期望输出 tab 呈 ✓/✗ 行级对照（本地后端在位验证）。
4. `website/books/` 任一书页：`auto` 围栏有 ▶ Run，展开 autorun 运行、收起还原；含 `import` 围栏呈锁提示（EN 与 `.cn.md` 各验一页）。
5. `cargo run -p auto-playground` 打开的页面与 website `/playground` 同为 Notes Explorer；IDE 模式切换可达全功能（文件树/调试/回放）。
6. `/api/examples` 与 manifest 计数一致（curl 验证）；删除 `crates/auto-playground/notes.json` 后回退目录扫描仍可用。
7. `cd packages/auto-playground-vue && npm run build`、`cd website && npm run build`、`cargo check -p auto-playground` 全绿零警告。
8. `scripts/build-playground.mjs` 不再写 `website/public/playground/`；仓库内无旧 SPA 残留产物。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T1 useNotes：加载与索引**
  文件：`packages/auto-playground-vue/src/composables/useNotes.ts`（新建）
  操作：`fetchNotes(base)` 加载 manifest → `groups`/`flatNotes`/`byId` 索引 + 加载/错误态。
  验证：`cd packages/auto-playground-vue && npx vue-tsc -b && echo OK`。

- **T2 useNotes：搜索**
  文件：`packages/auto-playground-vue/src/composables/useNotes.ts`
  操作：`search(q)`——标题+tags `includes` 匹配，返回命中笔记列表（分组归属保留）。
  验证：`cd packages/auto-playground-vue && npx vue-tsc -b && echo OK`。

- **T3 NotesExplorer：布局骨架**
  文件：`packages/auto-playground-vue/src/components/NotesExplorer.vue`（新建；侧栏可拆 `NotesSidebar.vue`）
  操作：左栏分组树（折叠/计数徽章）+ 右栏笔记头（标题/来源 chip→GitHub 链接/类型徽章/说明折叠）+ PlaygroundCard 挂载；CSS 用 VitePress 变量 + fallback token。
  验证：`cd packages/auto-playground-vue && npm run build`。

- **T4 NotesExplorer：深链与键盘**
  文件：`packages/auto-playground-vue/src/components/NotesExplorer.vue`
  操作：`#/notes/<id>` 读写（hashchange 监听，不触发 VitePress 路由）；↑/↓ 切换当前笔记、Ctrl+Enter 触发运行（事件转发 PlaygroundCard）。
  验证：`cd packages/auto-playground-vue && npm run build`；`npm run dev` 冒烟（改 hash 定位、键盘切换）。

- **T5 ExpectedOutputPanel 组件**
  文件：`packages/auto-playground-vue/src/components/ExpectedOutputPanel.vue`（新建）
  操作：三态（未运行/✓ 一致/✗ 差异行级高亮）；比对规则=trim 行尾空白+末尾空行后逐行精确比对。
  验证：`cd packages/auto-playground-vue && npm run build`。

- **T6 PlaygroundCard 接期望输出 tab**
  文件：`packages/auto-playground-vue/src/components/PlaygroundCard.vue`
  操作：note.expectedOutput 非空时输出区加"期望输出"tab 挂 ExpectedOutputPanel（实际 stdout 来自 usePlayground 状态）。
  验证：`cd packages/auto-playground-vue && npm run build`；本地起后端 + dev，打开 `01_basics` 任意 vm-golden 笔记冒烟 ✓/✗ 两态。

- **T7 项目型笔记文件 tab**
  文件：`packages/auto-playground-vue/src/components/PlaygroundCard.vue`
  操作：kind=project 时文件 tab（entry 锁 `main.at`、多文件切换编辑）；运行请求构造复用 `usePlaygroundFull` 的 files 形态。
  验证：`cd packages/auto-playground-vue && npm run build`；笔记站（或 dev 页）打开 `demo/09-multi-module` 冒烟多文件运行。

- **T8 "在 IDE 中打开"切换入口**
  文件：`packages/auto-playground-vue/src/components/PlaygroundCard.vue`、`NotesExplorer.vue`
  操作：按钮触发 `ide-mode` 事件；SPA 宿主（后端 frontend）渲染 `AutoPlaygroundFull` 并 loadExample 对应笔记；VitePress 宿主无后端时禁用+提示。
  验证：`cd packages/auto-playground-vue && npm run build`。

- **T9 无后端降级态**
  文件：`packages/auto-playground-vue/src/composables/usePlayground.ts`、`components/SnippetRunner.vue`、`components/PlaygroundCard.vue`
  操作：首个请求失败置 `backendDown`（可重试）；动作位降级为引导卡（`cargo run -p auto-playground` + 部署说明锚点）。
  验证：`cd packages/auto-playground-vue && npm run build`；`cd website && npx vitepress preview --port 4173 &` 后浏览器开 `/playground`（无后端）确认浏览完整、Run 位引导。

- **T10 website /playground 改造**
  文件：`website/playground.md`、`website/zh/playground.md`、`website/.vitepress/theme/index.ts`（如需注册全局组件）
  操作：内嵌 `<NotesExplorer />` 替换 `<AutoPlayground />`，保留头部与后端引导块；EN/ZH 同步。
  验证：`cd website && npm run build`；`npm run dev` 冒烟（树/搜索/深链）。

- **T11 旧静态页退役**
  文件：`scripts/build-playground.mjs`、`website/public/playground/`（清空）、`website/public/playground/index.html`（新重定向页）
  操作：mjs 删除 website 同步分支（保留后端 dist 分支）；public/playground 清空后提交仅含 meta-refresh → `/playground` 的 index.html。
  验证：`cd website && npm run build && grep -i refresh .vitepress/dist/playground/index.html public/playground/index.html`。

- **T12 manifest 第二输出**
  文件：`scripts/build-playground-notes.mjs`、`crates/auto-playground/.gitignore`（无则新建）
  操作：脚本增加输出 `crates/auto-playground/notes.json`（同内容）；gitignore 加 `notes.json`。
  验证：`node scripts/build-playground-notes.mjs && cmp website/public/playground-data/notes.json crates/auto-playground/notes.json && echo SYNCED`。

- **T13 /api/examples 读 manifest**
  文件：`crates/auto-playground/src/routes/examples.rs`
  操作：启动探测 `CARGO_MANIFEST_DIR/notes.json`，存在→serde 反序列化并映射为现有 `Example` 响应（schema 兼容）；缺失→现有目录扫描回退。
  验证：`cargo check -p auto-playground`；起服 `curl -s http://127.0.0.1:<port>/api/examples | jq '.examples | length'` 与 manifest 笔记总数一致；删 notes.json 重启后回退可用。

- **T14 后端 frontend 换宿主**
  文件：`crates/auto-playground/frontend/src/App.vue`
  操作：默认渲染 `NotesExplorer`，顶部"IDE 模式"切换渲染 `AutoPlaygroundFull`（接 T8 事件）；`build-playground.mjs` 后端 dist 同步分支保持。
  验证：`cd crates/auto-playground/frontend && npm run build`；`cargo run -p auto-playground` 打开页面冒烟（笔记站 + IDE 切换 + 项目笔记运行）。

- **T15 电子书围栏 Run**
  文件：`website/.vitepress/theme/components/AutoFence.vue`（新建）、`website/.vitepress/theme/index.ts`
  操作：lang=auto 围栏包裹组件（▶ Run/展开 SnippetRunner autorun/收起还原/`import` 锁提示+链接笔记站）；theme 接线 markdown fence 渲染。
  验证：`cd website && npm run build`；`npm run dev` 打开 rust 书 `ch01-getting-started` 对应页面冒烟（Run 展开运行；`.cn` 版同验一页）。

- **T16 e2e 最小集与验收清单**
  文件：`website/tests/`（新增 playground-notes spec，命名随现有惯例）
  操作：三条断言（分组渲染 ≥4/笔记点击出卡片/无后端降级态可见；后端相关仅本地起服启用）；逐条跑"验收标准"1–8 并记录。
  验证：`cd website && npm run test:e2e -- playground-notes`（本地无后端路径）+ 验收清单全绿。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **② "在 IDE 中打开"的 VitePress 宿主形态**（T8 落地时定）：无后端时禁用+提示是底线方案；若后端与 website 同域部署则可直接链后端 UI。
- **电子书围栏 ↔ manifest 笔记互链**（Playground 设计 §11 演进项）：本期不做，仅锁提示链接到笔记站。
- **`/playground/` 旧 URL 的外部引用**（部署历史链接）：重定向页兜底；若 GitHub Pages 路由行为异常改 `_redirects`/404 兜底（T11 验证时裁定）。
