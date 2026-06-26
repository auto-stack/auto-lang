# Plan 338: M1 — 把 015-notes 扩成中等 CRUD 基准 app

> **类型**:完整计划(实施)
> **状态**:设计待确认,实施未开始
> **日期**:2026-06-26
> **战略文档**:[docs/design/16-app-generation-and-ai-authoring.md](../design/16-app-generation-and-ai-authoring.md)(M1 = 基准阶梯第一级)
> **前身**:015-notes(已是最完整 ui 示例:解耦前后端、多模块、`#[api]` REST、CRUD+搜索)
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

## 1. 目标

把 `examples/ui/015-notes` 从"单视图 CRUD demo"升级为**第一个真正的基准 app(M1)**:
1. **加 routing**(Rung 1 app shell —— 当前 015 完全没有 `routes{}`),把单视图拆成多页。
2. **CRUD 加深**:标签/文件夹分类 + Markdown 渲染(从"玩具"到"中等")。
3. **后端持久化**:JSON 文件存储 + 类型化契约保持整洁(为后续 Rung 2 类型化打基础)。
4. **产出一份"再生 spec"**:用人话写、供 AI 仅凭 spec 复现整个 app 的规格书 —— 这是基准的"输入"。配合后续 gap 分析。

完成后,015-notes 成为能力阶梯 M1 的标定基准:一个解耦、多模块、多页、带后端持久化与类型化契约的中等 CRUD app,可供"AI 再生 → 量修复轮次 N"评测。

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **路由结构** | `routes { "/" -> notes_list; "/note/:id" -> editor; "/archive" -> archive; "/tags/:tag" -> tagged }` | 最小但像样的多页 app;暴露 list/detail/archive/filtered 四种路由模式 |
| **状态归属** | app 级持 `notes`/`active_id`/`archive`;每页只持自己的 UI 局部状态 | 为 Rung 4 共享 store 预热;避免每页重复 fetch |
| **标签模型** | `Note.tags: []str` + `/tags/:tag` 过滤页 | 典型 CRUD 加深;引入"集合的过滤/分类"模式 |
| **Markdown** | 后端只存原文;前端渲染用 marked(纯前端) | 加深"展示型"复杂度;后端不关心渲染 |
| **持久化** | 后端 `db.at` 读写 `data/notes.json`(现状已是 JSON) | 保持简单;为 Rung 2 的"loading/error/save 状态"提供真实 I/O 场景 |
| **类型化契约** | 共享 `Note` 类型已在 `back/api.at`;前端 `types.at` 镜像(现状) | 暂不引入 derive,只保证手动镜像一致 + 加测试钉住 |
| **再生 spec** | `examples/ui/015-notes/SPEC.md`:人话功能 + 路由 + 数据模型 + API | 基准的输入;故意不附代码,逼 AI 从需求生成 |

## 3. 非目标(留给后续)

- 拖拽、流式、auth —— 分别归 M2/M3/M5。
- 后端类型化契约的自动 derive(Rung 2 专项 Plan)。
- `auto dev` 热重载(Rung 5 专项 Plan)。

---

## 实施计划

> **Repo rules (CLAUDE.md):** 在专用 worktree 开发;改 codegen/CLI 后 `cargo build -p auto`;UI 改动用 `auto build`(015-notes 生成 vue)肉眼/构建验证。
>
> **Goal:** 015-notes 成为多页(routing)、带标签+Markdown、后端持久化的中等 CRUD app,并附再生 SPEC。
>
> **Tech Stack:** Auto(`routes{}` + 多 widget + `#[api]`)、Vue(渲染目标)、Rust(api:rust 后端)。

## Pre-flight: Worktree

```bash
git worktree add -b plan-338/m1-notes-benchmark ../auto-lang-338
cd ../auto-lang-338
```

---

## Phase 1 — Routing:单视图拆多页

**Files:** `examples/ui/015-notes/src/front/{app.at, sidebar.at, pages/*.at}`

### Task 1.1: app.at 加 `routes{}` + outlet

**Step 1:** 把 `app.at` 改为 app shell:顶部标题栏 + `<outlet>`(内容由路由填充),去掉内联的 sidebar+editor。加 `routes { ... }`(见 §2)。把现有 sidebar/list/editor 抽到 `src/front/pages/notes_list.at`、`note_editor.at`、`archive.at`、`tagged.at`。

**Step 2:** 每页 `widget NotesListPage { ... }`(etc.),各自 `use back.api: ...` 取数据;app 级 `use pages:*`。

