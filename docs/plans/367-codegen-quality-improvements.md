# Plan 367: 生成代码质量改进 — 让 Auto 产物达到手写水平

> **目标**: 把"Auto 生成的 Vue/TS/Rust 代码看起来像转译器输出的"变成"看起来像专家手写的"。
>
> **来源**: 基于 015-notes 完整代码审查（.at 源码 + Vue/TS/Rust 产物 vs 行业惯例）。
>
> **原则**: 只改转译器和 DSL，不改 .at 源码的语义——同样的 .at 输入，生成更高质量的产物。

---

## 1. 问题全景

当前生成的代码和手写代码的差距集中在 7 个点：

```
质量从高到低：
  api.ts    ████████████████████░  9/10  （几乎完美）
  Rust      ██████████████████░░░  8/10  （search 参数 + 时间硬编码）
  store     ████████████████░░░░░  7/10  （any 泛滥）
  .vue 组件 ██████████░░░░░░░░░░░  5/10  （主要问题区）
  .at 源码  ████████░░░░░░░░░░░░░  4/10  （sidebar 400 行复制粘贴）
```

每个问题归为三类：**转译器能力不足**、**DSL 缺失特性**、**.at 源码风格**。

---

## 2. P0 — 低成本高收益（改转译器，1-2 天）

这组改进不改 .at 语法、不改语义，纯粹让生成器输出更干净的代码。

### P0-1: 去掉无消费者的 emit

**问题**: 几乎每个 handler 末尾都有 `emit('X')`，但大部分父组件根本不监听。

```typescript
// 当前（噪音）
function Cancel(): void {
  editing.value = false;
emit('Cancel')     // ← App.vue 不监听 Cancel，这行是死代码
}

// 期望（干净）
function Cancel(): void {
  editing.value = false;
}
```

**方案**: 在 `generate_script` 里，只对 `emit_events` 中**父组件实际监听的事件**生成 `emit()` 调用。

检测方法：App.vue 模板里 `@_delete="DeleteActive"` 表示监听 `delete`。生成器已知哪些事件被父组件绑定（在 `node_to_html` 的事件处理循环里收集），把"被监听的事件"存到一个集合，script 生成时只对这些事件 emit。

**改动文件**: `crates/auto-lang/src/ui_gen/vue.rs` — `generate_script` + 事件收集逻辑。

### P0-2: 修正 ts_adapter 缩进和换行

**问题**: 函数体里语句缩进不一致，if 体没换行。

```typescript
// 当前
function DeleteActive(): Promise<void> {
  await delete_note(...);
 store.notes = await list_notes();   // ← 多了一格
 if (store.notes.length > 0) {store.active_id = 0;
}                                     // ← if 体没有换行
```

**方案**: ts_adapter 的 `transpile_handler_body` 在拼接语句时，统一用 `\n  ` 换行+缩进。`if { ... }` 的内部语句也要换行。

**改动文件**: `crates/auto-lang/src/ui_gen/ts_adapter.rs` — `transpile_handler_body`。

### P0-3: 去掉重复的 onMounted

**问题**: EditorPanel.vue 有两个 onMounted，做几乎一样的事。

```typescript
onMounted(() => {                          // ← .Init handler 生成
  if (props.note.title == '') { ... }
})
onMounted(() => {                          // ← 生成器硬编码的兜底
  if (!props.note?.title) { ... }
})
```

**方案**: 删除 `generate_script` 里硬编码的"空标题检测"onMounted（line ~1554）。这个逻辑应该由 .at 的 `.Init` handler 负责，不是生成器替用户写。

**改动文件**: `crates/auto-lang/src/ui_gen/vue.rs` — 删除 line 1554 附近的硬编码 onMounted。

### P0-4: emit 声明带 payload 类型

**问题**: `EditBody(str)` 的参数信息丢失，emit 声明全是空数组。

```typescript
// 当前
defineEmits<{ EditBody: []; RemoveTag: [] }>()

// 期望
defineEmits<{ EditBody: [string]; RemoveTag: [string] }>()
```

