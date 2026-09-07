---
plan_id: PLAN-582
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: playground-notes-explorer
author: [zhaopuming]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/playground-vue/project.md: 修改——PlaygroundCard 扩展（expectedOutput 期望输出 tab/files 文件 tab/projectDir/ideMode 四 props+hasRun 态+运行后自动切换对照）；usePlayground 增项目请求构造（projectRequestBody files 形态，run/transpile/transpileAll 三处）与 backendDown/retryBackend；SnippetRunner 降级卡替换 581 临时提示+动作位离线态；AutoPlaygroundFull defineExpose(loadExample)"
  - "docs/specs/website/project.md: 修改——/playground（EN/ZH）AutoPlayground→NotesExplorer+后端引导块 id=backend 锚点；theme index.ts 注册 NotesExplorer/AutoFence；public/playground 旧 SPA 退役为 meta-refresh 重定向页（构建产物 5 件 git rm）；prepare-content 增围栏外裸尖括号转义器（P581-D1 预存红清偿，不动 crates 源）"
  - "docs/specs/auto-playground/project.md: 修改——/api/examples 启动探测 notes.json 单一事实源（schema 兼容映射，解析失败/空清单/缺失三路回退目录扫描）；frontend/src/App.vue 宿主换 NotesExplorer+笔记站/IDE 双模式切换（ide-mode→loadExample）"
new_spec_components:
  - "docs/specs/playground-vue/project.md: 新增——useNotes composable（manifest fetch/groups/flatNotes/byId/search，NoteWithGroup 分组归属保留）；NotesExplorer+NotesSidebar 组件族（分组树折叠/计数徽章/搜索过滤/深链 #/notes/<id> replaceState 不触发路由/↑↓ Ctrl+Enter 键盘/来源 chip GitHub blob 链接/类型徽章/--nx-* token=var(--vp-c-*,fallback)/ScrollArea 浮动滚动条）；ExpectedOutputPanel 三态行级对照（toLines=CRLF 统一+去行尾空白+去末尾空行；expectedKind 通道分派 stdout|result——P581-D3 清偿）"
  - "docs/specs/website/project.md: 新增——AutoFence 书页围栏 Run（默认槽保留 shiki 原围栏=收起还原/SnippetRunner autorun/use|import 启发式锁+笔记站链接）；config.ts markdown.config→auto-fence-md.ts fence 钩子（仅裸 ```auto，属性变体不包）；playwright.config+playground-notes e2e 三断言（分组≥4/点击出卡+深链/降级卡）"
  - "docs/specs/auto-playground/project.md: 新增——build-playground-notes.mjs 第二输出 crates/auto-playground/notes.json（byte-identical 双写，gitignore 构建期产物）；App 壳单模式（用户裁定：去顶栏，左侧品牌+笔记导航直选载入，右侧 IDE 直嵌；笔记元信息经 noteMeta 入 IDE 标题栏）"
touched_goals:
  - "goal-014: Playground 在线体验层收官——Notes Explorer 笔记站+三宿主合一+电子书 Run 嵌入（581 组件/manifest 地基的消费闭环；P581-D1 build 预存红顺带清偿）"

affects: [playground-vue, website, auto-playground]   # specs 路径
current_step: 16
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
  [✅ 已完成] useNotes.ts 新建；groups/flatNotes/byId computed 索引 + isLoading/error；vue-tsc -b OK

- **T2 useNotes：搜索**
  文件：`packages/auto-playground-vue/src/composables/useNotes.ts`
  操作：`search(q)`——标题+tags `includes` 匹配，返回命中笔记列表（分组归属保留）。
  验证：`cd packages/auto-playground-vue && npx vue-tsc -b && echo OK`。
  [✅ 已完成] search(q) 实现（title+tags 小写 includes；空查询返回全集；NoteWithGroup 保留分组）；vue-tsc -b OK

- **T3 NotesExplorer：布局骨架**
  文件：`packages/auto-playground-vue/src/components/NotesExplorer.vue`（新建；侧栏可拆 `NotesSidebar.vue`）
  操作：左栏分组树（折叠/计数徽章）+ 右栏笔记头（标题/来源 chip→GitHub 链接/类型徽章/说明折叠）+ PlaygroundCard 挂载；CSS 用 VitePress 变量 + fallback token。
  验证：`cd packages/auto-playground-vue && npm run build`。
  [✅ 已完成] NotesExplorer.vue + NotesSidebar.vue 新建（分组树折叠/计数徽章/搜索框/来源 chip→GitHub blob/类型徽章/说明 details/PlaygroundCard :key=noteId）；CSS --nx-* token=var(--vp-c-*,fallback)；index.ts 导出；npm run build ✓（549ms）

