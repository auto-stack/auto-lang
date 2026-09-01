---
plan_id: PLAN-508
status: executing              # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-protocol-stage6-remote-policy
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-09-01

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 2
total_steps: 9
---

# [PLAN-508] 桌面协议 Stage 6——默认策略裁定 + 远程 command 流消费（RenderQueue 线收官）

## 变更摘要

RenderQueue/分离渲染线的**收官计划**（Stage 1–5 已落：协议五通道 → 两进程 →
真桌面壳 → 并行渲染模式 → 全 widget 覆盖）。两块：

1. **默认策略裁定（进程模型）**：桌面孵化 App 的默认形态——进程内直挂
   （现状）vs 进程外 attach（queue/independent，隔离 + 统一 RenderQueue
   路径 + 4.81MiB/App 内存模型）。以**实测数据裁定**：配置位
   `shell.apps.process_model: inproc|outproc` 先行，跑对比（启动延迟/
   稳态内存/交互延迟，480 基线工具复用），裁定成文 + 分阶段翻转路径
   （可能维持 inproc 默认——以数据定，不预设结论）。
2. **远程 command 流消费（web 端）**：宿主增 **WebSocket transport**
   （Transport trait 第五实现）+ 远程会话接纳（v1 回环 + token）；新
   `packages/drawlist-renderer/`（TS/Canvas2D：Quad=fillRect/圆角、
   Text=`fillText`——**D1 定案"字符串+样式、宿主侧 shaping"的 web 同义
   实现**，零 glyph 协议工作；布局词汇 = `ui/style::BoxLayout` 同款
   tailwind 子集）；输入回发走 D3 交互区表。端到端验收：**浏览器远程渲染
   002-counter 并点击闭环**——远程渲染的解锁演示。

设计文档 §1.3 分期表随本期收尾（Stage 6 落地注记 + RenderQueue 线终态
小结）。

## 目标

- **G1 进程模型配置位**：`shell.apps.process_model`（缺省 inproc=现状零
  变化）；outproc 时孵化走 broker/双模链（既有）。
- **G2 对比实测与裁定**：同一示例批两形态实测三指标 → 裁定文档（含分阶段
  翻转或维持的路径与触发条件）。
- **G3 WS transport**：`Transport` trait 新实现（内部线程桥接同步接口，
  codec 帧承载 WS binary 消息）——loopback/pipe 同族单测覆盖。
- **G4 远程会话**：宿主 WS 监听（v1 `127.0.0.1` + 握手 token）+ 会话泵
  复用（帧/输入/控制通道既有多路语义）。
- **G5 web 渲染器 v1**：`packages/drawlist-renderer/`——DrawList →
  Canvas2D（Tier1+2 覆盖集即渲染集）；输入命中 → `InputMsg` 编码回发；
  断线重连（ReconnectPolicy 语义对齐）。
- **G6 端到端**：浏览器（Chromium/Playwright）远程渲染 002-counter：
  帧到达、点击 button → handler → revision 递增 → 新帧文本变化断言。
- **非目标**：vue 桌面 `remote_window` 深度集成（后续计划，465 宿主机制
  消费本包）；TLS/跨网鉴权（v1 回环 + token，远程安全另立）；默认翻转
  的执行（本期只裁定+配置位）；移动端。

## 架构方案

```
宿主侧                                 远程浏览器
┌ WS listener (:17800, 回环+token) ◀── WS ──┐ packages/drawlist-renderer
│ Transport 第五实现(线程桥+codec 帧)        │  DrawList → Canvas2D
│ 会话泵(帧/输入/控制, 既有五通道)           │  交互区表 → InputMsg 编码回发
└ queue 臂 DrawList (Stage5 覆盖集) ────────┘  重连(ReconnectPolicy 同语义)

进程模型: shell.apps.process_model ─ inproc(现状) | outproc(broker 孵化)
```

- **传输**：WS 消息体 = 既有二进制 codec 帧（信封不变，传输层追加式——
  §1 演进纪律）；宿主 WS 用 `tokio-tungstenite`（**RenderQueue 线首个
  新三方依赖**，理由：手写 WS 帧协议得不偿失；见待澄清①）。
- **渲染器落点**：`packages/drawlist-renderer/`（独立包，vue 桌面/任意
  web 宿主后续可消费；vite demo 页作验收载体）。

## 技术栈

`tokio-tungstenite`（新，仅宿主 WS 侧）+ TS/Canvas2D（渲染器包）。其余
既有。

## 需求分析与背景调查

（取材设计文档 §1.3/§1.3.2 定案 + 500/507 归档与在途 + 现场核验 2026-08-31）

- **D1/D3 定案的远程红利**：D1=A（字符串+样式、宿主侧 shaping）→ web 端
  `fillText` 同语义零协议工作；D3 交互区表（`Vec<(WRect,kind,action)>`）
  → web 命中判定直接复用矩形表。**远程可行性由 Stage 4 的决策直接铺好**。
