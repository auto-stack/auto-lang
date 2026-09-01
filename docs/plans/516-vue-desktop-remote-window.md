---
plan_id: PLAN-516
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: vue-desktop-remote-window
author: [zhaopuming]
created_at: 2026-09-01
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 8
---

# [PLAN-516] vue 桌面 remote_window 集成——远程 App 进虚拟桌面

## 变更摘要

兑现 508 非目标预留的"vue 桌面深度集成"：把 Stage 6 落地的
`packages/drawlist-renderer/`（TS/Canvas2D 渲染器：codec/messages/render/
connect 四模块，Rust↔TS golden 对拍）接进 **vue 虚拟桌面宿主（465）**——
远程 App（桌面协议 WS 会话）作为**一种虚拟窗类型**进 vue 桌面的 WM
（store/layout/键盘/任务栏），而非 508 demo 页的独立窗口形态。

场景定位：**远程访问形态**——用户在浏览器/tauri 壳里打开 vue 虚拟桌面，
桌面里的某个"App"实际运行在远端（或本机另进程）的桌面宿主上，帧以
DrawList command 流过来，输入经 HitTable 回发。这是 508"远程渲染解锁"
面向用户的出口。

四块：

1. **RemoteWindow 组件**（`packages/widgets/` 的 wm 族，VirtualWindow
   同级）：canvas + drawlist-renderer connect + 会话状态面（连接中/
   在线/断线重连）。
2. **WM 集成**：remote 会话注册进 vue 桌面 store（窗口条目/任务栏图标/
   布局参与/键盘路由透传），与本地 App 虚拟窗同权混排。
3. **会话配置 v1**：桌面配置声明远程条目（url/token/目标 app）——配置
   注入形态（URL 参数或桌面配置文件），设置 UI 后置。
4. **端到端验收**：vue 桌面开一个远程窗渲染 002-counter，点击闭环
   （Playwright），且窗口受 vue 桌面 WM 管理（拖动/任务栏聚焦）。

## 目标

- **G1 组件**：`RemoteWindow.vue`（wm 族）——canvas 尺寸随虚拟窗 rect、
  DPR 处理、帧渲染、连接状态面（508 ReconnectPolicy 语义）。
- **G2 WM 同权**：远程窗进 store 窗口表（布局/任务栏/键盘路由/最小化），
  与本地窗无差别管理；断线时窗口保留+状态面可见。
- **G3 输入回发**：canvas 指针/键盘事件 → HitTable 命中 → InputMsg
  编码回发（渲染器包既有能力，接线到 vue 桌面事件路由）。
- **G4 配置**：v1 桌面配置声明远程条目（url/token/app 名），boot 时建连；
  无配置零行为变化。
- **G5 端到端**：Playwright——vue 桌面 + 远程窗渲染 002-counter，点击
  button → 远端 revision 递增 → 帧文本变化断言；拖动窗口/任务栏聚焦
  留痕。
- **非目标**：远程会话设置 UI；多远程窗性能优化；`remote_window` 的
  .at widget 登记（v1 宿主内置面——待澄清①评估升格）；TLS/跨网鉴权
  （508 边界沿用：回环+token）；移动端。

## 架构方案

```
vue 虚拟桌面宿主（465：store/layout/keyboard/Taskbar/VirtualWindow）
  └─ RemoteWindow.vue（新，wm 族）
       ├─ packages/drawlist-renderer（508：connect/renderFrame/hitTest/InputMsg）
       ├─ WS ──▶ 桌面宿主 :17800（508 listener，回环+token）
       └─ 远端 App（queue 臂 DrawList 帧，507 Tier1+2 覆盖集）
WM 集成：store 窗口表增 kind:"remote" 条目（title/icon/rect 状态同本地窗）
```

- **落点**：`packages/widgets/src/wm/RemoteWindow.vue` + store 扩展
  （remote sessions 状态切片）+ 桌面宿主入口配置消费；渲染器包零改动
  （预期——若需扩 API 走其包自身测试链）。
- **形态判定**：v1 为**宿主内置面**（与 Taskbar/dock 同级），不做 .at
  widget 登记——I4/I8 纪律针对 shell 表面与 widget 声明源，宿主能力面
  不强制同源；升格 .at widget 的条件记待澄清①。

## 技术栈

既有（drawlist-renderer 包 + vue 桌面宿主 + Playwright）。零新依赖。

## 需求分析与背景调查

（取材 508 归档交付 + 465 宿主机制 + 现场核验 2026-09-01）

- **508 交付**：`packages/drawlist-renderer/`（codec/messages/render/
  connect；Hello/Welcome/FrameReady/HitTable tag9/InputMsg 编解码；重连
  ReconnectPolicy 对齐；Rust↔TS golden 双侧对拍防漂移）+ 宿主 WS listener
  （:17800 回环+token）+ demo 页与 002-counter 远程闭环（Playwright 已有
  一条）。
- **465 宿主机制**：vue 虚拟桌面 = store/layout/keyboard/VirtualWindow/
  Taskbar（`@/wm/VirtualWindow`，aura.at:5070 登记）；桌面宿主配置/装载
  机制承载新窗口类型扩展点。
- **与 508 demo 的差异**（本计划的增量本质）：demo 页是独立单页；本期把
  远程窗**纳入 WM**——虚拟窗 rect 驱动 canvas、任务栏条目、布局混排、
  键盘路由、断线保留。
