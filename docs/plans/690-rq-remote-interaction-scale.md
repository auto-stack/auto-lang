---
plan_id: PLAN-690
status: drafting               # drafting → executing → execution_done → reviewed → archived
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

### preedit cursor 上行（T-03）

`live_input_from_input_method` 签名保留矩形（`Option<Rectangle>`）→
`LiveInput::ImePreedit{ text, cursor }` → rqhost 填真值 → headless 注入
`Preedit(text, rect)`（App 侧 iced text_input 自绘 on-the-spot 组合串，rect
供其内部定位）。

### 光标形状（T-05 部分）

App 侧 hover 命中（iced Cursor + widget hover 语义）得出目标光标形状 →
下行 `ControlMsg::SetCursor{wid, kind}` → daemon `window::ChangeCursor`。
v1 不追求 widget 级精细——text_input/按钮两级先行，其余 default。

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

- [ ] T-01 勘定：iced_runtime `UserInterface::update` 的 window 动作暴露面
      （InputMethod 请求从何截获——iced_test 先例对照）；产出决策段回填
      §详细设计。[关联 AC-02] 验证：勘定记录 + headless 截获点代码锚定。
- [ ] T-02 IME 下行通道：wire ControlMsg::ImeRequest + headless 截获转发 +
      daemon `window::InputMethod` task。[关联 AC-01/02] 验证：环测绿
      （管道注入→task 观测行）。
- [ ] T-03 preedit cursor 上行修复：`live_input_from_input_method` 矩形保留
      + rqhost 真值 + headless 注入；typing 环 remote 变体增中文用例。
      [关联 AC-02] 验证：环测绿。
- [ ] T-04 Windows 中文 IME 实机走查（003）：组合/候选/上屏/取消录证；
      残缺项按预案混合兜底呈报裁定；D6 销号。[关联 AC-01]
- [ ] T-05 交互核验：hover 对照截图（003/004）+ Tab 焦点环 + 光标形状
      （SetCursor 通道 + 两级先行）。[关联 AC-03]
- [ ] T-06 规模化实测：帧体积/帧率观测行 + charts/gallery 代表 + N 窗内存
      曲线 → 报告 + 债登记。[关联 AC-04]
- [ ] T-07 D5 自愈臂：daemon 0x0 检测 + 恢复策略。[关联 AC-05]
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
