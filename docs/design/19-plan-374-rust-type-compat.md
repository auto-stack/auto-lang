# Plan 374: Rust UI 代码生成器 — 类型兼容性深度修复

## 状态：第三轮进行中（39→0）

---

## 第一+二轮修复（已完成 ✅，已提交 fba936ea）

| # | 修复项 | 影响 |
|---|--------|------|
| 1 | Handler参数绑定：pattern key含params | ~15 |
| 2 | msg回调类型过滤+no-op | ~10 |
| 3 | 递归NotesStore跳过 | 1 |
| 4 | convert_condition中prop/stvar加self | 1 |
| 5 | Expr::None/Some/If处理 | 2 |
| 6 | store字段init移到sync_init之前 | ~2 |
| 7 | TypeDecl类型别名 | 5 |
| 8 | value_prop_names + needs_index_access | ~15 |
| 9 | view闭包中self.t→t (text元素) | 2 |
| 10 | 跳过无Msg变体的handler | 2 |
| 11 | on_change条件fn ptr cast | ~4 |
| 12 | value_loop_vars检测改进 | ~20 |
| 13 | NewNoteInFolder Msg变体添加 | 1 |
| 14 | all_tags computed属性添加 | 1 |
| 15 | NavTree构造参数字母排序+Component.props HashMap→Vec | ~4 |
| 16 | arg_to_rust: self.引用自动.clone() | ~4 |
| 17 | convert_condition后处理.contains().as_str() | ~5 |
| 18 | computed property Vec返回类型+.iter()+.collect() | ~3 |
| 19 | STORE_COMPUTED_NAMES+resolve_dotted_path | ~3 |

---

## 第三轮：39个剩余错误的详细分析+解决方案

### 错误分类总览（39个）

| 类别 | 错误数 | 根因摘要 |
|------|--------|----------|
| **A: bool字段用String** | 4 | `note["pinned"].as_str().to_string()` 用在if条件，应`.as_bool()` |
| **B: .map()未collect到View** | 5 | `iter().enumerate().map(...)` 直接传入`.child()`，需`.collect()`或`.children()` |
| **C: on_change类型不匹配** | 1 | `fn(String)->Msg` 与 `View<Msg>` 类型参数不匹配 |
| **D: &str→String字面量** | 2 | `NewNoteInFolder("work")` 需要 `.to_string()` |
| **E: &Value→String/clone** | 1 | `SelectTag(t)` 中 `t` 是 `&Value`，需要 clone 或转换 |
| **F: view fragment active未定义** | 2 | `NoteItem` 参数 `active` 未替换 |
| **G: self.t in NavTree button** | 1 | loop var `t` 被错误加 `self.` |
| **H: Vec<&Value> vs Vec<Value>** | 2 | computed property `.iter().filter()` 返回引用 |
| **I: .push() on String** | 2 | tags数组的push操作被转为String.push |
| **J: if缺else** | 1 | `if note != None { ... }` 缺else分支 |
| **K: Option<Value>.field** | 2 | `note.title` 但note是 `Option<&mut Value>` |
| **L: E0609 Value残留dot** | 5 | `note["id"]`/`note["pinned"]`/`note["tags"]` 仍有dot访问 |
| **M: E0507 move** | 4 | `self.search`/`self.store.X` 在view的`&self`中move |
| **N: E0599 iter on String/Value** | 2 | `tags.iter()` 但tags被转成String; `Value.iter()` |
| **O: 其他E0308** | 5 | pinned_notes filter返回String非bool等 |

---

### Fix A: bool字段用String (E0308 ×4, E0600 ×1)

**根因**: `value_field_access` 对所有字段统一用 `.as_str().unwrap_or_default().to_string()`。
但 `pinned` 是bool字段，用在 `if` 条件中应返回bool。

**位置**:
- `main.rs:226` — `if self.note["pinned"].as_str()...to_string() { ... }` (2处)
- `main.rs:409` — `n["pinned"].as_str()...to_string()` 在filter中
- `main.rs:496` — `note["pinned"].as_str()...to_string()` 在if条件中

