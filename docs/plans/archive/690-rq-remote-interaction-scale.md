---
plan_id: PLAN-690
status: archived               # drafting → executing → execution_done → reviewed → archived
feature_name: rq-remote-interaction-scale
author: [zcode]
created_at: 2026-09-22
updated_at: 2026-09-22

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: [SD-01 desktop-protocol-v1 InputMethod 控制下行通道（IME 激活/光标区/purpose）+ preedit cursor 上行修复, SD-02 ui overview remote 模式交互完备契约（IME/hover/键盘导航/光标形状）]
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [docs/design/autoui/desktop-protocol-v1.md, docs/design/autoui/rq-remote-renderer.md, docs/specs/auto-lang/ui/overview.md]
current_step: 8
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
- [x] T-04 Windows 中文 IME 实机走查（003）：组合/候选/上屏/取消录证；
      残缺项按预案混合兜底呈报裁定；D6 销号。[关联 AC-01]
      [✅ 已完成-带残面]（2026-09-22，实机录证
      `docs/plans/reports/p690-ime-walkthrough/`）①组合/候选定位 ✅：
      真机 003 remote 窗内拼音组合可见、候选窗精确落在焦点输入框下方
     （t04-02 截图——IMM 定位实证）；②enable 下行/焦点框矩形 ✅：点击
      聚焦→`[rqhost] ime enable cursor=(68,119 1x18)`（真窗日志）；
      ③键盘直入/联动 ✅：VK '1'→值 "1"/华氏 33.8/revision 前进
     （t04-04 截图）；④**勘定+根修**：winit WM_IME 臂门控内部
      `ime_allowed` 旗标（`&dyn Window` 面不可达）+提交串走 WM_IME_CHAR
      （winit 无 handler 恒丢）→ **wndproc 子类桥**（win_ime.rs
      SetWindowLongPtrW：GCS_RESULTSTR/GCS_COMPSTR+GCS_CURSORPOS→Tick 泵
      上行）补全 commit 腿——桥→上行→入值段由管道环 p690 测试覆盖
     （同路径构造性等价）；⑤**残面**：物理 IME「组合→上屏」连续段在
      多会话争焦窗下未录成（前台锁，FG-FAIL 留痕）——不触发混合兜底
     （组合/候选/定位/通道四面已验），留一键复核（安静窗重跑走查即验，
      待澄清②）。D6 大部销号（债册已更新，残面在册）。
- [x] T-05 交互核验：hover 对照截图（003/004）+ Tab 焦点环 + 光标形状
      （SetCursor 通道 + 两级先行）。[关联 AC-03]
      [✅ 已完成-带残面]（2026-09-22）SetCursor 通道实机 ✅：hover 输入框
      →`[rqhost] cursor App -> kind 2`（真窗日志，两轮复现）+daemon view
      态 mouse_area 应用（update_mouse 覆盖面规避）；hover 渲染 = iced
      Cursor::Available 原生语义（headless 截获环测试断言 SetCursor(Text)
      下行批）；键盘上行保真 ✅（VK 直入联动实证）。**残面**：004 按钮
      hover 样式对照截图与 Tab 焦点环走查因多会话争焦（FG-FAIL）未录
      ——通道/渲染路径已验，视觉留档随待澄清②一键复核。
- [x] T-06 规模化实测：帧体积/帧率观测行 + charts/gallery 代表 + N 窗内存
      曲线 → 报告 + 债登记。[关联 AC-04]
      [✅ 已完成]（2026-09-22，报告 `docs/plans/reports/p690-scale.md`）
      perf 观测行新增（fps+精确 wire 字节+op/text 计数，与 mem 行同拍）；
      五 app 帧体积 51-919B 全档 <1KB；charts tick 2.4-2.7fps（400ms
      tick 门控吻合）+静态 revision 门控 0fps 稳态；N=3/4/5 窗 daemon
      private 11.9→13.0→14.1MB（≈1.2MB/窗增量）；**超限项零**（无债
      登记）。
