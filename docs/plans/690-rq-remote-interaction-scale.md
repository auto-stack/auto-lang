---
plan_id: PLAN-690
status: executing               # drafting → executing → execution_done → reviewed → archived
feature_name: rq-remote-interaction-scale
author: [zcode]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 desktop-protocol-v1 InputMethod 控制下行通道（IME 激活/光标区/purpose）+ preedit cursor 上行修复, SD-02 ui overview remote 模式交互完备契约（IME/hover/键盘导航/光标形状）]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design/autoui/desktop-protocol-v1.md, docs/design/autoui/rq-remote-renderer.md, docs/specs/auto-lang/ui/overview.md]
current_step: 0
total_steps: 8
---

# [PLAN-690] rq-remote-interaction-scale——remote 模式交互完备与规模化（方案 2 第二阶段）

> 来源：PLAN-683 交付后的下一阶段路线（2026-09-22 用户批准「OK」）。683 交付
> 了 remote 模式三试点渲染+交互闭环（ASCII）；本计划把方案 2 从「三试点通过」
> 推到「日用可依赖」：**P1 交互完备**（中文 IME 闭环打头 + hover/光标/键盘
> 导航核验）+ **P2 规模化稳态**（帧率/帧体积/多窗曲线/D5 自愈）。
> 设计依据 [rq-remote-renderer](../design/autoui/rq-remote-renderer.md)（§7 spike
> 已绿，架构已实施）。

## 变更摘要

1. **IME 激活门控补全（致命缺口，本会话勘定）**：winit 只在窗口被
   `request_input_method` 后才发 IME 事件——App 侧 iced text_input 聚焦发出的
   `InputMethod::Enabled{cursor,..}` 请求在无窗 headless 宿主内无处落地，daemon
   canvas 无人请求 → **daemon 窗内中文组合根本不触发**。修法 = InputMethod
   请求**下行控制通道**（App 截获请求 → wire Control 下行 → daemon 对自己窗口
   执行 `window::InputMethod` task）；候选窗定位 = App 下发焦点框光标矩形 →
   daemon `enable_ime(cursor)` → winit `set_ime_cursor_area`。
2. **preedit cursor 上行修复**：`live_input_from_input_method` 的
   `Preedit(text, _)` 丢弃矩形 → 保留回填 `InputMsg::ImePreedit.cursor`（现
   rqhost 硬编码零矩形）。
3. **交互面核验补全**：hover（指针位同步已在 headless——核验 003/004 hover
   类生效）；光标形状（手型/文本杆——daemon 侧 set_cursor 下行位）；Tab
   键盘焦点导航与焦点环。
4. **规模化实测**：代表 app（charts/gallery）DisplayList 帧体积与 120Hz
   帧率曲线；N 窗单 daemon 内存/fps 曲线（单窗 12MB 已实测）；P683-D5
   最小化自愈臂。

## 目标

- **G1（IME 闭环）**：Windows 中文 IME 在 remote 模式实机可用——组合串
  on-the-spot 渲染、候选窗定位正确（不遮挡焦点框）、commit 入值、Esc 取消。
  红线兜底：若 winit/iced IME 面在 daemon 窗残缺，按预案走混合方案（text_input
  类交互保留 daemon 侧原生组件）——决策档呈报后裁定。
- **G2（交互核验）**：hover 态渲染生效（003/004 hover 类对照）；Tab 焦点序
  +焦点环；光标形状随 hover 目标切换。
- **G3（规模化）**：帧体积/帧率实测报告在档（charts 级 app + gallery 代表）；
  N≥3 窗单 daemon 内存/fps 曲线；超限项登记债（diff/脏区优化另立）。
- **G4（稳态）**：D5 最小化 0x0 自愈臂落地（daemon 检测 resized 0x0 →
  SW_RESTORE 策略或忽略零帧）。

**非目标**：不做子树 diff/脏区优化（实测超限另立）；不翻转 desktop_render
缺省（F-3 另立爬坡门）；不做跨机远程；不动 v1 legacy 轨。

