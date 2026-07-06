# Plan 354: 把 015-notes 从 CRUD demo 扩展成真正可用的笔记应用

> **类型**:完整计划(实施)
> **状态**:设计待确认
> **日期**:2026-07-06
> **战略文档**:[Design 16](../design/16-app-generation-and-ai-authoring.md)(M1 基准)、[Design 18](../design/18-shared-store.md)(SharedStore)
> **前置**:015-notes 现有后端通讯已稳定;codegen fixes 已合并(self. strip、v-model、conditional style、callback prop 等)
> **归档**:025-notes-extended 的 store + routing 概念验证已被吸收;不再维护
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

## 1. 目标

把 `examples/ui/015-notes` 从"最小 CRUD demo"扩展为一个**展示 AutoUI 全部核心能力的、真正可用的笔记应用**——可以指导开发者如何用 Auto 构建真实软件。

完成后,015-notes 展示:
- ✅ 后端持久化(`#[api]` REST + JSON)
- ✅ SharedStore(跨页共享状态)
- ✅ 多页路由(URL 变化 + 浏览器历史)
- ✅ 排序 + 置顶(数据操作)
- ✅ Markdown 内容(AutoDown 集成)
- ✅ 标签系统(创建/管理/过滤)
- ✅ 全文搜索(title + body)
- ✅ 加载/错误/空状态(异步 UX)
- ✅ 暗色模式(CSS 变量切换)

---

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **store vs 后端** | store = 客户端缓存/状态;后端 = 持久化 | `.Init -> { notes = list_notes() }`,CRUD 先调 API 再刷新 store |
| **多页结构** | `/` 列表页,`/note/:id` 详情页,`/settings` 设置页 | URL 变化 + 浏览器历史(用户明确要求) |
| **导航机制** | **前置必须解决**:`nav()` 或 `link` 在 handler/view 里不解析。需修复 parser 或加 `navigate()` builtin | 025 因为此问题放弃了多页路由 |
| **Markdown 渲染** | AutoDown(`@autodown/vue`)| 跨工程集成,非 auto-lang 内部 |
| **025 处理** | 归档 | 其代码已被吸收 |
| **后端 schema 变更** | 加 `pinned bool`、`tags []str`、`updated_at str` 字段 | 需更新 `db.at` + `api.at` + 前端 `types.at` |

---

## 3. 前置任务(必须先完成)

### 3.1 路由导航:修复 `nav()` / `link` parser

**问题**:在 handler body 里写 `nav("/note/123")` 不解析(parser 把 `nav` 当 TokenKind,不是 ident);在 view 里写 `link "/note/123"` 也不解析。

**方案**:加一个 **`navigate(path)` builtin**——在 view DSL 里用 `navigate("/note/123")` 作为一个 onclick handler 的替代;或在 handler body 里调 `router_push("/note/123")`。

**最小实现**:在 codegen 里,当 handler body 遇到 `nav("path")` 或 `navigate("path")` 调用时,生成 `router.push("path")`。需要 `useRouter` 已在组件 script 里。

**验证**:capability-test canary `nav-handler-routing`(点击按钮 → URL 变化 → 路由切换)。

### 3.2 后端 schema 扩展

**Files:** `src/back/db.at`、`src/back/api.at`

加字段:
- `pinned: bool`(默认 false)
- `tags: []str`(默认 [])
- `updated_at: str`(默认等于 created time)

加 API:
- `#[api] list_notes_sorted(sort_by str, pinned_first bool) []Note`
- `#[api] toggle_pin(id int) Note`
- `#[api] update_tags(id int, tags []str) Note`
- `#[api] search_notes(query str) []Note`(搜索 title + body)

---

## 4. 三层增量(每层独立可交付)

### 第一层:核心体验(Tier 1)

#### Task 1.1: SharedStore

**Files:** `src/front/notes_store.at`(新建)

```auto
store NotesStore {
    model {
        var notes []Note = []
        var active_id int = 0
        var sort_mode str = "updated"
        var search str = ""
        var loading bool = false
        var error str = ""
    }
    msg Msg { Init, Refresh, SelectNote(int), NewNote, SaveNote(int, str, str),
              DeleteNote(int), SetSort(str), TogglePin(int), Search(str) }
    on {
        .Init -> { .loading = true; .notes = list_notes(); .loading = false }
        .Refresh -> { .notes = list_notes() }
        .SelectNote(id) -> { .active_id = id }
        .NewNote -> { let n = create_note("Untitled", ""); .notes.push(n); .active_id = n.id }
        .SaveNote(id, title, body) -> { update_note(id, title, body); .notes = list_notes() }
        .DeleteNote(id) -> { delete_note(id); .notes = list_notes() }
        .SetSort(mode) -> { .sort_mode = mode }
        .TogglePin(id) -> { toggle_pin(id); .notes = list_notes() }
        .Search(q) -> { .search = q }
    }
}
```

**验证**:store composable 生成 + 所有页面 `use store:` + vue-tsc GREEN。

#### Task 1.2: 多页路由

