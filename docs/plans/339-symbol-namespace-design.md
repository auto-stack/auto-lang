# Plan 339：AutoVM Symbol 命名空间 — 设计与实施计划

> **For Claude:** 这是一个架构级设计。当前 VM 把所有模块（db.at、api.at、widget）的符号（函数、全局变量）扁平化到一个命名空间，导致同名冲突。015-notes 暴露出三类冲突：① `create_note`（api.at vs db.at）；② `notes`（db.at 全局变量 vs App.notes state 字段）；③ `use back.api: create_note` 导入别名歧义。当前 workaround（last-wins 去重、module#name 回退、global_vars always-global）脆弱且不完整。本计划设计 Symbol 命名空间系统，彻底解决冲突。

## 1. 问题陈述

### 1.1 冲突类型

#### 类型 A：同名函数跨模块冲突
```
back/api.at:  pub fn create_note(title str, body str) Note { ... }
back/db.at:   pub fn create_note(title str, body str) Note { ... }
```
widget module 扁平化后，两个 `create_note` 同名，当前 last-wins 去重保留 db.at 的。

#### 类型 B：全局变量与 state 字段同名
```
back/db.at:   var notes List<Note> = ...
front/app.at: model { var notes []Note = [] }
```
`db.notes`（后端数据）和 `App.notes`（前端快照）同名，实际是不同的东西。

#### 类型 C：use 导入别名歧义
```
front/app.at:  use back.api: create_note, delete_note, ...
```
裸调用 `create_note` 是 `api.create_note` 还是 `db.create_note`？当前因 last-wins，解析为 `db.create_note`。

#### 类型 D：widget handler 命名冲突
Plan 337 通过 `handler_<Widget>_<Event>` 前缀解决了 handler 命名空间，但函数/变量的跨模块冲突仍未解决。

### 1.2 当前 workaround（都脆弱）

| Workaround | 解决 | 问题 |
|------------|------|------|
| last-wins 去重 | 类型 A | api.at 的 create_note 被丢弃，无法独立调用 |
| module#name linker lookup | 类型 A（递归） | 只用了合成模块名，不是原始模块名 |
| global_vars always-global | 类型 B | 名字仍冲突，只是存储不冲突 |
| 模型层 `.notes` vs 全局 `notes` | 类型 B | 靠 `.` 前缀区分，无真正隔离 |

## 2. 目标架构

### 2.1 模块限定名

每个符号（函数、全局变量）有一个**完整的模块限定名**：

```
db:create_note      — db.at 的 create_note
db:notes            — db.at 的 var notes
api:create_note     — api.at 的 create_note
App:notes           — App widget 的 model.notes（state 字段）
App:active_index    — App widget 的 model.active_index
EditorPanel:editing — EditorPanel widget 的 model.editing
```

**规则**：
- `back/` 下模块用 `模块名:` 前缀（如 `db:`、`api:`）
- `front/` 下 widget 用 `Widget名:` 前缀（如 `App:`、`EditorPanel:`）
- `use` 导入只创建**别名映射**，不改变源符号的限定名

### 2.2 符号解析规则

调用/引用符号时的查找顺序：

1. **当前作用域**（局部变量、函数参数）
2. **use 别名映射**（`create_note` → `api:create_note`）
3. **当前模块限定名**（在 `back/db.at` 中，`create_note` → `db:create_note`）
4. **跨模块限定名**（`db:create_note`、`api:create_note` 直接匹配）
5. **widget state 字段**（`.field` → 当前 widget 的 state 字段）
6. **全局变量**（`notes` 如果在 global scope，解析为当前模块的全局变量）

### 2.3 use 语句语义

```
// 在 front/app.at 中
use back.api: create_note, list_notes, delete_note, update_note
```

含义：创建别名映射：
- `create_note` → `api:create_note`
- `list_notes` → `api:list_notes`
- 等等

解析时，裸调用 `create_note(...)` 先查别名，找到 `api:create_note`，然后调用该限定函数。

```
// 在 db.at 的函数体内
db.create_note(...)     // 直接引用 db 模块的函数（即使存在 use 别名）
```

## 3. 实现方案

### Phase 1 — codegen 层面实现模块限定名

**文件**：`crates/auto-lang/src/vm/codegen.rs`

改动：在编译函数/全局变量时，记录它所属的**模块名**。

