---
plan_id: PLAN-754
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: pg-unify
author: [agent]
created_at: 2026-10-10
updated_at: 2026-10-10
plan_revision: 1

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [playground-vue, auto-playground, website]
current_step: 0
total_steps: 5
---

# [PLAN-754] playground 双端统一：运行契约对齐 + IDE 深链桥

## 0. 变更摘要

统一 playground 的两个消费面——**独立版**（auto-playground 后端自服务 SPA，
`frontend/src/App.vue` = NotesSidebar + AutoPlaygroundFull IDE）与 **website 版**
（VitePress `/playground` 页 = NotesExplorer + PlaygroundCard 卡片）——在两条
已知的分裂点上收口：

1. **运行契约分裂**：website 卡片链（SnippetRunner → usePlayground）不发
   `prepend_lib`（PLAN-752 只接了 usePlaygroundFull）→ vm-bootstrap 组笔记在
   website 跑不了（`Undefined symbol`），独立版已可跑。统一：usePlayground
   补对称通道，卡片链按 note.id 组前缀透传。
2. **IDE 桥休眠**：组件层 `ideMode`/`ide-mode` 契约早已存在
   （NotesExplorer.vue:79/89/150，payload `{noteId, source, projectDir, files}`），
   但 website 端无 props → 卡片显示禁用的"在 IDE 中打开"；独立版 App.vue 无
   URL 深链，即便有桥也无处可跳。统一：website 有可达后端时桥激活并跳
   独立 SPA 笔记深链；App.vue 支持 `?note=<id>` 深链直接载入。

**非统一项（历史裁定保留）**：两端的宿主形态不动——sidebar+IDE（独立版，
dfcaf9117 用户裁定）与 Explorer+Card（website，Plan 582 设计）各自保留；
不重做 UI、不动 VitePress 结构、不合并 manifest 管线（notes.json 三输出
已是单一事实源）。

## 1. 目标

- G-1：双端运行行为一致——同一笔记在 website 与独立版产出相同结果
  （含 vm-bootstrap 前置拼接、60s 超时）。
- G-2：website 用户一键进 IDE：卡片"在 IDE 中打开"在有可达后端时可用，
  跳转独立 SPA 并直接载入该笔记。
- G-3：深链可分享：`http://<backend>/?note=<id>` 打开即载对应笔记。

## 2. 架构方案

```
website /playground（VitePress，静态）
  NotesExplorer ──探测 /api 可达──▶ ideMode = true
    PlaygroundCard ──note.id 前缀──▶ SnippetRunner(prependLib)
      │ Run/Trans（契约与独立版对齐：60s + prepend_lib）
      └─ "在 IDE 中打开" ──emit ide-mode──▶ 跳 <backend-origin>/?note=<id>
                                                │
独立 SPA（App.vue）                              ▼
  启动解析 ?note= ──useNotes.byId──▶ 自动 select+loadNote（现有侧栏载路）
```

- `usePlayground` 增 `setPrependLib`（与 Full 对称）；`SnippetRunner` 增
  `prependLib?: boolean` prop → 请求体；`PlaygroundCard` 由 `note-id` 前缀
  （已有 prop，NotesExplorer.vue:48 传入）派生 `prependLib` 传入 SnippetRunner。
- NotesExplorer 增 `backendUrl?: string`（探测/桥目标）；website playground.md
  传入并由轻量探测（fetch `${api}/examples`，复用 usePlayground 的
  backendDown/retryBackend 语义）决定 ideMode 与跳转 origin；`@ide-mode` 处理
  = `window.open(`${backendOrigin}/?note=${payload.noteId}`)`。
  探测失败保持现状（ideMode=false + 降级提示，零回归）。
- App.vue 启动时读 `URLSearchParams(location.search).get('note')` → byId 命中则
  onSelect（复用现有点选载路，不新造载入逻辑）；无效 id 静默回退欢迎态。

## 3. 技术栈

Vue3/TS（组件包 + 两宿主）、Rust 不动（后端契约 PLAN-746/752 已就绪）、
website VitePress 页 props 接线；验证＝vue-tsc + dist 构建 + e2e（standalone
深链 + 卡片请求体断言沿用 run-timeout.spec 拦截模式）+ website 构建冒烟。

