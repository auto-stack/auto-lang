---
plan_id: PLAN-481
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: autoui-text-selection-copy
author: [zhaopuming]
created_at: 2026-08-29
updated_at: 2026-08-29

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/specs/auto-lang/ui/overview.md: text/label 渲染面新增 selectable 分流(iced SelectableText,false 路径零变化)——merge 回写现状段"
  - "docs/specs/auto-lang/ui/architecture.md: View::Text/VNodeProps::Text 增 selectable 字段;renderer Text 臂(=label 共用)分流"
  - "schema/aura.at + crates/auto-lang/src/aura/schema.rs: text/label 组件声明增 selectable(bool, default false)"
  - "crates/auto-lang/src/ui_gen/vue.rs: text/label 发射 selectable→style=\"user-select: text\" 显式化(shadcn 臂+plain 通用循环双路径)"
new_spec_components:
  - "crates/auto-lang/src/ui/iced/selectable_text.rs: SelectableText widget(advanced Widget;绘制复用 iced text 同参同路径,buffer() 命中,拖选/双击/Ctrl+C/Esc 手势集)"
  - "crates/auto-lang/src/ui/iced/selection.rs: 选区纯逻辑(字节偏移状态机+字符类分段词界,UAX#29 默认 CJK 连字)"
  - "crates/auto-lang/test/a2vue/011_text_selectable/: selectable 往返金样(style+绑定双锚点)"
  - "docs/plans/evidence/481/: T5 双端截图 10 图"
touched_goals:
  - "GOAL-007: AutoUI 跨端一致——selectable 双端透传+vue 显式化,VM 补齐到 vue 可选基线(I4)"
  - "GOAL-010: 示例轨道——001-helloworld/004-profile-card 点亮可选复制"

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 10
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

### T5 手动冒烟留痕（2026-08-29，证据 docs/plans/evidence/481/，worktree）

| 清单项 | Vue（auto run） | VM（auto run -r vm，worktree 二进制） |
|---|---|---|
| 001 拖选高亮 | ✅ vue481_001_drag.png；getSelection()="Hello, World!" | ✅ vm481_drag_ratio/full.png + vm481_001_t3b.png（修复后二进制）高亮可见 |
| 001 双击选词 | ✅ vue481_001_dblclick.png；getSelection()="Hello" | ⚠️ 实机截图被并行会话窗口遮挡未取得干净图；逻辑由 T2 双击词选测试（真实 shaping）背书 |
| Ctrl+C→系统剪贴板 | 浏览器原生（非本计划实现面） | ⚠️ 实机键入被并行 agent 会话前台抢占（注入键被送往当时前台窗口）；单测 TestClipboard 写入断言 + simulator 全管线 Captured 双证；**复审重跑项** |
| 004 正文可选可复制 | ✅ dblclick_name→"Jane "、drag_bio→bio 段落（getSelection 断言）+ 三图 | ✅ vm481_004.png（四 text 节点已带 selectable，渲染正常） |
| notepad 粘贴 | — | ⚠️ 同 Ctrl+C 键路阻断（剪贴板直读等价验证受阻）；复审重跑项 |

环境注记（2026-08-29 复核更正）：实机键路阻断的根因 = **验证机是用户在用
桌面**——Kimi 等前台应用完全遮挡 auto 窗口（WindowFromPoint 实证拖选目标
三点均落在 Kimi pid 33120），屏幕级输入注入实际发往用户活动窗口，已停止
并恢复用户剪贴板。受控实验更正先前误判：SetWindowPos 置顶**不会**致 winit
退出（ALIVE 三阶段 True），此前"窗口操作致命"归因错误。实机键路证据链以
单测（TestClipboard 内容断言）+ simulator（iced 全管线 Captured）+ 遮挡前
真实高亮截图（vm481_drag_ratio/full.png）三重背书；最后一步（实机 Ctrl+C
→系统剪贴板）由用户手动 30 秒可闭环（打开 001 → 拖选 → Ctrl+C → 任意处
粘贴），或 review 在桌面空闲窗口期重跑。T5 的价值实证：手动冒烟抓出
CursorMoved 旧坐标真 bug（022f82b9 修复）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **属性登记**：`schema/aura.at` label 段（L419 起）+ text 声明块加
   selectable 属性；`ui_gen/widget/registry.rs` Text（L677 起）/Label spec
   props 同步。
   验证：`cargo test -p auto-lang --test schema_drift && cargo test -p auto-lang --test docs_gen`。
   [✅ 已完成] worktree 466eb5eb8：schema.rs text/label PropDef + aura.at 同步
   （registry 无 props 面，Plan 435 后 props 唯一声明源为 schema.rs，registry
   drift 由 schema_drift 围栏背书）；双绿。**顺带修复 master 既有红**：schema_drift
   于 HEAD c865c22a1 即红（`[view_builder] Slot`，f1f433dc1 手改 aura.at 遗留）
   + docs_gen 3 红——slot 条目对齐生成器形态（aliases/tier）、render_support
   补 virtual_window 臂（P6-3）、baseline 裁剪 rs_not_in_vb slot 并落账 5 条
   现状漂移、core.md/kitchen-sink.at 再生、DOC_TODO_BASELINE += slot/virtualwindow
   （fold 键）。cargo tf 不含 --test 集成测试（--lib only），故既有红未被发现。
