# Plan 354: 把 015-notes 从 CRUD demo 扩展成真正可用的笔记应用

> **类型**:完整计划(实施)
> **状态**:实施中（Tier 1 核心体验 + 后端 schema + 标签/搜索已完成；三列布局 + AutoDown 编辑器 + block + 文件夹组织待实施）
> **日期**:2026-07-06（初版）/ 2026-07-14（v2 融合三列布局 + AutoDown + block 方案）
> **战略文档**:[Design 16](../design/16-app-generation-and-ai-authoring.md)(M1 基准)、[Design 17](../design/17-blocks-first-class.md)(Block 三层模型)、[Design 18](../design/18-shared-store.md)(SharedStore)
> **前置**:015-notes 现有后端通讯已稳定;codegen fixes 已合并(self. strip、v-model、conditional style、callback prop 等)
> **归档**:025-notes-extended 的 store + routing 概念验证已被吸收;不再维护
> **For Claude:** 实施部分使用 `superpowers:executing-plans` 逐任务执行,在专用 worktree 内进行。

---

## 1. 目标

把 `examples/ui/015-notes` 从"最小 CRUD demo"扩展为一个**展示 AutoUI 全部核心能力的、真正可用的笔记应用**——参考 Notion/Apple Notes/Bear/Obsidian 等主流笔记 APP。

完成后,015-notes 展示:
- ✅ 后端持久化(`#[api]` REST + JSON)
- ✅ SharedStore(跨页共享状态)——**已完成**
- ✅ 排序 + 置顶(数据操作)——**已完成**
- ✅ 标签系统(创建/管理/过滤)——**已完成**
- ✅ 全文搜索(title + body)——**已完成**
- ✅ 加载/错误/空状态(异步 UX)
- ✅ 暗色模式(CSS 变量切换)
- 🆕 **三列布局**（导航/列表/编辑器，取代原计划的多页路由）
- 🆕 **文件夹/笔记本组织**（侧边栏文件夹树 + 快捷分类 Tab + 标签云）
- 🆕 **AutoDown WYSIWYG 编辑器**（集成 `@autodown/editor` Tiptap 编辑器，取代 textarea）
- 🆕 **Block 层设计**（用 Plan 342/343 的 block 体系组织 UI，而非 widget 拼凑）

### v2 变更说明（2026-07-14）

| 维度 | v1（初版） | v2（当前） |
|---|---|---|
| **布局** | 多页路由（`/`、`/note/:id`、`/settings`） | **三列布局**（导航/列表/编辑器）—— 更符合主流笔记 APP |
| **目录组织** | 无（扁平列表） | **文件夹树 + 快捷 Tab + 标签云** |
| **Markdown** | `@autodown/vue`（只渲染） | **`@autodown/editor`**（完整 Tiptap WYSIWYG） |
| **Block 层** | 未提及 | **三个 block 包接入**（note-list/note-editor/sidebar-nav） |

多页路由方案保留为**可选增强**（三列布局内的编辑器可以额外支持 `/note/:id` 深链接），但不再是主要导航方式。

---

## 2. 关键决策

| 决策点 | 结论 | 理由 |
|---|---|---|
| **布局** | 三列布局（导航/列表/编辑器） | 参考 Notion/Apple Notes/Bear——笔记 APP 标准模式 |
| **导航组织** | 快捷 Tab + 文件夹树 + 标签云（三段式侧边栏） | 融合三种组织方式，覆盖不同使用场景 |
| **编辑器** | `@autodown/editor`（Tiptap WYSIWYG） | 完整 block 编辑、Markdown、code highlight、math、slash command |
| **Block 接入** | 先设计 block 包再接入 | 遵循 Plan 342/343 的"spec + reference + gotchas"流程 |
| **store vs 后端** | store = 客户端缓存/状态;后端 = 持久化 | `.Init -> { notes = list_notes() }`,CRUD 先调 API 再刷新 store |
| **后端 schema 变更** | 加 `folder: str` 字段 + 文件夹 CRUD API | 支持文件夹层级组织 |

---

## 3. 已完成项（无需重做）

| 任务 | 状态 | 说明 |
|---|---|---|
| 后端 schema（pinned/tags/search/updated_at） | ✅ | `types.at`、`db.at`、`api.at` 已含 |
| SharedStore | ✅ | `notes_store.at` 已实现 |
| 排序 + 置顶 | ✅ | store 有 SetSort/TogglePin |
| 标签系统 | ✅ | tags 字段 + update_tags API + 编辑器内标签管理 |
| 全文搜索 | ✅ | search_notes API + 前端搜索栏 |

---

## 4. 待实施任务（按阶段）

### 阶段 A：后端文件夹扩展

#### Task A.1: Note 加 folder 字段 + 文件夹 CRUD

**Files:** `src/front/types.at`、`src/back/db.at`、`src/back/api.at`

