# Plan 272: Auto-UI DevTools Panel

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** 为 Auto-UI 桌面端实现 Chromium DevTools 风格的调试面板，支持元素检查、组件树、控制台、源码查看与热编辑。

**Architecture:** iced 窗口右侧 300px 面板，F12 开关，三个 tab（元素/检查/控制台）。通过 `DebugRenderCtx` 在渲染 DFS 中收集组件树和样式元数据。

**Tech Stack:** Rust, iced 0.14, AutoLang interpreter

---

## 状态: Phase A-G 已完成

## 已完成

| Phase | 功能 | 状态 |
|-------|------|------|
| A | 面板框架 + click-to-select + 属性检查器 | ✅ 已完成 |
| B | Console Tab（print() 输出拦截） | ✅ 已完成 |
| C | 源码查看器（带行号 + 热重载刷新） | ✅ 已完成 |
| D | 组件树可视化 Elements Tab | ✅ 已完成 |
| - | F12 直接打开面板 + 白色主题 + 合并 Inspector tab | ✅ 已完成 |
| E | Inspector 分割线 + 语法高亮 + 源码定位 + 表单编辑 + 写回热重载 | ✅ 已完成 |
| F | 全文件语法高亮缓存 + 自动滚动居中 + 直接源码文本编辑 | ✅ 已完成 |
| G | 动态面板宽度 + 源码高亮修复 + 可拖拽分隔栏 | ✅ 已完成 |

## 当前面板布局

```
┌──────────────────────────────────────────┬─────────────────────┐
│                                          │ [元素] [检查] [控制台]│
│                                          │─────────────────────│
│          应用主窗口                       │                     │
│          (白色主题)                       │  (当前 tab 内容)     │
│                                          │                     │
│                                          │                     │
└──────────────────────────────────────────┴─────────────────────┘
```

- Tab: 元素（组件树）/ 检查（属性+源码合并）/ 控制台
- 白色主题，与主 UI 一致
- F12 直接打开面板

---

## Phase E: 源码定位 + 可视化编辑（规划）

### 需求列表

| # | 需求 | 说明 |
|---|------|------|
| E1 | Inspector 分割线 | 属性和源码之间有明显分割线 + "源码" 标题 |
| E2 | 源码语法高亮 | 关键字、字符串、注释、类型等彩色显示 |
| E3 | 点击元素定位源码 | 点击 UI 组件时，源码自动滚动到对应 view 代码行，框选高亮 |
| E4 | 源码组件可编辑 | 框选的源码组件可切换为表单编辑模式 |
| E5 | 编辑写回 + 热重载 | 修改后写回 .at 文件，触发组件重新加载 |

### 难度分析

#### E1: 分割线 — ⭐ 简单（30 分钟）

在 `render_inspector_tab` 中属性和源码之间加一个分隔容器，深色细线 + "源码" 标题文字。纯 UI 调整。

**文件**: `crates/auto-lang/src/ui/iced/renderer.rs` 的 `render_inspector_tab`

#### E2: 源码语法高亮 — ⭐⭐ 中等（半天）

iced 没有内置的代码高亮。方案选择：

| 方案 | 优点 | 缺点 |
|------|------|------|
| **A: 手动 token 着色** | 无依赖，可控 | 需要实现简易 tokenizer |
| **B: tree-sitter** | 精确高亮 | 引入大依赖，编译慢 |
| **C: syntect** | 成熟，TextMate 语法 | 依赖大，嵌入语法文件 |

推荐方案 A：实现一个简易的 AutoLang token 着色器，只处理关键字、字符串、注释、数字、类型名这几种情况。逐行处理，输出带颜色的 `text()` 组件。

**文件**: 新增 `crates/auto-lang/src/ui/iced/source_highlight.rs`

**关键技术点**：
- 用正则或简易状态机逐行扫描
- 关键字（fn, let, var, if, for, col, row, text, ...）→ 蓝色
- 字符串 `"..."` → 绿色
- 注释 `//...` → 灰色
- 数字 → 橙色
- 组件标签（col, row, text, button, ...）→ 紫色

#### E3: 点击元素定位源码 — ⭐⭐⭐⭐ 困难（3-5 天）— 核心阻塞项

**这是整个 Phase E 最难的部分。**

当前 pipeline 完全没有源码位置信息：

```
Parser (有 Pos 但不保留) → ViewNode (无 span) → AuraNode (无 span) → View (无 span)
```

