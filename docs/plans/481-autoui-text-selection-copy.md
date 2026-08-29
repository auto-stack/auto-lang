---
plan_id: PLAN-481
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: autoui-text-selection-copy
author: [zhaopuming]
created_at: 2026-08-29
updated_at: 2026-08-29

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 10
---

# [PLAN-481] AutoUI 展示型文字组件的选择/复制（VM selectable text）

## 变更摘要

补齐 AutoUI 展示型文字组件（`text`/`label`）在 VM/iced 端的**选择与复制**能力：
上游 iced 0.14 的 `text` widget 纯展示、无选择（[iced#36](https://github.com/iced-rs/iced/issues/36)
长期开放），本仓所有 label/文字展示在 VM 端均不可选不可复制——而 vue 端浏览器
默认可选，形成双端行为差。本计划：

- `text`/`label` 组件新增 `selectable` 属性（**v1 默认 false，opt-in**），
  走 `schema/aura.at` + WidgetRegistry + VNode 的 I4 双端登记流程；
- VM 端新自研 `selectable_text` widget（`ui/iced/selectable_text.rs`）：复用
  iced advanced text 的 Paragraph 命中测试与 code_editor/autodown_editor 已验证
  的选区绘制模式，不引入第三方 crate（社区 `iced_selection` 仅作参考）；
- 手势集 v1：拖选、双击选词、Ctrl+C 复制（on_event 的 iced Clipboard 句柄，
  arboard 桥兜底）、Esc/单击清除；
- vue 端显式化 `user-select: text`（防应用级 none 吞掉），prop 透传；
- a2vue 金样 + iced_test 交互测试 + `examples/ui/001-helloworld`、
  `004-profile-card` 双示例点亮。

## 目标

- **G1**：`text`/`label` 声明 `selectable: true` 后，VM 端可用鼠标拖选（含高亮）、
  双击选词、Ctrl+C 把选中文本写入 OS 剪贴板。
- **G2**：选区状态为 widget 本地状态，不进 DesktopSession/WmState——对
  VirtualWindow/WM/投影协议零影响（桌面集成零改动）。
- **G3**：双端一致性：vue 端保持可选基线并显式化；`selectable` prop 双端
  透传进 a2vue 金样。
- **G4**：示例点亮：001-helloworld 与 004-profile-card 的正文文字可选可复制，
  双端（`auto run` / `auto run -r vm`）行为一致。
- **非目标**：右键"复制"上下文菜单（后置，Plan 402 先例可复用）；富文本/
  markdown/chat_message 等复合组件的选区推广；默认值翻转为 true（另立决策）；
  OS↔桌面剪贴板桥（= 473 Phase 2 的通道侧，本计划是其"生产者半边"）。

## 架构方案

**数据流（既有 I4 登记管线的加法）**：

```
schema/aura.at (label 段 L419 起 + text 声明)  ── prop: selectable(bool, default false)
  → ui_gen/widget/registry.rs  Text/Label WidgetSpec（registry.rs:677 起登记族）
  → ui/vnode.rs  VNodeProps::Text { content, selectable, .. }
  → ui/iced/renderer.rs:10646  (VNodeKind::Text, VNodeProps::Text) 臂分流
      selectable=false → iced::widget::text（现状，不变）
      selectable=true  → ui/iced/selectable_text.rs::SelectableText
```

**SelectableText widget 结构**（新文件，advanced Widget trait）：

- `layout`：与 `text` 同参测量（Paragraph min_bounds）；
- `draw`：文本 + 选区高亮 quad（Paragraph hit_test → 偏移 → 行内区间矩形；
  链路以 T3 spike 定案，code_editor `draw.rs` 同型先例）；
- `on_event`：ButtonPressed 记 anchor、CursorMoved 拖选扩展、ButtonReleased
  固化、双击词边界、Ctrl+C → clipboard 写入、Esc 清空；
- 选区纯逻辑抽 `selection.rs`（range 归一/词边界/扩展/清空），全平台单测。

**与既有资产的关系**：
- 剪贴板：on_event 携带 iced Clipboard 句柄（首选）；handler 侧 arboard 桥
  （`ui/clipboard.rs`，Plan 418）兜底；
- 手势包装：mouse_area 先例（renderer.rs Plan 402 右键、debug hover 捕获）；
- 编辑器先例：`ui/code_editor/iced/widget.rs`（cosmic-edit 架构）与
  `autodown_editor` 已验证 glyph 命中/选区绘制全套。

## 技术栈

iced 0.14 advanced widget API（`Widget::on_event`/`Paragraph` 命中测试）、
iced_test（交互测试）、arboard（兜底）。**零新三方依赖**。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui 模块 + 现场代码核验 2026-08-29）

- **ui 模块现状**：桌面线 462–465/472/478 已归档，473（native dock）7/10、
  479（通知中心）execution_done、480（桌面协议 Stage3）在途——本计划不碰
  session/layout（G2），renderer.rs 改动集中在 10646 臂与新增文件，与 480 的
  S3/S8 并行冲突可控（后合者 rebase）。
- **上游事实**：iced 0.14（crates.io 原版，crates/auto-lang/Cargo.toml:141）
  `text` 无选择能力（iced#36 开放中）；`text_input`/`text_editor` 有——故
  input/textarea 已可选，缺口只在展示型组件。社区过渡 crate `iced_selection`
  0.3.1 存在，本计划不引入、仅参考实现。
- **本仓先例核验**：`ui/clipboard.rs`（Plan 418 arboard 桥，含 headless 跳过
  式测试先例）；`ui/code_editor/`（core/{fold,highlight,render}.rs +
  iced/widget.rs + draw.rs + theme.rs）与 `ui/autodown_editor/` 均为自研带选区
  组件；renderer.rs 已有 mouse_area 包装先例（Plan 402）。
- **双端基线**：`packages/widgets/registry/`（label.vue 等，reka-ui/shadcn 系）
  未设 user-select——浏览器默认可选，vue 端无需功能开发，仅显式化 + prop
  透传；I4 对拍目标 = VM 补齐到 vue 现状。
- **登记管线**：WidgetRegistry（`ui_gen/widget/registry.rs`，Text 登记
  L677 起含 alias "text"）+ `schema/aura.at`（Plan 435 唯一声明源）+
  schema_drift/docs_gen 测试守门。
- **战略定位**：473 Phase 2（OS↔桌面剪贴板通道）的生产者半边——没有本计划，
  通道打通后展示型文字也无物可复制；A4/A5（文本互灌）用例的直接前置。

## 详细设计

### 1. 属性与登记

- `schema/aura.at`：label 声明段（L419–430）加
  `{ name: "selectable", type: "bool", default: false, description: "..." }`；
  text 组件声明块同型（执行时以 schema_drift 对齐确认其在 aura.at 的落点，
  registry 侧 Text 为独立 WidgetSpec）。
- `ui_gen/widget/registry.rs`：Text/Label WidgetSpec props 增 selectable
  （vue 后端映射透传）。
- `ui/vnode.rs` + `ui/node_converter.rs`：`VNodeProps::Text` 增
  `selectable: bool`（缺省 false），转换器透传；Label 同型。

### 2. selectable_text widget（ui/iced/selectable_text.rs + selection.rs）

- `selection.rs` 纯逻辑：`Selection { anchor: usize, head: usize }` + 归一
  range、词边界切分（Unicode word boundary，双击语义）、拖选扩展、清空。
- `selectable_text.rs`：如 §架构方案；高亮色走主题（对齐 text_editor 选区
  色板），仅 VM 侧存在（`#[cfg(feature = "ui-iced")]` 自然隔离）。

### 3. renderer 分流

- `ui/iced/renderer.rs:10646` Text 臂按 `selectable` 分流（false 路径零改动，
  I3 单路径原则的配置差异形态）；Label 臂执行时定位同型处理。

### 4. vue 端

- `packages/widgets/registry/label/`（及 text 对应生成模板）加显式
  `user-select: text`；`selectable` prop 透传为无操作属性（默认可选是 vue
  现状，prop 仅作语义声明与金样锚点）。

### 5. 金样与示例

- a2vue fixture：`crates/auto-lang/test/a2vue/009_text_selectable/`（.at 源 +
  期望 vue 快照，断言 selectable prop 往返）。
- `examples/ui/001-helloworld`、`examples/ui/004-profile-card`：正文文字节点
  加 `selectable: true`。

### 6. 桌面集成说明

选区为 widget 本地状态：VirtualWindow 裁剪/合成自动正确（高亮随内容走），
Ctrl+C 经桌面级键盘路由在焦点窗口内生效（既有机制），WM/投影协议零改动。

## 测试设计

1. **T1 纯单元**（进 `cargo t` 日常档）：`selection.rs` 状态机——归一/词边界
  （中英文混合词界）/拖选扩展/清空；node_converter selectable 透传。
2. **T2 iced_test 交互**：构造 SelectableText 实例，注入 按下→拖动→释放
   →断言选中文本；双击→断言词选；Esc→断言清空。
3. **T3 剪贴板往返**：Ctrl+C 后 arboard 读回比对（arboard 不可用的 headless
   环境跳过，Plan 418 测试同款 guard）。
4. **T4 双端对拍**：a2vue 金样 009_text_selectable 绿；schema_drift /
   docs_gen / component_registry_test 不回归。
5. **T5 手动冒烟**：001/004 双端（`auto run` / `auto run -r vm`）：拖选高亮、
   双击选词、Ctrl+C 粘贴进 notepad 验证真实剪贴板；结果逐行记入 §验收标准下。

## 验收标准

1. 001-helloworld 与 004-profile-card 在 VM 端：拖选可见高亮、双击选词、
   Ctrl+C 后系统剪贴板含选中文本（T5 手动清单 PASS 留痕）。
2. `cargo t text_selection`（T1/T2）全绿；T3 在有剪贴板环境全绿。
3. schema 三件套不回归：`cargo test -p auto-lang --test schema_drift`、
   `--test docs_gen`、component_registry_test 绿。
4. a2vue 金样新增且全绿；`cargo t ui` 不回归；`cargo check -p auto-lang`
   零警告。
5. `selectable` 缺省（不声明）时渲染输出与改动前逐项一致（false 路径零行为
   变化，截图/快照对拍）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **属性登记**：`schema/aura.at` label 段（L419 起）+ text 声明块加
   selectable 属性；`ui_gen/widget/registry.rs` Text（L677 起）/Label spec
   props 同步。
   验证：`cargo test -p auto-lang --test schema_drift && cargo test -p auto-lang --test docs_gen`。
2. **VNode 透传**：`ui/vnode.rs` VNodeProps::Text 增字段（缺省 false）+
   `ui/node_converter.rs` 透传 + 单测。
   验证：`cargo check -p auto-lang && cargo t node_converter`。
3. **选区纯逻辑**：新建 `ui/iced/selection.rs`（归一/词界/扩展/清空）+ 单测，
   在 `ui/iced/mod.rs` 登记。
   验证：`cargo t selection`。
4. **widget 骨架 + hit-test spike**：新建 `ui/iced/selectable_text.rs`
   （layout/draw 先与 text 逐像素一致、无交互），验证 Paragraph hit_test →
   偏移 → 区间矩形链路（spike 结论回写本文档）。
   验证：`cargo check -p auto-lang && cargo t selectable_text`。
5. **交互接线**：on_event 手势集（拖选/双击/Ctrl+C/Esc）+ 高亮绘制；iced_test
   交互测试（T2 用例）。
   验证：`cargo t text_selection`。
6. **renderer 分流**：`ui/iced/renderer.rs:10646` Text 臂 + Label 臂按
   selectable 分流；缺省路径快照对拍（验收 5）。
   验证：`cargo t ui`。
7. **vue 端显式化**：`packages/widgets/registry/label/` 等生成模板加
   `user-select: text` + selectable prop 透传。
   验证：`cargo t vue`（vue_capabilities/ui_snapshots 不回归）。
8. **a2vue 金样**：`crates/auto-lang/test/a2vue/009_text_selectable/` fixture
   + 期望快照。
   验证：对应 a2vue 套件绿（随 `cargo t vue` 档）。
9. **示例点亮**：`examples/ui/001-helloworld`、`examples/ui/004-profile-card`
   正文节点加 `selectable: true`。
   验证：双端手动 `auto run` / `auto run -r vm` 各一轮（截图留痕）。
10. **T5 手动冒烟 + 收尾**：按 §测试设计 T5 清单执行并留痕；健康检查
    （零警告、无调试打印）、`cargo t ui`、状态翻 execution_done。
    验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **默认值策略**：v1 opt-in（false）已定；翻转为默认 true 会改变所有 text 的
  命中测试/事件路由面，另立决策（建议示例全量点亮并收集使用反馈后评估）。
- **hit-test API 细节**：iced 0.14 advanced text Paragraph 的 hit_test/span
  区间矩形具体 API 形态以 T4 spike 定案（code_editor draw.rs 先例参照）；
  若 Paragraph 不暴露区间矩形，备选 = 按 glyph run 手工累积（code_editor 同型）。
- **词边界语义**：双击选词按 Unicode word boundary；中文逐字还是连字成词
  （UAX#29 默认连字）在 T3 单测里固化为期望行为，双端以 VM 为准（vue 端跟随
  浏览器原生双击行为，允许实现差异，金样只锁 prop 透传不锁选区行为）。
- **排程**：与 480 并行时 renderer.rs 以后合者 rebase 为准；建议 473 合入后
  开工（剪贴板故事闭环顺序：本计划（生产者）→ 473-P2（通道））。
- aura.at 中 text 组件是否独立声明块（registry 侧 Text 为独立 spec）以 T1 的
  schema_drift 结果为准对齐，不预改。