**Files:**
- `src/front/app.at`(改为 router shell)
- `src/front/pages/notes_list.at`(列表 + 搜索 + 排序 + 侧栏)
- `src/front/pages/note_detail.at`(编辑器 + 标签 + 置顶)
- `src/front/pages/settings.at`(排序偏好 + 暗色模式)

**app.at 结构:**
```auto
use store: NotesStore

widget App {
    routes {
        "/" -> use notes_list
        "/note/:id" -> use note_detail
        "/settings" -> use settings
    }
    view { col { outlet; style: "h-screen" } }
    on { .Init -> { store.Init() } }
}
```

**note_detail.at** 从路由参数 `:id` 取到 note id → 从 store 读对应 note → 编辑器(复用现有 EditorPanel,加 callback prop)。

**验证**:三个路由页面生成;URL 变化;浏览器后退/前进工作。

#### Task 1.3: 排序 + 置顶

- 列表页:排序下拉(updated/created/title)→ store.SetSort
- 列表页:每条 note 有 pin 按钮 → store.TogglePin
- store 的排序逻辑:pinned first + sort_mode

#### Task 1.4: 列表页高亮 + 空状态

- 当前选中 note 高亮(conditional style)
- notes 为空时显示"还没有笔记,点 + New 创建"

**Tier 1 DoD:**
- [ ] SharedStore 管理 notes 状态
- [ ] 三页路由(/、/note/:id、/settings),URL 变化
- [ ] 排序(updated/created/title)+ 置顶
- [ ] 列表高亮 + 空状态
- [ ] auto build + vue-tsc GREEN
- [ ] 后端 CRUD 仍正常(persistence 不丢)

---

### 第二层:内容增强(Tier 2)

#### Task 2.1: Markdown 编辑/预览

- note_detail 页:read mode 用 `@autodown/vue` 渲染 body;edit mode 用 textarea 编辑原文
- 需要在生成的 Vue 项目 package.json 加 `@autodown/vue` 依赖

#### Task 2.2: 标签系统

- 后端:`update_tags(id, tags)` API
- store:`active_tag` + `SetTag(tag)` action
- note_detail:编辑标签(添加/删除 chip)
- notes_list:tag 筛选栏(点击 tag 过滤)

#### Task 2.3: 全文搜索

- 搜索栏搜索 title + body(`search_notes(query)` 后端 API,或前端 filter)
- 搜索结果高亮匹配关键词

**Tier 2 DoD:**
- [ ] Markdown body 渲染(AutoDown)
- [ ] 标签创建/管理/过滤
- [ ] 全文搜索(title + body)
- [ ] auto build + vue-tsc GREEN

---

### 第三层:UX 打磨(Tier 3)

#### Task 3.1: 加载/错误状态
- store 的 `loading` + `error` 在列表页消费(spinner / error message)

#### Task 3.2: 暗色模式
- settings 页有 toggle → CSS 变量 `:root` ↔ `.dark` 切换
- store 持 `dark_mode` 状态

#### Task 3.3: 响应式布局
- 移动端:侧栏折叠为 drawer(按钮触发)
- CSS media query + 状态管理

**Tier 3 DoD:**
- [ ] loading/error 状态在 UI 里展示
- [ ] 暗色模式 toggle + 正确渲染
- [ ] 移动端布局可用
- [ ] auto build + vue-tsc GREEN

---

## 5. 风险

| 项 | 风险 | 缓解 |
|---|---|---|
| **路由导航 parser** | `nav()`/`link` 不解析 → 无法多页 | 前置任务 3.1 必须先解决;否则退回单页 + store |
| **AutoDown 集成** | 跨工程依赖,版本/API 可能不匹配 | Tier 2 才用到;先做 Tier 1 验证基础 |
| **后端 schema 迁移** | 现有 data/notes.json 无新字段 | db.at 加默认值;旧数据自动兼容 |
| **另一个 agent 改 015** | 冲突 | 在专用 worktree 做;合并时处理 |

---

## 6. 实施顺序

```
前置 3.1 (nav parser) ──→ 前置 3.2 (backend schema)
                                      │
                    ┌─────────────────┘
                    ▼
            Tier 1 (core) ──→ Tier 2 (content) ──→ Tier 3 (polish)
            1.1 store         2.1 markdown          3.1 loading/error
            1.2 routing       2.2 tags             3.2 dark mode
            1.3 sort/pin      2.3 search           3.3 responsive
            1.4 highlight
```

每层独立交付;每层完成后 `auto build` + `vue-tsc` + 后端 CRUD 全绿才进下一层。

---

## 7. 025-notes-extended 处理

- **不删除** 025 目录(保持 git 历史)
- README 标注"已归档,能力已迁移至 015-notes"
- 后续不再修改 025

---

## Definition of Done (全部三层)

- [ ] 015-notes 是一个多页、有 SharedStore、支持 Markdown + 标签 + 搜索 + 排序 + 置顶 + 暗色模式 + 加载状态的笔记应用
- [ ] 后端 CRUD 持久化正常(重启不丢数据)
- [ ] URL 变化 + 浏览器历史/刷新工作
- [ ] `auto build` + `vue-tsc` GREEN
- [ ] 025 归档标注
- [ ] worktree 分支在全部绿后合并回 `master`