**成功标准**：中文输入法在 remote 窗内完成「打中文→候选→上屏→入值」全环
（实机录证）；hover/Tab/光标三面核验通过；规模化报告在档且无未登记超限。

## 架构方案

```
App（headless iced 宿主）                     daemon（iced 窗）
 text_input 聚焦 → iced 发                     收 Control::Ime{Enabled,
 window::InputMethod::Enabled{cursor}           Disabled,cursor} 下行
      │ headless 截获（不发 OS）──────────►    → window::InputMethod task
      │                                         → enable_ime(cursor)
 winit 组合事件（daemon 窗）                         │
      │                                         OS IME 候选窗（定位=
 Ime::Preedit(text, rect) ── InputMsg 上行 ──►   cursor 矩形）
      ▼                                          │
 UserInterface::update(InputMethod)          rect 随上行回传（修复②）
 → text_input 渲染 preedit → 下一帧
```

要点：①App 侧截获点是 `UserInterface::update` 后的 window 命令流
（T-01 勘定 iced_runtime 暴露面——iced_test 同型问题先例）；②cursor 矩形
坐标系 = app 视口坐标，daemon 侧加窗偏移换算（多窗时按焦点窗）；③purpose
（Password 等）透传。

## 需求分析与背景调查

- **授权记录**：2026-09-22 用户批准下一阶段 = P1+P2 合批（「OK」——本会话
  上一轮路线图建议：IME 闭环打头 + hover/光标/键盘导航 + 帧率与多窗实测 +
  D5 修复；P3 缺省翻转另立）。
- **IME 勘定（本会话完成，代码级）**：
  - 上行链路在：`rqhost.rs:1070` `Event::InputMethod(Ignored 门)` →
    `session.rs:2555 live_input_from_input_method` → `InputMsg::Ime*` →
    `headless.rs:622-629` 三态映射 → `UserInterface::update`；
  - **缺口①激活门控**：`rqhost.rs` 全文无 `request_input_method`/
    `window::InputMethod` 调用——daemon 窗从不 enable IME（winit 缺省
    allowed=false → 组合事件不发生）；
  - **缺口②矩形丢弃**：`session.rs:2560 Preedit(text, _)` 弃 rect；
    `rqhost.rs:840` 补 `WRect::new(0,0,0,0)` 硬编码；
  - iced 0.14 API 面核实：`iced_winit window.rs:212 request_input_method
    (InputMethod::Enabled{cursor, purpose, preedit})` 完整承接
    （enable_ime + preedit overlay）；`iced_core input_method.rs` enum 公开。
- **hover 现状**：`headless.rs:196/697` 指针位同步（Cursor::Available）已在
  ——hover 渲染预期已通，本计划核验+登记差距即可。
- **P683 债承接**：D5（最小化 0x0 环境怪）→ G4 自愈臂；D6（IME 未实机）→
  本计划 G1 主项（完成后 D6 销号）。

## 详细设计

### IME 下行控制通道（T-02 主体）

- wire：`ControlMsg` 增 `ImeRequest { wid, enabled: Option<ImeReq> }`——
  `None`=Disabled，`Some{cursor: WRect, purpose: u8}`=Enabled（追加式 tag，
  旧端未知忽略语义沿用）；
- headless 截获：T-01 勘定 iced_runtime `UserInterface::update` 返回的
  window 动作暴露面（若 update 返回 `Vec<window::Action>` 直读；若不暴露，
  从 iced_winit conversion 层借路径——iced_test 仿真同型，先例可查）；
- daemon 落地：`rq_update` 收 Control → `iced::window::InputMethod::*`
  task 发到该 wid 窗（多窗偏移换算在 daemon 侧按窗 rect）。

**T-01 勘定结论（2026-09-22，代码级三锚点，待澄清①销号）**：