要实现点击 UI 元素定位到源码，需要贯穿整个 pipeline 传播 span：

**改动链**：

| 层 | 文件 | 改动 |
|----|------|------|
| **Parser** | `parser.rs` 的 `parse_view_node` | 在创建 `ViewNode` 时记录 `start_pos` 和 `end_pos` |
| **AST** | `ast/ui.rs` 的 `ViewNode` | 每个 variant 添加 `span: Option<(usize, usize)>` |
| **AURA** | `aura/extract.rs` | `extract_view_node` 传播 span 到 `AuraNode` |
| **AURA 类型** | `aura/types.rs` | `AuraNode` 每个 variant 添加 `span` 字段 |
| **View 构建** | `ui/aura_view_builder.rs` | `convert_node` 传播 span 到 View 或 DebugRenderCtx |
| **Renderer** | `ui/iced/renderer.rs` | `wrap_debug` 从 View/AuraNode 获取 span，存入 `DebugElementInfo` |

**难点**：
1. Parser 中 `parse_view_node` 是递归的，需要在进入和退出时记录位置
2. `ViewNode` 有 7 个 variant，全部需要加 span 字段
3. `AuraNode` 有 7 个 variant，同样全部需要加
4. 所有 match 和构造的地方都需要更新
5. byte offset → 行号的转换需要额外的 source map

**替代方案（降低难度）**：
- **方案 A（精确）**：完整 span 传播链 — 改动量大（~500 行），但结果精确
- **方案 B（启发式）**：用 tag 名 + props 模式在源码中搜索匹配 — 不精确但实现快（~100 行）
- **方案 C（混合）**：只在 `ViewNode` 和 `AuraNode` 层加 span，View 层用 side-channel 映射

#### E4: 源码组件可编辑 — ⭐⭐⭐ 困难（2-3 天）— 依赖 E3

需要 E3 的 span 信息来确定可编辑区域。UI 部分：
- 源码中被框选的组件显示为可点击区域
- 点击后切换为表单：每个 prop 变成一行 input（key: value）
- 需要处理 AutoLang 的属性语法（`key: value`, `key: {expr}`, `class: "..."` 等）

**难点**：
1. 属性值的解析和序列化（表达式 vs 字面量 vs f-string）
2. 编辑后需要重新生成有效的 AutoLang 源码片段
3. 嵌套组件的边界识别

#### E5: 编辑写回 + 热重载 — ⭐⭐ 中等（1-2 天）— 依赖 E3 + E4

**已有基础**：热重载机制完整（500ms 轮询 + mtime 检测 + 全量重解析 + 状态迁移）。

**需要的额外工作**：
1. 将编辑后的源码片段替换回原文件对应位置
2. 触发热重载（修改 mtime 或直接调用 reload）
3. 确保编辑后的组件 ID 稳定（当前 counter 每帧重置，已 OK）

**难点**：
1. 精确替换文件中的字节范围（需要 E3 的 span）
2. 多次编辑的增量更新 vs 全量替换
3. 替换后行号偏移的修正

### 实施优先级

```
E1 (分割线)     → E2 (语法高亮) → E3 (源码定位)  → E4 (可编辑) → E5 (写回+重载)
   ⭐ 简单           ⭐⭐ 中等        ⭐⭐⭐⭐ 最难       ⭐⭐⭐ 困难    ⭐⭐ 中等
   30 min            半天             3-5 天          2-3 天       1-2 天
```

**建议**：E1 和 E2 可以立即实施。E3 是核心阻塞项，决定后续 E4/E5 是否可行。如果 E3 采用方案 B（启发式），整个 Phase E 可以在 3-4 天内完成；方案 A（精确 span）则需要 7-10 天。

### 技术决策点

| 决策 | 选项 | 推荐 |
|------|------|------|
| 语法高亮方案 | A(手动) / B(tree-sitter) / C(syntect) | A — 简易够用 |
| 源码定位方案 | A(精确span) / B(启发式) / C(混合) | A — 长期正确 |
| 编辑模式 | 纯文本编辑 / 表单编辑 / 混合 | 表单编辑 — 更安全 |
| 热重载触发 | mtime轮询 / notify监听 / 手动触发 | mtime轮询 — 已有 |

## Phase E 实施方案（E3 修复 + E4 + E5）

### E3 修复：Span 侧通道传播

**问题**：span 传播链在 `AuraViewBuilder → AbstractView` 处断裂。ViewNode/AuraNode 都有 span 字段，但 `convert_node` 用 `..` 丢弃了 span，导致 `wrap_debug` 中 `span: None`。