- **Transport 现状**：trait 四方法同步接口（send/try_recv/pending/
  is_eof），loopback/命名管道两实现——WS 实现为第三实现，线程桥接同
  pipe 的阻塞语义。
- **内存/压测工具**：480 的 N=1/3/5 压测与 `K32GetProcessMemoryInfo`
  采样管线可直接复用于 G2 对比。
- **排程**：507（Stage 5）drafting 待领——**G5 渲染集 = 507 的 Tier1+2
  覆盖集**，故远程部分的开工前置 = 507 合入；G1/G2（进程模型）无前置可
  先行。建议排程：本计划可在 507 之后整领，或拆先行（G1/G2 先）——按
  泳道空闲定，待澄清④。
- 499/502（charts/diagram）与本期零交叠。

## 详细设计

### 1. 进程模型配置位与孵化分支（G1）

- storage 键 `shell.apps.process_model`（缺省 `inproc`）；boot 读入；
  outproc 时 App 孵化改走 broker `request_incubation` 链（既有），shell
  特权面（dock/switcher/settings/desktop 本体）恒 inproc。
- I3 纪律：同一 App 装载管线两形态共用，差异仅在进程归属与帧通道。

### 2. 对比实测与裁定（G2）

- 批次：001–005 + 一个中型示例（如 009-article-feed）× 两形态；
- 指标：冷启动到首帧、稳态内存（Private 口径，480 方法）、交互往返延迟
  （点击→帧更新）；N=1/3/5 递增；
- 产出：`docs/plans/reports/508-process-model-verdict.md`（数据表 + 裁定
  + 翻转路径/触发条件或维持理由）。

### 3. WS transport（G3/G4）

- `transport.rs` 增 `WsTransport`（accept 端）：tokio-tungstenite +
  内部线程 + 有界通道桥接同步 trait；codec 帧为 WS binary 载荷；
- 宿主 `Wsws listener`（端口 `:17800` 常量入协议文档偏差表）：握手 =
  首条 `Hello` + token 头校验（v1 静态 token 配置键 `shell.remote.token`，
  缺省拒绝）；会话泵与 pipe 会话同代码路径（帧/输入/控制多路复用不变）。

### 4. drawlist-renderer 包（G5）

- `packages/drawlist-renderer/`：`renderFrame(ctx, drawList, boxLayout)`
  （Quad/圆角/清屏、Text 字符串+样式）、`hitTest(rectTable, x, y)`、
  `connect(url, token)`（重连预算/退避，ReconnectPolicy 对齐）+
  `InputMsg` 二进制编码（与 Rust codec 同 tag 表——TS 侧生成或镜像常量，
  执行期定，倾向代码生成防漂移）；
- demo 页：`examples/remote/viewer/`（vite 单页：连宿主 → 渲染 → 点击
  回发），Playwright 断言用。

### 5. 端到端与收尾（G6）

- 002-counter 全链：spawn（outproc queue）→ WS 会话 → 浏览器帧渲染 →
  点击 → revision 递增 → 帧文本变化；
- 设计文档 §1.3 分期表 Stage 6 注记 + RenderQueue 线终态小结（一段）。

## 测试设计

1. **T1 transport 单测**：WsTransport round-trip/golden bytes/EOF（loopback
   同族）；token 拒收路径。
2. **T2 会话集成**：WS 会话孵化 → 帧到达 → 输入回发 → handler 闭环
   （Rust 侧 headless 断言，不经浏览器）。
3. **T3 渲染器单测**（vitest）：Quad/Text 渲染快照（canvas mock 或像素
   断言）、hitTest 表驱动、InputMsg 编码字节断言（与 Rust codec 对拍）。
4. **T4 Playwright 端到端**：demo 页连宿主 → 002-counter 点击闭环
   （autoui-verifier 脚本入口复用）。
5. **T5 对比实测**：G2 数据表成文。
6. **T6 实机**：浏览器真窗口演示 + outproc 模式桌面跑示例批。

## 验收标准

1. G6 端到端绿（Playwright 断言留痕）；G2 裁定文档成文。
2. T1–T3 绿；配置位缺省 inproc 时**全桌面行为零变化**（回归门禁）。
3. 协议演进纪律：`PROTOCOL_VERSION` 仍 1；既有 pipe/loopback 会话零改动。
4. 设计文档分期表收尾注记 + 终态小结成文；`cargo t ui`、
   `desktop_protocol` 套件不回归；零警告。
