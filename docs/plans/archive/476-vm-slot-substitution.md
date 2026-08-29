---
plan_id: PLAN-476
status: archived                 # drafting → executing → execution_done → reviewed → archived
feature_name: vm-slot-substitution
author: [zcode]
created_at: 2026-08-29
updated_at: 2026-08-29

# /auto-plan:review 结束时填写：
supersedes_spec_components: []  # 无取代——补缺性新机制（审查裁定）
new_spec_components:
  - ui/aura_view_builder:slot-substitution（SlotFills 父作用域捕获 + outlet 渲染臂 + 五容器×双胎兄弟拼接）
touched_goals: [GOAL-007]      # AutoUI 跨端视觉一致（vue/vm slot parity 缺口闭合）

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 7
total_steps: 7
---

# [PLAN-476] VM 前端管线 slot（内容模板）替换机制

## 变更摘要

VM(iced) 轨补齐 widget slot 插座/填充机制：调用位的 `slot(name: "X") { … }` 填充
子树渲染到子 widget 视图中对应 outlet 位置，填充按**父作用域**求值（父 state /
computed / use 导入）、事件路由到父 handler、随每渲染帧重求值。改动集中在
`crates/auto-lang/src/ui/aura_view_builder.rs`（构建层替换），`iced/renderer.rs`
零改动。清偿 auto-musk **KD-048 UPSTREAM①**（musk PLAN-050 T5 移交需求
`docs/designs/009-vm-slot-substitution-requirement.md`，2026-08-29）。

## 目标

1. **插座填充**：widget 调用位的 `slot(name: "X") { subtree }` 在 VM 轨渲染到
   子 widget 视图中对应 `slot(name: "X")` outlet 位置（需求 §3.1）。
2. **父作用域求值**：填充子树按调用位（父）作用域求值——`.store.session_list`
   解析到父的 store 单例、computed/`use` 导入按父解析（§3.2）。
3. **逐帧重求值**：父状态变化后下一渲染帧填充内容跟随更新（§3.3）。
4. **事件保留父绑定**：填充内 `onclick: .ParentFn($event)` 路由到父 handler（§3.4）。
5. **默认插槽**：空名 `slot { … }` 与具名插槽并存；调用位非 slot 裸子节点并入
   默认插槽（对齐 vue.rs `slot_child_to_html` 语义）（§3.5）。
6. **校验沿用**：填充给了无插座 widget 沿用 `slot_outlet_names` 构建期 warn；
   name 不匹配的填充不渲染（§3.6）。
7. **范围外**（登记 debt，不实现）：teleport、动态 slot 名（`slot(name: expr)`）、
   多层 widget 嵌套的 slot 透传、`for` 循环体内直接出现的 outlet 拼接（§3.7）。

## 架构方案

**选型：方向 B 精化——构建期作用域切换替换（View 构建层），非 AuraView 数据克隆。**

需求文档给了 A（父预求值+标记拼接）/ B（AuraView 数据替换+作用域栈）/ C（最小编号版）
三个候选。勘验结论（见"需求分析与背景调查"）：VM 轨的"作用域"不是一个可切换的
栈，而是散在 `AuraViewBuilder` 的字段组合（`widget_name` / `override_state_obj_id`
/ `computed` / `routes` + 共享的 `bridge`/`widget_registry`/`import_stmts`）。
因此最自然的落点是：

- **不克隆子 widget 的 `view_tree`**（方向 B 原案），也不做事后 View 树标记遍历
  （方向 A 原案），而是在**子构建器转换到 outlet 节点时**，切回父 builder 求值
  填充子树。父 builder 本身就是父作用域的完整载体，零字段拷贝、零作用域栈改造。
- 填充多子节点时采用**兄弟拼接**（splicing）：fill 的各子节点作为 outlet 位置的
  多个兄弟加入 outlet 父容器的 children——精确对齐 Vue 语义（`<template #x>` 的
  子节点成为 outlet 父元素的直接子节点，布局轴向由该父容器决定）。这避免了
  "单返回值必须包一层 Row/Column" 的方向猜测：musk actions（2 按钮在 flex-row
  头）需要横向、033 默认槽（文本+按钮在 col 体）需要纵向，固定包装无法两者都对。

