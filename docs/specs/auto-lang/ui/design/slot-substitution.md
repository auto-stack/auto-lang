# VM 轨 slot（内容模板）替换机制

> 来源：Plan 476（2026-08-29），清偿 auto-musk KD-048 UPSTREAM①
> （需求 `auto-musk docs/designs/009-vm-slot-substitution-requirement.md`）。
> 代码：`crates/auto-lang/src/ui/aura_view_builder.rs`；vue 轨对照
> `ui_gen/vue.rs` `generate_slot_outlet_html` / `slot_child_to_html`。

## 语义（与 vue 轨对齐）

widget 调用位 `slot(name: "X") { … }`（及裸子节点=默认槽）渲染到子 widget
视图 `slot(name: "X")` / `slot` outlet 位置；填充按**父作用域**求值
（state/computed/事件路由父 handler），随每渲染帧重求值；outlet 自带
children 为 fallback（无填充时子作用域求值渲染）。

## 机制：构建期作用域切换 + 兄弟拼接

VM 轨"作用域"= `AuraViewBuilder` 字段组合（`widget_name`/
`override_state_obj_id`/`computed`/`routes` + 共享 `bridge`/registry）。
`SlotFills<'a> { parent: &AuraViewBuilder, fills, bindings }` 在
widget 调用位捕获**父 builder 引用**与调用位循环变量；子构建器经
`slot_fills` 字段持有之。子树转换到 outlet 时切回 `fills.parent` 求值
填充——零字段拷贝、零作用域栈改造。事件（`DynamicMessage::Typed` 带父
widget_name → 命名空间 handler 跑根 state）与逐帧重求值（handler 后全树
重建）天然成立，分发层零改动。

**多子填充走兄弟拼接**（非单包装）：五大容器（col/row/container/scroll/
grid × untracked/tracked 双胎）的 children 收集经
`expand_children_spliced*` 展开——fill 各子节点作为 outlet 位置多个兄弟
加入容器 children，布局轴向由 outlet 父容器决定（Vue 精确语义：musk
actions 双按钮在 flex-row 头横排、033 默认槽双节点在 col 纵排，同一机制
两态皆对）。非拼接上下文（outlet 不在五容器 children）走
`render_slot_outlet` 兜底：单子直通/多子 Column 包装。

## 已知限制（详见 KNOWN-DEBT 476 条目）

- 范围外：teleport、动态 slot 名、多层嵌套 slot 透传、`for` 体内 outlet
  拼接（走兜底）。
- 源索引容器（scroll/container/grid tracked）多子填充 probe 共享 outlet
  路径（快照降级，渲染/事件不受影响）。
- **VM registry 只注册 `use` 导入的 widget**：同项目隐式组件调用落 tag
  fallback（组件自身视图丢失，children 直出）——示例须 `use x: X` 导入
  才走真组件路径（033-slots 即此假阳性现场）。
