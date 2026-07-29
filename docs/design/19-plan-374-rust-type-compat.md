# Plan 374: Rust UI 代码生成器 — 类型兼容性深度修复

## 状态：第二轮进行中（41→0）

---

## 第一轮修复（已完成 ✅）

| # | 修复项 | 文件 | 修复数 |
|---|--------|------|--------|
| 1 | Handler参数绑定：pattern key含params | `aura/extract.rs` | ~15 |
| 2 | msg回调类型过滤+no-op | `ui_gen/rust.rs` | ~10 |
| 3 | 递归NotesStore跳过 | `ui_gen/rust.rs` | 1 |
| 4 | convert_condition中prop/stvar加self | `ui_gen/rust.rs` | 1 |
| 5 | Expr::None/Some/If处理 | `ui_gen/rust.rs` | 2 |
| 6 | store字段init移到sync_init之前 | `ui_gen/rust.rs` | ~2 |
| 7 | TypeDecl类型别名 | `auto-man/rust_ui.rs` | 5 |
| 8 | value_prop_names + needs_index_access | `ui_gen/rust.rs` | ~15 |
| 9 | view闭包中self.t→t (text元素) | `ui_gen/rust.rs` | 2 |
| 10 | 跳过无Msg变体的handler | `ui_gen/rust.rs` | 2 |
| 11 | on_change条件fn ptr cast | `ui_gen/rust.rs` | ~4 |
| 12 | value_loop_vars检测改进 | `ui_gen/rust.rs` | ~20 |
| 13 | NewNoteInFolder Msg变体添加 | `notes_store.at` | 1 |
| 14 | all_tags computed属性添加 | `notes_store.at` | 1 |

---

## 第二轮：41个剩余错误的详细解决方案

### 错误1: NavTree构造参数顺序错误 (E0061, E0308)
**位置**: `main.rs:102,108` — `NavTree::new(active_id, search, folder, tag)` 应为 `NavTree::new(search, folder, tag, active_id)`
**根因**: `find_constructor_args_for_child` 按props出现顺序组装参数，但构造函数参数顺序由widget声明定义，不一致。
**文件**: `ui_gen/rust.rs` `find_constructor_args_for_child`
**修复**: 按widget的props声明顺序排列构造参数，而非view tree中props的出现顺序。
**影响**: ~4 errors

---

### 错误2: convert_condition中Value prop的dot访问 (E0609)
**位置**: `main.rs:226` — `self.note.pinned` 应为 `self.note["pinned"]`
**根因**: `convert_condition`处理`self.note.pinned`时，`note`和`pinned`之间的dot被识别为method call dot。虽然`note`在`value_prop_names`中，但修复代码错误地又加了一次`self.`前缀。
**文件**: `ui_gen/rust.rs` `convert_condition`
**修复**: 当`is_method_call=true`且前面标识符在`value_prop_names`中时，将`var.field`替换为`var["field"]`格式的bracket access，而不是再次加`self.`。
**影响**: ~2 errors

---

### 错误3: String:Pattern trait bound (E0277)
**位置**: `main.rs:多处` — `.contains(self.store.active_tag)` 其中`active_tag`是`String`
**根因**: `str::contains`期望`impl Pattern`，`String`不直接实现Pattern。需要`.as_str()`。
**文件**: `ui_gen/rust.rs` `ast_expr_to_rust` BinOp处理
**修复**: 在生成`.contains(rhs)`时，如果rhs是String类型，添加`.as_str()`转换。
**影响**: ~5 errors

---

### 错误4: computed property中Vec的filter/map (E0599)
**位置**: `main.rs:409,413` — `self.notes.filter(...)` 应为 `self.notes.iter().filter(...)`
**根因**: computed property表达式在Rust中直接翻译，但`Vec<T>`没有`.filter()`(只有Iterator有)。
**文件**: `ui_gen/rust.rs` `generate_computed_impl`
**修复**: 对computed property中的`.filter()`/`.map()`调用，在前面插入`.iter()`。
**影响**: ~2 errors

---

### 错误5: all_tags方法调用语法 (E0615)
**位置**: `main.rs:496` — `self.store.all_tags.iter()` 应为 `self.store.all_tags().iter()`
**根因**: computed property生成的是方法(`fn all_tags()`)，但view代码中作为字段访问。
**文件**: `ui_gen/rust.rs` `ast_expr_to_rust` Dot处理 或 `generate_view_tree`
**修复**: 在Dot访问处理中，如果右端是已知的computed property名，自动添加`()`。
**影响**: 1 error