**文件**: `ui_gen/rust.rs` `value_field_access`
**修复**: 根据**字段名**选择访问方式。已知bool字段(`pinned`, `done`, `completed`, `active`, `editing`, `loading`, `dark_mode`, `show_tag_input`)用`.as_bool().unwrap_or(false)`；int字段(`id`, `idx`, `count`)用`.as_i64().unwrap_or(0) as i32`；其余用`.as_str()`。

**影响**: ~5 errors

---

### Fix B: .map()未collect到View (E0308 ×5)

**根因**: `self.store.notes.iter().enumerate().map(|(i, note)| { ... })` 直接作为 `.child()` 参数传入。
`.child()` 期望 `View<M>`，但 `.map()` 返回 `Map<...>` 迭代器。

**位置**: `main.rs:496` 多处 `.child(self.store.notes.iter().enumerate().map(...))`

**文件**: `ui_gen/rust.rs` `generate_view_tree` ForLoop处理
**修复**: 当 ForLoop 的结果传给 `.child()` 时，改用 `.children(iterator.collect())` 或 `.children(iterator.collect::<Vec<_>>())`。具体：ForLoop生成改为 `View::col().children(...)` 包装。

**影响**: ~5 errors

---

### Fix C: on_change类型不匹配 (E0308 ×1)

**根因**: `on_change(EditorPanelMsg::EditTitle as fn(String) -> EditorPanelMsg)` 使整个View的类型参数变为 `fn(String)->Msg` 而非 `Msg`。

**位置**: `main.rs:226` — `View::input(...).on_change(...).build()` 的返回类型

**文件**: `ui_gen/rust.rs` 或 `ui/view.rs`
**修复**: 方案A：修改 `ViewInputBuilder::on_change` 接受 `fn(String) -> M` 并保持 `M` 不变；方案B：改用闭包 `move |s| Msg::Variant(s)`（但闭包不实现Debug）。
**推荐方案A**: 改 View API — `on_change` 参数从 `M` 改为 `Box<dyn Fn(String) -> M>`。

**影响**: 1 error（连锁修复其他View类型错误）

---

### Fix D: &str→String字面量 (E0308 ×2)

**根因**: `NavTreeMsg::NewNoteInFolder("work")` — `"work"` 是 `&str`，但变体是 `NewNoteInFolder(String)`。

**位置**: `main.rs:496` — `NewNoteInFolder("work")`, `NewNoteInFolder("personal")`

**文件**: `ui_gen/rust.rs` `handler_to_rust_direct_msg` 或 `ast_expr_to_rust`
**修复**: 当字符串字面量作为 `Msg::Variant(String)` 的参数时，自动追加 `.to_string()`。

**影响**: 2 errors

---

### Fix E: &Value→String/clone (E0308 ×1)

**根因**: `NavTreeMsg::SelectTag(t)` 中 `t` 是 `&Value`（来自 `|t|` 闭包参数迭代 `all_tags()`），但 `SelectTag(String)` 需要 `String`。

**位置**: `main.rs:496` — `on_click(|_| NavTreeMsg::SelectTag(t))`

**文件**: `ui_gen/rust.rs` 闭包事件处理器
**修复**: 当闭包捕获的loop var是 `&Value` 类型时，在传递给Msg变体时加 `.to_string()` 或 clone+as_str。

**影响**: 1 error

---

### Fix F: view fragment active未定义 (E0425 ×2)

**根因**: `view fn NoteItem(note: Note, active: bool, indent: str)` 内联展开时，参数 `active` 未替换为调用方的实参（如 `i == .store.active_id`）。

**位置**: `main.rs:496` — `if active { ... }`

**文件**: `aura/extract.rs` `expand_fragment_node`
**修复**: 实现参数→实参的AST替换。当展开fragment时，将fragment body中对参数的引用（`.active`、`.note`）替换为调用方传入的实参表达式。

**影响**: 2 errors

---

### Fix G: self.t in NavTree button (E0609 ×1)

**根因**: `format!("{}", self.t)` 在 `|t|` 闭包内。button标签处理中，loop var 加了 `self.` 前缀。

**位置**: `main.rs:496` — `View::button(format!("{}", self.t))`

**文件**: `ui_gen/rust.rs` `generate_view_tree` button text处理
**修复**: 在button标签的format string中检查 `is_loop_var`，不加 `self.` 前缀。