**Step 3:** `auto build`(015 目录)→ 生成的 vue 含 vue-router 配置、多路由。`pnpm dev` 肉眼验证四个路由可切换。

**Step 4:** Commit `feat(015-notes): multi-page app shell with routes + outlet`。

### Task 1.2: 导航与选中态

**Step 1:** sidebar(nav)用路由 `link`(参考 auto-musk nav_rail.at 写法);当前 note 高亮。
**Step 2:** 路由参数 `/note/:id` → editor 页拿到 id 取该 note。
**Step 3:** Commit `feat(015-notes): nav rail + route params`。

---

## Phase 2 — CRUD 加深:标签 + Markdown

### Task 2.1: 标签模型 + 过滤页

**Files:** `back/api.at`(Note 加 tags、加 `list_notes_by_tag`)、`pages/notes_list.at`(显示 tag chips)、`pages/tagged.at`(按 tag 过滤)。

**Step 1:** 后端 `Note = { id, title, body, time, tags []str }`;`db.at` 读写适配;加 `#[api] list_notes_by_tag(tag str) []Note`。
**Step 2:** 列表页渲染 tag chips;`/tags/:tag` → tagged 页调用 `list_notes_by_tag`。
**Step 3:** `auto build` → 验证过滤生效。
**Step 4:** Commit `feat(015-notes): tags model + filter-by-tag page`。

### Task 2.2: Markdown 渲染

**Files:** `pages/note_editor.at`(只读态渲染 Markdown)。

**Step 1:** 生成的 vue 项目引入 `marked`(在 `generate_package_json` 或 015 自己的 package.json 声明)。前端 `text .note.body` 的只读分支改为渲染 marked 输出。
**Step 2:** 验证只读态显示渲染后 HTML;编辑态仍是 textarea(原文)。
**Step 3:** Commit `feat(015-notes): render note body as markdown`。

---

## Phase 3 — 后端持久化 + 契约一致

### Task 3.1: 真正读写 data/notes.json

**Files:** `back/db.at`。

**Step 1:** `db.at` 用 `auto` 的文件/json stdlib 在 create/update/delete 时落盘到 `data/notes.json`(若 stdlib 尚未齐备,记录缺口,改用启动时载入 + 内存 + save 的最小可用形态)。
**Step 2:** 重启 app,数据保留。
**Step 3:** Commit `feat(015-notes): persist notes to data/notes.json`。

### Task 3.2: 前后端类型镜像测试

**Files:** `examples/ui/015-notes/src/front/types.at` 与 `back/api.at` 的 `Note`。

**Step 1:** 加一个最小校验(脚本或测试)断言两边 `Note` 字段一致(字段名+类型)。
**Step 2:** Commit `test(015-notes): front/back Note type mirror invariant`。

---

## Phase 4 — 再生 SPEC + 基准就绪

### Task 4.1: SPEC.md

**Files:** `examples/ui/015-notes/SPEC.md`

**内容**:人话功能描述 + 路由表 + 数据模型 + API 端点 + 关键交互。**不附 Auto 代码**(目的是让 AI 仅凭 spec 复现)。

**Commit:** `docs(015-notes): regeneration SPEC for M1 benchmark`。

### Task 4.2: README 记录基准身份

**Files:** `examples/ui/015-notes/README.md`

**内容**:说明 015-notes 现为基准阶梯 M1(链 design 16);如何用作 AI 再生基准的流程。

**Commit:** `docs(015-notes): mark as M1 benchmark, link design doc`。

---

## Definition of Done

- [ ] `routes{}` + outlet,4 个路由(列表/详情/归档/标签)可切换;nav 高亮。
- [ ] 标签模型 + `/tags/:tag` 过滤页;Markdown 渲染只读态。
- [ ] `data/notes.json` 真持久化;重启不丢。
- [ ] 前后端 `Note` 类型镜像有测试钉住。
- [ ] `SPEC.md`(无代码)就绪;README 记录 M1 基准身份。
- [ ] `auto build` 绿、生成的 vue `pnpm dev` 可用、核心 CRUD+路由+过滤端到端肉眼通过。
- [ ] worktree 分支在 build 绿后合并回 `master`。

---

## 后续(不在本 Plan)

- **Gap 分析**(研究):让 AI 仅凭 `SPEC.md` 再生 015-notes,记录失败模式 → 驱动 Rung 2(类型化契约/数据生命周期)投资。
- **M2-M6**:见 design 16;各自单独立 Plan(022-kanban、017-chat、016-calendar、023-realworld、auto-musk)。