---

### 错误6: NavTree button中self.t → t (E0609)
**位置**: `main.rs:496` — `format!("{}", self.t)` 在`|t|`闭包内
**根因**: button标签的text内容处理中，对loop var加了`self.`前缀。之前的修复只覆盖了`text`元素，没覆盖`button`的标签。
**文件**: `ui_gen/rust.rs` `generate_view_tree` button text处理
**修复**: 在生成button的text_styled调用时，也检查`is_loop_var`。
**影响**: 1 error

---

### 错误7: handler body中Value数组操作 (E0599)
**位置**: `main.rs:210,214` — `.push()` on String, `.iter()` on String
**根因**: `self.note["tags"]`的Value被转换成了String再操作，应该直接操作Value/JSON数组。
**文件**: `ui_gen/rust.rs` `ast_expr_to_rust` Dot处理
**修复**: 对于Value类型的`.push()`和`.iter()`调用，生成正确的JSON数组操作代码。
**影响**: ~2 errors

---

### 错误8: view fragment内联参数替换 (E0425)
**位置**: `main.rs:496` — `if active { ... }` 但`active`未定义
**根因**: `view fn NoteItem(note, active, indent)`内联时，参数`active`未替换为调用方的实参。
**文件**: `aura/extract.rs` view fragment展开逻辑
**修复**: 实现参数→实参的AST级别字符串替换。
**影响**: ~2 errors

---

### 错误9: Option<Value>.field访问 (E0609)
**位置**: `main.rs:383` — `note.title`但`note`是`Option<&mut Value>`
**根因**: `var note = None; ... note = Some(n); ... note.title`中，`note`是Option类型，需要unwrap。
**文件**: `ui_gen/rust.rs` `ast_expr_to_rust`
**修复**: 检测Option类型的dot访问，插入unwrap或使用`as_ref().map()`。
**影响**: ~2 errors

---

### 错误10: if缺少else分支 (E0317)
**位置**: `main.rs:496` view chain中
**根因**: multiline view链中的某个if表达式没有else分支。
**文件**: `ui_gen/rust.rs` AuraNode::Conditional处理
**修复**: 为所有if表达式生成else分支（即使为空也要`else { View::Empty }`）。
**影响**: 1 error

---

### 错误11: update_note函数参数 (E0061)
**位置**: `main.rs:383` — `update_note(id, note.title, note.body)` 参数不匹配
**根因**: Option unwrap问题导致参数数量或类型错误。
**文件**: API stub生成 + 上述Option修复
**修复**: 修复Option访问后自动解决。
**影响**: ~1 error（随错误9修复）

---

### 错误12: mismatched types (E0308, ~10 errors)
**位置**: 多处
**根因**: 各种类型不匹配：String vs &str, i32 vs usize等。
**文件**: `ui_gen/rust.rs` 多处
**修复**: 
- String比较用`.as_str()`
- 索引转换加`as usize`
- 随其他修复大部分自动解决
**影响**: ~10 errors（大部分随其他修复解决）

---

## 实施顺序

按影响面从大到小：

1. **Fix 1**: NavTree构造参数顺序 → ~4 errors
2. **Fix 3**: String:Pattern contains → ~5 errors
3. **Fix 2**: convert_condition Value bracket → ~2 errors
4. **Fix 4**: computed property iter → ~2 errors
5. **Fix 5**: all_tags() 方法调用 → 1 error
6. **Fix 6**: self.t in button → 1 error
7. **Fix 7**: Value数组push/iter → ~2 errors
8. **Fix 8**: view fragment inlining → ~2 errors
9. **Fix 9**: Option<Value> → ~3 errors
10. **Fix 10**: if missing else → 1 error
11. 剩余类型不匹配 → ~18 errors（大部分随1-10解决）

## 修改文件清单

| 文件 | 修改项 |
|------|--------|
| `crates/auto-lang/src/ui_gen/rust.rs` | Fix 1-7, 9-10 |
| `crates/auto-lang/src/aura/extract.rs` | Fix 8 |
| `examples/ui/015-notes/src/front/notes_store.at` | 源文件修复（已完成）|