**方案**: 在生成 emit 声明时，从 `widget.handler_params` 或 msg 声明里提取参数类型，映射为 TS 类型（`str` → `string`、`int` → `number`）。

**改动文件**: `crates/auto-lang/src/ui_gen/vue.rs` — emit 声明生成逻辑。

---

## 3. P1 — 中成本（改 DSL + 转译器，3-5 天）

### P1-1: Auto 类型 → TS 类型映射

**问题**: 所有 props 都是 `any`。

```typescript
// 当前
defineProps<{ note: any; search: any; active_id: any }>()

// 期望
defineProps<{ note: Note; search: string; active_id: number }>()
```

**方案**:

1. 在 AuraProp 里保留类型信息（目前 `note: str` 的 `str` 被 parse 了但没传到 AuraProp）。
2. 建立类型映射表：

| Auto 类型 | TS 类型 |
|-----------|---------|
| `str` | `string` |
| `int` / `float` | `number` |
| `bool` | `boolean` |
| `[]Note` / `List<Note>` | `Note[]` |
| 自定义 type（如 `Note`） | 引用同名 interface |
| `msg` | `() => void` |
| `?Note` | `Note \| null` |

3. `generate_script` 生成 `defineProps<{...}>` 时用映射后的类型。

**改动文件**: `crates/auto-lang/src/aura/extract.rs`（保留 prop 类型）、`crates/auto-lang/src/ui_gen/vue.rs`（使用映射）。

### P1-2: v-if / v-else 支持

**问题**: 两个独立的 `v-if` 代替 `v-if/v-else`，模板冗余且 Vue 会两次求值条件。

```auto
// 当前 .at 只能这样写
if .editing == false { ... }
if .editing == true { ... }
```

```vue
<!-- 生成的（冗余）-->
<template v-if="editing == false"> ... </template>
<template v-if="editing == true"> ... </template>

<!-- 期望 -->
<template v-if="!editing"> ... </template>
<template v-else> ... </template>
```

**方案**:

在 view 解析器里，检测**连续的两个 if 块**，如果第二个的条件是第一个的逻辑取反（`== false` vs `== true`，或 `== "x"` vs `!= "x"`），自动标记为 else 分支。生成器产出 `v-else`。

**改动文件**: `crates/auto-lang/src/aura/extract.rs`（view 解析）、`crates/auto-lang/src/ui_gen/vue.rs`（模板生成）。

### P1-3: a2r query 参数提取

**问题**: `fn search_notes(query str)` 的 query 参数在生成的 Rust handler 里丢失。

```rust
// 当前（忽略 query，返回全部）
pub async fn search_notes(State(db): State<Db>) -> JsonResponse<Vec<Note>> {
    let items = db.lock().unwrap();
    JsonResponse(items.clone())
}

// 期望
pub async fn search_notes(
    State(db): State<Db>,
    axum::extract::Query(params): axum::extract::Query<SearchParams>,
) -> JsonResponse<Vec<Note>> {
    let query = params.query.to_lowercase();
    let items = db.lock().unwrap();
    let filtered: Vec<Note> items.iter()
        .filter(|n| n.title.to_lowercase().contains(&query) || n.body.to_lowercase().contains(&query))
        .cloned().collect();
    JsonResponse(filtered)
}
```

**方案**: a2r 检测 `#[api(method = "GET")]` 且函数有非路径参数时，为每个非路径参数生成 `Query<struct>` 提取器 + 过滤逻辑。

**改动文件**: `crates/auto-man/src/api_gen.rs`（a2r 后端生成）。

### P1-4: callback prop 正确映射

**问题**: `on_delete: msg` 生成为 `on_delete: any`，且通过事件转发而非直接传递。

```typescript
// 当前（间接：prop → handler → emit）
const props = defineProps<{ on_delete: any }>()
function Delete() { props.on_delete(); emit('Delete') }

// 期望（直接 prop callback）
const props = defineProps<{ on_delete: () => void }>()
function Delete() { props.on_delete() }
```