- **前置**：508 ✅（渲染器包与 listener 就绪）；507 ✅（Tier1+2 覆盖集
  即远程可渲染集）。
- **排程**：vue 线——与 509（Smithay）/513（清理）/515（DEBT 二期）
  零交叠，可并行。

## 详细设计

### 1. RemoteWindow.vue（G1）

- props：会话配置（url/token/appId）+ 虚拟窗 rect（store 注入）；
- 生命周期：mount 建连 → 帧到达渲染（canvas，DPR 缩放）→ unmount 优雅
  断连；连接状态面（spinner/重连倒计时/失败原因）覆盖在 canvas 上。
- 帧节奏：requestAnimationFrame 合帧（多帧取末帧，渲染器包若有节流则复用）。

### 2. store/WM 集成（G2）

- 窗口表条目 `kind:"remote"`（title/icon/rect/minimized 与本地窗同构）；
  布局算法零改动（rect 由 store 统一分配，RemoteWindow 消费）；
- 键盘路由：焦点在远程窗时按键转 InputMsg（KeyExited/CharTyped），本地
  窗语义不变；
- 断线：窗口保留 + 状态面"重连中"；会话终止（远端 Exit）→ 窗口关闭走
  既有销毁链。

### 3. 配置（G4）

- 桌面配置（465 装载机制）增 `remote_apps: [{id, url, token?, title,
  icon?}]`；boot 建连失败 → 窗口仍出现（状态面可见），不阻断桌面启动。
- token 来源：v1 同配置明文（回环场景），跨网安全沿 508 边界外置。

### 4. 端到端（G5）

- 环境：本机桌面宿主（WS listener）+ 002-counter（queue 臂 attach）+
  vue 桌面（dev server 或 tauri 壳）；
- Playwright：断言帧渲染（文本可见）→ 点击 → 文本变化 → 拖动窗口
  rect 变化 → 任务栏聚焦切换；复用 508 e2e 脚本基础设施
  （autoui-verifier 入口）。

## 测试设计

1. **T1 组件测**（vitest）：RemoteWindow 状态机（连接中/在线/重连/失败
   面）、rect→canvas 尺寸、配置缺省零渲染。
2. **T2 store 测**：remote 条目注册/断线保留/关闭销毁；布局混排（本地+
  远程各一）。
3. **T3 输入路由测**：焦点远程窗按键 → InputMsg 编码断言（mock 连接）。
4. **T4 Playwright 端到端**（G5 全链）。
5. **T5 回归**：无远程配置时 vue 桌面行为零变化；508 demo 页与渲染器包
   测试不回归。

## 验收标准

1. T1–T4 绿；T5 零回归。
2. 远程窗与本地窗在 vue 桌面内同权（布局/任务栏/键盘/最小化）演示留痕。
3. 断线重连状态面可见且窗口不消失（演示留痕）。
4. 无配置零行为变化（回归门禁）；渲染器包零改动或改动带其自身测试绿。
5. vue 档测试绿；零新依赖。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **store 扩展**：`packages/widgets/src/wm/`（store 所在）增 remote
   sessions 切片 + `kind:"remote"` 窗口条目 + T2 单测。
   验证：`pnpm -C packages/widgets test`（或既有 vue 测试入口）。
2. **RemoteWindow 组件**：新建 `packages/widgets/src/wm/RemoteWindow.vue`
   （canvas/connect/状态面）+ T1。
   验证：同上。
3. **WM 接线**：布局/任务栏/键盘路由消费 remote 条目 + T2/T3。
   验证：同上。
4. **配置装载**：桌面宿主入口消费 `remote_apps` 配置（465 装载机制）+
   boot 建连 + 失败降级。
   验证：无配置零变化断言（T5 部分）。
5. **Playwright 端到端**：T4 全链脚本（复用 508 e2e 基建）。
   验证：脚本绿 + 留痕。
6. **演示与断线场景**：拖动/聚焦/断线重连状态面留痕。
   验证：截图/录屏留痕。
7. **回归**：T5 全量（vue 桌面无配置行为 + 508 demo/渲染器包）。
   验证：相关套件绿。
8. **收尾**：健康检查；状态翻 execution_done。
   验证：vue 档测试绿 + `cargo check -p auto-lang`（如涉 rust 侧零改动
   则仅确认）。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **① .at widget 升格条件**：v1 宿主内置面；若 .at 应用需要声明式内嵌
  远程窗（`remote_window` widget），升格走 aura.at+registry 登记（vm 臂
  n/a/web_component 形态）——触发=出现真实声明式需求。
- **② 会话与窗口的生命周期绑定**：v1 一窗一会话；多窗共享一会话（同一
  远程桌面多窗映射）列为后续形态，需求出现再扩。
- **③ 帧节奏参数**：rAF 合帧为 v1；显式节流/背压（远端产帧快于渲染）
  以实测数据定，必要时渲染器包加 API（带其 golden 维护）。
- **④ 远端窗口语义映射**：远程会话的 Welcome.wh/rect 与 vue 虚拟窗 rect
  的协商（远端按 vue 窗 resize 跟随，v1 以 vue 侧为源）——与 512 fit
  动态重测的远端联动语义在协议文档补一行注记。
- **排程提示**：508/507 均已合入，无前置阻塞；但若 515 的 scissor 算子
  先合入，远程窗的 scroll 内容裁剪自动受益（渲染器包 golden 同步关系），
  领取顺序不设硬约束。