- **T4 NotesExplorer：深链与键盘**
  文件：`packages/auto-playground-vue/src/components/NotesExplorer.vue`
  操作：`#/notes/<id>` 读写（hashchange 监听，不触发 VitePress 路由）；↑/↓ 切换当前笔记、Ctrl+Enter 触发运行（事件转发 PlaygroundCard）。
  验证：`cd packages/auto-playground-vue && npm run build`；`npm run dev` 冒烟（改 hash 定位、键盘切换）。
  [✅ 已完成] hash 读写（replaceState 不触发路由；hashchange 监听外部变更）+ ↑/↓（可编辑目标豁免）+ Ctrl+Enter→PlaygroundCard defineExpose(run)；修复双 watch 竞态与 defineModel 未命名 bug；build ✓；浏览器冒烟：深链 #/notes/demo/01-hello 定位 Hello、搜索 fib→2 组 2 笔记、点击 Fibonacci→hash 写回、合成键盘事件切换+运行转发通过（IAB 真实按键不达页面=环境限制，非代码问题）

- **T5 ExpectedOutputPanel 组件**
  文件：`packages/auto-playground-vue/src/components/ExpectedOutputPanel.vue`（新建）
  操作：三态（未运行/✓ 一致/✗ 差异行级高亮）；比对规则=trim 行尾空白+末尾空行后逐行精确比对。
  验证：`cd packages/auto-playground-vue && npm run build`。
  [✅ 已完成] ExpectedOutputPanel.vue（pending/match/diff 三态 + 行级 ±/· 高亮 + 行号 + 差异计数；toLines=CRLF 统一/去行尾空白/去末尾空行）；index.ts 导出；build ✓（302ms）

- **T6 PlaygroundCard 接期望输出 tab**
  文件：`packages/auto-playground-vue/src/components/PlaygroundCard.vue`
  操作：note.expectedOutput 非空时输出区加"期望输出"tab 挂 ExpectedOutputPanel（实际 stdout 来自 usePlayground 状态）。
  验证：`cd packages/auto-playground-vue && npm run build`；本地起后端 + dev，打开 `01_basics` 任意 vm-golden 笔记冒烟 ✓/✗ 两态。
  [✅ 已完成] PlaygroundCard expectedOutput prop + 条件 'Expected' tab + hasRun 态 + 运行后自动切换对照；NotesExplorer 传 expected-output；build ✓；冒烟（后端 3030 + dev 5199）：vm-basics/hello 未运行态提示→运行 ✓"输出一致（1 行）"→改代码运行 ✗"输出有差异（1 / 2 行不一致）"+ 行级 +EXTRA 高亮

- **T7 项目型笔记文件 tab**
  文件：`packages/auto-playground-vue/src/components/PlaygroundCard.vue`
  操作：kind=project 时文件 tab（entry 锁 `main.at`、多文件切换编辑）；运行请求构造复用 `usePlaygroundFull` 的 files 形态。
  验证：`cd packages/auto-playground-vue && npm run build`；笔记站（或 dev 页）打开 `demo/09-multi-module` 冒烟多文件运行。
  [✅ 已完成] usePlayground 增 projectRequestBody（run/transpile/transpileAll 三处接入）+ projectFiles/projectDir 透出；PlaygroundCard 文件 tab 条（main.at 锁标+entry 徽章）+ fileBuffers 切换回写 + syncProjectRun；NotesExplorer 从 sourcePath 推导 projectDir；build ✓；冒烟：multi-module 双文件 tab 渲染、运行输出 hello（files 形态生效）、切换 main→helpers→main 内容保持（use helpers / pub fn greet 往返）