**方案**：不修改 `AbstractView`（跨框架类型），利用已有的 `DynamicComponent.find_element_span(kind, index)` — 通过 DFS 按 tag 名 + 出现索引查找 span。在 `wrap_debug` 中用 `counter` 作为出现索引直接调用。

#### Step 1: `renderer.rs` — `wrap_debug` 中用 `find_element_span` 查 span

```rust
// wrap_debug 中，kind + counter 即可定位
let span = state.component.find_element_span(kind, counter_val);
```

需要将 `DynamicComponent` 引用传入 `DebugRenderCtx`。

#### Step 2: `renderer.rs` — `DebugRenderCtx` 添加 `component` 引用

```rust
struct DebugRenderCtx {
    // ...existing fields...
    component: &'a DynamicComponent,  // 用于 find_element_span
}
```

#### Step 3: `renderer.rs` — `wrap_debug` 使用 component 查找 span

```rust
let counter_val = *self.counter.borrow();
let span = self.component.find_element_span(kind, counter_val);
self.element_styles.borrow_mut().insert(id.clone(), DebugElementInfo {
    kind: kind.to_string(),
    props,
    span,
});
```

**验证**：`cargo build` + F12 点击元素 → Inspector 源码高亮定位到对应行。

### E4：源码可编辑

在 Inspector tab 的源码高亮区域添加"编辑"按钮，点击后切换为表单编辑模式。

#### Step 4: `renderer.rs` — DynamicState 添加编辑状态字段

```rust
editing_element: RefCell<Option<String>>,
edit_values: RefCell<HashMap<String, String>>,
edit_span: RefCell<Option<(usize, usize)>>,
edit_error: RefCell<Option<String>>,
```

#### Step 5: `renderer.rs` — 添加编辑消息常量和处理

- `__edit_{id}` — 进入编辑模式
- `__edit_apply` — 应用编辑
- `__edit_cancel` — 取消编辑
- `__edit_field_{key}` — prop 值变化

#### Step 6: `renderer.rs` — 修改 `render_inspector_tab`

- 高亮区域添加 `[编辑]` 按钮
- 编辑模式：表单 UI（每个 prop 一行 text_input + 保存/取消按钮）

#### Step 7: `renderer.rs` — 添加 `parse_element_props` 辅助函数

从源码片段解析 `key: value` prop 对。

### E5：编辑写回 + 热重载

#### Step 8: `dynamic.rs` — 添加 `write_source_range` 方法

```rust
pub fn write_source_range(&self, offset: usize, len: usize, new_content: &str) -> Result<String, String>
```

读取文件 → 替换字节范围 → 写回 → 更新 `last_modified`。

#### Step 9: `renderer.rs` — 添加 `reconstruct_element_source` 辅助函数

原始源码片段 + 编辑的 prop 值 → 重构后的源码。替换 `key: value` 模式中的 value。

#### Step 10: `renderer.rs` — 实现 `apply_edit` 函数

1. 从 `edit_span` 获取偏移量
2. 从 `source_code` 取原始片段
3. `reconstruct_element_source` 生成新源码
4. `component.write_source_range` 写回文件
5. 刷新 `source_code` 和 `source_line_offsets` 缓存
6. 500ms 热重载 tick 自动检测变更并 reload

### 实施文件清单

| 文件 | 步骤 | 改动 |
|------|------|------|
| `crates/auto-lang/src/ui/iced/renderer.rs` | E3-E5 | DebugRenderCtx 添加 component 引用、wrap_debug 查 span、编辑状态、消息处理、Inspector 编辑 UI、辅助函数 |
| `crates/auto-lang/src/ui/dynamic.rs` | E5 | 添加 `write_source_range()` 方法 |

### 验证步骤

1. `cargo build` 编译通过
2. 运行 `auto examples/ui/010-contact-form/src/front/app.at`
3. F12 → 点击元素 → Inspector 源码定位高亮（E3）
4. `[编辑]` → 表单 → 修改 prop → 取消退出（E4）
5. `[编辑]` → 修改 prop → 保存 → 文件更新 + UI 重载（E5）

## Phase F: 源码查看器三大优化

### 需求