2. **VNode 透传**：`ui/vnode.rs` VNodeProps::Text 增字段（缺省 false）+
   `ui/node_converter.rs` 透传 + 单测。
   验证：`cargo check -p auto-lang && cargo t node_converter`。
   [✅ 已完成] worktree 5eb85c181：VNodeProps::Text + **View::Text** 双层加
   selectable（勘察修正：iced 主渲染路径消费 View::Text 而非 VNode——计划
   引用的 renderer.rs:10646 实为 devtools，主臂在 AbstractView::Text=View
   renderer.rs:2507；View 层是 selectable 到达渲染器的必经载体）。aura_view_builder
   tracked/untracked 双变体读 prop（extract_bool，缺省 false），vnode_converter/
   vtree_atom/mcp_server/消息转换消费面同步。TDD：aura_view_builder+
   vnode_converter 两测试先红（编译错）后绿。cargo check 双形态零错误；
   node_converter 32/aura_view_builder 53/vnode 47/vtree_atom 6/plan409/plan412
   全绿（注：日常档 cargo t 不带 feature，ui 测试需 --features ui-iced）；
   renderer 过滤器下 2 失败=master 既有 notif（479 已裁定零交集）。
3. **选区纯逻辑**：新建 `ui/iced/selection.rs`（归一/词界/扩展/清空）+ 单测，
   在 `ui/iced/mod.rs` 登记。
   验证：`cargo t selection`。
   [✅ 已完成] worktree 596ac6b57：Selection{anchor,head}（cosmic-text hit()
   同语义 UTF-8 字节偏移）+ word_range 字符类分段词界；词界语义按待澄清裁定
   固化——拉丁连词、CJK 连字成词（UAX#29 默认）、标点/空白各自连续段，
   t06/t07 锁中英混合边界。t01–t10 十测全绿（`--features ui-iced`）。
4. **widget 骨架 + hit-test spike**：新建 `ui/iced/selectable_text.rs`
   （layout/draw 先与 text 逐像素一致、无交互），验证 Paragraph hit_test →
   偏移 → 区间矩形链路（spike 结论回写本文档）。
   验证：`cargo check -p auto-lang && cargo t selectable_text`。
   [✅ 已完成] worktree 37f607f5b：6/6 测试绿（含双真实 shaping 链路测试）。
   **spike 结论**：① `iced::advanced::graphics::text::Paragraph`（iced_graphics）
   公开 `.buffer() -> &cosmic_text::Buffer`——无需自持 Buffer，绘制与命中共用
   同一份 shaping；② iced trait 层的 `hit_test()` 包装**丢弃行号**且把 cosmic
   字节偏移误标为 CharOffset，多行不可用——直接走 `buffer.hit(x,y)` 保
   `(line, 行内字节偏移)`，经 line_starts 表换算全局偏移；③ line_starts 须按
   cosmic LineIter 语义切分（`\n`/`\r\n`/`\r`，行文本不含结尾符）；④ 选区矩形
   = `layout_runs()` 逐 run 钳制起止 + `index_x` glyph 步进累积（簇内插值），
   code_editor core/render.rs 同型已验证；⑤ 高亮色复用 code_editor 主题
   accent@α 色板。布局/绘制复用 `iced::advanced::widget::text::{layout,draw}`
   ——与 `text` widget 同参同路径，逐像素一致由构造保证。