- **T8 "在 IDE 中打开"切换入口**
  文件：`packages/auto-playground-vue/src/components/PlaygroundCard.vue`、`NotesExplorer.vue`
  操作：按钮触发 `ide-mode` 事件；SPA 宿主（后端 frontend）渲染 `AutoPlaygroundFull` 并 loadExample 对应笔记；VitePress 宿主无后端时禁用+提示。
  验证：`cd packages/auto-playground-vue && npm run build`。
  [✅ 已完成] PlaygroundCard ideMode prop（null 不渲染/false 禁用+tooltip/true 点击 emit）+ NotesExplorer ide-mode 重发（载荷 noteId/source/projectDir/files，NoteFile→ProjectFile 映射）；SPA 宿主接线归 T14；build ✓；冒烟：禁用态按钮+提示文案渲染正确

- **T9 无后端降级态**
  文件：`packages/auto-playground-vue/src/composables/usePlayground.ts`、`components/SnippetRunner.vue`、`components/PlaygroundCard.vue`
  操作：首个请求失败置 `backendDown`（可重试）；动作位降级为引导卡（`cargo run -p auto-playground` + 部署说明锚点）。
  验证：`cd packages/auto-playground-vue && npm run build`；`cd website && npx vitepress preview --port 4173 &` 后浏览器开 `/playground`（无后端）确认浏览完整、Run 位引导。
  [✅ 已完成] usePlayground backendDown（run/runCode/transpile 网络异常置位、成功清除）+ retryBackend 探测；SnippetRunner 动作条 WifiOff+禁用+输出区降级卡（替换 581 临时提示）；PlaygroundCard Run 按钮降级（"后端离线"）+ 输出区降级卡 + 重试；build ✓；冒烟（dev 宿主）：杀 3030→Run→降级卡+52 组浏览完整→重启后端→重试连接→降级消失→复跑 ✓"输出一致"（website preview 无后端路径并入 T10 冒烟）

- **T10 website /playground 改造**
  文件：`website/playground.md`、`website/zh/playground.md`、`website/.vitepress/theme/index.ts`（如需注册全局组件）
  操作：内嵌 `<NotesExplorer />` 替换 `<AutoPlayground />`，保留头部与后端引导块；EN/ZH 同步。
  验证：`cd website && npm run build`；`npm run dev` 冒烟（树/搜索/深链）。
  [✅ 已完成] playground.md（EN/ZH）换 NotesExplorer + 后端引导块加 id="backend" 锚点（降级卡链接目标）；theme/index.ts 注册全局组件；附带修复 P581-D1 预存红：prepare-content 增通用裸尖括号转义器（围栏/行内代码/Listing 管线跳过+合法 HTML 允许清单——Map<tableKey…>/<prefix>_pts/N<slot>/<msg> 全族覆盖，只改复制产物不动 crates 源），website build 由红转绿（68.99s）；preview 4173 冒烟：EN 深链定位 Hello/52 组/搜索 fib→2 组/Run→降级卡+后端离线按钮/IDE 禁用；ZH 页同验 ✓

- **T11 旧静态页退役**
  文件：`scripts/build-playground.mjs`、`website/public/playground/`（清空）、`website/public/playground/index.html`（新重定向页）
  操作：mjs 删除 website 同步分支（保留后端 dist 分支）；public/playground 清空后提交仅含 meta-refresh → `/playground` 的 index.html。
  验证：`cd website && npm run build && grep -i refresh .vitepress/dist/playground/index.html public/playground/index.html`。
  [✅ 已完成] build-playground.mjs website 同步分支删除（保留后端 dist）；git rm 旧 SPA 5 产物（assets×2/favicon/icons/旧 index）→ 新 index.html 仅 meta-refresh+/playground 链接（双语+noindex）；build ✓ grep refresh 双命中；浏览器实测：/playground/ 浏览器路径经 VitePress 客户端路由归一 /playground#… 直接渲染笔记站（无 404/无循环），/playground/index.html 直访 410B 重定向页；GH Pages 静态托管目录优先行为留待部署观测（待澄清③在案）

- **T12 manifest 第二输出**
  文件：`scripts/build-playground-notes.mjs`、`crates/auto-playground/.gitignore`（无则新建）
  操作：脚本增加输出 `crates/auto-playground/notes.json`（同内容）；gitignore 加 `notes.json`。
  验证：`node scripts/build-playground-notes.mjs && cmp website/public/playground-data/notes.json crates/auto-playground/notes.json && echo SYNCED`。
  [✅ 已完成] build-playground-notes.mjs 双写（同一 JSON 串，byte-identical）；crates/auto-playground/.gitignore 新建（notes.json）；vm=461/aavm=158/book=634/demo=28；cmp → SYNCED