数据流（一帧渲染内）：

```text
父 builder 转换到 widget 调用位（Component/Element 命中 registry）
  ├─ extract_slot_fills(children)：slot(name:X){..}→fills[X]；bare slot{..}→fills[""]；
  │   非 slot 裸子节点→追加 fills[""]（Vue 默认槽语义）
  ├─ SlotFills { parent: &父builder, fills, bindings: &调用位bindings(循环变量) }
  └─ render_child_widget(_tracked)(.., slot_fills=Some(&fills))
        └─ 子 builder（override_state_obj_id=子 state、widget_name=子名）.slot_fills=Some
              └─ 转换子 view_tree 到 slot outlet 元素（tag=="slot"）
                    ├─ 有匹配 fill → fills.parent（父作用域）逐个转换 fill 子节点
                    │     · 容器拼接路径：多子作为兄弟插入父容器 children（轴向随容器）
                    │     · 非拼接容器路径：单子直通 / 多子 Column 包装（兜底，罕见位形）
                    └─ 无匹配 fill → 渲染 outlet 自身 children（fallback，子作用域）；空→Empty
```

事件与重求值不需新机制（勘验证实）：
- fill 内事件由父作用域 builder 生成 `DynamicMessage::Typed { widget_name: 父名 }`；
  VM 轨分发（`dynamic.rs` ~行 995-1045）按 widget_name 找命名空间 handler
  `handler_<父>_<事件>`、统一跑在根 state 上——父 handler 天然生效。
- VM 轨在每次 handler 后置 dirty 并全树重建（`view_with_debug_gated`），fill 随
  每帧重求值——§3.3 免费获得。

## 技术栈

Rust（crate `auto-lang`，`ui` 模块）；无新依赖。验证用 `cargo t`/`cargo tf`、
`examples/capability-tests/033-slots`（`auto run -r vm`）与
`.agents/skills/autoui-verifier/scripts/test_vm_mcp.py`。

## 需求分析与背景调查

**出处**：auto-musk `docs/designs/009-vm-slot-substitution-requirement.md`（PLAN-050
T5 调查移交，2026-08-29）+ auto-musk `KNOWN-DEBT-AND-RISKS.md` KD-048 UPSTREAM①。
症状：musk `NavSidebar` widget 声明 `slot(name:"list")`/`slot(name:"actions")` 插座，
调用位传入填充后 **Vue 轨正常、VM 轨整个填充子树不渲染**（`autoui_vtree` 导出无
slot 子树节点；证据 auto-musk `tmp/plan050-survey/04-session-nav.vtree.txt`）。

**现状勘验**（本仓 master 09e64c391，本计划起草时逐点核实）：

| 位置 | 现状（行号为起草时） |
|---|---|
| `crates/auto-lang/src/aura/types.rs:223` | `slot_outlet_names()` + `slot_children_warnings()`：构建期校验在；`slot_element_name()`（types.rs:250，私有 fn）提取 outlet 名 |
| `crates/auto-lang/src/aura/schema.rs:2478` | `slot` ElementDef 已声明（解析层 OK） |
| `crates/auto-lang/src/ui_gen/vue.rs:3669/3770` | vue 轨完整：outlet→`<slot name>`；调用位 `slot(name:X){..}`→`<template #X>`、bare `slot{..}`/裸子节点→默认槽；outlet 自身 children=fallback 内容 |
| `crates/auto-lang/src/ui/aura_view_builder.rs` | **零 slot 处理**（除无关的 D-GAP-4 槽位编号）——缺陷主体 |
| `crates/auto-lang/src/ui/iced/renderer.rs` | 零 slot 处理——但构建层替换后渲染器只见完整 View 树，**无需改动** |