## 4. 需求分析与背景调查

**授权**：用户 2026-10-10 "尝试把 playground 的独立版和 website 版统一"——
标准四技能流程 + worktree（lang-754 组），范围以本合同为准（非目标清单
即保留裁定的边界）。

**现状锚点**（主检出 master @ f3a7548b5 调查）：
- 契约分裂实证：`usePlayground.ts` projectRequestBody 只有 `timeout_secs=60`
  （PLAN-746），无 prependLib；`SnippetRunner.vue:124-139` 用 usePlayground；
  `PlaygroundCard.vue` 持有 `:note-id`（NotesExplorer.vue:48）但未派生组标志。
- 桥契约现存：NotesExplorer `ideMode?: boolean`（默认 false）+ emit
  `ide-mode {noteId, source, projectDir, files}`（:79/:89/:150）；
  PlaygroundCard 渲染 IDE 按钮（ideMode !== null 时，:14-20）。
- website 接线：`website/playground.md:12` 裸 `<NotesExplorer />`——无
  ideMode/无 apiBase/无 @ide-mode。
- 独立版无深链：`frontend/src/App.vue` 无 location/hash 解析（grep 实证）；
  载入路 = onSelect → loadNote（App.vue:71-76）。
- 数据源已统一：notes.json 三输出（website/public、backend、frontend/public）。

**历史裁定（非目标依据）**：dfcaf9117"后端壳单模式化——去顶栏直嵌 IDE（用户
裁定）"；Plan 582 三层组件（Snippet→Card→Full）是刻意分层，能力差异非缺陷。

## 5. 详细设计

### 运行契约对齐（G-1）

- `usePlayground`：`const prependLib = ref(false)` + `setPrependLib`（导出）；
  projectRequestBody 追加 `if (prependLib.value) body.prepend_lib = true`。
- `SnippetRunner`：`prependLib?: boolean`（默认 false）→ setup 调
  `setPrependLib(props.prependLib)`（watch 同步 prop 变化）。
- `PlaygroundCard`：`const prependLib = computed(() => props.noteId?.startsWith('vm-bootstrap/'))` 传入 SnippetRunner。
- 验收抽样：website 链路下 a2r_hello 卡片运行输出 = `ok`（独立版同源）。

### IDE 深链桥（G-2/G-3）

- `NotesExplorer` 增 `backendUrl?: string`（默认 ''）：空 → ideMode 恒 false
  （现状）；非空 → mount 时 `fetch(backendUrl + '/api/examples')` 成功则
  ideMode=true（失败保持 false，不弹错——复用静默降级哲学）。
- website `playground.md`：`<NotesExplorer backend-url="" />` 静态页不能硬编
  → 传空由页内脚本注入？**简化裁定**：VitePress 页内 `<script setup>` 不行
  （markdown）；改由 theme 层包裹组件 `website/.vitepress/theme/components/
  PlaygroundPage.vue`：探测 `location.origin`（部署形态 website 与后端同域）
  + fallback `http://127.0.0.1:3030`；`@ide-mode` → `window.open(
  `${backendOrigin}/?note=${noteId}`, '_blank')`。playground.md 换用
  `<PlaygroundPage />`。
- App.vue 深链：onMounted 读 `?note=` → `byId` 命中 → `activeNoteId`+`loadNote`；
  深链命中后可选 `history.replaceState` 清参（防刷新重复载入——载入幂等，
  可不清理，取不清理）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/playground-vue/project.md | before：prependLib 仅 usePlaygroundFull 通道；after：双 composable 对称支持，卡片链按 note-id 组前缀自动派生 | 双端契约一致 | AC-01 |
| SD-02 | add | docs/specs/playground-vue/project.md | ideMode 桥契约补全：backendUrl 探测激活 + ide-mode 跳 `<backend>/?note=<id>` 深链；App.vue 启动解析 ?note 载入 | 双端入口互通可分享 | AC-02, AC-03 |
| SD-03 | add | docs/specs/website/project.md | /playground 页经 PlaygroundPage 包裹：后端探测（同源→127.0.0.1:3030 fallback）决定 IDE 桥可用性 | website 模块现状记录 | AC-02 |