- **T13 /api/examples 读 manifest**
  文件：`crates/auto-playground/src/routes/examples.rs`
  操作：启动探测 `CARGO_MANIFEST_DIR/notes.json`，存在→serde 反序列化并映射为现有 `Example` 响应（schema 兼容）；缺失→现有目录扫描回退。
  验证：`cargo check -p auto-playground`；起服 `curl -s http://127.0.0.1:<port>/api/examples | jq '.examples | length'` 与 manifest 笔记总数一致；删 notes.json 重启后回退可用。
  [✅ 已完成] examples.rs 增 NotesManifest 反序列化（serde rename sourceType/sourcePath）+ load_from_manifest 映射（fence→single、project_dir 从 sourcePath 推导、entry=main.at、空源跳过、解析失败/空清单 warn 回退）+ load_from_dir_scan 保留原扫描；cargo check 零警告（auto-playground 自身；172 警告为 auto-lang 依赖预存）；起服验证：/api/examples=1281=manifest 总数（project=4，日志"读 notes.json（1281 条"）；删 notes.json 重启→回退 28 条目录扫描可用；恢复→1281（jq 缺席改用 node 等价比对）

- **T14 后端 frontend 换宿主**
  文件：`crates/auto-playground/frontend/src/App.vue`
  操作：默认渲染 `NotesExplorer`，顶部"IDE 模式"切换渲染 `AutoPlaygroundFull`（接 T8 事件）；`build-playground.mjs` 后端 dist 同步分支保持。
  验证：`cd crates/auto-playground/frontend && npm run build`；`cargo run -p auto-playground` 打开页面冒烟（笔记站 + IDE 切换 + 项目笔记运行）。
  [✅ 已完成] App.vue 重写（顶栏 笔记站/IDE 模式 双态切换；NotesExplorer ide-mode → onIdeMode → nextTick loadExample）；AutoPlaygroundFull 增 defineExpose(loadExample)；ide ref 用结构化类型（InstanceType 触发宿主 vue-tsc 过深实例化，P581-D2 家族）；frontend npm run build ✓（482.95kB js）；cargo run 冒烟：3030 根页=笔记站（52 组/arithmetic/IDE 按钮可用）→ demo/09-multi-module 点"在 IDE 中打开"→ IDE 模式激活+use helpers 载入→Run→Hello 输出（注：后端须在 dist 已构建后启动，dist 缺失时走 npm dev 分支会因 PATH 无 npm 降级 404——重启即恢复）