| # | 需求 | 说明 |
|---|------|------|
| F1 | 全文件语法高亮缓存 | 所有代码行彩色显示，tokenization 只在源码加载时做一次，缓存 `Vec<(String, Color)>` 结构 |
| F2 | 自动滚动居中 | 点击 UI 元素时，Inspector 源码自动滚动到高亮区域，使高亮行居中显示 |
| F3 | 直接源码文本编辑 | 替换旧的表单编辑模式，使用 `text_editor` 多行编辑器直接修改源码 |

### 实施方案

#### F1: 语法高亮缓存

- 新增 `cached_highlighted: RefCell<Option<Vec<Vec<(String, Color)>>>>` 到 `DynamicState`
- 将 `highlight_line()` 重构为 `tokenize_line()` 纯函数（返回数据不创建 widget）
- 新增 `build_highlight_cache()` 一次性为所有行做 tokenization
- 源码加载/热重载/编辑写回时自动构建缓存
- `render_inspector_tab()` 改为渲染全部行（不再 viewport clipping），从缓存读取颜色

#### F2: 自动滚动居中

- 使用 iced 0.14 的 `scrollable::scroll_to(id, AbsoluteOffset)` API
- 新增 `inspector_scroll_id` 和 `pending_scroll_to_center` 状态
- `render_devtools_panel()` 中给 scrollable 附加 ID
- 选中元素时计算高亮起始行号，设置 pending_scroll
- `update()` 末尾返回 scroll Task，将高亮区域滚动到视口 1/3 位置
- 延迟滚动处理：首次点击时 styles 可能尚未包含 span，下一帧自动补算

#### F3: 直接源码文本编辑

- 替换旧的 `edit_values: HashMap` + 表单编辑为 `edit_textarea_key: Option<String>` + `text_editor`
- 复用项目已有的 `get_textarea_content()` + `TEXTAREA_CONTENTS` 静态存储
- `[编辑]` 按钮点击后提取 span 范围源码，初始化 text_editor content
- 保存时直接从 `TEXTAREA_CONTENTS` 读取编辑后文本，写入文件
- 删除旧的 `parse_element_props()`、`reconstruct_element_source()` 函数

### 改动文件

| 文件 | 改动 |
|------|------|
| `crates/auto-lang/src/ui/iced/renderer.rs` | DynamicState 新字段、tokenize_line/build_highlight_cache、渲染重写、scroll Task、text_editor 编辑 |

## Phase G: 动态面板宽度 + 源码高亮修复 + 可拖拽分隔栏

### 需求

| # | 需求 | 说明 |
|---|------|------|
| G1 | 面板宽度动态化 | 面板宽度改为动态状态（默认 ~420px），窗口初始大小增大到 1600×900，监听 resize 事件更新窗口尺寸 |
| G2 | 源码高亮修复 | 修复 tag 名映射和 counter 语义不匹配导致 span 查找失败的 bug |
| G3 | 可拖拽分隔栏 | 主窗口和面板之间添加 6px 可拖拽分隔栏，支持鼠标拖拽实时调节面板宽度 |

### G2 根因分析

span 查找失败的两个原因：

1. **tag 名不匹配**：AuraNode 中 `center` 的 tag 是 `"center"`，但 `AuraViewBuilder` 将其转换为 `View::container()`，渲染中 `wrap_debug` 查找 key 为 `("container", idx)`。`"center"` ≠ `"container"`
2. **counter 语义不同**：`wrap_debug` 使用全局递增 counter，而 `collect_all_spans_dfs` 使用 per-tag counter。即使 tag 匹配，idx 也不对

### G2 修复方案

- `wrap_debug` 改用 per-kind counter（新增 `kind_counters` 字段）
- `collect_all_spans_dfs` 加入 tag 别名映射（`center→container`, `div→container`），确保 `(kind, per_kind_idx)` key 匹配

### G3 实现方式

- 在 `row![main, panel]` 之间插入 6px `mouse_area` 作为拖拽把手
- 全局 subscription 监听 `CursorMoved` 和 `ButtonReleased` 事件
- 拖拽中根据鼠标 X 坐标实时计算面板宽度 `win_w - mouse_x`
- 拖拽中把手变蓝色，正常时灰色
- 面板宽度限制在 200px ~ (window_w - 200px) 之间

### 改动文件

| 文件 | 改动 |
|------|------|
| `crates/auto-lang/src/ui/iced/renderer.rs` | DynamicState 新字段、动态面板宽度、拖拽把手渲染、鼠标事件处理、per-kind span 查找 |
| `crates/auto-lang/src/ui/dynamic.rs` | `collect_all_spans_dfs` 改为 per-tag counter + tag 别名映射 |
