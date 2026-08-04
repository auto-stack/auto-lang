# Plan 374: a2r 支持 store composable + view fn fragment — Rust 模式 015-notes Parity

> **状态**: ✅ 已完成并归档（2026-08-04）。实测确认全部 4 个 Task 已实现：view fn fragment（parser `ViewFragmentDecl` + `parse_view_fragment_decl_body` + rust.rs `tag_to_view_fn`/`View::{fn}` 生成）、store composable（`rust_ui.rs` `collect_store_decls` 预扫描 + `ui_gen/rust.rs` `register_store`/`STORE_NAMES` thread-local + store 字段持久化）、store 导入处理、aura/extract 复杂表达式+参数名支持。
> **目标**: 让 a2r 转译器支持 `store` composable 和 `view fn` fragment，使 `auto run -r rust` 生成的 015-notes 达到与 VM 模式一致的功能 parity。
>
> **核心挑战**: a2r 的 UI 生成管线（`auto-man/rust_ui.rs` → `aura/extract.rs` → `ui_gen/rust.rs`）完全跳过了 `StoreDecl` 和 `ViewFragmentDecl`，导致 store 字段（dark_mode/accent_color/active_folder 等）和 view fn 片段（NoteRow/NoteItem）在生成的 Rust 代码中缺失。

---

## 1. 问题根因

### 1.1 架构路径

```
.at 源码 → parser → AST → rust_ui.rs:compile_at_file → aura/extract.rs → ui_gen/rust.rs → main.rs
```

`compile_at_file`（`auto-man/rust_ui.rs:318-332`）只处理 `Stmt::WidgetDecl`，跳过了：
- `Stmt::StoreDecl`（store composable 声明）→ 静默丢弃
- `Stmt::ViewFragmentDecl`（view fn 片段声明）→ 静默丢弃

### 1.2 store composable 缺失

- `notes_store.at` 的 `store NotesStore { ... }` 被完全忽略
- `use notes_store: NotesStore` 不加载任何 store 定义
- `store.X` 引用在生成的代码中变成未定义标识符（`store.active_id` → 编译错误或运行时 panic）
- 缺失字段：notes, active_id, active_folder, active_tag, dark_mode, accent_color, sort_mode, loading, all_tags
- 缺失方法：store.NewNote(), store.TogglePin(), store.SetAccent(), store.ToggleDarkMode()

### 1.3 view fn fragment 缺失

- `sidebar.at` 的 `view fn NoteRow(...)` / `view fn NoteItem(...)` 被解析但从不注册到 `VIEW_FRAGMENTS`
- `extract_view_node` 找不到已注册的片段 → `NoteItem(...)` 调用不被内联展开
- 生成的 NavTree 中 `NoteItem` 调用参数顺序/数量错误（fragment 参数 vs widget 参数不匹配）

### 1.4 VM 模式的参考实现

VM 模式（`lib.rs`）已正确处理这两个特性：
- **StoreDecl → view-less WidgetDecl**（`lib.rs:2559-2581`）：store 被转换为无视图的子组件，字段合并进 root state
- **ViewFragmentDecl → register_view_fragment**（`lib.rs:2408-2413`）：在每个模块提取 widget 前注册片段

---

## 2. 方案

### Task 1: view fn fragment 注册（最小改动，最高杠杆）

**文件**: `auto-man/rust_ui.rs` 的 `compile_at_file`（~line 318）

在 `WidgetDecl` 循环之前，添加 fragment 注册（镜像 `lib.rs:2408-2413`）：

```rust
auto_lang::aura::extract::clear_view_fragments();
for stmt in &ast.stmts {
    if let auto_lang::ast::Stmt::ViewFragmentDecl(frag) = stmt {
        auto_lang::aura::extract::register_view_fragment(frag);
    }
}
```

同样修改 `ui_gen/api.rs:transpile_aura`（~line 51）。

**效果**: `NoteItem(...)` 调用在 `extract_view_node` 中自动内联展开为 `button { text note.title ... }`。`ui_gen/rust.rs` **无需修改**——它只看到普通的 `AuraNode::Element` 节点。

**验证**: 重新生成 015-notes，检查 NavTree 的 `view()` 输出是否包含内联的笔记列表项按钮（含 `onclick: SelectNote(i)`）。

### Task 2: store composable 转译 — 方案选择

**方案 B（推荐）: 独立 struct + 委托调用**

生成一个独立的 `NotesStore` Rust struct，包含 store 的字段和方法：

```rust
pub struct NotesStore {
    pub notes: Vec<serde_json::Value>,
    pub active_id: i32,
    pub active_folder: String,
    pub active_tag: String,
    pub dark_mode: bool,
    pub accent_color: String,
    // ...
}

impl NotesStore {
    pub fn new() -> Self { /* 初始值 */ }
    
    pub fn on(&mut self, msg: NotesStoreMsg) {
        match msg {
            NotesStoreMsg::NewNote => { /* ... */ }
            NotesStoreMsg::TogglePin(idx) => { /* ... */ }
            // ...
        }
    }
}
```