- **T15 电子书围栏 Run**
  文件：`website/.vitepress/theme/components/AutoFence.vue`（新建）、`website/.vitepress/theme/index.ts`
  操作：lang=auto 围栏包裹组件（▶ Run/展开 SnippetRunner autorun/收起还原/`import` 锁提示+链接笔记站）；theme 接线 markdown fence 渲染。
  验证：`cd website && npm run build`；`npm run dev` 打开 rust 书 `ch01-getting-started` 对应页面冒烟（Run 展开运行；`.cn` 版同验一页）。
  [✅ 已完成] AutoFence.vue（默认槽保留原 shiki 围栏=收起还原；SnippetRunner autorun 展开；import/use 启发式锁=manifest isStandalone 同规则+笔记站链接）；fence 钩子经 config.ts markdown.config→auto-fence-md.ts（markdown-it renderer.rules.fence 仅裸 ```auto 包裹，属性变体不包；plan 写 theme/index.ts，实际挂点在 config.ts——VitePress 1.6 唯一 markdown-it 注入口，theme 侧只注册组件）；修复 :code 误绑编码 prop 的 bug（改绑解码 source）；build ✓（55.51s）；dev 5188 冒烟：ch01 三围栏 3 Run（前两个教学片段合法报错、第三个 Hello World 输出 "Hello, world!"）、收起还原 display:block、.cn 版 3 Run 同验；byte-of-python/ch08 锁态 2 锁（含 /playground 链接）+1 Run

- **T16 e2e 最小集与验收清单**
  文件：`website/tests/`（新增 playground-notes spec，命名随现有惯例）
  操作：三条断言（分组渲染 ≥4/笔记点击出卡片/无后端降级态可见；后端相关仅本地起服启用）；逐条跑"验收标准"1–8 并记录。
  验证：`cd website && npm run test:e2e -- playground-notes`（本地无后端路径）+ 验收清单全绿。
  [✅ 已完成] website/playwright.config.ts 新建（preview 4173 webServer+reuse）+ tests/playground-notes.spec.ts 三断言；首跑 3 failed 定位为陈旧 preview 实例 assets ERR_ABORTED（杀旧服后 playwright 自起干净实例）→ 3 passed（17.6s）；全量 test:e2e 中 spa-routes 5 失败为预存红（master 同源：public/ui/gallery/index.html 标题已漂移 "widgets-gallery"，与本计划无关，见待澄清④）；test-results/playwright-report 入 website/.gitignore
  [验收清单] ① EN/ZH Notes Explorer+52 组+徽章+搜索+深链 ✓（e2e+浏览器），旧 /playground/ 重定向页+客户端路由归一 ✓ ② 静态预览全笔记可浏览+降级引导卡 ✓（e2e 断言3） ③ vm-basics/hello 运行后 ✓"输出一致"→改码 ✗"+EXTRA"行级高亮 ✓（后端 3030 在位） ④ ch01 EN 3 围栏 3 Run、Hello World 实跑输出、收起还原；.cn 版 3 Run 同验；byte-of-python/ch08 import 围栏 2 锁+笔记站链接 ✓ ⑤ 3030 根页=笔记站（52 组），IDE 切换载入 multi-module 并运行输出 ✓ ⑥ /api/examples=1281=manifest 总数（project=4）；删 notes.json 回退 28；恢复 1281 ✓ ⑦ 三门禁全绿零警告（auto-playground 自身；172 警告为 auto-lang 依赖预存）✓ ⑧ build-playground.mjs website 分支删除；旧 SPA 产物 git rm 仅存 meta-refresh index.html ✓

## 用户裁定变更全录（2026-09-07 会话，复审后增改批次）

以下为复审通过后用户在会话中逐一提出的修改（每条含落点提交与验证方式），全部已实现并验证：

| # | 用户裁定 | 落点提交 | 验证 |
|---|---|---|---|
| U1 | 侧栏滚动条换 ScrollArea 组件（弃原生滚动条） | 358caa792 | 浏览器实测 scroll-area 结构生效 |
| U2 | 右侧内容占满高度、消除下方空白（后演变为双列全高对齐+呼吸间距） | 358caa792 + dfcaf9117 | 实测 gap=0；三底边 704/704/704 对齐 |
| U3 | 期望输出对照修双语义假阳性（复审发现，随轮修复） | 358caa792 | arithmetic(result)/hello(stdout) 双通道 ✓ |
| U4 | 后端壳单模式化：去顶栏双模式切换，右侧直嵌 IDE，左侧导航保留 | dfcaf9117 | 实测单模式布局+IDE 全功能 |
| U5 | 笔记元信息（题名/badge/来源链接）并入 IDE 标题栏（noteMeta prop） | dfcaf9117 | 实测标题栏三件套联动 |
| U6 | "Auto Playground" 品牌移入侧栏搜索框上方 | dfcaf9117 | 实测品牌行渲染 |
| U7 | 双列占满窗口高度（76vh 上限在确定高度壳内解除） | dfcaf9117 | 实测 bottom 对齐 |
| U8 | 侧栏树三级归类：Playground Demo / 书籍示例 / 测试用例（parity 后加为第四级） | 73e8d47dc + 0cb047cc8 | 实测 28/634/619/51 四分类 |
| U9 | 书籍展示序：tapl(官方)→Rust→Think Python→Byte of Python→Typescript→Typescript Deepdive→Little C→Modern C | e7d1a0a3f | 实测顺序 8/8 精确 |
| U10 | 书内增章节目录层（每章一个目录，笔记剥章节前缀显示） | e7d1a0a3f | 实测 Rust 15 章排序/展开/选择联动 |
| U11 | 标题栏紧凑化：示例名与 git 链接合为单链接（名字+外链 icon，路径收 tooltip） | df18c9e83 | 实测链接/图标/tooltip |
| U12 | 删除 ExampleSelector 下拉（导航已在侧栏） | df18c9e83 | 实测仅剩转译目标功能件 |
| U13 | Load Replay 暂隐藏（功能未完善；emit 链路与测试钩子保留） | df18c9e83 | 实测按钮消失 |
| U14 | 工具栏排序 Run→Trans→Debug→Share | 68684f289 | DOM 读序精确匹配 |
| U15 | parity 语料收录为顶级分类 "Parity Demo"（与测试用例并排，内分 Rust/Python 等家族） | 0cb047cc8 | 实测五家族 51 条渲染 |
| U16 | parity 笔记可运行（files-only 物化运行，修 Module not found） | afd96114a | 实测 base64·Decode 10 ok/0 not ok |
| U17 | 品牌标题加紫色闪电图标（favicon 同形）+indigo 文字+字号加大 | a47112bb4 | 截图确认 |
| U18 | AutoUI 迁移（Vue 手写→Auto 语言双端化）写入设计文档，本期不执行 | 041c0ed1e | playground-architecture.md §12 |

附随工程修正（非用户直接提出，随轮落地并在复审记录留痕）：expectedKind 双语义分派（U3 同轮）、首条自动载入移除（覆写竞态破坏宿主 e2e 确定性）、bytecode-meta 几何断言改对 IDE 容器、examples.rs entry files[0] 回退（保 api=manifest 不变量）。

## 复审记录

- **复审人**：ZCode（/auto-plan:review，2026-09-07；同日修正轮复验）
- **复审方式**：worktree（`.wt/lang-582/auto-lang`）实代码逐条复验——门禁全部重跑、浏览器实测复跑、回退路径重演；不信任执行期勾选。
- **首验结论**：不通过——标准③对 183 条 result 语义 vm-golden 笔记呈假阳性 ✗（P581-D3 预警项在计划与实现中双遗漏）；其余 7 条 pass。
- **修正轮（同日）**：用户另报两处 UI 问题（侧栏原生滚动条/卡片下方残白）随轮一并修复。commit 358caa792：manifest `expectedKind` 判别字段（--check 幂等绿 278+183=461）+ ExpectedOutputPanel `actualResult` 通道分派 + NotesSidebar 换 ScrollArea 浮动滚动条 + 卡片 flex 拉伸消除底边残白（实测 gap=0）。复验：arithmetic/hello 双通道 ✓ 一致、e2e 3/3、包/website 构建绿。
- **用户裁定增改（同日，commit dfcaf9117 + 73e8d47dc + df18c9e83 + e7d1a0a3f）**：后端壳单模式化——去顶栏双模式切换，右侧直嵌 AutoPlaygroundFull（IDE），笔记元信息（题名/类型 badge/来源 chip→GitHub）经 PlaygroundLayout 新 `noteMeta` prop 并入 IDE 标题栏；"Auto Playground" 品牌经 NotesSidebar 新 `title` prop 移入侧栏搜索框上方；侧栏选笔记即载入 IDE。双列占满全高（bottom 704/704 对齐 + 16px 呼吸距）。侧栏树三级归类（用户裁定）：Playground Demo(28)/书籍示例(634，8 书组子级)/测试用例(619，vm 43 组+aavm 组子级)——manifest 平铺单一事实源不变，纯展示层 sections 推导，section+组两级独立开合、活动笔记自动展开、搜索态强制全开）。书籍层二次细化（用户裁定）：展示序 tapl(官方书籍)→Rust→Think Python→Byte of Python→Typescript→Typescript Deepdive→Little C→Modern C，书内增章节目录层（chNN stem 聚合+numeric 排序+笔记剥章节前缀显示）。标题栏紧凑化（用户裁定）：示例名与 git 链接合而为单链接（名字+外链 icon，路径收 tooltip）、ExampleSelector 移除（导航在侧栏）、Load Replay 暂隐藏（emit 链路与测试钩子保留）。附带：去首条自动载入（与宿主 e2e 初始交互存在覆写竞态，致 10 失败；移除后 frontend 套件 19/19 绿）+ bytecode-meta 几何断言改对 IDE 容器右缘（侧栏布局用户裁定后的合理更新）。VitePress 宿主不受影响（NotesExplorer 卡片式保留；`height:100%` 对 auto 父安全回退）。复验：frontend e2e 19/19、website e2e 3/3、website build 87.05s 绿、浏览器实测书籍序/章节展开/选择联动全过。。parity 语料收录落地（用户裁定，commit 0cb047cc8）：manifest 增 parity 五家族 51 条（Rust Cookbook 13/Python 20/Lang 8/Consumer 9/Framework 1，sourceType=parity，standalone=false——lib import/python FFI 外部依赖），侧栏顶级第四分类 "Parity Demo"（与测试用例并排，家族为子目录）；examples.rs entry 增 files[0] 回退保住 api=manifest 不变量（1332=1332，project=35）。Run 行为：python 单文件需后端 --features python（FFI），.at 多文件族需 lib import 解析——本迭代可浏览、Run 如实报依赖错误（581 勘察在案），后端 FFI 仍为后续计划；**多文件运行能力已本轮补齐**（commit afd96114a）：run 路径新增 files-only 物化（prepare_files_temp_dir 嵌套写盘 + 根 entry main.at——run_with_capture_and_path 以 entry 父目录种 source_dirs，temp 根使 auto/<lib>.at 可解析），前端两 composable 无 project_dir 时也随请求发 files/entry；实证 base64 · Decode 浏览器 Run = 10 ok/0 not ok（与 581 勘察封闭全绿一致）。backend 宿主 e2e 19/19（debug-vs-run 1 例全量并发偶发、单跑两度过——已知 flaky 非本轮引入）。
**终态：8/8 pass，无阻塞债 → `status: reviewed`，可进入 `/auto-plan:merge`。**

### 复审发现的阻塞缺陷（标准③ fail）

P581-D3 明文要求："582 期望输出对照 UI 需按 sourcePath 后缀分派（.result 对照 RunResponse.result、.out 对照 stdout），否则差异高亮会误报"。计划 G3/§3 只写了 stdout 比对（计划对已知债遗漏），执行按计划实现——复审实证假阳性：

- `vm-basics/arithmetic`（code=`1 + 2`，expectedOutput="3" 取自 `.expected.result`）：实跑 stdout=""、result="3" → 面板判"输出有差异"（应为 ✓ 一致）。文件系统实测 out=275 / **result=183** 条，全部误报。
- 根因：manifest 未携带判别字段（sourcePath 恒为 `.at`，客户端无法区分期望语义），面板无条件取 stdout。

**修复清单**（/auto-plan:work）：
1. `scripts/build-playground-notes.mjs` collectVmGolden：note 增 `expectedKind: 'stdout' | 'result'`（按 expectedPath 后缀判定）；重新生成两处 manifest。
2. `packages/auto-playground-vue/src/types.ts` NoteMeta 增 `expectedKind?: 'stdout' | 'result'`。
3. `ExpectedOutputPanel.vue` props 增 `actualResult`，按 expectedKind 选 actual（缺省 'stdout' 兼容）；`PlaygroundCard.vue` 把 resultCode 透传给面板；NotesExplorer 传 note.expectedKind。
4. 复验：arithmetic → ✓"输出一致"（result 通道）；hello → ✓（stdout 通道）；`node scripts/build-playground-notes.mjs --check` 幂等绿；三门禁重跑。

### 验收标准逐条裁定（首次复审）

| # | 标准 | 裁定 | 复验证据 |
|---|---|---|---|
| 1 | EN/ZH Notes Explorer + 旧 URL 重定向 | **pass** | e2e 复跑 3/3（分组≥4+徽章断言）；浏览器实测 EN 深链定位/搜索 fib→2 组/ZH 页 52 组；`/playground/` 客户端路由归一渲染笔记站，`index.html` 直访 meta-refresh |
| 2 | 无后端全笔记可浏览+降级引导 | **pass** | e2e 断言 3（降级卡含启动命令+浏览不受影响）复跑绿 |
| 3 | vm-golden 期望输出 ✓/✗ 对照 | **pass**（修正后复验） | 首验 fail（.result 语义 183 条假阳性）；修正轮 manifest 增 `expectedKind`（278 stdout/183 result，--check 幂等绿），面板按语义分派通道：arithmetic（result）→✓"输出一致"、hello（stdout）→✓"输出一致"均实测 |
| 4 | 书页围栏 Run/收起还原/import 锁 | **pass** | 新 dist preview 实测：.cn 页 3 围栏 3 Run、收起还原 display:block；byte-of-python/ch08 2 锁+笔记站链接；完整运行输出执行期经 dev 代理已证（"Hello, world!"） |
| 5 | 后端自服务页同体验+IDE 切换 | **pass** | 3030 根页=笔记站（52 组）→ multi-module"在 IDE 中打开"→IDE 激活+载入实证 |
| 6 | /api/examples=manifest+回退 | **pass** | 1281=manifest 总数（project=4）；删 notes.json 重启→28；恢复→1281 |
| 7 | 三门禁全绿零警告 | **pass** | 包 build 320ms/website 44.75s 绿；cargo check -p auto-playground 本体零警告（111 警告全在依赖 auto-lang，master 同在=预存）；e2e 3/3 |
| 8 | mjs 无 website 分支+无 SPA 残留 | **pass** | mjs 仅退役注释；`git ls-files website/public/playground/` 仅 index.html；diff 无 TODO/FIXME/HACK |

### 遗漏/延后/Workaround 清查

- **改动范围**：diff 仅触 `packages/auto-playground-vue`、`website`、`scripts`、`crates/auto-playground`——与 affects 一致，未触 `crates/auto-lang`（禁跑 tf 门禁正确适用）。
- **计划文本遗漏（本次 fail 根因）**：P581-D3 的 expectedKind 分派要求未进入计划 G3/§3——计划起草时未吸收已知债条目。修复随上面清单，属计划缺陷非执行走样。
- **范围内部新增（已记账非 silent）**：① prepare-content 裸尖括号转义器（P581-D1 清偿，验收⑦必要）；② AutoPlaygroundFull defineExpose(loadExample)（T14 最小 API）。
- **延后项**：待澄清②③均为计划预授权的部署期事项。
- **workaround 扫描**：无 TODO/FIXME/HACK。

### 债务候选（已登记 KNOWN-DEBT）

- **P582-D1** spa-routes e2e 5 失败 master 预存红（/ui/* 资产标题漂移，非本计划引入）
- **P582-D2** 键盘 ↑/↓ 真实按键未经真机验证（IAB 沙箱限制；合成事件全链路已证）
- **P582-D3** 后端 dist 缺失时启动走 npm dev 分支的降级路径（PATH 无 npm 时静态页 404）
- **P582-D4** Playground 前端 Vue→AutoUI 双端化迁移（用户裁定未来项；设计全文入
  playground-architecture.md §12——复刻清单/依赖映射/三期路线/CodeMirror 等价性风险；本期不执行）

### 分支状态

分支 `plan-582-dev` 基于 42ee4e7cb，master 已前进 11 提交——diff 中 stdlib/auto/term*.at"删除"为漂移伪象（master 新增，非本分支删除），fold 时由 merge 处理。

## 待澄清事项

- **② "在 IDE 中打开"的 VitePress 宿主形态**（T8 落地时定）：无后端时禁用+提示是底线方案；若后端与 website 同域部署则可直接链后端 UI。
  → T8 落地：底线方案（禁用+tooltip 提示 `cargo run -p auto-playground`）；同域部署形态留待部署期。
- **电子书围栏 ↔ manifest 笔记互链**（Playground 设计 §11 演进项）：本期不做，仅锁提示链接到笔记站。
- **`/playground/` 旧 URL 的外部引用**（部署历史链接）：重定向页兜底；若 GitHub Pages 路由行为异常改 `_redirects`/404 兜底（T11 验证时裁定）。
  → T11 实测：直访 index.html meta-refresh 生效；浏览器无扩展名路径经 VitePress 客户端路由归一渲染笔记站（无 404/无循环）。GH Pages 真实托管行为留部署期观测，重定向页已兜底。
- **④（执行期新增）spa-routes e2e 5 失败为 master 预存红**：`public/ui/gallery/index.html` 等旧 SPA 资产标题已漂移（"widgets-gallery" ≠ 期望 "Auto Language - Components"），与本计划无关（worktree diff 仅触 public/playground/）。建议单独 L0 计划重生成 /ui/* 资产或修标题断言。
- **⑤（执行期新增）键盘 ↑/↓ 真实按键未在 IAB 环境验证**：合成 KeyboardEvent 全链路通过（监听→切换→hash 写回），IAB 沙箱不派发真实 CDP 按键到页面（探针证实环境限制）。真实键盘行为留 review/用户验收时人工确认。
- **⑥（执行期新增）后端启动时序**：`cargo run -p auto-playground` 在 frontend/dist 缺失时会尝试拉起 npm dev（PATH 无 npm 时降级为仅 API、静态页 404）。先 `npm run build`（或 scripts/build-playground.mjs）再起服即正常；主检出日常路径不受影响。