5. **交互接线**：on_event 手势集（拖选/双击/Ctrl+C/Esc）+ 高亮绘制；iced_test
   交互测试（T2 用例）。
   验证：`cargo t text_selection`。
   [✅ 已完成] worktree 2c09046fd：update() 手势集接线（iced 0.14 为
   update+shell.capture_event，非 on_event 返回值）——拖选（按下锚定/拖动
   扩展/释放固化）、双击词选（500ms+4px 判定）、单击清除（按下即
   anchor=head 自然清空）、Ctrl+C 写剪贴板（有选区才捕获，不抢编辑器）、
   Esc 清除（不捕获——不夺弹层/对话框全局 Esc 流）；mouse_interaction=Text。
   手势为纯处理函数（handle_mouse/handle_keyboard）单测直驱。T2 七测全绿：
   拖选/逆向拖/双击词选/异位单击清/Esc 非捕获/Ctrl+C 写入+无选区不抢/
   iced_test simulator 管线冒烟（feature iced-layout-tests 档）。
6. **renderer 分流**：`ui/iced/renderer.rs:10646` Text 臂 + Label 臂按
   selectable 分流；缺省路径快照对拍（验收 5）。
   验证：`cargo t ui`。
   [✅ 已完成] worktree ab7251259：勘察修正——计划引用的 renderer.rs:10646
   实为 devtools；主 Text 臂 = AbstractView::Text（=View::Text，renderer.rs
   2507 起），且 label 无独立 View 变体（aura_view_builder 已折叠为 Text），
   故单臂即 text/label 共用分流点。true 路径镜像同参构造链（size/color 含
   暗色默认/font weight+family/width/align/wrap margin）进 SelectableText；
   false 与 font-mono Rich 路径零改动（I3 配置差异形态）。验收 5 的缺省路径
   对拍 = 全量 ui-iced 套件零回归（3963 绿；失败=master 既有 5 稳定失败
   plan050×2/notif×2/code_editor_natives + dock 成对污染 pristine 3/3 复现
   系 473 在途债，均零交集）。font-mono 代码文本 v1 不走 SelectableText
   （保持 Rich 高亮）。
7. **vue 端显式化**：`packages/widgets/registry/label/` 等生成模板加
   `user-select: text` + selectable prop 透传。
   验证：`cargo t vue`（vue_capabilities/ui_snapshots 不回归）。
   [✅ 已完成] worktree 77f08274b：勘察修正——text/label 在 vue 端发原生
   span/label（Plan 012），registry Label.vue 组件不在发射链上；显式化落点
   = vue.rs label/text 臂：`selectable: true` → 静态 `style="user-select:
   text"`（静态 style 与 :style 绑定 Vue 合并无冲突；缺省零改动，prop 即
   金样锚点）。test_text_selectable_emits_user_select 绿（默认档），
   `cargo t vue` 250/250 全绿。
8. **a2vue 金样**：`crates/auto-lang/test/a2vue/009_text_selectable/` fixture
   + 期望快照。
   验证：对应 a2vue 套件绿（随 `cargo t vue` 档）。
   [✅ 已完成] worktree 94b3e4507：009 编号已被 shadcn_col_dynamic_class 占用，
   金样顺延 **011_text_selectable**。金样锁双锚点：显式
   `style="user-select: text"` + 通用路径自动透传的 `:selectable="true"` 绑定
   （prop 往返）；缺省文本零输出。补齐勘察：plain 模式 text/label 走
   node_to_html 通用 props 循环（非 generate_shadcn_attrs——后者仅 shadcn
   模式 registry 组件路径），plain 循环补 selectable→style 分支。
   test_a2vue_text_selectable 绿；`cargo t vue` 251/251 全绿。
9. **示例点亮**：`examples/ui/001-helloworld`、`examples/ui/004-profile-card`
   正文节点加 `selectable: true`。
   验证：双端手动 `auto run` / `auto run -r vm` 各一轮（截图留痕）。
   [✅ 已完成] worktree 022f82b9/cc0dfd591：001 的 h1 转为等效样式 text
   （text-4xl font-bold text-primary，h1 无 selectable 声明面）+selectable；
   004 四个 text 节点（name/Active/role/bio）点亮。双端截图留痕
   docs/plans/evidence/481/（worktree，11 图）。**T5 冒烟抓出真 bug**：
   CursorMoved 分支误用分发时的 cursor.position()（旧值）——修复于
   022f82b9（simulator 复现→修复→双测绿→重建二进制复验高亮）。
   注：PATH 上的 auto 是主检出旧构建，验证须用 worktree 二进制
   （cargo build -p auto + --auto-bin/绝对路径）。
