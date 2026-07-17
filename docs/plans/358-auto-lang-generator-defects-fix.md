# Plan 358: Auto 语言生成器/编译器缺陷系统性修复

> **类型**: 架构改进 + Bug 修复
> **状态**: 待评审
> **日期**: 2026-07-17
> **来源**: 015-notes 开发过程中发现的所有语言/生成器缺陷
> **原则**: 所有修复必须是编译器/转译器/生成器层面的架构正确方案，而非 workaround

---

## 1. 缺陷总览

| # | 缺陷 | 类别 | 严重度 | 当前状态 |
|---|---|---|---|---|
| D1 | Vue 生成器 OOM（for + style:if + msg/on） | 生成器 | 🔴 高 | Workaround：内联到 App.vue |
| D2 | Store handler 数组操作（`+` 变字符串拼接） | 转译器 | 🟡 中 | Workaround：computed getter |
| D3 | Store handler 方法间调用（链式调用 bug） | 转译器 | 🟡 中 | Workaround：避免方法间调用 |
| D4 | `tag` 保留关键字冲突 | Parser | 🟢 低 | Workaround：改用 `t` |
| D5 | View DSL for 迭代器不支持 `ident.field` | Parser | 🟡 中 | Workaround：避免嵌套 for |
| D6 | a2r 不支持多类型 CRUD | 代码生成 | 🟡 中 | Workaround：Folder 硬编码 |
| D7 | Vue 生成器根元素 style:if 丢失 | 生成器 | 🟢 低 | ✅ 已修复 |
| D8 | Store composable 非单例 | 生成器 | 🟢 低 | ✅ 已修复 |
| D9 | autodown_editor 导致生成器卡住 | 生成器 | 🔴 高 | Workaround：用 textarea |
| D10 | a2r 不读取 db.at seed data | 代码生成 | 🟢 低 | ✅ 已修复 |

---

## 2. 详细分析与架构方案

### D1: Vue 生成器 OOM（for + style:if + msg/on）

**现象**：当 widget 同时有 `for` 循环 + `style:if` 条件样式 + `msg`/`on` 块时，生成器消耗 1.7GB+ 内存。

**根因推测**：
- `generate_script` 扫描 view 树寻找 handler 绑定时，遇到 `for` + `style:if` 组合，可能对每个迭代变体展开 handler 代码
- 或者 `extract_classes` 处理 `Expr::If` 时与 handler 扫描产生笛卡尔积
- 精确触发条件：`for tag in .all_tags { button { style:if tag == "a" { ... } else { ... } } }` + widget 有 `msg Msg { ... }` 和 `on { ... }`

**架构方案**：
- **分离模板生成与 handler 分析为两阶段**：
  1. **分析阶段**：单次遍历 view 树，收集所有 handler 绑定（O(N)）
  2. **生成阶段**：用收集的数据独立生成 template 和 script
- 消除模板生成与 handler 分析的交叉递归
- 这是 Plan 356 的策略 C

**实施**：
1. 在 `generate_sfc` 入口加节点计数日志，用最小复现 case 定位爆炸函数
2. 根据诊断结果修复具体算法
3. 加压力测试：100 个嵌套 if/for 节点，验证时间 < 5s、内存 < 200MB

---

### D2: Store handler 数组操作（`+` 变字符串拼接）

**现象**：Auto 的 `var tags []str = []` 被生成为 JS 的 `let tags = ''`（空字符串），导致 `tags + [t]` 变成字符串拼接。

**根因**：
- `store_init_to_js`（vue.rs:7876）对 `Expr::Array(_)` 返回 `"[]"`，这是正确的
- 但 `transpile_handler_body` 在转译 `var tags []str = []` 时，可能走了不同的路径（通过 ts_adapter 的 `transpile_stmt`），把 `[]str` 类型推断为 string
- 或者 `var` 声明的初始值 `[]` 被丢失，默认为空字符串

**架构方案**：
- **ts_adapter 的 `transpile_stmt` 在处理 `Stmt::Store` 时，根据 store 的类型标注（`[]str`）决定初始值**：
  - `[]str` / `[]Note` → `[]`（空数组）
  - `str` → `''`
  - `int` → `0`
  - `bool` → `false`
- 确保 `+` 运算符在数组上下文中生成 `push` 或展开运算 `[...arr, item]`

**实施**：
1. 在 `ts_adapter.rs` 的 `transpile_stmt` 中，`Stmt::Store` 分支检查类型标注
2. 如果类型是数组，初始值用 `[]`
3. 如果 `+` 的右操作数是数组字面量 `[x]`，生成 `[...left, ...right]` 而非 `left + right`

