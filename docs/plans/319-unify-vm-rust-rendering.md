# 统一 VM / Rust 渲染：单一转换器 + `View::Grid` 变体

> **For Claude:** 核心 `crates/auto-lang/src/ui/iced/renderer.rs` + `crates/auto-lang/src/ui/view.rs`。每次改完 `cargo build -p auto`；回归 `cargo test -p auto-lang --lib`。手动验证：`examples/ui/016-calendar` 在 `render:vm` 和 `render:rust` 下渲染一致。**在专用 worktree 里执行**（per CLAUDE.md）。

## Context（为什么做）

VM 模式和 Rust 模式渲染同一个 `AbstractView<M>` 却走**两套转换器**：
- `IntoIcedElement::into_iced(self) -> Element<M>`（[renderer.rs:647](crates/auto-lang/src/ui/iced/renderer.rs#L647)）—— 泛型 `M`，rust 模式用。
- `render_dynamic_view(view: AbstractView<IcedMessage>, …) -> Element<IcedMessage>`（[renderer.rs:5819](crates/auto-lang/src/ui/iced/renderer.rs#L5819)）—— `IcedMessage` 专用，VM 模式用。

`render_dynamic_view` 自己重写了 **6 个 arm**（Input / Textarea / Column / Row / Container / Scrollable），其余通过 `_ => view.into_iced()` 委托。这 6 个 arm 的目的只是：① 给每个子节点 push/pop `path` 并 `wrap_debug`（DevTools 插桩）；② Input/Textarea 接 VM 专属的 `textarea_perform_action` 文本捕获。**结果：行/列/容器的 widget 构造逻辑（spacing、padding、justify-spacer、width、children 循环）在两处各写一遍，会漂移。**

grid 更严重：`View` 枚举**没有 Grid 变体**，grid 在**构造 View 树时**就被拆成 col-of-rows，拆分逻辑写了**三遍**（`ui_gen/rust.rs:1211`、`convert_grid` `aura_view_builder.rs:1239`、`convert_grid_tracked_ctx` `aura_view_builder.rs:678`）。本周它直接造成 bug：rust 模式日历渲染成竖直"高塔"——VM 那条路靠 `wrap_debug` 的副作用没塌，rust 这条路塌了，两条路漂移。（高塔已由本会话的 `w-full` 补丁临时修复；本计划是**根治**，让漂移不再可能发生。）

**目标产出**：① widget 的形状/样式逻辑只写一次；② grid 拆分只写一次；③ 立一条规矩——以后新增 widget 只在 `into_iced` 加一个 arm，`render_dynamic_view` 永远不重复实现 widget 形状。

## 架构决策（为什么是"共享泛型 builder"而非 thread-local）

`into_iced` 是**泛型 `M`** 的，而 `wrap_debug`/`DebugRenderCtx` 是 `IcedMessage` 专用的——无法在泛型 `into_iced` 里产出 `Element<IcedMessage>`。所以"把插桩塞进 `into_iced`（thread-local）"的方案**不成立**。

正确边界：**提取共享的、泛型 `M` 的 widget builder**（`build_row<M>` / `build_column<M>` / `build_container<M>` / `build_scrollable<M>` / `build_grid<M>`）。这其实是对**已存在模式**的自然延伸——`apply_row_style`/`apply_column_style`/`apply_container_style` 已经是泛型、被两个转换器共用、且接收 `widget_id: Option<String>`（让 IcedMessage 路注入 id、泛型路传 `None`）。新的 `build_*` 就是再往上一层（包住 `apply_*_style` + children 循环）。

- `into_iced` 的 Row/Column/Container/Scrollable/Grid arm 变薄：`children.into_iter().map(into_iced).collect()` → `build_row(children, …)`。
- `render_dynamic_view` 的同名 arm 变薄：给每个 child 插桩（path push/pop + 递归 `render_dynamic_view` 让其自包裹）→ 调**同一个** `build_*`（传 instrumented children + widget_id）→ `wrap_debug` 结果。
- 叶子（Text/Button/…）维持现状（已通过 `_ => into_iced()` 共享）。
- Input/Textarea：形状（width/placeholder/password）抽成 `build_input_shape<M>`/`build_textarea_shape<M>`，**on_change/on_submit/textarea key 仍由各自 caller 接**（两条路的 message 接线本质不同；textarea 的 key 必须留 caller 算，否则 `get_textarea_content` 缓存会错）。

**为什么不"重建 children 后直接对父节点调 into_iced"**：那样会丢掉每个子节点的独立 `wrap_debug` 插桩（整行变成一个不透明节点）——已验证不成立。共享 builder 之所以对，正是因为它接收**已构建、已插桩的 children**。

## 任务（B 先于 A）

### Phase 1 — Part B：提取共享泛型 builder（先发，零行为变化）

1. **renderer.rs** 新增 `fn build_row<M>(children: Vec<iced::Element<'static,M>>, spacing, padding, style: Option<&Style>, widget_id: Option<String>) -> Element<M>`（紧邻 `apply_row_style` ~473）。函数体 = 现 `into_iced` Row arm（~756-779）和 `render_dynamic_view` Row arm（~5936-5968）里重复的 justify-spacer + children-push + `apply_row_style`。
2. 同文件 `build_column<M>`（~451 附近）、`build_container<M>`、`build_scrollable<M>`（scrollable 的 id 设置内联进 builder，不新增 `apply_scrollable_style`——它没有 container 那种视觉样式包裹）。
3. **重写 `into_iced` 的 Row/Column/Container/Scrollable arm**（756-961）：薄壳，调 `build_*`，传 `widget_id=None`。
4. **重写 `render_dynamic_view` 同 4 个 arm**（5916-6019）：保留 per-child 插桩（递归 `render_dynamic_view(child,…)` —— 它自己会 `wrap_debug`，**别改成手动逐子 wrap**），调同一个 `build_*`（传 widget_id），末尾 `wrap_debug` 父节点。
5. 新增 `build_input_shape<M>(placeholder, value, width, password, style) -> TextInput<'static,M>` 和 `build_textarea_shape<M>(…) -> TextEditor<'static,M>`；两处 Input/Textarea arm 改用它们，on_change/on_submit/key 留 caller。
6. **验证（必须零行为变化）**：`cargo build -p auto`；`cargo test -p auto-lang --lib`；`016-calendar` 在 vm/rust 两模式目视一致。

### Phase 2 — Part A：新增 `View::Grid` 变体（依赖 Part B）

7. **view.rs**：在 `Image`（~393）后加 `Grid { cols: usize, gap: u16, cells: Vec<View<M>>, style: Option<Style> }`。
8. **view.rs**：`ViewBuilderKind` 加 `Grid`；加 `View::grid()` 构造 + `.cols(n)`/`.gap(n)`；`build()`（~699）加 Grid arm。
9. **renderer.rs 新增 `build_grid<M>(cols, gap, cells: Vec<Element<M>>, style, widget_id)`**：含拆分（chunks、末行补齐、每行 `w-full`、gap）——这是**取代 3 处删除点的唯一真相源**。
10. 给**所有穷举 match** 加 Grid arm（编译器强制，逐个修）：
    - `into_iced`（647）→ `build_grid`。
    - `render_dynamic_view`（5819，`_` arm）→ 逐 cell 插桩 + `build_grid` + `wrap_debug`（**必须显式加**，否则 grid 绕过插桩）。
    - `extract_view_style`（5767）→ `style.as_ref()`。
    - `view_kind`（5795）→ `"grid"`。
    - `patch_input_values`（6035）和 `patch_input_values_iced`（6077，均 `_ => {}`）→ **显式递归 cells**（否则 grid 内 input 值静默不更新）。
    - `map_msg_with_arc`（view.rs:1115）→ 递归 cells（rust 模式 DevTools 每帧走这条，错则崩）。
    - `vnode_converter.rs::extract_kind_and_props`（202）→ 复用 `VNodeKind::Column`（MVP，DevTools 树把 grid 显示为 Column；加 `VNodeKind::Grid` 列为 follow-up）。
    - `vnode_converter.rs::extract_children`（392，`_ => Vec::new()`）→ **显式返回 `cells.clone()`**（否则 VTree 丢掉 grid 子节点）。
    - `snapshot_builder.rs::traverse_view`（64）→ kind `"Grid"` + 递归 cells。
    - **`gpui/renderer.rs`（~600，穷举）→ 内联拆成 col-of-rows**（GPUI div/flex 直接渲染；它是第三后端，无法共享 iced 的 `build_grid`，记为已知例外）。
11. **删除 3 处拆分**：
    - `convert_grid`（aura_view_builder.rs:1239）→ 改为构造 `View::Grid { cols, gap, cells, style }`。
    - `convert_grid_tracked_ctx`（:678）→ 同上，保留 tracked cell 递归。
    - `ui_gen/rust.rs:1211` codegen → 改为输出 `View::grid().cols(N).gap(G).child(…).build()`。

### Phase 3 — Part C：立规矩
12. 在 `into_iced`（646）上方加模块 doc 注释：**新 widget = `into_iced` 一个 arm（有带样式子节点则配一个 `build_*`）；`render_dynamic_view` 永不重复实现 widget 形状，只加插桩。**

## 风险与缓解

| 风险 | 缓解 |
|---|---|
| 通配 match 的**静默**回归（`extract_children`/`patch_input_values×2`/`render_dynamic_view` 的 `_`） | 任务 10 给这 4 处都显式加 Grid arm；补单测：Grid 内放 Input → `patch_input_values` → 断言值已更新；Grid → `view_to_vtree` → 断言子节点数对。 |
| `map_msg_with_arc` 在 rust DevTools 热路径 | 显式 arm（编译器强制）。 |
| `VNodeKind` 无 Grid | MVP 复用 Column；`VNodeKind::Grid` 作 follow-up。 |
| GPUI 无共享 `build_grid` | 内联拆分，文档标注为"第三后端"例外。 |
| Textarea key 分歧（`placeholder.len()` vs `msg.widget_event`） | `build_textarea_shape` **不接收 key**，caller 算。 |
| a2r 快照基线 | 改 codegen 后 grid 用例的生成源会变；跑 a2r conformance，**有意**重置相关基线。 |
| `016-calendar` 可能不触发 grid | 先确认它确实含 `<grid>`（本会话已确认 app.at 有 `grid{ cols:7 }`）；不够则补 `#[test]`：7-cell Grid → `into_iced` + `render_dynamic_view` 双路渲染、断言结构。 |

## Verification

1. **Phase 1（Part B）零行为变化**：`cargo build -p auto` 干净；`cargo test -p auto-lang --lib` 全过；`016-calendar` vm/rust 两模式目视一致。
2. **Phase 2（Part A）**：`cargo build -p auto-lang` —— 在 7 处穷举 match 补齐 Grid arm 前会编译失败，编译器即安全网；`cargo clippy -p auto-lang` 审通配点。新单测：(a) 7-cell Grid → vtree 子节点=7；(b) Grid 内 Input → patch 后值更新；(c) Grid → map_msg 子节点消息已重映射；(d) Grid → into_iced 含 ceil(cells/cols) 个 Row；(e) 同上走 render_dynamic_view + DebugRenderCtx 每 cell 被 wrap。`016-calendar` vm/rust 一致且无高塔。a2r：`cargo test -p auto-lang -- trans`（含 a2r），重置 grid 基线。GPUI：相应 feature 下编译过。
3. 全程在专用 worktree；master 保持稳定；Plan 全绿后再 merge 回 master。