**关键调用链缺陷**（`aura_view_builder.rs`，行号为起草时）：

1. widget 实例化四个 registry 命中点全部**丢弃调用位 children**（slot 填充所在）：
   - Component untracked：~532-548 `render_child_widget(child_widget, &prop_values, events, bindings)`
   - Component tracked：~782-796 `render_child_widget_tracked(.., path, id_map, probe)`
   - Element fallback tracked：~1021-1028（`convert_element_tracked_ctx` 的 `_` 臂）
   - Element fallback untracked：~2463（`convert_element` 的 `_` 臂）
2. `render_child_widget`（~2798）/`render_child_widget_tracked`（~2840）：构造子
   builder 只带 `widget_name=子名`、`override_state_obj_id=Some(子state)`、
   `computed=子的`——**父作用域信息全部留在父 builder 里**（这正是本方案复用点）。
3. 子树里的 `slot` 元素目前落 `convert_element`（~1726）`_` fallback → registry
   无名 → 最终 fallback 渲染 outlet 自身 children（通常空）→ `View::Empty`——
   即 outlet 处空白、填充在上游已被丢弃。

**作用域事实**（决定选型）：`Bindings = HashMap<String, Value>` 只是循环变量绑定
（aura_view_builder.rs:44）；真正作用域 = builder 字段组合。`read_state`（:221）
按 `override_state_obj_id` 先读子 state、miss 回退根 state（store 单例合并进根）。
VM 分发（`dynamic.rs` ~995）：所有 handler 以根 `state_obj_id` 跑，按
`widget_name` 找 `handler_<Widget>_<Event>` 命名空间函数；子 widget props 经
`sync_child_props_to_root` 临时回灌。=> 父作用域 builder 生成的 Typed 消息天然
路由到父 handler；handler 后全树重建保证逐帧重求值。

**spec 关联**：auto-lang/ui 模块（aura/ + ui/ 渲染 + ui_gen/，spec 已随 Plan 471
刷新）；本计划补 VM 轨与 vue 轨（ui_gen/vue.rs 既有 slot codegen）的 parity 缺口，
属桌面轨道（活跃线 1）双端 parity（455 线）延伸。

**现成验收资产**：
- `examples/capability-tests/033-slots`（app.at + panel.at）：具名+默认插槽、
  父作用域 state 绑定（`f"clicked ${.clicks}"`）、父事件（`onclick: .Clicked`）
  ——VM 轨此前填充不渲染，本计划直接作实机验收样例。
- musk 现网用例（零改动受益）：`nav_sidebar.at`（双具名插座）/`chats_view.at`
  （填充含父 store for 循环、嵌套 NavListItem、父事件）/`nav_item.at`。

## 详细设计

全部改动集中在 `crates/auto-lang/src/ui/aura_view_builder.rs`（另有
`aura/types.rs` 一处可见性导出）。以下行号为起草时基线，实施时以现场为准。

### D1. `SlotFills` 结构与 builder 新字段

```rust
/// Plan 476: widget 调用位的 slot 填充集。parent 是父作用域 builder 的
/// 直接引用——widget_name / override_state_obj_id / computed / routes /
/// registry 等"父作用域"全部由它承载，fill 求值时切回它即可，无需拷贝。
/// bindings 是调用位的循环变量绑定（fill 内 `.item` 等按调用位解析）。
struct SlotFills<'a> {
    parent: &'a AuraViewBuilder<'a>,
    fills: HashMap<String, Vec<&'a AuraNode>>,   // "" = 默认槽
    bindings: &'a Bindings,
}
```

`AuraViewBuilder` 增字段 `slot_fills: Option<&'a SlotFills<'a>>`（构造点
`new`/`with_registry`/（如有）其他构造器补 `slot_fills: None`；子 builder 构造
点（:2816/:2855 两处 struct literal）显式设置）。生命周期自洽：`SlotFills` 在
`render_child_widget*` 调用栈内构造并借给子 builder，子构建在该栈内完成。