5. 新依赖仅 `tokio-tungstenite`（宿主 WS），理由注记待澄清①。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **进程模型配置位**：`crates/auto-lang/src/ui/session.rs` boot 读
   `shell.apps.process_model` + 孵化分支（outproc→broker 链）+ 单测。
   验证：`cargo t session`。
   [✅ 已完成] commit a84dee048（worktree）：ProcessModel 枚举+from_storage、
   DesktopState.process_model/outproc_spawner/outproc_children、launch_app
   outproc 臂（re-exec spawn→broker 受理→同步泵 attach→registry_id 回填）、
   renderer boot 读键+resolver 目录名兜底；`cargo t session --features
   ui-iced` 83/83 绿（含 process_model_storage_parse /
   launch_app_outproc_lands_window 真子进程孵化落地 0.13s /
   launch_outproc_child_body）。
2. **对比实测与裁定**：跑两形态三指标批次 →
   `docs/plans/reports/508-process-model-verdict.md` 成文。
   验证：报告含数据表+裁定。
   [✅ 已完成] worktree commits 5be10ddf8/b28e57be9(合并 master 同步)/
   裁定报告落库：harness p508_g2_inproc_arm/p508_g2_outproc_arm（两轮
   复跑内存差 <1%）；裁定=维持 inproc 缺省、outproc 保留隔离选项；
   总边际 0.86 vs 7.64MiB/App、启动 2–17ms vs 25–250ms、交互 0.07 vs
   1.54ms；009 未覆盖降级像素臂 +210MiB（507 前置实证）；翻转三闸
   T-覆盖/T-稳定性/T-远程。
3. **WS transport**：`crates/auto-lang/src/ui/desktop_protocol/transport.rs`
   增 WsTransport（tokio-tungstenite dep 入 `crates/auto-lang/Cargo.toml`）
   + T1 单测。
   验证：`cargo t desktop_protocol --features ui-iced`。
4. **远程会话接纳**：`:17800` 监听 + token 校验 + 会话泵复用 + T2 集成。
   验证：`cargo t desktop_protocol --features ui-iced`。
5. **渲染器包骨架**：`packages/drawlist-renderer/`（renderFrame/hitTest/
   connect/InputMsg 编码）+ T3 单测。
   验证：`pnpm -C packages/drawlist-renderer test`（或既有前端测试入口）。
6. **demo 页**：`examples/remote/viewer/`（vite 单页）连宿主渲染。
   验证：手动起宿主+demo 页冒烟截图。
7. **Playwright 端到端**：002-counter 远程闭环脚本（autoui-verifier 入口）
   + T4。
   验证：脚本绿 + 留痕。
8. **实机演示 + 文档收尾**：T6 清单；设计文档 §1.3 分期表 Stage 6 注记 +
   RenderQueue 终态小结。
   验证：文档 diff + 实机留痕。
9. **收尾**：健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **⑤（执行期 2026-09-01）远程部分阻塞**：步骤 3–8（WS transport/远程
  会话/渲染器包/demo/Playwright）按④前置注记须 507（Stage 5 覆盖）
  先合入；当前 507 仍 drafting 未领（阶段1 已按分期纪律折叠 master：
  4510f3de2）。**处置**：G1/G2 已收口；远程部分等 507 合入后本计划
  续领（worktree `.worktrees/plan-508-dev` 与分支保留），或用户裁定
  改拆先行折叠/另立计划。
  **[已解除 2026-09-01]** 507 合入归档（32b7b48e6 + 沉淀 ced770bc4：
  Tier1+2 全量 69/388 + 漂移闸门 + 门禁 ui-iced 化）——远程部分续领。
- **⑥（执行期 2026-09-01）存量破坏通报**：`crates/auto-lang/tests/
  osconfig_integration.rs:220` 在 `--features ui-iced` 编译下缺
  LaunchSpec `fit`/`name` 字段（Plan 504 引入时遗留，非本计划改动；
  日常门禁不含该测试目标故未拦截）。候选入 KNOWN-DEBT（复审时定）。
- **① 新依赖**：`tokio-tungstenite` 为 RenderQueue 线首个新三方——理由
  （手写 WS 帧协议的工程量/风险不划算）已记；若复审不接受，退路 = 纯
  std 的最小 WS 服务端（HTTP 升级 + 帧解析，仅 server 侧子集）。
- **② InputMsg TS 编码**：代码生成（Rust 常量导出 → TS 镜像脚本）vs 手工
  镜像常量——倾向生成防漂移，T5 定；对拍测试兜底无论哪条路。
- **③ 端口与安全**：`:17800` 回环 v1；token 静态配置。远程跨网/TLS 另立
  计划，本计划文档明示边界。
- **④ 排程**：G5 渲染集依赖 507（Tier1+2 覆盖）；若泳道紧张可拆先行
  （G1/G2 先行折叠，远程部分等 507）——默认整领，前置注记写明
  "远程任务开工前 507 须已合入"。
- **裁定倾向声明**：G2 不预设结论——若 outproc 在交互延迟上显著劣化，
  维持 inproc 默认、outproc 作为隔离选项即为合法终态。
