# Plan 337：单 VM Widget 树 — 消除子组件独立 VM

> **For Claude:** 本计划重构 VM UI 架构：从"每个子组件一个独立 VM"改为"一个 VM 跑整个 widget 树"。核心改动：① 多 widget handler 编译进同一 module（命名空间化）；② render_child_widget 不建新 VM（同一 heap 上操作 child state）；③ 事件路由带 widget_name。

## 问题
当前 `render_child_widget` 为每个子组件创建临时 VM（`VmBridge::new`），导致：
1. Handler 隔离：EditorPanel 的 `.Edit`/`.Save`/`.Delete` 在临时 child VM 里，事件路由到 App VM → 找不到
2. State 不通：child handler 写的 `editing`/`edit_title` 在 child VM 里，App VM 读不到
3. 每帧重建：每次渲染都 `new_with_imports`（重新编译 back.api）→ NewNote 卡 10 秒

## 目标架构
```
App VM (唯一 VM)
├── 合成 module: App + EditorPanel + NoteItem 的 handler 全编译进去
│   handler 命名: handler_App_SelectNote, handler_EditorPanel_Edit, ...
├── State heap: 多个 GenericInstanceData
│   App state (heap_A), EditorPanel state (heap_E), NoteItem state (heap_N)
├── 渲染: 遇到 Component(EditorPanel) → 不建新 VM
│   在同一 heap 创建/更新 child state object → 展开 child view_tree
└── 事件: DynamicMessage 已带 widget_name
    call_handler("EditorPanel", "Edit", heap_E) → handler_EditorPanel_Edit
```

## Phase 1 — 合成多 widget module（handler_codegen.rs）
- 新增 `synthesize_multi_widget_module(root_widget, child_widgets, import_stmts)`
- 遍历 root + 所有 registry 中的子组件，handler 加 `<WidgetName>_` 前缀
- 所有 state type 声明编译进同一 module
- VmBridge 新增 `child_state_map: HashMap<String, u64>` + `ensure_child_state()` 方法

## Phase 2 — render_child_widget 不建新 VM（aura_view_builder.rs）
- 不调用 `VmBridge::new`，改用 `self.bridge.ensure_child_state(child_widget, &props)`
- AuraViewBuilder 新增 `override_state_obj_id: Option<u64>` 字段
- 用同一个 bridge 展开 child view_tree

## Phase 3 — 事件路由带 widget_name（vm_bridge.rs, dynamic.rs, iced/renderer.rs）
- `call_handler(widget_name, event_name, state_obj_id, args)` → 查 `handler_<widget>_<event>`
- `on_with_input(widget_name, event_name, input)` — DynamicMessage 已携带 widget_name
- iced renderer 转发 `msg.widget_name`

## Phase 4 — 验收
- 015-notes vm：列表 + 切换 + Edit + Save + Delete + New（不卡顿）+ Search
- 016-calendar vm 回归
- handler_codegen 测试通过

## 不做
- 不改 VNode 结构（state flat，VNode 不挂 state）
- 不改 vue/rust 模式
- 不改 back.api/db 跨模块链接
