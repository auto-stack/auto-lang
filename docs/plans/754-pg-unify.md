---
plan_id: PLAN-754
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: pg-unify
author: [agent]
created_at: 2026-10-10
updated_at: 2026-10-10
plan_revision: 2

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

affects: [playground-vue, auto-playground, website]
current_step: 0
total_steps: 6
---

# [PLAN-754] playground 双端统一：运行契约对齐 + IDE 深链桥

## 0. 变更摘要

**r2（2026-10-10 用户裁定）**：website `/playground` 与独立版**同形**——双端共用
新组件 `PlaygroundIDEApp`（侧栏 NotesSidebar + AutoPlaygroundFull IDE + 笔记组
运行契约 + `?note=` 深链 + apiBase 可参），统一为 sidebar+IDE 形态。
r1 的"双形态保留 + IDE 桥"方案废止（Explorer+Card 组件保留导出，书页内嵌
运行器仍用；`/playground` 页不再用它）。运行契约统一由"同组件同请求路径"
天然达成；卡片链 prepend_lib 对称化随 r1 一并废止（书页不嵌 bootstrap 语料）。

r1 原始动机（双端分裂调查）保留为背景：website 卡片链缺 prepend_lib、
ideMode 桥两头空、App.vue 无深链——r2 以更彻底的方式一并解决。

## 1. 目标

- G-1：**双端同形**——website `/playground`（含 zh）渲染与独立版一致的
  sidebar+IDE 宿主（同一组件 `PlaygroundIDEApp`）。
- G-2：**同契约**——同一笔记双端运行请求一致（timeout 60s / prepend_lib
  组判定，组件内聚，不再依赖宿主记得传）。
- G-3：**可分享深链**——`?note=<id>` 双端打开即载。
- G-4：独立版零回归（e2e 全绿）；website 构建修复（dedupe 依赖入册）。

**非目标（r2）**：
- 不删 NotesExplorer/PlaygroundCard/SnippetRunner（书页内嵌等宿主仍用，
  仅 `/playground` 页弃用 Explorer 形态）。
- 不动后端 Rust；不动 manifest 管线。
- 不做 VitePress 以外的 website 结构改动。

## 2. 架构方案

```
packages/auto-playground-vue
  └─ PlaygroundIDEApp.vue（新, 宿主壳内聚）
       ├─ NotesSidebar（分组树+搜索, 现有）
       ├─ AutoPlaygroundFull（IDE, 现有）
       │    └─ usePlaygroundFull({ apiBase })   ← API_BASE 参数化(原常量 '/api')
       ├─ noteMeta 构造 + prependLib 组前缀判定（自 App.vue 迁入）
       └─ ?note=<id> 深链启动载入 + history.replaceState 清参

独立版 frontend/src/App.vue     → <PlaygroundIDEApp :api-base="'/api'" />
website /playground（theme 包裹）→ <PlaygroundIDEApp :api-base="探测" />
                                    探测: fetch(origin+/api/examples) → 同源
                                    失败 → http://127.0.0.1:3030（本地后端约定）
```

后端缺席时：manifest 静态可浏览（笔记树/代码/期望输出可看），Run 走
usePlaygroundFull 错误面（现有 Error 展示 + Plan 582 backendDown 降级哲学）。

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

### PlaygroundIDEApp 组件（新）

- props：`apiBase?: string`（默认 `/api`）、`manifestBase?: string`
  （默认 `/playground-data/notes.json`）。
- 内部聚合：useNotes(manifestBase) + 搜索过滤（App.vue 现有 visibleGroups
  逻辑迁入）+ noteMeta 构造（含 prependLib 组前缀）+ AutoPlaygroundFull
  watch 同步 + `?note=` 启动载入。
- 深链：onMounted 解析 → byId 命中 → select+loadNote → replaceState 清参。

### usePlaygroundFull apiBase 参数化

- `usePlaygroundFull(options?: { apiBase?: string })`；模块级 `API_BASE`
  常量改为 options 回落（默认 '/api'），fetch 调用点改用实例值。

### website 接线

- `website/.vitepress/theme/components/PlaygroundIDEPage.vue`（新）：探测
  apiBase（同源 → fallback 127.0.0.1:3030）后渲染 `<PlaygroundIDEApp>`；
  `playground.md`/`zh/playground.md` 换用该组件（保留 backend 提示块）。