---

### D3: Store handler 方法间调用（链式调用 bug）

**现象**：store handler 中 `.RefreshTags()` 被生成为 `list_notes().RefreshTags()`，即被链接到前一行的 API 返回值上。

**根因**：
- ts_adapter 在转译多语句 handler body 时，语句间的分隔不清晰
- 前一行 `notes.value = await list_notes()` 的 `list_notes()` 返回值被当作 `.RefreshTags()` 的接收者
- ts_adapter 可能缺少分号或换行来明确语句边界

**架构方案**：
- **ts_adapter 的语句转译必须保证每条语句以分号 `;` 结尾**，确保 JS 解析器不会把下一行的方法调用误解为链式调用
- 对于 store 内部方法调用（`.MethodName()`），识别为独立语句而非链式调用：
  - 检查方法名是否在 store 的 handler 列表中
  - 如果是，生成 `MethodName()` 而非 `prevExpr.MethodName()`

**实施**：
1. 在 `transpile_handler_body` 中，确保每条语句后加分号
2. 对 `.MethodName()` 模式，检查是否是 store handler，如果是则生成独立调用

---

### D4: `tag` 保留关键字冲突

**现象**：`tag` 是 `TokenKind::Tag`（Auto 的 enum/tag 声明关键字），在 handler 参数名中使用 `tag` 导致解析失败。

**根因**：parser 在所有上下文中都将 `tag` 识别为关键字 token，不作为标识符。

**架构方案**：
- **`tag` 应该是上下文关键字（contextual keyword）**，只在声明位置（`tag X { ... }`）是关键字，在其他位置（参数名、变量名、属性名）是普通标识符
- 这与 Auto 已经实现的 `view`/`on`/`model` 上下文关键字处理一致（这些在参数模式位置是关键字，在其他位置是标识符）

**实施**：
1. 在 parser 的 `parse_handler_params` 和 `parse_msg_variants` 中，允许 `TokenKind::Tag` 作为参数名
2. 或者更通用：在标识符期望位置（identifier position），所有 contextual keywords 都可以被接受为标识符

---

### D5: View DSL for 迭代器不支持 `ident.field`

**现象**：`for tag in note.tags` 解析失败，因为 `parse_view_for_loop` 只接受 `.field`（点前缀）、数字范围、或单个标识符，不支持 `ident.field` 链。

**根因**：
```rust
// parser.rs ~11216
fn parse_view_for_loop(&mut self) -> AutoResult<AuraNode> {
    // 迭代表达式解析：
    // 1. .field → Expr::Ident(".field")
    // 2. 数字范围 → Expr::Range(...)
    // 3. 单 ident → Expr::Ident("name")
    // 没有 ident.field → Expr::Dot(Expr::Ident("note"), "tags") 的处理
}
```

**架构方案**：
- **`parse_view_for_loop` 的迭代表达式解析应复用 `parse_expr` 的 dot-access 路径**
- 即：先解析一个表达式（可能是 `ident`、`ident.field`、`ident.field.subfield`），然后用作迭代源
- 这是 parser 层面的一行改动——把 "只接受一个 ident" 改为 "接受一个完整的 dot-access 表达式"

**实施**：
1. 在 `parse_view_for_loop` 中，替换 "只接受一个 ident token" 为 "调用 `parse_dot_access` 或类似函数解析完整表达式"
2. 确保生成的 AuraNode::ForLoop 的 `iterable` 字段可以是 `Expr::Dot` 而非只能是 `Expr::Ident`
3. 确保 Vue 生成器的 ForLoop 处理能正确转译 `Expr::Dot` 迭代表达式

---

### D6: a2r 不支持多类型 CRUD

**现象**：后端 API 中如果有 `list_folders() []Folder` 和 `list_notes() []Note` 两种类型，a2r 生成的 Rust 代码把所有 API 都当作对同一个 `Vec<Note>` 的操作。

**根因**：
- a2r 的 `generate_main_rs` / `generate_api_rs` 假设只有一个主类型（`primary_type`），所有 state 都存在同一个 `Db = Arc<Mutex<Vec<PrimaryType>>>` 中
- 没有机制处理多个类型需要独立存储

**架构方案**：
- **a2r 支持多类型 state**：根据 API 返回类型自动生成多个 state 容器
  ```rust
  pub type NoteDb = Arc<Mutex<Vec<Note>>>;
  pub type FolderDb = Arc<Mutex<Vec<Folder>>>;
  pub type AppState = (NoteDb, FolderDb);
  ```
