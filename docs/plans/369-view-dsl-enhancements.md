# Plan 369: View DSL 增强三连 — else-if / view fn / store computed

> **目标**: 消除 .at 源码的三个核心表达力缺陷，让 sidebar.at 从 400 行降到 ~150 行。
>
> **来源**: Plan 367 P2 深入调研。三个特性都基于已有基础设施（AST 变体、解析函数、生成逻辑已部分存在），改动量比预期小得多。

---

## 1. 调研核心发现

三个特性的复杂度评估都远低于初步预估，因为大量基础设施已存在：

| 特性 | 初步预估 | 实际评估 | 为什么更简单 |
|------|----------|----------|-------------|
| else-if | "低" | **极低（~25行）** | 只改 parser 一处 peek + generator 一处检测 |
| view fn | "高（语言级新特性）" | **中等（~300行）** | `ViewNode::Component` 变体已定义但从未被解析器生成（死代码）；方案 A（内联展开）零生成器改动 |
| store computed | "高" | **中低（~80行）** | `parse_computed_block_inner` 已存在；widget computed 全链路已实现；闭包 `n => expr` 已支持 |

---

## 2. Phase 1: else-if 链式语法（~25 行改动）

### 2.1 问题

当前 `if A { } else { if B { } else { } }` 必须嵌套写。不支持 `else if B`。

### 2.2 方案

**解析器改动**（`parser.rs:11398` 之后，1 处）：

```rust
// 在 expect_ident("else") 之后、expect(LBrace) 之前插入：
if self.cur.text.as_str() == "if" {
    let nested = self.parse_view_conditional()?;
    Some(vec![nested])
} else {
    // 原有的 else { ... } 逻辑
    self.expect(TokenKind::LBrace)?;
    ...
}
```

**Vue 生成器改动**（`vue.rs:~2650`，1 处）：

当 `else_body` 恰好是 `vec![Conditional{..}]`（单元素嵌套 Conditional）时，生成 `v-else-if` 而非 `<template v-else><template v-if>`：

```rust
// 检测 else_body 是否是单 Conditional（else-if 链式）
if let Some(else_nodes) = else_body {
    if else_nodes.len() == 1 {
        if let AuraNode::Conditional { condition, then_body, else_body: inner_else, .. } = &else_nodes[0] {
            // 生成 v-else-if + 递归处理 inner_else（可能继续是 else-if）
            ...
        }
    }
    // 否则正常 v-else
}
```

### 2.3 改动文件

| 文件 | 改动 | 行数 |
|------|------|------|
| `parser.rs` | else 后加 if peek | ~6 |
| `vue.rs` | else_if 检测 + v-else-if 生成 | ~15 |
| 测试 | 3 个测试用例 | ~40 |

### 2.4 测试用例

```auto
// 简单 if / else if / else
if .status == "active" { text "Active" }
else if .status == "pending" { text "Pending" }
else { text "Other" }
```
→
```html
<template v-if="status == 'active'">Active</template>
<template v-else-if="status == 'pending'">Pending</template>
<template v-else>Other</template>
```

---

## 3. Phase 2: view fn — 视图片段 / 内联展开（~300 行改动）

### 3.1 问题

sidebar.at 里笔记列表项的渲染逻辑被复制 15+ 次（~400 行）。无法抽取可复用单元。

### 3.2 方案 A（内联展开）：推荐首选

**语法**：
```auto
// 声明（在 widget 外或 .at 文件顶层）
view fn NoteItem(note: Note, active: bool) {
    button {
        text note.title { style: "block truncate" }
        text note.time { style: "block text-xs text-muted-foreground mt-0.5" }
        onclick: .SelectNote(i)    // 使用父 widget 的 handler
        style: if active {
            "w-full text-left px-3 py-2 rounded-lg bg-accent text-accent-foreground"
        } else {
            "w-full text-left px-3 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors"
        }
    }
}

// 使用（在 view 块内）
for i, note in .store.notes {
    if note.folder == "" {
        NoteItem(note, i == .store.active_id)
    }
}
```