### 规范增量（r2 修订）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/playground-vue/project.md | PlaygroundIDEApp 宿主契约：双端同形组件；apiBase/manifestBase props；?note= 深链；组运行契约内聚 | 双端统一（r2 用户裁定） | AC-01, AC-02, AC-03 |
| SD-02 | add | docs/specs/website/project.md | /playground 页形态=PlaygroundIDEApp（apiBase 探测：同源→127.0.0.1:3030）；Explorer+Card 降级为书页内嵌宿主 | website 现状记录 | AC-01, AC-03 |
| SD-03 | add | docs/specs/website/project.md | website 构建依赖修复：dedupe 契约包（highlight.js 等）须入 package.json 依赖（PLAN-718 dedupe 强制根解析的入册要求） | 构建稳健性（本次截图受阻实证） | AC-04 |

(r1 的 SD-01/02/03 由上述替代；卡片链 prepend_lib 对称化随 r1 废止。)

## 6. 测试设计

- standalone e2e（frontend/tests/unify.spec.ts 新）：
  ① `/?note=demo/04-fibonacci` 打开 → 编辑器载入斐波那契且 Run 出数；
  ② `/?note=vm-bootstrap/a2r_hello` → Run 输出 `ok`（组契约经宿主内聚生效）；
  ③ 无效 note id → 回退欢迎态不崩。
- website 构建冒烟：`npm run build` 过 + preview 渲染截图（sidebar+IDE 形态）。
- 回归：既有 e2e 全量（run-timeout 7 条等）绿 = App.vue 瘦壳零行为漂移。

## 7. 验收标准

| id | 标准 | 验证方法 |
|---|---|---|
| AC-01 | website 构建产物 `/playground` 渲染 sidebar+IDE 宿主（preview 截图/e2e：侧栏分组树 + IDE 工作区 + 选中笔记载入） | website build + preview 截图 |
| AC-02 | `?note=<id>` 深链双端打开即载（demo 笔记 + vm-bootstrap 笔记各一） | e2e ①② |
| AC-03 | apiBase 探测：后端在场可 Run；缺席时静态浏览可用且不白屏（错误面降级） | preview 双场景走查 |
| AC-04 | 独立版零回归：frontend e2e 全量绿；website 依赖入册后构建稳定（dedupe 包全在 package.json） | 全量 e2e + package.json 审计 |
| AC-05 | 复审门禁：cargo t 红集对拍（零 Rust 改动应恒等）、vue-tsc 过、spec 增量落库 | 复审档 |

## 8. 执行步骤

- [ ] T-0 website 依赖入册：dedupe 契约包审计（highlight.js/@codemirror/*/
  vue/lucide-vue-next/@lezer/highlight）全部入 website/package.json 依赖 +
  lockfile。验证：删除根 node_modules 后 npm ci && npm run build 过。
- [ ] T-1 usePlaygroundFull apiBase 参数化 + PlaygroundIDEApp 组件新建
  （聚合 App.vue 全部宿主逻辑）。文件：packages/auto-playground-vue。
  验证：vue-tsc + 导出登记 index.ts。AC-02
- [ ] T-2 standalone App.vue 瘦壳化（换用 PlaygroundIDEApp）。文件：
  crates/auto-playground/frontend/src/App.vue。验证：既有 e2e 全量绿 +
  新 unify spec ①②③。AC-02, AC-04
- [ ] T-3 website 接线：PlaygroundIDEPage.vue + playground.md/zh 换用 +
  apiBase 探测。验证：npm run build + preview 截图（AC-01/AC-03 走查）。
- [ ] T-4 验收档：scratch/playground-check/p754-unify-report.md（截图 +
  e2e 实录 + 双端对照）。AC-01, AC-03
- [ ] T-5 复审门禁：cargo t 红集对拍、vue-tsc、spec 预埋（merge 档落库）。
  AC-04, AC-05

## 9. 复审记录

- 2026-10-10 **plan_revision 1→2（用户裁定）**：website 与独立版同形——
  r1"双形态保留+IDE桥"废止，改为双端共用 PlaygroundIDEApp（sidebar+IDE）。
  任务/AC/SD 全量重订（T-0..T-5）；r1 卡片链 prepend_lib 对称化一并废止。
  Explorer+Card 组件保留导出（书页内嵌宿主用）。授权：用户 2026-10-10
  "不对，website上也要做成sidebar+IDE形态吧"+ "OK，开工"。
- 2026-10-10 `/auto-plan:new` r1 起草：`stage: new`，PLAN-754 revision 1。
  `outcome: pass`（用户已批统一诉求，授权范围内可开工；非目标清单=保留
  历史裁定的边界）。`next: work`。Q-1：website 桥探测的同源/fallback 顺序
  实现期按部署实况（deploy/nginx-auto-playground.conf）复核。

## 10. 待澄清事项

- Q-1：website 生产部署中静态站与后端的域关系（同源 nginx 反代？分域？）——
  决定 backendUrl 探测顺序；实现期读 deploy/nginx-auto-playground.conf 定谳，
  若分域则需网站构建期注入配置（不在本计划则按同源+localhost fallback）。