### D2. 填充提取 `extract_slot_fills`

新私有 fn：入参 `children: &[AuraNode]`，出参 fills 映射（不持有 parent/bindings，
由调用点组装 `SlotFills`）。规则（对齐 vue.rs:3770 `slot_child_to_html`）：

- `slot`/`Slot` 元素带 `name` prop（Str/Ident，复用 `slot_element_name` 语义）
  → `fills[name]` = 该元素的 children；
- `slot`/`Slot` 无 `name` → 其 children 追加 `fills[""]`；
- 非 slot 子节点 → 追加 `fills[""]`（Vue 默认槽语义；此前这些子节点在 VM 轨
  widget 调用下被整体丢弃，属同源缺陷的修复）。

`aura/types.rs:250` 的 `slot_element_name` 从私有改 `pub(crate)` 供复用。

### D3. 四个 registry 命中点接线

- `render_child_widget`（:2798）与 `render_child_widget_tracked`（:2840）签名各增
  `slot_fills: Option<&SlotFills>`；子 builder struct literal 设
  `slot_fills: slot_fills`。
- 四个调用点（:548、:789、:1024、:2463）先 `extract_slot_fills(children)`，
  有 fills 才构造 `SlotFills { parent: self, fills, bindings }` 传
  `Some(&fills)`，否则 `None`（保持既有路径零变化）。
- `render_outlet`（:2621 内对 `render_child_widget` 的调用）传 `None`（页面路由
  无调用位填充）。

### D4. outlet 渲染臂（子作用域侧）

- `convert_element`（:1726）match 增早期臂 `"slot" | "Slot" => self.render_slot_outlet(props, children, bindings)`。
- `convert_element_tracked_ctx`（:819）的 tag 分发同步增臂（带 path/id_map/probe
  的 tracked 双胎，文件 D-GAP 规则）。
- `render_slot_outlet` 语义：
  1. `name = slot_element_name(props)`；
  2. `self.slot_fills` 命中 `fills.get(name)`：
     - 单子 → `fills.parent.convert_node_with(子, fills.bindings)` 直通返回；
     - 多子 → 兜底 `View::Column` 包装逐子转换结果（**拼接感知容器不会走到
       这里**，见 D5；此臂覆盖 outlet 出现在非拼接上下文的罕见位形）；
  3. 未命中 → 渲染 outlet 自身 children（fallback 内容，**子作用域** `self`
     转换；对齐 vue `generate_slot_outlet_html`）；空 → `View::Empty`。

### D5. 兄弟拼接 `convert_children_spliced`（Vue 布局精确语义）

- untracked helper：`fn convert_children_spliced(&self, children: &[AuraNode], bindings: &Bindings) -> Vec<View<DynamicMessage>>`
  —— 逐 child：是 outlet 且有匹配 fill → `fills.parent.convert_node_with` 逐个
  fill 子节点产出**多个**兄弟视图；是 outlet 无 fill → fallback children（子作用域，
  产出 0..1 个，空则丢弃）；非 outlet → 常规单视图。
- tracked 双胎 `convert_children_spliced_tracked_ctx(.., path, id_map, probe, bindings)`：
  fill 子节点继续 path/id_map/probe（父作用域 builder 的 tracked 转换），编号遵循
  既有 D-GAP-4 "RESULTING slot" 约定（视觉空子节点出列、后续槽位左移）。
- 接线容器转换器（children 收集循环替换为 helper；布局轴向由容器自身决定，
  这是拼接的意义所在）：
  - untracked：`convert_column`(:2881)、`convert_row`(:3017)、`convert_container`(:3245)、
    `convert_scroll`(:2972)、`convert_grid`(:3173)；
  - tracked：`convert_column_tracked_ctx`(:1042)、`convert_scroll_tracked_ctx`(:1115)、
    `convert_grid_tracked_ctx`(:1170)、`convert_row_tracked_ctx`(:1322)、
    `convert_container_tracked_ctx`(:1431)。
  - 其余转换器（for/conditional/button-content 等收集点）不接线——outlet 出现在
    那些位置的位形落 D4 兜底路径，登记 debt（见范围外）。