**展开逻辑**：在 aura extract 阶段，遇到 `ViewNode::Component { name: "NoteItem" }` 时，查片段表，把片段体**内联展开**——用调用参数替换片段参数引用。

**核心发现**：`ViewNode::Component` 和 `AuraNode::Component` 变体**已存在**（定义在 ast/ui.rs:301 和 aura/types.rs:549），所有生成器（Vue/Ark/Jet/Rust）都已处理它们。但**解析器从未生成过它们**——是死代码。

### 3.3 改动文件

| 文件 | 改动 | 行数 |
|------|------|------|
| `ast/ui.rs` | 加 `ViewFragmentDecl` 结构 + `Stmt::ViewFragmentDecl` 变体 | ~15 |
| `ast.rs` | Display 分支 | ~5 |
| `dialect/ui.rs` | `TokenKind::View` + `fn` 分支 → 解析片段声明 | ~5 |
| `parser.rs` | `parse_view_fragment_decl()` + `parse_view_node` 里 PascalCase 调用检测 | ~80 |
| `aura/extract.rs` | 片段收集 + 内联展开（参数替换遍历） | ~150 |
| `aura/types.rs` | `AuraModule` 加 `fragments` 字段 | ~15 |
| 生成器（vue/ark/jet/rust） | **零改动** | 0 |

### 3.4 内联展开 vs 子组件

| 维度 | 方案 A（内联展开） | 方案 B（Vue 子组件） |
|------|-------------------|---------------------|
| `.store.xxx` | ✅ 直接用父组件的 store | ❌ 需要 Provide/Inject |
| `onclick: .Handler` | ✅ 直接用父组件的 handler | ❌ 需要 callback prop 转发 |
| 新文件 | 不需要 | 每个片段一个 .vue |
| 生成器改动 | 零 | 需要（props/事件传递） |
| 适用场景 | 同一 widget 内复用 | 跨 widget 复用 |

**决策**：先做方案 A。015-notes 的 sidebar.at 全在 NavTree 一个 widget 内，方案 A 完全够用。

### 3.5 展开算法

```
extract_view_node(Component { name: "NoteItem", props: [(note, expr1), (active, expr2)] }, fragments)
  → 查 fragments["NoteItem"]
  → 获取片段体 body: ViewNode（一个 button 元素）
  → 参数绑定: { "note" → expr1, "active" → expr2 }
  → 遍历 body 树，把 `note` 引用替换为 expr1，`active` 引用替换为 expr2
  → 返回展开后的 AuraNode（button 元素，内联了具体参数值）
```

### 3.6 sidebar.at 改造前后

**改前（~400 行）**：15 个几乎相同的 for-if-if-button 块

**改后（~80 行）**：
```auto
view fn NoteItem(note: Note, active: bool, indent: str) {
    button {
        text note.title { style: "block truncate" }
        text note.time { style: "block text-xs text-muted-foreground mt-0.5" }
        onclick: .SelectNote(i)
        style: if active {
            "w-full text-left {indent} py-2 rounded-lg bg-accent text-accent-foreground"
        } else {
            "w-full text-left {indent} py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors"
        }
    }
}

view {
    ...
    for i, note in .store.notes {
        if .active_folder == "all" {
            if note.folder == "" {
                if .store.active_tag == "" || note.tags.contains(.store.active_tag) {
                    NoteItem(note, i == .store.active_id, "px-3")
                }
            }
        }
        ...
    }
}
```

---

## 4. Phase 3: store computed（~80 行改动）

### 4.1 问题

sidebar.at 的过滤逻辑散落在模板的嵌套 v-if 里。无法用 computed 抽取。

### 4.2 方案

在 store 里加 computed 支持。语法复用 widget 已有的 `=> ` 表达式语法：

```auto
store NotesStore {
    model {
        var notes []Note = []
        var active_folder str = "all"
        var active_tag str = ""
        ...
    }

    computed {
        filtered_notes => .notes.filter(n =>
            .active_folder == "all" || n.folder == .active_folder
        )
        pinned_notes => .notes.filter(n => n.pinned)
        all_tags_set => .notes.flat_map(n => n.tags).to_set()
    }

    on { ... }
}
```