1. **App 侧截获点 = `UserInterface::update` 返回的 `State::Updated`**
   （iced_runtime-0.14 user_interface.rs:615-640）：`State::Updated {
   mouse_interaction, redraw_request, input_method: InputMethod,
   has_layout_changed }` 直接公开 IME 请求与鼠标光标形状——无需任何
   hack 或降级路径（headless.rs:127/177 现以 `let _ = ui.update(..)`
   丢弃 State，截获即改捕获）。text_input 在 `RedrawRequested` 事件臂
   发 `shell.request_input_method(Enabled{cursor, purpose, preedit})`
  （iced_widget text_input.rs:1348-1353）→ update 返回时已 merge 好。
   cursor 矩形 = App 视口逻辑坐标（text_input 内部 layout 推得）。
2. **daemon 落地点 ≠ 计划原设的 `iced::window::InputMethod` task**
  （勘误：iced 0.14 无此公开 task——`window::Action` 枚举无 IME 变体，
   iced_winit 的 `request_input_method` 是内部方法，lib.rs:969 每帧
   自刷）。实际路径 = **`iced::window::run(id, closure)`
   （iced_runtime/window.rs:463）→ iced_winit `run_action` 在事件循环
   线程执行闭包（lib.rs:1633）→ `&dyn Window`（HasWindowHandle）→
   `RawWindowHandle::Win32` → HWND → IMM API**。IMM 语义照抄 winit
   0.30.13 ime.rs:115-151：enable = `ImmAssociateContextEx(hwnd, 0,
   IACE_DEFAULT)`；cursor area = `ImmSetCompositionWindow(CFS_POINT)`
   + `ImmSetCandidateWindow(CFS_EXCLUDE)`；purpose = Windows no-op
  （winit `set_ime_purpose` 空实现）。逻辑→物理换算用 daemon 窗
   scale factor（`iced::window::scale_factor(id) -> Task<f32>` 开窗后
   取一次，client 登记）。非 Windows daemon = 观测行 + no-op（remote
   桌面 v1 主战场 Windows）。
3. **daemon 侧 preedit 可视化 v1 裁定 = 缺省不建**：iced 0.14.2
   text_input **不自绘** preedit 内容（draw 只用 `state.value`，
   text_input.rs:589 preedit 仅门 placeholder）；真窗的组合串可见性
   来自 iced_winit runtime overlay（window.rs:263 draw_preedit）或
   OS IME 缺省 UI（ImmSetCompositionWindow 定位）。remote 窗 v1 依赖
   OS 缺省 composition UI（下行走通后候选窗/组合串由系统绘在定位
   点）；T-04 实机若出现双绘/不绘再立 overlay 补件。

### preedit cursor 上行（T-03）

**语义勘正**（iced 0.14 源级）：`input_method::Event::Preedit(String,
Option<Range<usize>>)` 第二参 = 组合串内**字节选区**（byte-wise，
winit conversion.rs:322 `(start, end) -> start..end`），**非矩形**。
适配实现：`live_input_from_input_method` 保留 `Option<(usize, usize)>`
→ `LiveInput::ImePreedit { text, selection }` → wire
`InputMsg::ImePreedit` 的 `cursor: WRect` 字段**原位重定义为
`selection: Option<(u32, u32)>`**（现硬编码零矩形、headless 弃读——
双端同仓同版，无兼容包袱；SD-01 注记）→ headless 注入
`Preedit(text, selection.map(|(s,e)| s..e))`（App 侧 text_input 收进
`state.preedit`，随后经 `InputMethod::Enabled.preedit` 原样上报——
T-01 截获面即环测断言口）。

### 光标形状（T-05 部分）

App 侧 hover 命中（iced Cursor + widget hover 语义）得出目标光标形状 →
下行 `ControlMsg::SetCursor{wid, kind}` → daemon 应用。v1 不追求
widget 级精细——text_input/按钮两级先行，其余 default。