### D6. 不改动的部分（明确）

- `iced/renderer.rs`：零改动（替换在 View 构建层完成）。
- `slot_outlet_names`/`slot_children_warnings` 构建期校验：原样沿用（需求 §3.6）。
- vue 轨（ui_gen/vue.rs）、a2r（trans/rust.rs）、schema：零改动。
- `Bindings`/`read_state`/事件分发（dynamic.rs）：零改动（勘验证实已兼容）。

## 测试设计

**单元测试**（`aura_view_builder.rs` 尾部 tests 模块，沿用 `make_test_widget` /
`VmBridge::new` / `AuraViewBuilder::with_registry` / `AuraNode::element` 惯例）：

1. `test_slot_named_fill_renders_at_outlet`：父调用子 widget 传
   `slot(name:"header"){ text "X" }`，子视图 outlet 在 row 内——构建结果含
   fill 文本（View 树断言/递归查找）。
2. `test_slot_fill_resolves_parent_scope`：父 state `label="parent"`、子 state
   `label="child"`，fill `text .label` → 渲染出 "parent"（父作用域胜出）。
3. `test_slot_fill_event_routes_to_parent`：fill 内 `button onclick: .Clicked` →
   `DynamicMessage::Typed.widget_name == 父名`。
4. `test_slot_unmatched_name_not_rendered` + outlet fallback：fill 名
   `slot(name:"nope")` 不渲染；outlet 自带 children 且无 fill → fallback 内容
   （子作用域）渲染。
5. `test_slot_default_fill_from_bare_children`：调用位裸子节点（无 slot 包装）
   渲染到子视图 `slot`（默认 outlet）位置。
6. `test_slot_multi_child_fill_spliced_as_siblings`：fill 2 子节点在子视图 row
   outlet → 产出 2 个兄弟视图（Row children 数量断言），而非单包装。
7. `test_slot_fill_probe_tracked`：`build_with_debug` 后 fill 子树路径在
   BuildProbe snapshot 中有条目（tracked 双胎覆盖）。

**集成验证**：`examples/capability-tests/033-slots`：
- `auto run -r vm`（autoui-verifier 技能，`test_vm_mcp.py`）：VM 快照含 header
  badge 文本 "3 pending"、默认区文本与按钮；点击按钮 → `clicks` 计数递增
  （父 state 逐帧重求值 + 父事件路由的端到端证据）。
- vue 轨 `auto run` 对照零回归。

**门禁**：`cargo check -p auto-lang` → `cargo t iced`（快速档）→ 终局
`cargo tf` 全量（含 1M churn，Category B 规约）。

## 验收标准

1. 033-slots VM 轨实机：具名槽 badge、默认槽文本+按钮可见；点击按钮计数递增
   （对齐需求 §5.1/5.2/5.3 的 auto-lang 侧等价证据）。
2. 新增单测 7 项全绿：slot 替换树变换、父作用域求值、事件路由父绑定、未匹配
   name 不渲染、默认槽、多子兄弟拼接、tracked probe 覆盖（对齐 §5.4）。
3. `cargo tf` 全量绿（含 ui-iced）。
4. vue 轨零回归：033 vue 模式对拍正常；`cargo t` 无新失败。
5. musk 侧零源改动即可受益（KD-048 UPSTREAM① 清偿路径打通；musk 侧 NavSidebar
   实机验收由 musk 仓 PLAN-050 后续批次执行，本仓以 1-4 为等价证据）。
6. 范围外项（多层 slot 透传、for 体内 outlet 拼接、teleport/动态名）登记
   KNOWN-DEBT-AND-RISKS.md。