```rust
// 新增字段
pub struct Codegen {
    // ... 现有字段 ...
    /// Plan 339: 当前编译的模块名（如 "db", "api", "App"）
    pub current_module: String,
}
```

**影响**：
- 函数导出名从 `create_note` → `db:create_note`
- 全局变量名从 `notes` → `db:notes`
- 调用时使用限定名：`db:create_note()`

### Phase 2 — collect_module_imports 传递模块名

**文件**：`crates/auto-lang/src/lib.rs`（`collect_module_imports`）

在收集每个模块的函数时，标记所属模块名。模块名从文件路径提取：
- `back/db.at` → `"db"`
- `back/api.at` → `"api"`
- `front/editor.at` → `"EditorPanel"`

```rust
fn collect_module_imports(..., module_name: &str) {
    for stmt in &ast.stmts {
        if let Stmt::Fn(f) = stmt {
            // 导出名为 "db:create_note" 而非 "create_note"
            let qualified = format!("{}:{}", module_name, f.name);
            out.push(/* stmt with modified name */);
        }
    }
}
```

### Phase 3 — linker 支持限定名解析

**文件**：`crates/auto-lang/src/vm/loader.rs`

符号表存储限定名：`db:create_note`、`api:create_note`。

重定位解析：
1. 精确匹配 `db:create_note` → 直接找到
2. 回退 `create_note` → 检查 use 别名 → `api:create_note`

### Phase 4 — use 别名映射

**文件**：`crates/auto-lang/src/vm/codegen.rs`

新增 `use_map: HashMap<String, String>`（原始名 → 限定名）。

编译 `use back.api: create_note` 时：
```rust
self.use_map.insert("create_note".into(), "api:create_note".into());
```

在函数/表达式编译中，解析裸调用时：
```rust
fn resolve_call(&self, name: &str) -> String {
    self.use_map.get(name).cloned().unwrap_or_else(|| name.into())
}
```

### Phase 5 — widget state 字段解析

**文件**：`crates/auto-lang/src/ui/handler_codegen.rs`

`.` 前缀的字段（`.notes`、`.active_index`）解析为当前 widget 的 state 字段。

在当前合成模块中，不需要额外限定——handler 已通过 `__state` 参数访问状态。但需要确保：
- `.notes`（state 字段）和 `db:notes`（全局变量）不冲突
- LOAD_GLOBAL 使用限定名（`db:notes`），LOAD_STATE_FIELD 使用 `.notes`

### Phase 6 — 清理 workaround

移除：
- `last-wins` 去重（恢复为 first-wins，因为不会有冲突了）
- `module#name` linker lookup（不再需要，改用限定名直接匹配）
- `global_vars always-global`（全局变量已通过限定名隔离）

## 4. 实施顺序

| Phase | 内容 | 影响面 | 产出 |
|-------|------|--------|------|
| 1 | codegen `current_module` + 限定名导出 | codegen.rs | 函数/变量名带前缀 |
| 2 | `collect_module_imports` 传递模块名 | lib.rs, handler_codegen.rs | 收集时知道来源 |
| 3 | linker 限定名解析 | loader.rs | 符号表存储限定名 |
| 4 | use 别名映射 | codegen.rs, handler_codegen.rs | `use` 语义正确 |
| 5 | widget state 字段隔离 | handler_codegen.rs, vm_bridge.rs | `.field` vs `db:var` |
| 6 | 清理 workaround | 多处 | 移除 last-wins 等 |

## 5. 验收标准

### 5.1 回归（不破坏现有功能）
- [ ] 015-notes vm：list、select、new、save、delete、search（全部通过）
- [ ] 016-calendar vm：回归正常
- [ ] `plan337_tests`：7/7 通过
- [ ] `handler_codegen` 测试：不变

### 5.2 新能力
- [ ] `db:create_note` 和 `api:create_note` 作为不同符号共存
- [ ] `use back.api: create_note` 正确解析到 `api:create_note`
- [ ] `db:notes` 全局变量和 `App.notes` state 字段不冲突
- [ ] last-wins workaround 可移除

## 6. 与后续计划的关系

- **Plan 338（List shims）**：独立于本计划，可在之前或之后做
- **Plan 335（ListData\<Value\> 完整支持）**：部分依赖本计划（filter/map 需要正确的符号解析）
- **Plan 337（单 VM widget 树）**：handler 命名空间是本计划的特殊情况（已通过前缀解决）