**T-05 补充勘定（2026-09-22）**：①App 侧来源 = T-01 同一截获面
`State::Updated.mouse_interaction`（iced_core mouse/interaction.rs，
~27 变体——wire v1 两级映射：Pointer→1 / Text→2 / 其余→0）。②daemon
应用路径 = **view 态而非 Win32**：`&dyn Window` 只有 rwh 句柄面无
set_cursor；且 iced_winit 每帧 `window.update_mouse(mouse_interaction)`
会以 daemon 自身 UI 的 interaction 覆盖 OS 光标（lib.rs:969）。
裁定 = `rq_view` 内容外包 `mouse_area().interaction(映射图标)`——
SetCursor 下行更新 client 状态 → 下一帧 daemon UI 自带该 interaction
（hover 期 update_mouse 读到的即它，纯 iced 跨平台）。③Tab 键盘焦点
导航勘定：iced 0.14 runtime/`UserInterface::update` **无内建 Tab 遍历**
（全库仅 focus_next/focus_previous operation 定义，无 runtime 调用
方——遍历是 app 级显式 opt-in）。remote 臂的 Tab 语义 = 上行保真
（Tab 事件到达 App widget 树，与 native 轨同语义）；app 级遍历缺省
不在本计划展开，实机走查按 parity 口径记录。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/design/autoui/desktop-protocol-v1.md | 控制通道无 IME 面 → 增 ImeRequest/SetCursor 下行 + ImePreedit.cursor 真值 | G1/G2 通道载体 | AC-01/02 |
| SD-02 | modify | docs/specs/auto-lang/ui/overview.md | remote 模式交互契约未成文 → IME/hover/Tab/光标四面完备契约 + 实测数据行 | G1-G3 验收口径 | AC-01..05 |

## 测试设计

- 单测：ImeRequest/SetCursor codec round-trip；live_input 矩形保留；headless
  Preedit(text, rect) 注入 → 帧含组合串（text_input 内部渲染）断言；
- 环测（协议级）：管道注入 ImeEnabled{cursor} → daemon 窗 `window::InputMethod`
  task 断言（可观测行）；Preedit/Commit 环 remote 变体（沿 683 typing 环形制，
  增中文三段串用例）；
- 实机：Windows 中文 IME 手工走查脚本（003-converter：聚焦→拼「你好」→
  候选窗位置截图→上屏→联动；Esc 取消）——录证入 reports；
- 规模：`AUTO_FIT_TRACE` 同型观测行扩展（帧字节数/ops 数）+ 帧率采样
  （万拍均值）；N=1/2/4 窗 daemon 内存曲线复用 `[rqhost] mem` 观测行。

## 验收标准

- [ ] AC-01 Windows 中文 IME 实机全环：003 remote 窗内拼音组合可见
      （on-the-spot）、候选窗不遮挡输入框、上屏入值联动、Esc 取消——
      录证（截图/日志）入档。
- [ ] AC-02 ImeRequest/SetCursor wire + daemon task 链单测/环测绿
      （含 codec round-trip）。
- [ ] AC-03 hover/Tab 核验：003/004 remote 窗 hover 类样式生效对照截图；
      Tab 焦点环遍历可走查。
- [ ] AC-04 规模化报告在档：≥2 代表 app 帧体积+帧率 + N≥3 窗内存曲线；
      超限项全部登记债（无未登记项）。
- [ ] AC-05 D5 自愈臂落地：模拟最小化（0x0 resize）后渲染恢复（单测或
      实机走查脚本验证）。
- [ ] AC-06 既有门禁零新增红（desktop_protocol/fit/typing 全套基线持平）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

- [x] T-01 勘定：iced_runtime `UserInterface::update` 的 window 动作暴露面
      （InputMethod 请求从何截获——iced_test 先例对照）；产出决策段回填
      §详细设计。[关联 AC-02] 验证：勘定记录 + headless 截获点代码锚定。
      [✅ 已完成]（2026-09-22）三锚点：①截获点 = `State::Updated{input_method,
      mouse_interaction}`（user_interface.rs:615-640，headless.rs:127/177 现
      丢弃待捕获）；②daemon 落地 = `window::run`→HWND→IMM（勘误：无
      `window::InputMethod` task；winit ime.rs:115-151 语义照抄）；③preedit
      第二参勘正 = 字节选区 `Option<Range<usize>>` 非矩形。决策段已回填
      §详细设计；待澄清①降级路径销号。