## 执行步骤
（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] **T1** `crates/auto-lang/src/aura/types.rs`：`slot_element_name`（:250）
  改 `pub(crate)`；`crates/auto-lang/src/ui/aura_view_builder.rs`：新增
  `SlotFills<'a>` 结构 + `AuraViewBuilder.slot_fills` 字段（含全部构造点补
  `None`）+ `extract_slot_fills` helper + `is_slot_outlet` 判定 helper。
  验证：`cargo check -p auto-lang`。
- [x] **T2** `aura_view_builder.rs`：`render_child_widget`(:2798)/
  `render_child_widget_tracked`(:2840) 增 `slot_fills` 参数并设入子 builder；
  四个 registry 命中点（:548/:789/:1024/:2463）提取 fills 并传入；
  `render_outlet` 调用点传 `None`。
  验证：`cargo check -p auto-lang`。
- [x] **T3** `aura_view_builder.rs`：`convert_element`(:1726) 与
  `convert_element_tracked_ctx`(:819) 增 `"slot" | "Slot"` 臂 →
  `render_slot_outlet`（u+t 双胎：匹配 fill 单子直通/多子 Column 兜底/父作用域
  转换；无 fill 渲染 outlet fallback children/Empty）。
  验证：`cargo t slot`（新增单测 1/4/5 随本步落）。
- [x] **T4** `aura_view_builder.rs`：`convert_children_spliced`（u+t）helper +
  十个容器转换器接线（untracked :2881/:2972/:3017/:3173/:3245；tracked
  :1042/:1115/:1170/:1322/:1431）。
  验证：`cargo t slot`（单测 6/7 随本步落）。
- [x] **T5** 单测补齐：测试设计 7 项全部落在 tests 模块（含父作用域胜出
  2、事件路由 3）。
  验证：`cargo t aura_view_builder`（或 `cargo t iced` 快速档）全绿。
- [x] **T6** 实机验证：`examples/capability-tests/033-slots` 下 `auto run -r vm`
  + autoui-verifier `test_vm_mcp.py`：快照含 fill 内容、点击计数递增；
  `auto run` vue 轨对照正常。
  验证：VM 快照/交互证据记录于本节。
- [x] **T7** 终局门禁 + 复审：`cargo tf` 全量绿；`/auto-plan:review` 四步
  （清单审计/遗漏扫描/健康检查/spec-impact 元数据）；范围外项登记
  `docs/plans/KNOWN-DEBT-AND-RISKS.md`。
  验证：`cargo tf` 输出 + review 记录。


### 执行证据（worktree plan-476-dev）

- [x] T1 [✅ 已完成] `cargo check` 过（注：ui 模块在 `ui` feature 门后，check 需
  `--features ui-iced` 才真正编译本文件——见 T5 勘误）；`SlotFills`/`slot_fills`
  字段/`extract_slot_fills`/`slot_outlet_parts`/`slot_element_name` pub(crate) 落位。
- [x] T2 [✅ 已完成] 四个 registry 命中点 + `render_outlet`(None) 接线；
  `slot_fills_for` 统一组装。
- [x] T3 [✅ 已完成] `convert_element`/`convert_element_tracked_ctx` 增
  `"slot"|"Slot"` 臂 → `render_slot_outlet`(_tracked_ctx)（fill 单子直通/多子
  Column 兜底/未命中 fallback children 子作用域求值）。
- [x] T4 [✅ 已完成] `expand_children_spliced`(u) / `expand_one_child_spliced`(t)
  / `expand_children_spliced_resulting`(t-幸存槽位) / `expand_children_spliced_source`(t-源索引)
  + `expand_one_child_untracked`；十容器接线（col/row/container/scroll/grid u+t）。
- [x] T5 [✅ 已完成] 7 单测全绿（`cargo test -p auto-lang --lib --features ui-iced
  test_slot`：12 passed 含 7 新增）；模块回归 49/49 全绿。
  **勘误**：AGENTS.md 的 `cargo check -p auto-lang`/`cargo t` 默认特性不含 `ui`
  feature——ui 模块测试不在日常档，须 `--features ui-iced` 触发（Plan 476 发现，
  复审时在 KNOWN-DEBT 登记测试盲区）。