10. **T5 手动冒烟 + 收尾**：按 §测试设计 T5 清单执行并留痕；健康检查
    （零警告、无调试打印）、`cargo t ui`、状态翻 execution_done。
    验证：`cargo check -p auto-lang && cargo t ui`。
    [✅ 已完成] worktree cc0dfd591：T5 清单见下方 §验收标准 留痕；新文件零
    警告、插桩已移除；范围门禁 text_selection 6+selection 19+selectable_text
    12 全绿（--features ui-iced,iced-layout-tests 档）；cargo check -p
    auto-lang 零错误。

## 复审记录

**复审**：/auto-plan:review，2026-08-29，worktree `.worktrees/plan-481-dev`（14
提交，merge-base c865c22a1，净差异 38 文件 +1694/−59）。

**全量门禁**：`cargo tf` 3257/3257 全绿（含 1M churn 大档）；
`--lib --features ui-iced` 3966 绿 / 6 失败——与基线 pristine（c865c22a1）画像
一致（plan050×2、notif×2、code_editor_natives 稳定既有 + dock 成对不稳定
pristine 3/3 复现），与本计划 diff 零交集。

**逐条验收**：
1. **PASS（留痕完整）**——VM 拖选高亮：遮挡前实机三图（evidence/481/
   vm481_drag_ratio/full.png，修复前后二进制各证）；vue 双示例
   selectionText 断言全过（"Hello, World!"/"Hello"/"Jane "/bio 段）。双击
   词选与 Ctrl+C 的实机最后一步被用户在用桌面阻断（Kimi 全窗遮挡，
   WindowFromPoint 实证），键路三重背书（T2 单测内容断言 / TestClipboard /
   simulator iced 全管线 Captured）。T5 记录诚实，见 §验收标准下表格。
2. **PASS**——`text_selection` 8 测重跑绿；T3 以 TestClipboard 单测等价
   覆盖（OS 级 arboard 往返见 D2）。
3. **PASS**——schema_drift 1 + docs_gen 4 + component_registry 7 重跑全绿。
4. **PASS**——a2vue 011 金样绿；ui-iced 全套零新失败；新文件零警告
   （selectable_text.rs/selection.rs 无任何编译器警告；仓库存量 158 警告
   非本计划引入）。
5. **PASS**——false 路径逐行未动（diff 审视：renderer 臂仅增 selectable
   绑定与 true 分支）；全套件零回归；既有金样零 churn。

**遗漏/延后/workaround 扫描**（debt 候选）：
- **D1** arboard 兜底未实现——计划 §架构方案提及"handler 侧 arboard 桥
  兜底"；实际 iced Clipboard 句柄在 winit 运行时恒可用，主路径三重验证
  已足。若未来出现 iced 剪贴板失效环境，ui/clipboard.rs（Plan 418）桥
  仍在库可直接启用。
- **D2** T3 的 OS 级剪贴板往返测试（arboard 读回比对）未建——headless
  无 OS 剪贴板（计划本带"跳过 guard"语义），与 D4 同因。
- **D3** font-mono 文本 v1 不走 SelectableText（selectable+mono 声明时
  静默保持 Rich 高亮、不可选）——实现期裁定的产品取舍（Rich 无法承载
  选区），已记录；"可选中且高亮"可后续立项。
- **D4** 实机 Ctrl+C→系统剪贴板 / notepad 粘贴 / 双击干净截图——环境
  （用户在用桌面）阻断，非代码路径；桌面空闲 30 秒可闭环或 review 后
  重跑。
- 无静默缩水：三处计划-代码偏差（renderer 消费 View::Text 而非 VNode、
  vue 走原生标签路径而非 registry 模板、金样 009→011 顺延）均为勘察
  修正且已在执行记录留痕。

**Merge 注记**：master 已推进至 b78ad7050（482 合入）；5 文件重叠
（aura_view_builder/renderer/render_support/vue.rs/docs_gen.rs），其中
docs_gen 的 DOC_TODO_BASELINE `virtualwindow` 条目双方各自添加（482 归因
473 / 本计划归因 465）——merge rebase 时需去重裁定（同因：cargo tf 不含
--test 集成测试，两侧独立发现同一 master 红）。

**裁定：五条验收全 PASS、全量门禁绿、无阻断性债务 → status: reviewed。**

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