- `Note` 类型加 `folder: str`（默认 `"default"`）
- 新建 `Folder` 类型：`{ id int, name str, icon str }`
- DB 加文件夹种子数据：`default`（默认）、`work`（工作）、`personal`（个人）、`study`（学习）
- API 新增：
  - `#[api] list_folders() []Folder`
  - `#[api] create_folder(name str) Folder`
  - `#[api] move_note(id int, folder str) Note`
  - `#[api] list_notes_by_folder(folder str) []Note`

**验证**: 后端 CRUD + 文件夹列表正常。

---

### 阶段 B：Block 包设计

按"先设计 block 包再接入"原则，在 `blocks/` 目录下设计/升级 block 包。

#### Task B.1: 升级 `data-display/note-list` block

**Files:** `blocks/data-display/note-list/spec.md`、`reference/default.at`、`gotchas.md`

当前 note-list block 只有搜索 + 列表。升级为支持：
- `folder` 过滤 props（按文件夹筛选笔记）
- `tag` 过滤 props（按标签筛选）
- 多选/批量操作扩展点

spec 的 palette 增加 `badge`（标签徽章）、`separator`（分隔线）。
dataSource 增加 `by_folder = "(folder) -> []Note"`、`by_tag = "(tag) -> []Note"`。

#### Task B.2: 新建 `editor/note-editor` block

**Files:** `blocks/editor/note-editor/spec.md`、`reference/default.at`、`gotchas.md`

封装 AutoDown 编辑器的 block：
- palette: `autodown_editor`（新 widget）、`input`（标题）、`badge`（标签）、`button`
- extension_points: `title`, `editor`, `toolbar`, `tags`
- variants: `default`（编辑+预览）、`readonly`（只读渲染）
- dataSource: `save = "(id, title, body) -> Note"`

#### Task B.3: 新建 `navigation/sidebar-nav` block

**Files:** `blocks/navigation/sidebar-nav/spec.md`、`reference/default.at`、`gotchas.md`

三段式导航 block：
- palette: `button`, `text`, `badge`, `separator`
- extension_points: `shortcuts`（快捷 Tab）, `folders`（文件夹树）, `tags`（标签云）
- variants: `default`（三段全显）、`compact`（仅快捷 Tab + 文件夹）
- dataSource: `folders = "() -> []Folder"`, `tags = "() -> []Tag"`

#### Task B.4: 新建 `autodown_editor` widget 注册

在 `WidgetRegistry` 注册 `autodown_editor` 为 Vue 后端自定义组件。Vue 生成器遇到此标签时：
- 输出 `<AutoDownEditor>` PascalCase 组件名
- 生成 import：`import { AutoDownEditor } from '@autodown/editor'`
- props 映射：`content`、`canEdit`、`showActions`
- emits 映射：`@update`、`@save`、`@cancel`

**验证**: `auto block check` 通过；reference `.at` `auto build` 绿。

---

### 阶段 C：AutoDown 编辑器集成

#### Task C.1: Vue 生成器增强——自定义组件透传

当前 Vue 生成器对未知标签 `_ => tag` 原样输出（小写 HTML 标签）。需要增强：
- 注册的自定义组件（如 `autodown_editor`）→ 输出 PascalCase 组件名
- 生成对应 import 语句
- props/emits 正确绑定

#### Task C.2: 015-notes 生成项目配 AutoDown 依赖

- `gen/front/vue/package.json` 加 `"@autodown/editor"` 依赖
- 如果 `@autodown/editor` 未发布 npm，配 `file:` 或 workspace 依赖指向 `../auto-down/autodown/packages/editor`

#### Task C.3: AutoDownEditor widget Vue 模板

- 编辑器组件的 props 传递（`content` 绑定 note.body）
- `@update` 事件绑定到 handler（实时更新）
- `@save` 事件绑定到后端保存

**验证**: 生成的 Vue 项目 vue-tsc 绿；编辑器渲染 Markdown 内容；保存回后端。

---

### 阶段 D：前端两栏树状导航重构（v3）

> **v3 变更（2026-07-16）**：原 v2 是三栏（导航 + 列表 + 编辑器），搜索/筛选分散在两栏。
> v3 改为**两栏**（统一导航栏 + 编辑器），导航栏内是树状结构，所有搜索/筛选/笔记列表统一在一栏。

#### 新布局设计

```
col (h-screen)
├── header (row: 标题 "Notes" + 搜索框 + + New + Dark)
└── body (row: flex-1)
    ├── NavTree (col: w-80)           ← 唯一导航栏（树状）
    │   ├── 快捷入口：📌 Pinned | 🕐 Recent | All Notes
    │   ├── separator
    │   ├── 搜索结果区（当有搜索词时，直接显示匹配笔记列表）
    │   ├── 文件夹树：
    │   │   📁 Default
    │   │     ├── 笔记1 (当前选中高亮)
    │   │     ├── 笔记2
    │   │     └── 笔记3
    │   │   📁 Work
    │   │     └── 会议笔记
    │   │   📁 Personal
    │   │     └── 购物清单
    │   ├── separator
    │   └── 标签筛选区（点击 tag 过滤树）
    │
    └── EditorPanel (col: flex-1)     ← 编辑器
```

