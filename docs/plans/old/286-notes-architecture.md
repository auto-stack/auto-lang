# Plan 286: 015-notes 全栈架构重构

> 日期: 2026-06-06
> 状态: ✅ 已完成 — 多 Widget（App + Sidebar + NoteItem + Editor + types）、多模块、前后端分离架构均已实现。

## 目标

将 `examples/ui/015-notes` 从单文件纯前端笔记应用重构为 **多 Widget + 多模块 + 全栈** 的标杆示例，验证 AutoLang 的以下能力：

1. **多 Widget 组合** — 父子 widget 的 props/events 传递
2. **多模块导入** — `use` 语句在 UI 项目中的实际使用
3. **类型共享** — 跨前后端模块的结构体定义
4. **全栈能力** — Auto HTTP API + Tauri IPC 双模式通信
5. **数据持久化** — JSON 文件读写

## 架构决策

| 决策项 | 选择 | 理由 |
|--------|------|------|
| 项目结构 | 单体混合 | 前后端同一项目，结构简洁 |
| Widget 粒度 | 4 widget | App + Sidebar + NoteItem + EditorPanel + types |
| 数据持久化 | JSON 文件 (Auto 格式) | 验证 Auto 文件 I/O 和序列化 |
| 前后端通信 | HTTP + Tauri 双模式 | 参照 api-example，开发用 HTTP，打包用 Tauri |

## 文件结构

```
015-notes/
├── pac.at                        # 项目配置 (scene: "ui", 多 backend)
├── src/
│   ├── front/                    # 前端代码
│   │   ├── app.at                # App 根 widget: 状态容器 + 布局 + API 调用
│   │   ├── sidebar.at            # Sidebar widget: 搜索 + 笔记列表
│   │   ├── note_item.at          # NoteItem widget: 单条笔记摘要卡片
│   │   ├── editor.at             # EditorPanel widget: 笔记编辑区
│   │   └── types.at              # 共享类型: Note 结构体
│   └── back/                     # 后端代码
│       ├── api.at                # API 接口定义 (#[api] 标注)
│       ├── db.at                 # 数据服务层 (CRUD 操作)
│       └── store.at              # 文件存储层 (JSON 读写)
├── data/                         # 运行时数据目录
│   └── notes.json                # 笔记持久化文件
└── gen/                          # 生成产物
    └── vue/                      # Vue 前端 + Vite proxy 到后端
```

## 数据流

```
前端 Widget                    后端 API                    存储层
─────────────                  ──────────                  ──────
App (状态容器)
 │
 ├── Sidebar                  ┌──────────┐
 │   └── NoteItem      ───HTTP/IPC──▶  api.at  │
 │                                       │
 └── EditorPanel     ───HTTP/IPC──▶  db.at   ──▶  store.at
                                        │              │
                                        │         notes.json
                                 (内存 List<Note>)   (文件系统)
```

- App 是唯一调用后端 API 的 widget（单一数据源）
- 子 widget 通过 props 接收数据，通过 events 向上冒泡操作请求
- Sidebar 和 EditorPanel 之间没有直接通信，全由 App 中转

## 共享类型 (`src/front/types.at`)

```auto
type Note {
    id int
    title str
    body str
    time str
}

type CreateNoteRequest {
    title str
    body str
}

type UpdateNoteRequest {
    title str
    body str
}
```

## 后端设计

### 三层架构

**Layer 1: `store.at` — 文件存储层**
- `load_notes() -> str` — 从 `data/notes.json` 读取 JSON 字符串
- `save_notes(data str) -> void` — 将 JSON 字符串写入文件
- 初始化时创建默认文件（如不存在）

**Layer 2: `db.at` — 数据服务层**
- 启动时从 store 加载数据到内存 `List<Note>`
- 每次变更后自动写回文件
- 接口：
  ```auto
  pub fn init() void                      // 从文件加载初始数据
  pub fn all_notes() []Note               // 获取全部笔记
  pub fn get_note(id int) Note?           // 按 ID 获取
  pub fn create_note(title str, body str) Note   // 创建并持久化
  pub fn update_note(id int, title str, body str) Note?  // 更新并持久化
  pub fn delete_note(id int) bool         // 删除并持久化
  pub fn search_notes(query str) []Note   // 按标题/内容搜索
  ```