**方案**: 对 `msg` 类型的 prop，生成 `() => void` 类型；不生成 emit（已在 P0-1 处理）；直接调用 `props.on_xxx()`。

**改动文件**: `crates/auto-lang/src/ui_gen/vue.rs`。

---

## 4. P2 — DSL 表达力增强（深入调研后修订）

> 以下方案基于 Plan 369 的代码级深入调研。三个特性的复杂度都远低于初步预估。

### P2-1: else-if 链式语法 ✅ 已完成（P1-2 的延续）

**状态**: `else` 语法 view DSL 早已支持（`parser.rs:11397`）。`if/else` 在 P1-2 已用于 015-notes。
`else if` 链式只需在解析器 `else` 后加一个 peek（~6行）。

**改动**:
| 文件 | 改动 | 行数 |
|------|------|------|
| `parser.rs:~11399` | else 后 peek `if` → 递归 `parse_view_conditional` | ~6 |
| `vue.rs:~2650` | else_body 为单 Conditional 时生成 `v-else-if` | ~15 |

### P2-2: store computed（~80 行改动）

**核心发现**: `parse_computed_block_inner` 已存在可复用。widget computed 全链路已实现（但零测试）。闭包 `n => expr` 已支持。

**语法**（复用 widget 的 `=> ` 表达式语法）：
```auto
store NotesStore {
    model { var notes []Note = [] ... }
    computed {
        filtered_notes => .notes.filter(n =>
            .active_folder == "all" || n.folder == .active_folder
        )
    }
    on { ... }
}
```

**改动**:
| 文件 | 改动 | 行数 |
|------|------|------|
| `ast/ui.rs` | `StoreDecl` 加 `computed: Option<ComputedBlock>` | ~2 |
| `aura/types.rs` | `AuraStore` 加 `computed: Vec<AuraComputed>` | ~2 |
| `parser.rs:~10283` | store 解析 match 加 `"computed"` 分支（复用 `parse_computed_block_inner`） | ~3 |
| `aura/extract.rs` | `extract_store_from_decl` 读 computed | ~5 |
| `vue.rs` | `generate_store_composable` 加 computed getter 循环（用 ts_adapter 转译） | ~20 |

**store composable 生成**：
```typescript
get filtered_notes() {
    return notes.value.filter(n =>
        active_folder.value === "all" || n.folder === active_folder.value
    )
}
```

### P2-3: view fn — 视图片段 / 内联展开（~300 行改动）

**核心发现**: `ViewNode::Component` 和 `AuraNode::Component` 变体**早已存在但从未被解析器使用**——是死代码。所有四个生成器（Vue/Ark/Jet/Rust）都已处理 Component 节点。方案 A（内联展开）零生成器改动。

**语法**：
```auto
view fn NoteItem(note: Note, active: bool) {
    button {
        text note.title { style: "block truncate" }
        text note.time { style: "block text-xs text-muted-foreground mt-0.5" }
        onclick: .SelectNote(i)    // 使用父 widget 的 handler
        style: if active { "active-class" } else { "inactive-class" }
    }
}

// 使用
for i, note in .store.visible_notes {
    NoteItem(note, i == .store.active_id)
}
```

**展开逻辑**：在 aura extract 阶段，遇到 `ViewNode::Component { name: "NoteItem" }` 时查片段表，用调用参数替换片段参数引用，内联展开为普通元素节点。

**改动**:
| 文件 | 改动 | 行数 |
|------|------|------|
| `ast/ui.rs` | 加 `ViewFragmentDecl` 结构 + `Stmt::ViewFragmentDecl` 变体 | ~15 |
| `ast.rs` | Display 分支 | ~5 |
| `dialect/ui.rs` | `TokenKind::View` + `fn` 分支 | ~5 |
| `parser.rs` | `parse_view_fragment_decl()` + PascalCase 调用检测 | ~80 |
| `aura/extract.rs` | 片段收集 + 参数替换展开 | ~150 |
| `aura/types.rs` | `AuraModule` 加 `fragments` 字段 | ~15 |
| 生成器（vue/ark/jet/rust） | **零改动** | 0 |

