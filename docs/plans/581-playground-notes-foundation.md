---
plan_id: PLAN-581
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: playground-notes-foundation
author: [zhaopuming]
created_at: 2026-09-07
updated_at: 2026-09-07

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [playground-vue, website]   # specs 路径
current_step: 0
total_steps: 8
---

# [PLAN-581] Playground 组件分层 + Notes Manifest 管线（Playground 设计 · Plan A）

## 变更摘要

按 [Playground 设计](../design/documents/playground-architecture.md) §4/§5 落地 Playground 在线体验层的基础两件：

1. **组件三层收紧**：`packages/auto-playground-vue` 新增 `SnippetRunner`（拼图层，单 snippet 无工具栏）与 `PlaygroundCard`（卡片层，工具栏可配置、`exampleSelector` 默认关）；现有 `AutoPlayground.vue`（731 行）降级为兼容薄包装。编辑/运行逻辑复用 `usePlayground.ts`，不重写。
2. **Notes manifest 管线**：新增 `scripts/build-playground-notes.mjs`，构建期从仓内语料源（vm golden、aavm corpus、书籍围栏、playground-demo；parity 勘察后裁定）采集生成 `website/public/playground-data/notes.json`（单一事实源，确定性输出），接入 website 构建链（`prepare-content.js`），并提供 `--check` 幂等校验。

本计划是 Plan 582（Notes Explorer + 宿主合一 + 电子书嵌入）的前置依赖；不改变任何现有页面布局（website `/playground` 页在 582 重做，本计划仅保证其构建不破）。

## 目标

- G1: `SnippetRunner` / `PlaygroundCard` 从 `auto-playground-vue` 包导出，类型检查与构建零错误。
- G2: `AutoPlayground` 保留为向后兼容别名，website 构建通过（行为变化仅一处：工具栏不再默认显示 ExampleSelector——Playground 设计 §4.4 裁定）。
- G3: `notes.json` 生成覆盖全部仓内语料源，规模达到验收阈值（§验收标准 3），连续两次生成 byte-identical。
- G4: manifest 生成内联进 `website/scripts/prepare-content.js`（dev/build/deploy 三态自动触发）；`--check` 模式供 CI 防采集规则回归。
- G5: parity 语料收录范围完成勘察并登记裁定（Playground 设计 §9-①）。

## 架构方案

对应 [Playground 设计](../design/documents/playground-architecture.md)：

- §4 组件分层：`SnippetRunner`（核，无 chrome）⊂ `PlaygroundCard`（壳 + 可配置工具栏）⊂ `AutoPlaygroundFull`（IDE，不动）。本计划落前两层 + 兼容别名。
- §5 数据管线：`scripts/build-playground-notes.mjs` → `website/public/playground-data/notes.json`（schema v1：groups/notes，含 expectedOutput/standalone/kind）。manifest **不提交**（gitignore），每次构建重新生成；`--check` = 内存二次生成幂等比对 + 计数断言（非磁盘 diff，因 books 为 gitignore 生成物）。
- 后端 `/api/examples` 仍走目录扫描，582 T8 统一切到 manifest（本计划不动 `crates/`）。

## 需求分析与背景调查

（取材 [docs/specs/overview.md](../specs/overview.md) 与模块 specs）

