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

## 4. P2 — 高成本（改语言，1-2 周）

### P2-1: 子组件 / 可复用视图片段

**问题**: sidebar.at 的笔记列表项逻辑被复制 15+ 次（~400 行）。

**方案**: 引入 `fn` 视图片段语法：

```auto
// 新语法：视图片段（view fragment）
view fn NoteListItem(note: Note, i: int, active: bool, onclick: msg) {
    button {
        text note.title { style: "block truncate" }
        text note.time { style: "block text-xs text-muted-foreground mt-0.5" }
        onclick: onclick
        style: if active {
            "w-full text-left px-3 py-2 rounded-lg bg-accent text-accent-foreground"
        } else {
            "w-full text-left px-3 py-2 rounded-lg text-foreground hover:bg-accent/50 transition-colors"
        }
    }
}

// 使用
for i, note in .store.notes {
    if note.folder == "" {
        if .store.active_tag == "" {
            NoteListItem(note, i, i == .store.active_id, .SelectNote(i))
        }
        if .store.active_tag != "" {
            if note.tags.contains(.store.active_tag) {
                NoteListItem(note, i, i == .store.active_id, .SelectNote(i))
            }
        }
    }
}
```

**生成策略**: 视图片段翻译为 Vue 的**子组件**或**`<script setup>` 中的 render function**。如果片段简单（单根元素），可以直接内联展开（消除复制粘贴但保持性能）。

**改动范围**: 解析器（新语法）、aura 提取器、Vue 生成器。

### P2-2: computed 属性

**问题**: 列表过滤逻辑散落在模板的 v-if 嵌套里，而不是用 computed。

```auto
// 新语法
model {
    computed filtered_notes []Note = {
        if .active_folder == "all" {
            return .notes
        }
        if .active_folder == "pinned" {
            return .notes.filter(fn(n Note) bool { return n.pinned })
        }
        ...
    }
}

// view 里直接用
for i, note in .filtered_notes {
    NoteListItem(note, i, ...)
}
```

**改动范围**: 解析器、aura 提取器、Vue 生成器（computed → `computed(() => ...)`）。

### P2-3: else / else-if 支持

**问题**: 不支持 else，导致互斥条件必须写两个独立 if。

```auto
// 当前
if .editing == false { ... }
if .editing == true { ... }

// 期望
if .editing == false { ... }
else { ... }

// 或
if .active_folder == "all" { ... }
else if .active_folder == "pinned" { ... }
else { ... }
```

**改动范围**: 解析器（语法）、aura view 树（else 节点）、Vue 生成器（v-else / v-else-if）。

---

## 5. 实施路线

### 阶段 1: 清扫低垂果实（1-2 天）

P0 全部：
- [ ] P0-1 去掉无消费者 emit
- [ ] P0-2 修正 ts_adapter 缩进
- [ ] P0-3 去掉重复 onMounted
- [ ] P0-4 emit 带 payload 类型

**验证**: 快照测试（Plan 362 的 insta）对比改动前后的 SFC，确认噪音减少。
**预期收益**: .vue 文件可读性显著提升，代码量减少 ~20%。

### 阶段 2: 类型安全（2-3 天）

- [ ] P1-1 Auto → TS 类型映射
- [ ] P1-4 callback prop 映射

**验证**: `vue-tsc --noEmit` 通过（目前因为 `any` 不报错，改成具体类型后可能暴露新的 TS 错误，需要修复）。
**预期收益**: props/emits 有类型保护，IDE 智能提示可用。

### 阶段 3: DSL 表达力（3-5 天）

- [ ] P1-2 v-if/v-else
- [ ] P1-3 a2r query 参数

**验证**: 015-notes 改写为用 v-else + search 功能可用。
**预期收益**: 模板更简洁，search 功能真正可用。

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