**设计原则**：
1. **单一导航栏**：文件夹、笔记、搜索、筛选全部在一个树里，不分散
2. **树状结构**：文件夹是父节点，笔记是子节点（类似文件管理器）
3. **搜索即筛选**：搜索框在 header，输入后导航栏的树只显示匹配的笔记
4. **快捷入口**：Pinned/Recent 作为虚拟"文件夹"出现在树顶部
5. **标签筛选**：标签栏在底部，点击后树只显示含该标签的笔记

#### Task D.1: 两栏布局骨架（app.at 重写）

**Files:** `src/front/app.at`

从三栏改为两栏：
- header 含搜索框（从第二栏移上来）
- body 只有 NavTree + EditorPanel
- 去掉独立的"笔记列表"列

#### Task D.2: 树状导航组件（sidebar.at → navtree.at 重写）

**Files:** `src/front/sidebar.at`（重命名为 NavTree）

用 Auto 的 `for` + `if` 嵌套实现树状渲染：
- 快捷入口区：Pinned / Recent / All 按钮
- 搜索过滤：当 `.search != ""` 时，树只显示匹配的笔记（平铺，不分文件夹）
- 文件夹树：`for folder in folders` → `for note in notes where note.folder == folder.name`
- 笔记节点：缩进显示，选中高亮，点击选中
- 标签区：底部显示所有 tag，点击筛选

#### Task D.3: AutoDown 编辑器（editor.at，保持不变）

已有 AutoDown 编辑器集成，只需确保从导航栏点击笔记后正确加载。

#### Task D.4: NotesStore（保持不变）

已有 folder/tag 状态，无需额外扩展。

---

### 阶段 E：UX 打磨

#### Task E.1: 加载/错误状态
- store 的 `loading` + `error` 在列表页消费（spinner / error message）

#### Task E.2: 暗色模式
- header 有 toggle → CSS 变量 `:root` ↔ `.dark` 切换
- store 持 `dark_mode` 状态（已有，视图需消费）

#### Task E.3: 响应式布局（可选）
- 移动端：侧边栏折叠为 drawer（按钮触发）

---

## 5. 实施顺序

```
阶段 A (后端文件夹)
    │
    ▼
阶段 B (block 包设计)
  B.1 升级 note-list
  B.2 新建 note-editor block
  B.3 新建 sidebar-nav block
  B.4 注册 autodown_editor widget
    │
    ▼
阶段 C (AutoDown 集成)
  C.1 Vue 生成器自定义组件透传
  C.2 npm 依赖配置
  C.3 编辑器模板
    │
    ▼
阶段 D (前端两栏树状导航重构 v3)
  D.1 两栏布局骨架（搜索移到 header）
  D.2 树状导航组件（文件夹+笔记+搜索+标签统一）
  D.3 AutoDown 编辑器（保持不变）
  D.4 store（保持不变）
    │
    ▼
阶段 E (UX 打磨) — 可选
```

每阶段完成后 `auto build` + `vue-tsc` + 后端 CRUD 全绿才进下一阶段。

---

## 6. 风险

| 项 | 风险 | 缓解 |
|---|---|---|
| **@autodown/editor 发布** | npm 上可能未发布 | 先用 `file:` 依赖指向本地 `../auto-down`；或发布 npm |
| **Vue 生成器透传** | 未知标签输出为小写 HTML | Task C.1 先做——注册自定义组件映射 |
| **block palette 约束** | `autodown_editor` 不在 WidgetRegistry | Task B.4 先注册 |
| **后端 folder 迁移** | 现有 notes.json 无 folder 字段 | db.at 加默认值 "default"；旧数据兼容 |
| **auto-down 是独立项目** | 版本/API 可能不匹配 | 阶段 C 才集成；先做 A/B/D 的非编辑器部分 |

---

## 7. 025-notes-extended 处理

- **不删除** 025 目录(保持 git 历史)
- README 标注"已归档,能力已迁移至 015-notes"
- 后续不再修改 025

---

## Definition of Done

- [ ] 015-notes 是一个**两栏布局**的笔记应用（树状导航栏 + 编辑器）
- [ ] 导航栏是**统一树状结构**：文件夹是父节点，笔记是子节点（类似文件管理器）
- [ ] 搜索/筛选/快捷入口统一在导航栏内（不分散到多栏）
- [ ] 编辑器是 **AutoDown WYSIWYG**（Tiptap），支持 Markdown 编辑/预览
- [ ] UI 通过 **block 体系**组织（至少 note-list + note-editor + sidebar-nav 三个 block 接入）
- [ ] 后端 CRUD + 文件夹组织 + 持久化正常
- [ ] 加载/错误/空状态在 UI 里展示
- [ ] 暗色模式 toggle + 正确渲染
- [ ] `auto build` + `vue-tsc` GREEN
- [ ] 025 归档标注
- [ ] worktree 分支在全部绿后合并回 `master`