**影响**: 1 error

---

### Fix H: Vec<&Value> vs Vec<Value> (E0308 ×2)

**根因**: `self.notes.iter().filter(|n| ...)` 返回 `Filter<Iter, ...>` 即 `&Value`，但返回类型是 `Vec<Value>`。

**位置**: `main.rs:409` — `pinned_notes` computed property

**文件**: `ui_gen/rust.rs` `generate_computed_impl`
**修复**: computed property的 `.collect()` 加 `.cloned()` 或用 `.collect::<Vec<_>>()` 后类型推断。

**影响**: 2 errors（随Fix A的bool修复一并解决部分）

---

### Fix I: .push() on String (E0308 ×2)

**根因**: `self.note["tags"].as_str()...to_string().push(...)` — Value字段被转成String再push。

**位置**: `main.rs:210` — `self.note["tags"]...to_string().push(self.tag_input.clone())`

**文件**: `ui_gen/rust.rs` `ast_expr_to_rust` Dot处理
**修复**: 对Value类型数组的 `.push()` 调用，生成JSON数组操作：
```rust
// 旧: self.note["tags"].as_str().to_string().push(x)
// 新: self.note["tags"].as_array_mut().unwrap_or(&mut vec![]).push(serde_json::json!(x))
```

**影响**: 2 errors

---

### Fix J: if缺else (E0317 ×1)

**根因**: `if note != None { update_note(...) };` 缺else分支。

**位置**: `main.rs:356`

**文件**: `ui_gen/rust.rs` `ast_stmt_to_rust` If处理
**修复**: handler body中的if语句生成时确保有else（即使是空的 `else {}`）。

**影响**: 1 error

---

### Fix K: Option<Value>.field (E0609 ×2)

**根因**: `note.title` 但 `note` 是 `Option<&mut Value>`。

**位置**: `main.rs:374` — `note.title, note.body`

**文件**: `ui_gen/rust.rs` `ast_expr_to_rust` Dot处理
**修复**: 检测local var是Option类型时，生成 `note.as_ref().and_then(|n| n.get("title")).cloned()`。

**影响**: 2 errors

---

### Fix L: Value残留dot访问 (E0609 ×5)

**根因**: handler body中 `self.notes[idx].pinned`、`self.store.notes[idx].id` 仍用dot访问。

**位置**: `main.rs:94,361`

**文件**: `ui_gen/rust.rs` `ast_expr_to_rust` Dot处理
**修复**: 扩展 `needs_index_access` 检测：Vec<Value>的索引结果也是Value，其字段需要bracket access。

**影响**: 5 errors

---

### Fix M: E0507 move in view (×4)

**根因**: `self.search`、`self.store.active_folder` 等String在 `&self` view方法中被move。

**位置**: `main.rs:102,108` — `NavTree::new(self.search, ...)`、`EditorPanel::new(...)`

**文件**: `ui_gen/rust.rs` `find_constructor_args_for_child`
**修复**: 已有 `arg_to_rust` 添加 `.clone()`，但handler body的constructor_args走的是 `find_constructor_args_for_child` 路径，需要也使用 `arg_to_rust`。

**影响**: 4 errors

---

## 实施优先级

按修复难度（易→难）和影响面（大→小）排序：

1. **Fix A** (bool字段): value_field_access按字段名选择访问器 — ~5 errors
2. **Fix M** (E0507 move): find_constructor_args也用arg_to_rust — 4 errors
3. **Fix D** (&str→String): 字面量参数加.to_string() — 2 errors
4. **Fix L** (Value残留dot): needs_index_access扩展 — 5 errors
5. **Fix G** (self.t→t): button标签loop var — 1 error
6. **Fix E** (&Value→clone): 闭包捕获Value加转换 — 1 error
7. **Fix B** (.map未collect): ForLoop包装children — 5 errors
8. **Fix C** (on_change类型): View API修改 — 1+ errors
9. **Fix J** (if缺else): handler if加else — 1 error
10. **Fix I** (Value push): JSON数组操作 — 2 errors
11. **Fix K** (Option field): Option unwrap — 2 errors
12. **Fix F** (view fragment): 参数替换 — 2 errors
13. **Fix H** (Vec<&Value>): .cloned() — 2 errors