- **[playground-vue](../specs/playground-vue/project.md)**（active）：入口组件 AutoPlayground（精简）/ AutoPlaygroundFull（完整）；composables `usePlayground`/`useDebugger`/`useReplayPlayer`；lang/ CodeMirror 语言支持。现状问题：精简组件不纯——工具栏常驻 `ExampleSelector`（拉后端 `/api/examples`），无法作单 snippet 拼图嵌入。
- **[auto-playground](../specs/auto-playground/project.md)**（active）：axum 后端 `examples.rs` 仅扫 `examples/playground-demo/`（25 单文件 + 4 项目），语料面窄。
- **[website](../specs/website/project.md)**（active）：VitePress 站；`playground.md` 内嵌 `<AutoPlayground>`；`public/playground/` 为全量 SPA 同步产物（`build-playground.mjs` 双路同步）。书籍内容由 `scripts/prepare-content.js` 从外仓 `../book` 物化到 `website/books/`（gitignore，构建期生成）。
- **语料盘点（2026-09-07 实测，Playground 设计 §1.3）**：vm golden 42 组目录/322 个 `.at`+`.expected.out` 配对；aavm corpus 158 个 `.at`；书籍 ` ```auto ` 围栏 ~1282（双语重复）；playground-demo 25+4；parity 53 个 `.at`（多依赖环境）。

## 详细设计

### 1. 组件契约（Playground 设计 §4 表格为准）

```ts
// types.ts 新增
interface SnippetRunnerProps {
  code: string                                   // 必填
  apiBase?: string                               // 默认 ''（同源 /api）
  autorun?: boolean                              // 默认 false
  target?: 'run' | 'rust' | 'c' | 'python' | 'typescript' | 'abt'
  height?: string                                // 默认 'auto'（按行数）
}
interface PlaygroundCardProps extends SnippetRunnerProps {
  noteId?: string                                // manifest 笔记 id（582 用；本期仅预留透传）
  toolbar?: { transpile?: boolean; share?: boolean; debug?: boolean; live?: boolean }  // 默认全开
  exampleSelector?: boolean                      // 默认 false（旧默认行为显式选入）
}
```

- `SnippetRunner.vue`：编辑器 + 内联折叠输出区 + 单动作位（▶）；无 ExampleSelector/文件树；无后端时动作位报错提示（统一降级态在 582 实现）。
- `PlaygroundCard.vue`：工具栏从现 `AutoPlayground.vue` 迁移（样式与逻辑不动，仅按 `toolbar` 开关条件渲染）；`exampleSelector` 开启时才渲染 `<ExampleSelector>`。
- `AutoPlayground.vue`：改为 `<PlaygroundCard v-bind="$attrs" :example-selector="exampleSelector">` 薄包装，`api-url`/`code`/`height` 旧 prop 名透传映射；`index.ts` 导出全系 + `AutoPlayground` 标注 `@deprecated`（注释指向 PlaygroundCard）。

### 2. 采集脚本 `scripts/build-playground-notes.mjs`

- 源与规则（Playground 设计 §5.1）：
  - **vm golden**：`crates/auto-lang/test/vm/[0-9][0-9]_*/`（不含 `aavm2/`）每 case 目录内 `.at` 与同名 `.expected.out` 配对，无配对跳过；组名=目录名语义段（`05_loops` → "循环"，沿用 `display_name_from_stem` 风格 + 内置目录号中文映射表，未命中回退目录名）。
  - **aavm corpus**：`crates/auto-lang/test/vm/aavm2/corpus_{m1,m2,m3,m4,use,a2r}/*.at`，每目录一组。
  - **books**：`website/books/*/ch*.md`（prepare-content 物化后采集；`.cn.md` 不重复采）；`kind=fence`；围栏内含 `import` 行 → `standalone:false`。
  - **playground-demo**：`examples/playground-demo/` 单文件 + 项目目录（`main.at` 判定，`kind=project`，采全部文件入 `files`）。
  - **parity**：T1 勘察裁定后启用或后置（默认后置）。
- 输出：`website/public/playground-data/notes.json`，schema 见 Playground 设计 §5.2；**确定性**：groups 按 `order`/`id` 排序、notes 按 `id` 排序，**不写 builtAt**（保证 byte-identical）。
- `--check`：重新在内存生成第二遍与第一遍深比对 + 计数断言（见验收 3），失败非零退出。
- 体积：单文件起步；若 >5MB 改按组分片（Playground 设计 §9-③，T6 实测后裁定并登记）。

### 3. 构建接线

- `website/scripts/prepare-content.js` 末尾 `spawnSync(process.execPath, [REPO_ROOT/scripts/build-playground-notes.mjs])`（books 已物化后运行；book 仓缺失时跳过 books 源并打 warning，不失败）。
- `website/public/playground-data/` 加入 `website/.gitignore`（或根 .gitignore 对应条目）。
- deploy-website.yml 零改动（prepare-content 已在其构建链内）。

## 测试设计

- **分级门禁**：本计划不触 `crates/` Rust 源码 → **不跑 cargo t/tf/tv**（AGENTS.md Category A/B 判定：前端资产 + 脚本）。门禁 = ① `packages/auto-playground-vue` build（vue-tsc + vite）② `website` build ③ 采集脚本 `--check`。
- 组件：`cd packages/auto-playground-vue && npm run build`（类型+产物）；dev 冒烟（`npm run dev` + 本地后端，人工，验 SnippetRunner autorun / Card 工具栏开关 / 别名页不破）。
- 脚本：`node --check scripts/build-playground-notes.mjs`（语法）+ `node scripts/build-playground-notes.mjs --check`（幂等+计数）。
- website：`cd website && npm run build`（含 prepare-content 触发 manifest 生成）。

## 验收标准

1. `packages/auto-playground-vue` `npm run build` 零错误；`index.ts` 导出 `SnippetRunner`/`PlaygroundCard`。
2. `website` `npm run build` 通过；`/playground` 页编译不破（布局改造留给 582）。
3. `notes.json` 计数断言：vm-golden ≥300 笔记且全部带 `expectedOutput`；aavm ≥158；demo ≥29（含 ≥4 个 `kind=project`）；books >0（本地 book 仓在位时）。`--check` 退出 0。
4. 幂等：连续两次 `node scripts/build-playground-notes.mjs` 产物 byte-identical（`fc /b` 或 `cmp`）。
5. parity 收录裁定已登记于本文件"待澄清事项①"（含勘察证据路径）。
6. manifest 体积实测值与单文件/分片裁定登记于"待澄清事项③"。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- **T1 parity 语料勘察与收录裁定**
  操作：对 `parity/libs/**/*.at`（53 个）逐个/抽样 `auto run` 探测 stdout 可跑性与外部依赖（C 库/文件系统），产出 `scratch/p581/parity-survey.md`（表：路径、依赖、可跑性、建议）；据此在本文"待澄清①"登记裁定（收录清单 / 后置）。
  验证：`test -f scratch/p581/parity-survey.md && grep -c "可跑性" scratch/p581/parity-survey.md`（>0）。

- **T2 组件契约类型**
  文件：`packages/auto-playground-vue/src/types.ts`
  操作：新增 `SnippetRunnerProps` / `PlaygroundCardProps` / `NoteMeta`（manifest 笔记元信息，582 复用）类型定义。
  验证：`cd packages/auto-playground-vue && npx vue-tsc -b && echo OK`。

- **T3 SnippetRunner 组件**
  文件：`packages/auto-playground-vue/src/components/SnippetRunner.vue`（新建）
  操作：实现拼图层（props 见详细设计 §1；CodeEditor + ConsoleOutput 复用；`usePlayground` 驱动）。
  验证：`cd packages/auto-playground-vue && npm run build`。

- **T4 PlaygroundCard 组件**
  文件：`packages/auto-playground-vue/src/components/PlaygroundCard.vue`（新建）
  操作：卡片层 = SnippetRunner + 可配置工具栏（transpile/share/debug/live 条件渲染，样式从 `AutoPlayground.vue` 迁移）+ `exampleSelector`（默认 false）条件渲染 ExampleSelector。
  验证：`cd packages/auto-playground-vue && npm run build`。

- **T5 AutoPlayground 兼容薄包装与导出**
  文件：`packages/auto-playground-vue/src/AutoPlayground.vue`、`packages/auto-playground-vue/src/index.ts`
  操作：AutoPlayground.vue 改为 PlaygroundCard 薄包装（旧 prop 名 `api-url`/`code`/`height` 透传映射）；index.ts 导出 `SnippetRunner`/`PlaygroundCard`，AutoPlayground 标 `@deprecated`。
  验证：`cd packages/auto-playground-vue && npm run build && cd ../../website && npm run build`。

- **T6 manifest 采集脚本**
  文件：`scripts/build-playground-notes.mjs`（新建）
  操作：实现四源采集（vm golden 配对 / aavm corpus / books 围栏 / playground-demo；parity 按 T1 裁定）+ 确定性输出 + `--check`（内存幂等比对 + 计数断言），输出 `website/public/playground-data/notes.json`；实测体积并在待澄清③登记单文件/分片裁定。
  验证：`node scripts/build-playground-notes.mjs && node scripts/build-playground-notes.mjs --check && node -e "const m=require('./website/public/playground-data/notes.json');const c=t=>m.groups.filter(g=>g.id.startsWith(t)).reduce((s,g)=>s+g.notes.length,0);console.log('vm:'+c('vm-'),'aavm:'+c('aavm-'),'demo:'+c('demo'))"`。

- **T7 website 构建接线**
  文件：`website/scripts/prepare-content.js`、`website/.gitignore`（或仓库根 .gitignore）
  操作：prepare-content 末尾 spawn 采集脚本（book 仓缺失时 warning 跳过 books）；`website/public/playground-data/` 入 gitignore。
  验证：`cd website && npm run build && test -f public/playground-data/notes.json && echo WIRED`。

- **T8 验收清单执行**
  操作：逐条跑"验收标准"1–6 命令并记录结果到本节。
  验证：全绿；`cd packages/auto-playground-vue && npm run build && cd ../../website && npm run build && node scripts/../../scripts/build-playground-notes.mjs --check`（路径以仓库根执行为准）。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **① parity 收录范围**（T1 勘察后回填）：默认后置；若可跑子集 ≥10 个且无环境依赖则收录为 `parity` 组。
- **③ manifest 单文件 vs 分片**（T6 实测后回填）：默认单文件；阈值 5MB。
- **⑤ 人工 description overrides 机制**（Playground 设计 §9-⑤）：本期不做，582 或后续计划再议。