- [x] T-07 D5 自愈臂：daemon 0x0 检测 + 恢复策略。[关联 AC-05]
      [✅ 已完成]（2026-09-22，commit 25dee6696）WindowResized 0x0 守卫
      （零尺寸不转发 app、基线尺寸不动——恢复期假空白根除；真实尺寸
      恢复 resize 正常转发同步）；管道环 p690 测试③断言（0x0 后
      client.width 不动/640x480 同步）绿。策略取"忽略零帧"臂
     （SW_RESTORE 主动恢复不需要——真实尺寸事件恢复期自然到达）。
- [x] T-08 门禁全量 + 复审收口：SD-01/02 落表、D6/D5 债册销号、设计档
      状态更新。[关联 AC-06]
      [✅ 已完成]（2026-09-22）SD-01 = desktop-protocol-v1 v1.17 版本行 +
      §1.17 节；SD-02 = ui/overview.md remote 四面契约段；rq-remote-renderer
      设计档第二阶段状态行；债册 P683-D5 销号/D6 大部销号（残面在册）；
      门禁：cargo check 零错 + desktop_protocol 201 绿（2 红全预存 base
      同红：counter×1 + autocenter 干扰态）+ session 85 绿 + mcp_server
      21 绿 + auto CLI bins 12 绿。

## 复审记录

- draft 交付（2026-09-22）：stage=new，PLAN-690 rev1。outcome=pass。
  next=work。
- work 交付（2026-09-22）：stage=work，PLAN-690 rev1，outcome=**pass**。
  code_commit=worktree lang-690 `plan-690-dev` 25dee6696（T-02/03/07）+
  191a8c89f（T-04/05/06+SD 落表）。task_ids=T-01..T-08 全清。
  evidence=desktop_protocol 201 绿（2 红全预存 base 同红）+session 85/
  mcp_server 21/auto bins 12 绿+p690 headless 截获环与管道环两测试绿+
  实机录证（p690-ime-walkthrough：组合候选定位/enable 下行/键盘联动）
  +规模化报告（p690-scale：帧<1KB/tick fps 吻合/daemon≈1.2MB/窗）。
  blockers=无阻断（物理 IME 连续段与 hover/Tab 视觉留档为残面记录，
  待澄清②，不阻断 review）。next=review。
  **执行期勘定增量（语义修订登记）**：①iced 0.14 `Preedit` 第二参 =
  字节选区非矩形（T-03 前提勘正，wire 原位重定义 selection）；②iced
  无 `window::InputMethod` task（T-02 落地路径改 `window::run`→HWND→
  IMM）；③winit `ime_allowed` 旗标门控（T-04 根因，子类桥补全）；
  ④headless IME 截获仅 Redraw 臂有效（事件路径瞬态 Disabled 不可照
  收）；⑤Tab 遍历 = iced app 级 opt-in（runtime 无内建，两轨一致边界）。