### 4.3 改动文件

| 文件 | 改动 | 行数 |
|------|------|------|
| `ast/ui.rs` | `StoreDecl` 加 `computed: Option<ComputedBlock>` | ~2 |
| `aura/types.rs` | `AuraStore` 加 `computed: Vec<AuraComputed>` | ~2 |
| `parser.rs` | store 解析器 match 加 `"computed"` 分支（复用 `parse_computed_block_inner`） | ~3 |
| `aura/extract.rs` | `extract_store_from_decl` 读 computed | ~5 |
| `vue.rs` | `generate_store_composable` 加 computed getter 循环 | ~20 |

### 4.4 store composable 里的生成

用 ts_adapter 转译表达式（和 handler body 一样），生成 JS getter：

```typescript
// 生成的 store composable
export function useNotesStore(): any {
    return {
        notes,           // ref
        active_folder,   // ref
        ...
        // 用户声明的 computed（Plan 369 Phase 3）
        get filtered_notes() {
            return notes.value.filter(n =>
                active_folder.value === "all" || n.folder === active_folder.value
            )
        },
        // 自动生成的 getter（已有）
        get all_tags() { ... },
    }
}
```

### 4.5 sidebar.at 配合改造

```auto
// store 里加 computed
computed {
    visible_notes => .notes.filter(n =>
        .active_folder == "all" || n.folder == .active_folder
    )
}

// sidebar.at 里用 computed
for i, note in .store.visible_notes {
    NoteItem(note, i == .store.active_id, "px-3")
}
```

把嵌套的 folder/tag 过滤从模板移到 computed，sidebar.at 更简洁。

---

## 5. 实施路线

### 阶段 1: else-if（半天）
- [ ] parser.rs 加 else if peek
- [ ] vue.rs 加 v-else-if 生成
- [ ] 3 个测试用例
- [ ] 015-notes 里用 else-if 改写 Pinned/Recent 分支

### 阶段 2: store computed（1 天）
- [ ] 5 处加字段/分支
- [ ] store composable getter 生成
- [ ] 015-notes 的 notes_store.at 加 filtered_notes/pinned_notes computed
- [ ] sidebar.at 用 computed 替代模板内嵌 if

### 阶段 3: view fn 内联展开（2-3 天）
- [ ] AST 加 ViewFragmentDecl
- [ ] parser 加声明解析 + 调用检测
- [ ] aura extract 加片段收集 + 参数替换展开
- [ ] sidebar.at 用 view fn 消除 15x 复制粘贴
- [ ] 测试套件验证

### 阶段 4: 验证
- [ ] vue-tsc 严格模式通过
- [ ] 16/17 测试通过（无回归）
- [ ] sidebar.at 从 ~400 行降到 ~150 行
- [ ] 生成的 Vue 代码可读

---

## 6. 验收标准

### 可量化

| 指标 | 当前 | 目标 |
|------|------|------|
| sidebar.at 行数 | ~400 行 | ~150 行 |
| 嵌套 v-if 深度 | 3-4 层 | 1-2 层（用 computed 替代） |
| 复制粘贴的列表项代码 | 15+ 份 | 1 份（view fn） |
| else-if 语法 | 不支持 | 支持 |
| store computed | 不支持 | 支持 |

### 质量标准

- [ ] vue-tsc 严格模式零错误
- [ ] 生成的 Vue 代码接近手写水平
- [ ] 生成的 Rust 后端无回归
- [ ] 015-notes 全部功能保持正常（16/17 测试通过）

---

## 7. 与其他 Plan 的关系

- **Plan 367（代码质量）**: 本计划是 P2 的实现，消除了 P2-1/P2-2/P2-3 三个 DSL 缺陷
- **Plan 361（校验）**: view fn 和 computed 新增后，校验框架需要新增规则（如"片段调用参数匹配"）
- **Plan 363（Skill）**: 模式库里可以加 "view fn pattern" 和 "computed filter pattern"