- [x] T6 [✅ 已完成] 实机验证（关键勘误：VM registry 只注册 `use` 导入 widget——
  033 原示例无 use 导入，Panel 落 tag fallback（children 直出）造成"填充可见"假阳性；
  补 `use panel: Panel` 后走真组件路径）：
  - **worktree exe**（plan-476-dev）VM 快照：`row { text "Settings"; text "3 pending" }`
    + 默认槽 col{Notifications, button}——填充渲染在 outlet 位置、Panel 自身视图在；
    点击填充按钮 → `handler: .App.Clicked`、`clicks: 0 -> 1`、快照刷新 "clicked 1 times"
    （事件路由父 handler + 父 state 逐帧重求值，需求 §3.2/3.3/3.4 实机闭环）。
  - **master exe 对照**（同改后示例）：`row { text "Settings" }` 两 outlet 全空——
    musk KD-048 症状精确复现，修复前后对照成立。
  - **vue 轨**：`auto gen` 产物正确（`import Panel` + `<template #header>` +
    `<slot name="header"/>`/`<slot/>` 双 outlet），use 导入形态零回归。
- [x] T7 [✅ 已完成] `cargo tf` **3237/3237 全绿**（95 skipped 特性门控）；另跑
  `--features ui-iced` 档 **3876/3876 全绿**（含 ui 模块全量，超出验收 3 字面档）。
  KNOWN-DEBT 登记 4 条（范围外项/probe 共享路径/registry use 门/测试盲区）。

## 复审记录

**独立复审（/auto-plan:review 范式，2026-08-29，ZCode）——结论：PASS，status → reviewed**

1. **清单审计（4/4 PASS）**：
   - 验收 1（033 VM 实机）：PASS——快照结构对齐 panel.at 声明（row>[title, 具名填充]、
     col>[默认填充]），交互/计数证据在 T6。
   - 验收 2（7 单测）：PASS——`--features ui-iced test_slot` 12 passed（7 新增），
     模块 49/49。
   - 验收 3（全量门禁）：PASS——cargo tf 3237/3237 + ui-iced 档 3876/3876。
   - 验收 4（vue 零回归）：PASS——codegen 产物正确，cargo tf 无新失败。
   - 验收 5（musk 零源改动受益）：机制等价证据链闭合（master 对照复现症状）；
     musk 侧实机验收归 musk 仓后续批次（plan 原文如此约定）。
   - 验收 6（debt 登记）：PASS——KNOWN-DEBT 4 条（476 前缀）。
2. **遗漏/延后/workaround 扫描**：D3 四命中点/D5 十容器/D4 双臂逐一对码确认；
   延后项（for 内 outlet、probe 共享路径）均已登记；无未经批准的 deferral；
   033 的 use 导入为语义修正（附因注释），非 workaround。
3. **健康检查**：新增代码零警告（cargo check --features ui-iced 下本文件仅
   3 处存量警告：5609/5753/6179，均在非本次改动段）；无 debug print 残留；
   rustfmt——该文件 master 基线即全局非 fmt-clean（~400 处存量差异），本次新增
   段落遵循周边风格，不重排污染 diff（裁定记录在案）。
4. **spec-impact 元数据**：supersedes 无（补缺性机制，无组件被取代）；
   new = ui/aura_view_builder:slot-substitution；touched = GOAL-007。
5. **过程勘误（记录供后续计划借鉴）**：
   - `cargo check -p auto-lang`/`cargo t`/`cargo tf` 默认特性**不含 ui feature**——
     ui/ 模块不在日常档编译（已登记 KNOWN-DEBT）；ui 改动的 check/test 须显式
     `--features ui-iced`。
   - 本仓存在并发会话（477/478 同期施工+文档同步进程），取号后需复查 .next-id
     实际提交值（本次骨架被同步进程扫走、.next-id 终值 479，无撞号）。

## 待澄清事项

（无——需求文档语义完备；范围外清单以需求 §3.7 为准并入验收 6。）