## 6. 测试设计

- 组件 e2e（standalone frontend/tests 新增 unify spec）：
  ① `?note=demo/04-fibonacci` 打开 → 编辑器自动载入斐波那契（标题栏断言）；
  ② 卡片链契约：PlaygroundCard 渲染 vm-bootstrap 笔记时 /api/run 请求体含
  prepend_lib=true（组件级挂载或经 NotesExplorer 集成态）；
  ③ 非 bootstrap 笔记不含 prepend_lib。
- website 冒烟：`npm run docs:build`（或 vitepress build）过 + 页面渲染
  NotesExplorer（构建期即锚）；桥激活为运行时行为，由 ② 的组件级测试覆盖
  契约，运行时探测不单独 e2e（静态部署依赖后端在场）。
- 回归：run-timeout.spec 7/7 不动全绿（Full 链路与本计划正交）。

## 7. 验收标准

| id | 标准 | 验证方法 |
|---|---|---|
| AC-01 | website 卡片链运行 vm-bootstrap 笔记（a2r_hello 抽样）输出 `ok` 且请求体含 prepend_lib=true；非 bootstrap 笔记请求体不含 | e2e ②③ + 对照独立版输出 |
| AC-02 | 后端在场时 website 卡片"在 IDE 中打开"可点击并打开 `<backend>/?note=<id>`；后端缺席时保持禁用+提示（零回归） | 组件级测试 + 人工走查 |
| AC-03 | `http://127.0.0.1:3030/?note=demo/01-hello` 打开即载 Hello 笔记可直接 Run | e2e ① |
| AC-04 | 双端宿主形态未变（App.vue 仍 sidebar+IDE；website 仍 Explorer+Card）；既有 e2e 23/23 + plan746/752 测试全绿 | 全量 e2e + 单测 |
| AC-05 | 复审门禁：裸 `cargo t` 红集无新增（本计划零 Rust 改动应为恒等）、vue-tsc/dist 构建过、spec 增量落库 | 复审档 |

## 8. 执行步骤

- [ ] T-01 契约对称：usePlayground.setPrependLib + SnippetRunner prop +
  PlaygroundCard 组前缀派生。文件：packages/auto-playground-vue（composables/
  usePlayground.ts、components/SnippetRunner.vue、PlaygroundCard.vue）。
  验证：vue-tsc + 组件 e2e ②③。AC-01
- [ ] T-02 独立版深链：App.vue `?note=` 解析载入。文件：frontend/src/App.vue。
  验证：e2e ①。AC-03
- [ ] T-03 website 桥：PlaygroundPage.vue 包裹（探测+跳转）+ playground.md
  接线 + NotesExplorer backendUrl。文件：website/.vitepress/theme/components/
  PlaygroundPage.vue（新）、website/playground.md、zh 版、NotesExplorer.vue。
  验证：vitepress build + AC-02 组件级断言。AC-02
- [ ] T-04 全量 e2e + 回归：frontend/tests 全量 23+新 spec、plan746/752 单测。
  验收档：scratch/playground-check/p754-unify-report.md。AC-04
- [ ] T-05 复审门禁：cargo t 红集对拍（预期与 master 恒等）、vue-tsc、
  spec 增量落库（merge 档）。AC-05

## 9. 复审记录

- 2026-10-10 `/auto-plan:new` r1 起草：`stage: new`，PLAN-754 revision 1。
  `outcome: pass`（用户已批统一诉求，授权范围内可开工；非目标清单=保留
  历史裁定的边界）。`next: work`。Q-1：website 桥探测的同源/fallback 顺序
  实现期按部署实况（deploy/nginx-auto-playground.conf）复核。

## 10. 待澄清事项

- Q-1：website 生产部署中静态站与后端的域关系（同源 nginx 反代？分域？）——
  决定 backendUrl 探测顺序；实现期读 deploy/nginx-auto-playground.conf 定谳，
  若分域则需网站构建期注入配置（不在本计划则按同源+localhost fallback）。