**Layer 3: `api.at` — API 接口层**
- 使用 `#[api]` 注解标注，自动生成 HTTP 路由和 Tauri 命令
- RESTful 风格端点：
  | Method | Path | 函数 | 说明 |
  |--------|------|------|------|
  | GET | `/api/notes` | `listnotes` | 列出所有笔记 |
  | GET | `/api/notes/:id` | `getnote` | 获取单条笔记 |
  | POST | `/api/notes` | `createnote` | 创建笔记 |
  | PUT | `/api/notes/:id` | `updatenote` | 更新笔记 |
  | DELETE | `/api/notes/:id` | `deletenote` | 删除笔记 |
  | GET | `/api/notes/search?q=` | `searchnotes` | 搜索笔记 |

## 前端 Widget 设计

### Widget 1: `App` — 根容器

**职责:** 全局状态持有、API 调用、子 widget 协调、页面布局

```auto
use sidebar: Sidebar
use note_item: NoteItem
use editor: EditorPanel

widget App {
    msg Msg {
        LoadNotes                           // 初始化: 从后端加载
        SelectNote(id int)                  // 选中笔记
        NewNote                             // 新建笔记
        DeleteNote(id int)                  // 删除笔记
        SaveNote(id int, title str, body str)  // 保存编辑
        SearchChanged(query str)            // 搜索关键词变更
        NotesLoaded(notes []Note)           // 后端返回数据
        NoteCreated(note Note)              // 后端创建完成
        NoteUpdated(note Note)              // 后端更新完成
        NoteDeleted(id int)                 // 后端删除完成
    }

    model {
        var notes []Note = []
        var active_id int = -1
        var search str = ""
        var loading bool = false
    }

    computed {
        filtered_notes => /* notes 按 search 过滤 */
        active_note => /* notes 中 id == active_id 的笔记 */
    }

    view {
        col {
            // Header
            row {
                h2 "Notes" { style: "text-xl font-bold text-gray-800" }
                button "+ New" { onclick: .NewNote, style: "ml-auto px-4 py-2 bg-blue-500 text-white rounded-lg text-sm" }
                style: "w-full items-center p-4 border-b border-gray-200"
            }
            // Body: Sidebar + Editor
            row {
                Sidebar(notes: .filtered_notes, active_id: .active_id)
                EditorPanel(note: .active_note)
                style: "flex-1"
            }
            style: "w-full h-screen bg-white flex-col"
        }
    }

    on {
        // --- API 调用 ---
        .LoadNotes -> {
            .loading = true
            .notes = listnotes()        // 调用后端 API
            .loading = false
            if notes.len() > 0 {
                .active_id = notes[0].id
            }
        }
        .NewNote -> {
            let note = createnote("", "")    // 调用后端 API
            notes.push(note)
            .active_id = note.id
        }
        .DeleteNote(id) -> {
            deletenote(id)                   // 调用后端 API
            notes.remove(/* by id */)
            .active_id = notes.len() > 0 ? notes[0].id : -1
        }
        .SaveNote(id, title, body) -> {
            let note = updatenote(id, title, body)   // 调用后端 API
            // 更新本地 notes 数组
        }
        .SearchChanged(query) -> {
            .search = query
        }
        .SelectNote(id) -> {
            .active_id = id
        }
    }
}
```

### Widget 2: `Sidebar` — 侧栏

**职责:** 搜索输入 + 笔记列表容器

**Props 输入:** `notes []Note`, `active_id int`
**Events 输出:** `SelectNote(id)`, `SearchChanged(query)`

```auto
use note_item: NoteItem

widget Sidebar {
    msg Msg {
        SelectNote(id int)
        SearchChanged(query str)
    }

    model {
        var notes []Note = []
        var active_id int = -1
        var search str = ""
    }

    view {
        col {
            input { placeholder: "Search...", value: .search, oninput: .SearchChanged, style: "w-full px-3 py-2 text-sm border rounded-lg mb-2" }
            for note in .notes {
                NoteItem(note: note, is_active: note.id == .active_id)
            }
            style: "w-64 border border-gray-200 p-3 flex-shrink-0 gap-1 overflow-y-auto"
        }
    }

    on {
        .SelectNote(id) -> { /* emit 到父组件 */ }
        .SearchChanged(query) -> { .search = query; /* emit 到父组件 */ }
    }
}
```

### Widget 3: `NoteItem` — 笔记列表项

**职责:** 渲染单条笔记的摘要，处理选中状态

**Props 输入:** `note Note`, `is_active bool`
**Events 输出:** `SelectNote`, `DeleteNote`