在 `App` struct 中添加 `pub store: NotesStore` 字段。

**`store.X` 路径重写**（`ui_gen/rust.rs:ast_expr_to_rust`）：
- `store.field` → `self.store.field`
- `self.store.field` → `self.store.field`（已正确）
- `.store.field` → `self.store.field`
- `store.Method(args)` → `self.store.on(NotesStoreMsg::Method(args))`

**实现步骤**：

1. `compile_at_file` 中识别 `Stmt::StoreDecl`，调用 `extract_widget_from_decl` 转换为 `AuraWidget`（view=None）
2. `RustGenerator` 正常生成 `NotesStore` struct + `Component` impl（它已经能处理 view-less widget）
3. 在 `App` 的 struct 生成中，如果检测到 store 子组件，添加 `pub store: NotesStore` 字段
4. `ast_expr_to_rust` 中添加 `store.` 前缀重写规则
5. store 方法调用 `store.Method()` 重写为消息分发

### Task 3: store 导入处理

**文件**: `auto-man/rust_ui.rs` 的 `compile_at_file`

当遇到 `use notes_store: NotesStore` 时：
1. 解析被导入的模块文件（`notes_store.at`）
2. 提取其中的 `StoreDecl`
3. 转换为 `AuraWidget` 并生成

镜像 `lib.rs:2498-2527` 的 imported-store 逻辑。

### Task 4: 验证 — 重新生成 + 编译 + MCP 测试

1. `auto run -r rust`（从 `examples/ui/015-notes` 目录）
2. 检查生成的 `main.rs` 是否包含：
   - `NotesStore` struct（含 notes/active_id/dark_mode/accent_color 等字段）
   - `App` struct 含 `pub store: NotesStore` 字段
   - NavTree view 含内联的笔记列表项按钮
   - `store.X` 引用被正确重写为 `self.store.X`
3. `cargo build -p notes --features ui-iced` 编译通过
4. 运行 iced 窗口，通过 MCP 验证：
   - `autoui_exists(button, Dark)` → FOUND（暗黑模式按钮）
   - `autoui_exists(text, 📁 Notes)` → FOUND（文件夹标题）
   - `autoui_find(button, label=All)` → FOUND（view tab）
5. 运行 `.autotest` 套件（`--mode rust`），验证场景通过率提升

---

## 3. 实施顺序

1. **Task 1**（view fn 注册）— 最小改动（~10 行），立即解锁 NavTree 笔记列表渲染
2. **Task 2**（store composable 转译）— 核心工作，解锁 dark_mode/accent_color/folders/tags
3. **Task 3**（store 导入）— 依赖 Task 2
4. **Task 4**（验证）— 端到端

---

## 4. Parity 目标矩阵

| 功能 | 当前 Rust | Task 1 后 | Task 2-3 后 | VM/Web |
|------|----------|----------|------------|--------|
| 笔记列表（内联展开） | ❌ NoteItem 参数错误 | ✅ | ✅ | ✅ |
| 文件夹分组 | ❌ | ✅ | ✅ | ✅ |
| View tabs (All/Pinned/Recent) | ❌ 无 store.active_folder | ⚠️ | ✅ | ✅ |
| 搜索过滤 | ✅ 独有 | ✅ | ✅ | ✅ |
| 暗黑模式 | ❌ | ❌ | ✅ | ✅ |
| 主题色 5 色 | ❌ | ❌ | ✅ | ✅ |
| Pin 置顶 | ❌ | ❌ | ✅ | ✅ |
| Tag 筛选 | ❌ | ⚠️ | ✅ | ✅ |
| Edit/Save/Cancel | ✅ | ✅ | ✅ | ✅ |
| New/Delete | ✅ | ✅ | ✅ | ✅ |
| MCP 操作 | ✅ | ✅ | ✅ | N/A |

---

## 5. 技术风险

| 风险 | 影响 | 缓解 |
|------|------|------|
| store 方法调用的消息类型生成 | `store.NewNote()` 需要生成 `NotesStoreMsg::NewNote` | 复用现有 WidgetDecl 的消息生成逻辑 |
| `store.X` 重写遗漏边界情况 | 某些表达式路径未被重写 | 全面搜索 `ast_expr_to_rust` 中的 Dot/Ident 分支 |
| view fn 参数类型推断 | fragment 参数（如 `indent: str`）在 Rust 中需要类型 | aura extract 已处理参数替换，Rust 侧不需要额外类型 |
| 编译错误链式修复 | 修复 store 后可能暴露新的 a2r 代码生成缺陷 | 增量修复，每个缺陷独立 commit |

---

## 6. 不在本次范围

- 不修改 .at 源码（所有改动在 a2r 转译器侧）
- 不实现 store computed（Plan 367 Phase 4 的范畴）
- 不实现 a2r 对 autodown_editor 的原生支持（继续用 textarea 降级）
- 不处理 Vue+Rust 模式（那是浏览器端）