**内联展开 vs 子组件**：方案 A（内联展开）对 015-notes 最合适——`.store.xxx` 和 `onclick: .Handler` 直接用父组件的，不需要 props/callback 转发。

---

## 5. 实施路线

### 阶段 1: 清扫低垂果实（已完成 ✅）

P0 全部：
- [x] P0-1 去掉无消费者 emit
- [x] P0-2 修正 ts_adapter 缩进
- [x] P0-3 去掉重复 onMounted
- [x] P0-4 emit 带 payload 类型

### 阶段 2: 类型安全 + DSL 表达力（已完成 ✅）

- [x] P1-1 Auto → TS 类型映射（props 不再是 any）
- [x] P1-2 if/else 语法（view DSL 早已支持，改 .at 源码即可）
- [x] P1-3 a2r query 参数（search API 正确过滤）
- [x] P1-4 callback prop 类型 + emit 调用传参

**验证**: vue-tsc 严格模式零错误，16/17 测试通过。

### 阶段 3: else-if 链式（半天）

- [ ] parser.rs 加 else if peek（~6 行）
- [ ] vue.rs 加 v-else-if 生成（~15 行）
- [ ] 015-notes 用 else-if 改写 Pinned/Recent 分支

### 阶段 4: store computed（1 天）

- [ ] 5 处加字段/分支（AST + parser + aura + extract + composable）
- [ ] notes_store.at 加 filtered_notes/pinned_notes computed
- [ ] sidebar.at 用 computed 替代模板内嵌 if

### 阶段 5: view fn 内联展开（2-3 天）

- [ ] AST 加 ViewFragmentDecl + Stmt 变体
- [ ] parser 加声明解析 + PascalCase 调用检测
- [ ] aura extract 加片段收集 + 参数替换展开
- [ ] sidebar.at 用 view fn 消除 15x 复制粘贴

### 阶段 6: 验证

- [ ] vue-tsc 严格模式通过
- [ ] 16/17 测试通过（无回归）
- [ ] sidebar.at 从 ~400 行降到 ~150 行

### 阶段 4: 语言级改进（1-2 周，长期）

- [ ] P2-3 else/else-if（最简单的语言改动）
- [ ] P2-1 子组件/视图片段（消除 sidebar.at 400 行复制）
- [ ] P2-2 computed 属性

**验证**: sidebar.at 从 ~400 行降到 ~150 行，功能不变。
**预期收益**: .at 源码可维护性大幅提升。

---

## 6. 验收标准

### 可量化指标

| 指标 | 当前 | 目标 | 来源 |
|------|------|------|------|
| EditorPanel.vue 代码行数 | ~190 行 | ~150 行 | P0 减少 emit/onMounted |
| `any` 类型 prop 数量 | 全部 | 0（基本类型映射） | P1-1 |
| 无消费者 emit 数量 | ~8 个 | 0 | P0-1 |
| sidebar.at 代码行数 | ~400 行 | ~150 行 | P2-1 |
| search 功能 | 不工作 | 工作 | P1-3 |

### 质量对比

改完后，把生成的 EditorPanel.vue 拿给 Vue 专家看，他不应该能分辨出"这是转译器生成的"。

---

## 7. 与其他 Plan 的关系

- **Plan 361（校验）**: P0-1（去 emit）需要在校验框架里增加"未监听的 emit"警告
- **Plan 362（快照测试）**: 每个改动都用 insta snapshot 验证生成产物变化
- **Plan 363（Skill）**: 改进后的代码 pattern 补充到 Skill 的 generator-contracts
- **Plan 366（测试）**: 每个改动后跑 `pnpm test` 确认无回归