- 每个 endpoint 根据其返回类型自动路由到正确的 state 容器
- `generate_main_rs` 为每种类型生成独立的 seed data

**实施**：
1. 在 `generate_api` 中，收集所有 API 涉及的类型（从 endpoint 返回类型推断）
2. 为每种类型生成独立的 Db 类型和 state 容器
3. 每个 endpoint handler 根据返回类型选择正确的 state

**复杂度**：高。需要改动 api_gen.rs 的核心生成逻辑。建议作为独立 PR。

---

### D9: autodown_editor 导致生成器卡住

**现象**：当 widget 的 view 树包含 `autodown_editor { ... }` 节点时，Vue 生成器卡住（可能 OOM 或无限循环）。

**根因**：可能与 D1 相关（for + style:if + handler 组合），但触发条件不同——`autodown_editor` 是自定义组件，走 shadcn 属性生成路径，可能在该路径中有未处理的递归。

**架构方案**：
- 需要先诊断（同 D1 的插桩方法）
- 可能与 `generate_shadcn_attrs` 对 `autodown_editor` 的属性处理有关
- 修复取决于诊断结果

**实施**：
1. 用最小复现 case（widget 有 msg/on + 一个 autodown_editor 节点）定位卡住位置
2. 根据诊断修复

---

## 3. 修复优先级与路线图

### Phase 1：Parser 层修复（低风险、高收益）

| 缺陷 | 改动位置 | 预计工作量 | 收益 |
|---|---|---|---|
| **D4** `tag` 上下文关键字 | parser.rs ~3 处 | 0.5 天 | 消除关键字冲突 |
| **D5** for 迭代器支持 ident.field | parser.rs 1 处 | 1 天 | view DSL 表达力大幅提升 |

### Phase 2：转译器层修复（中风险、高收益）

| 缺陷 | 改动位置 | 预计工作量 | 收益 |
|---|---|---|---|
| **D2** store 数组操作 | ts_adapter.rs | 1 天 | store 可靠操作数组 |
| **D3** store 方法间调用 | ts_adapter.rs | 1 天 | store handler 可组合 |

### Phase 3：生成器层修复（高风险、最高收益）

| 缺陷 | 改动位置 | 预计工作量 | 收益 |
|---|---|---|---|
| **D1** OOM | vue.rs 核心 | 3-5 天 | 恢复独立组件架构 |
| **D9** autodown_editor | vue.rs | 1-2 天（依赖 D1） | 恢复 AutoDown 编辑器 |

### Phase 4：代码生成层修复（高风险、独立）

| 缺陷 | 改动位置 | 预计工作量 | 收益 |
|---|---|---|---|
| **D6** a2r 多类型 CRUD | api_gen.rs | 3-5 天 | 支持多类型后端 |

### 时间线

```
Phase 1 (Parser)      ████ ████              1.5 天
Phase 2 (转译器)       ████ ████              2 天
Phase 3 (生成器)       ████████████████        4-7 天
Phase 4 (a2r)          ████████████           3-5 天
                       ──────────────────────
                       总计：10-15 天（可分多个 PR）
```

---

## 4. 修复后可移除的 workaround

| Workaround | 依赖修复 | 移除后 |
|---|---|---|
| 导航栏内联到 App.vue（300+ 行） | D1 | 恢复 NavTree 为独立 widget |
| store computed `all_tags` getter | D2 | 改用 store handler 动态收集 |
| 避免方法间调用 | D3 | store handler 可调用 RefreshTags 等 |
| 参数名用 `t` 替代 `tag` | D4 | 参数名可自由使用 `tag` |
| 避免嵌套 `for tag in note.tags` | D5 | view DSL 原生支持 |
| Folder 硬编码在前端 | D6 | 后端支持 Folder CRUD API |
| textarea 替代 autodown_editor | D1+D9 | 恢复 WYSIWYG 编辑器 |

---

## 5. 剩余可能的 workaround（需讨论）

### W1: View DSL 不支持 CSS `group-hover`

**状态**：已通过 Tailwind `group` class 透传解决，不需要 workaround。

### W2: Dark mode 根元素条件 class

**状态**：已通过生成器检测 `ToggleDarkMode` handler 注入 `:class` 解决。

### W3: Store composable 单例模式

**状态**：已通过 `let _instance` 单例解决。长期方案是使用 Pinia 或 Vue 的 `provide/inject`。

### W4: pac.at npm_deps 配置

**状态**：已实现（Plan 354 Phase C + npm_deps 字段）。

### 无剩余需要讨论的 workaround

所有当前 workaround 都有明确的修复计划（Phase 1-4）。