- [x] T-02 IME 下行通道：wire ControlMsg::ImeRequest + headless 截获转发 +
      daemon `window::InputMethod` task。[关联 AC-01/02] 验证：环测绿
      （管道注入→task 观测行）。
      [✅ 已完成]（2026-09-22，commit 25dee6696）wire ImeRequest/SetCursor
      tag15/16+ImeReq 载荷；headless State::Updated 截获（勘定补遗：仅
      Redraw 臂——事件路径 shell 态瞬态 Disabled，照收误发 Disable，
      drive_and_draw 独占截获）+去重+drain_window_controls 泵缝；daemon
      window::run→HWND→IMM（win_ime.rs，winit ime.rs 同语义）+scale 取样。
      环测：p690_ime_downlink_arrival_and_zero_resize_guard 管道环绿（点击
      聚焦→ime_applied 落位含非退化 cursor 矩形）。
- [x] T-03 preedit cursor 上行修复：`live_input_from_input_method` 矩形保留
      + rqhost 真值 + headless 注入；typing 环 remote 变体增中文用例。
      [关联 AC-02] 验证：环测绿。
      [✅ 已完成]（2026-09-22，commit 25dee6696）语义勘正落地：第二参 =
      字节选区（非矩形）——LiveInput::ImePreedit{selection} 四构造消费点
      （session/rqhost/mcp/broker）+wire 原位重定义+headless 注入；
      环测：headless 截获环（preedit 上报回环/Commit winit 序镜像
      Preedit("") 先行/Esc 取消）+管道环中文上屏出帧 绿；codec round-trip
      含 Some((5,9))/None 两形态。
- [ ] T-04 Windows 中文 IME 实机走查（003）：组合/候选/上屏/取消录证；
      残缺项按预案混合兜底呈报裁定；D6 销号。[关联 AC-01]
- [ ] T-05 交互核验：hover 对照截图（003/004）+ Tab 焦点环 + 光标形状
      （SetCursor 通道 + 两级先行）。[关联 AC-03]
- [ ] T-06 规模化实测：帧体积/帧率观测行 + charts/gallery 代表 + N 窗内存
      曲线 → 报告 + 债登记。[关联 AC-04]
- [x] T-07 D5 自愈臂：daemon 0x0 检测 + 恢复策略。[关联 AC-05]
      [✅ 已完成]（2026-09-22，commit 25dee6696）WindowResized 0x0 守卫
      （零尺寸不转发 app、基线尺寸不动——恢复期假空白根除；真实尺寸
      恢复 resize 正常转发同步）；管道环 p690 测试③断言（0x0 后
      client.width 不动/640x480 同步）绿。策略取"忽略零帧"臂
     （SW_RESTORE 主动恢复不需要——真实尺寸事件恢复期自然到达）。
- [ ] T-08 门禁全量 + 复审收口：SD-01/02 落表、D6/D5 债册销号、设计档
      状态更新。[关联 AC-06]

## 复审记录

- draft 交付（2026-09-22）：stage=new，PLAN-690 rev1。outcome=pass。
  next=work。

## 待澄清事项

1. T-01 勘定若 iced_runtime 不暴露 InputMethod 请求（iced_test 同样受阻），
   降级路径 = headless 侧自算焦点 text_input 光标矩形（从 layout 树取——
   UserInterface 有 bounds 面），不依赖 iced 内部请求流。勘定后定。
2. 混合兜底（若 winit IME 在 daemon 窗残缺）：text_input 交互保留 daemon
   侧原生组件的拆分方案——仅 G1 红线时呈报裁定，不预先展开。
3. N 窗实测的窗数上限（4 窗起步，超限继续加）——T-06 现场定。