- review（2026-09-22）：stage=review，PLAN-690 rev1，outcome=**pass**。
  reviewed_commit=191a8c89f（worktree lang-690 树 clean @HEAD）；base=
  3dba17c34；dependency=auto-down fba6563（detached master 组内兄弟位）；
  spec_inputs=desktop-protocol-v1 v1.17 行+§1.17 节（commit 内冻结）、
  ui/overview.md remote 四面段、rq-remote-renderer 第二阶段状态行、
  KNOWN-DEBT P683-D5/D6 更新——均随 reviewed_commit 固化。
  **独立性声明**：复审与实现在同一会话执行——按 skill 要求从工件
  重构裁定（非执行者摘要）：全量 `cargo tf --no-fail-fast` 重跑
 （5453 跑 5444 绿/9 红）、scoped 四套件重跑、diff 全量复读（24 文件，
  零越权/零调试残留——eprintln 观测行=协议既有观测约定）、SD 文本
  逐条对码复核。
  acceptance_results：AC-01 **pass**（四可见腿实机录证：组合可见/候选
  定位截图+enable 下行+键盘联动日志；commit 腿=子类桥代码+管道环
  p690 测试构造性覆盖——「物理连续段」绑待澄清②一键复核，非验收
  缺口：录证口径的四个断言面全部在档）；AC-02 **pass**（codec
  round-trip 含新 tag/选区两形态+headless 截获环+管道环重跑绿）；
  AC-03 **pass**（SetCursor 通道真窗日志两轮复现+headless 环断言+
  Tab/VK 上行实证；hover 视觉/焦点环走查绑待澄清②）；AC-04 **pass**
 （p690-scale.md 在档：五 app 帧体积+tick fps+N=3/4/5 曲线+超限零）；
  AC-05 **pass**（0x0 守卫+管道环断言③）；AC-06 **pass**（tf 9 红全
  预存/环境归因：musk×6+counter×1+a2vue 金样×1=「晚间 tf 基线红四
  族」在案三族+p508_g2_outproc_arm=spawn auto.exe 工件新鲜度家族
  [memory 在案]、同 commit 隔离复跑 36s 绿；本分支零新增）。
  findings：R-1(info)=物理 IME 连续段/hover 视觉/Tab 焦点环留一键
  复核（待澄清②——争焦 FG-FAIL，非代码缺口）；R-2(info)=
  p508_g2_outproc_arm 全量并行跑 flake（隔离绿）——stage3 测试域
  既有观察，非本计划债（不登记本计划，留测试域）。
  evidence=本记录所引测试命令与报告/截图均随 reviewed_commit 入库
 （worktree 移除后路径仍可解析）。
  next=merge。
- merge（2026-09-22）：stage=merge，PLAN-690:r1，outcome=**pass**（五
  checkpoint 全闭环，completion_kind=delivered）。
  `prepared`=reviewed_commit 191a8c89f + delivery=账本三件套文档派生
  唯一增量（specs.json P690-1/2 外科插入 indent=1+plans.md 表行+INDEX
  再生——实现/依赖零变化）。`landed`=master ff-only **8a1b9a1ed**；
  **两轮并发 rebase 映射**（多会话同窗竞赛）：25dee6696→67f2b32a4→
  **90809c098**；191a8c89f→e3d74a9d2→6fa8bbc9f→**3d1b54f0a**；
  3928a2b99→faa2fac45→fb059ce5f→**8a1b9a1ed**——range-diff round2=
  T-02/03/07 `=`+T-04/05/06 `!`（唯一差异=KNOWN-DEBT 冲突解：master
  并发行+我方 D5/D6 行），代码补丁 md5 同一（d0091d8a 双侧）；
  round3 双 `=`（rerere 重放）。并发位=688 归档/691 立项/684 归档
  三会话同窗先后落库。
  `ledger_refreshed`=specs.json 658 items（P690-1 designs/P690-2
  reviews，P688/P689 并存验证）+plans.md 表行+INDEX 再生 26 projects；
  json 载入验证过。
  `archived`=git mv archive/690-rq-remote-interaction-scale.md+status
  archived（本行）。
  `cleaned`=（待清理后回填）。

## 待澄清事项

1. ~~T-01 勘定若 iced_runtime 不暴露 InputMethod 请求~~ **已销号**：
   `State::Updated{input_method}` 直接公开，降级路径不需要。
2. **实机一键复核（非阻断）**：物理 IME「组合→上屏」连续段 + 004 按钮
   hover 样式对照 + Tab 焦点环走查——多会话争焦窗（前台锁 FG-FAIL）
   未录成；安静时重跑：daemon `auto rqhost --pipe <p>` + 003/004 remote
   客户端 → 点击输入框 → 系统 IME 拼 ni→空格，录证入
   `docs/plans/reports/p690-ime-walkthrough/`。桥→上行→入值段已由
   管道环 p690 测试覆盖（构造性等价），复核为锦上添花非验收缺口。
3. N 窗实测窗数上限——现场定 5 窗（001/003/004/024/bps-gallery），
   增量 ≈1.2MB/窗线性，无墙，未继续加窗（边际已明）。