```auto
widget NoteItem {
    msg Msg {
        Select
        Delete
    }

    model {
        var note Note = { id: 0, title: "", body: "", time: "" }
        var is_active bool = false
    }

    view {
        button .note.title {
            onclick: .Select
            style: "w-full text-left p-3 rounded-lg text-sm font-semibold text-blue-600 hover:bg-blue-50"
        }
        // is_active 时显示高亮背景
    }

    on {
        .Select -> { /* emit SelectNote(note.id) 到父 */ }
        .Delete -> { /* emit DeleteNote(note.id) 到父 */ }
    }
}
```

### Widget 4: `EditorPanel` — 编辑面板

**职责:** 显示/编辑当前选中的笔记

**Props 输入:** `note Note` (当前选中笔记)
**Events 输出:** `SaveNote(id, title, body)`, `DeleteNote(id)`

```auto
widget EditorPanel {
    msg Msg {
        Edit
        Save
        Cancel
        EditBody
        EditTitle
        Delete
    }

    model {
        var note Note = { id: 0, title: "", body: "", time: "" }
        var editing bool = false
        var edit_title str = ""
        var edit_body str = ""
    }

    view {
        col {
            if .editing == false {
                text .note.title { style: "text-lg font-semibold" }
                text .note.time { style: "text-xs text-gray-400 mt-1" }
                text .note.body { style: "text-gray-700 flex-1 leading-relaxed" }
            }
            if .editing == true {
                input { value: .edit_title, oninput: .EditTitle, placeholder: "Note title..." }
                textarea { value: .edit_body, oninput: .EditBody, placeholder: "Start writing..." }
            }
            // Action bar
            row {
                if .editing == false {
                    button "Edit" { onclick: .Edit }
                }
                if .editing == true {
                    button "Save" { onclick: .Save }
                    button "Cancel" { onclick: .Cancel }
                }
                button "Delete" { onclick: .Delete, style: "ml-auto" }
            }
            style: "flex-1 flex-col"
        }
    }

    on {
        .Edit -> {
            .edit_title = .note.title
            .edit_body = .note.body
            .editing = true
        }
        .Save -> {
            // emit SaveNote(note.id, edit_title, edit_body) 到父
            .editing = false
        }
        .Cancel -> { .editing = false }
        .EditBody -> { /* 更新 edit_body */ }
        .EditTitle -> { /* 更新 edit_title */ }
        .Delete -> { /* emit DeleteNote(note.id) 到父 */ }
    }
}
```

## 实现步骤

### Phase 1: 前端多 Widget 拆分
1. 创建 `types.at`，定义 Note 类型
2. 创建 `note_item.at`，实现 NoteItem widget
3. 创建 `sidebar.at`，实现 Sidebar widget（引用 NoteItem）
4. 创建 `editor.at`，实现 EditorPanel widget
5. 重构 `app.at`，引用子 widget，删除内联逻辑
6. 验证：`auto build` + 浏览器中确认 UI 与原有一致

### Phase 2: 后端 API 层
7. 创建 `store.at`，实现 JSON 文件读写
8. 创建 `db.at`，实现内存 CRUD + 自动持久化
9. 创建 `api.at`，定义 #[api] 接口
10. 创建 `data/notes.json`，初始示例数据
11. 验证：API 端点可用，CRUD 操作正常

### Phase 3: 前后端联调
12. 在 App widget 中集成 API 调用
13. 配置 Vite proxy（开发模式）和 Tauri 命令注册（打包模式）
14. 端到端验证：创建、编辑、删除笔记，刷新后数据仍在

## 验证要点

- [ ] 多 widget `use` 导入正常工作
- [ ] Props 从父组件传递到子组件
- [ ] Events 从子组件冒泡到父组件
- [ ] 后端 API 的 GET/POST/PUT/DELETE 全部正常
- [ ] JSON 文件读写正常，数据重启后仍在
- [ ] HTTP 模式和 Tauri 模式均可运行
- [ ] UI 外观与原有单文件版本一致

## 风险与注意事项

1. **Widget 间通信语法**: 当前 AutoUI 的 props/events 传递机制可能需要验证或扩展。如果现有的 `Widget(prop: value)` 语法不能正确传递到子 widget，可能需要先完善编译器支持。
2. **use 模块解析**: `use sidebar: Sidebar` 的模块路径解析需要确认在 `src/front/` 目录下的行为。
3. **异步 API 调用**: 前端调用后端 API 可能涉及异步处理，需要确认 `.notes = listnotes()` 的同步/异步语义。
4. **文件路径**: `data/notes.json` 的相对路径在不同运行模式下（开发/打包）可能不同，需要处理。
