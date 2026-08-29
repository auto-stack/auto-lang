---
plan_id: PLAN-386
status: executing
feature_name: AutoUI RenderQueue / 分离渲染架构 Stage 1——桌面协议 loopback（五通道同进程走通）
author: [zcode]
created_at: 2026-08-28T00:00:00+08:00
updated_at: 2026-08-29T17:10:00+08:00

supersedes_spec_components: []
new_spec_components: []
touched_goals: []

current_step: 10
total_steps: 14
---

# Plan 386: AutoUI RenderQueue / 分离渲染架构（路线 B：进程外 App 与桌面协议）

> **状态**：🔄 **复活（2026-08-28，用户裁定）。Stage 1 即刻可开工**；Stage 2/3
> 按新前置依赖梯次解锁（见 §0）。
> **来源**：从 Plan 365 W5 独立出来。Plan 365 的 Host ①/②（in-process）已
> 完成；RenderQueue 是 Host ③（AutoOS 愿景），不影响 COSMIC 兼容性。
> **定位更新（2026-08-26，Design 23 / Plan 452）**：RenderCommand 重定位为
> 虚拟桌面 **AppWindow 接缝的渲染叶子后端（路线 B）**——宿主 = Plan 455 的
> 桌面进程（Win/Mac 单 OS 窗口、单 surface 合成虚拟窗口，不再每 App 一个
> OS 窗口，见 Design 23 R2/R7）。启动条件已改挂虚拟桌面（见下节更新与
> `docs/plans/autos-desktop-program.md` 仪表盘）。Stage 1→2→3 分期不变。
> **2026-08-28 复活进展注记**：①复活条件 I1 接缝 ✅（评审报告
> `reports/462-i1-seam-review.md`——view 侧零删除替换可达，462 接缝 v1 就位）；
> ②**协议蓝图已备**：Design 25 §7/§7.1（`autoui/autoshell.md`）——桌面协议五
> 通道（孵化握手/帧/输入/控制/观测）即 Stage 2"两进程"的施工图，双模 exe 入口
> 裁决与窗口形态迁移三层（L1 同进程换窗=459+462 机件）在案；③剩余条件：常驻
> App ≥ 3（待 463/464 落地）、进程内内存实测（建议 463 合入后即测——虚拟桌面
> 已可真实承载 N App）。届时复活 = 按图施工，无需重新调研。

## 0. 复活范围重构（2026-08-28，Design 25 §7/§7.1 = 施工图）

原复活三条件（I1 / 常驻 App ≥ 3 / 内存实测）重构为**依赖前置**，不再作为
统一门槛：

| Stage | 内容 | 前置依赖 | 交付/验收 |
|---|---|---|---|
| **Stage 1** | 桌面协议 loopback：五通道（孵化握手/帧/输入/控制/观测）在**同进程**内按协议编码走通（协议消息结构 + 序列化 + 双端状态机，frame 用内存缓冲模拟共享纹理） | 462 ✅ + Design 25 §7 蓝图 ✅（I1 ✅） | 协议规范文档（版本化）+ loopback demo：一个 App 的帧/输入/控制经协议通路渲染进 462 虚拟窗口，行为与直挂无差 |
| **Stage 2** | 两进程：spawn-client 双模 exe（`--autodesk-client=<pipe>` 入口裁决 + broker 命名管道）+ 帧 共享纹理/共享内存 + 输入 IPC 注入（E1 进程间版） | Stage 1 + 463/464 合入（常驻 shell/launcher 就位，孵化有真实消费方） | 独立 exe 双态启动（在桌面内/直接双击）+ L2 detach/attach 协议消息 |
| **Stage 3** | 多 App + 形态迁移 L1/L3：L1 同进程换窗（459+462 机件拼装）、L3 状态迁移 v2a 快照重启 | Stage 2 | **内存实测验收**（对比 1-5MB/App 目标，原复活条件转为此处度量）+ 多 App 桌面压测 |

*原"内存实测超标才做"的立论保留为 Stage 3 的**验收度量**而非启动门槛——
协议与双模 exe 的价值（隔离/发布形态/detach）独立于内存数字成立。*

## 背景

### 为什么独立出来

Plan 365（AutoUI 可插拔宿主架构）的 W5 原本包含 RenderQueue/共享内存
IPC/分离 compositor。但在 Plan 365 实施过程中明确了：

- **COSMIC 兼容不需要 RenderQueue**。Linux 上的 COSMIC 组件靠 Host ②
  （VTree → libcosmic Element，**in-process**）运行——和 COSMIC 原生组件
  （cosmic-applet、cosmic-panel 等）一样，都是各自独立的 in-process iced
  应用，直接通过 libcosmic + Wayland 与 cosmic-comp 交互。
- **RenderQueue 是 AutoOS 的内存优化**，为"100 个 app 共享 GPU"场景设计
  （当前单体架构每 app ~100MB → 分离架构目标 ~1-5MB/app）。它与 COSMIC
  兼容性正交。

因此，把 RenderQueue 从 Plan 365 独立为本计划，让 Plan 365 干净地收束为
"in-process 架构就绪"。短期目标是把 in-process 架构在 Windows（dev host）
和 Linux（libcosmic host）上都跑通；RenderQueue 是达成后的性能优化。

### 短期目标（不含本计划）

- **Windows**：in-process iced，每 app 独立渲染，mock 系统端口。已就绪
  （Plan 365 W1–W2）。只要保证新代码不影响现有逻辑即可。
- **Linux/COSMIC**：in-process libcosmic，每 app 独立渲染，真实系统端口。
  Plan 365 W3（host-libcosmic 脚手架）+ W4（ports-linux 脚手架）已就绪，
  真实实现在 Linux 环境上单独开发。

## 启动条件（Plan 365 D4）

本计划**不立即启动**。启动需同时满足：

1. **≥3 个复刻的 COSMIC app 在 Host ②（libcosmic in-process）上跑通**——
   证明 in-process 架构功能完备，分离架构是纯优化而非功能补全。
2. **测得内存/延迟预算证明需要拆分**——如果 3 个 in-process app 的总内存
   在可接受范围内（如 <1GB），分离架构的复杂度收益比不合理。

满足后，按 Plan 365 Migration 的 Stage 1→2→3 渐进推进。

> **更新（2026-08-26，Plan 452）**：条件 1 改挂虚拟桌面——"≥3 个 App 常驻
> 虚拟桌面（Plan 454/455 产物；任务栏/启动器/通知中心可计入）"；条件 2
> 保留原文；新增条件 3：454 的 AppWindow 接缝就位（Design 23 不变式 I1
> 通过一次评审）。实时状态见 `docs/plans/autos-desktop-program.md` 仪表盘。

## 工作项（启动后）

### Stage 1 — VTree → RenderCommand lowering + in-process loopback

| 项 | 内容 |
|----|------|
| 目标 | 在 in-process 内（无 IPC）把 VTree 降为 RenderCommand 流，用 loopback executor 执行，证明渲染等价性 |
| 验收 | 每个 example 的 VTree snapshot → RenderCommand 序列 golden-compared；loopback executor 渲染结果与 in-process iced 像素等价 |
| 文件 | `crates/auto-lang/src/ui/render_command.rs`（新）、`crates/auto-lang/src/ui/loopback_executor.rs`（新） |
| 关键约束 | 同一 VTree snapshot → 同一 RenderCommand 序列（可序列化 golden 比较） |

### Stage 2 — RenderQueue transport，两进程，单 app

| 项 | 内容 |
|----|------|
| 目标 | RenderQueue 共享内存传输（app 进程 → host 进程），单 app，Windows first（wgpu/winit 跨平台） |
| 验收 | 一个 example app 跑在独立进程，经 RenderQueue 发送 RenderCommand 到 host 进程渲染；opt-in |
| 文件 | `crates/autoui-host/`（新）、transport 层（Windows: `CreateFileMapping` + named events；Linux: memfd + eventfd） |
| 弹性 | app 检测 host 断连 → 等待重连；host 崩溃不杀 app（app 自有状态 + VTree，host 只持有可重建的 GPU 资源） |

### Stage 3 — 多 app 共享 host

| 项 | 内容 |
|----|------|
| 目标 | 多个 app 共享同一 host 进程（字体图集/纹理池/Pipeline 缓存集中化）— doc 20 的真正内存收益 |
| 验收 | ≥3 个 app 共享一个 host，总内存显著低于 3 × 独立进程 |
| 文件 | host 的窗口注册表（app 连接 → 窗口 → surface） |

## 设计输入（Plan 365 已记录，启动时参考）

- **Windows host 不是 compositor**——DWM 是。host 是 DWM client：winit
  多窗口进程，持有唯一 wgpu 上下文，每 app 一个 OS 窗口，执行该 app 的
  RenderCommand 流。窗口堆叠/装饰/焦点/最终合成由 Windows 负责。
  *(2026-08-26 注：本条拓扑已被 Design 23 R2 翻转——Win/Mac 宿主为单 OS
  窗口虚拟桌面，窗口语义归特权桌面 App；本条保留作历史参考。)*
- **Linux host**：同一二进制可作为 winit client 调试；AutoOS 阶段才生长
  smithay-based 真 compositor 变体（Linux-only）。
- **弹性**：host 是接受的 SPOF（与 Wayland compositor / Chrome GPU 进程
  同风险级别）→ 目标是快速无状态恢复，非消除 SPOF。详见 Plan 365 D2
  resilience requirements（host 代计数器、Full-frame 重发、watchdog 重启、
  RenderCommand 边界检查、可选按关键性分片）。
- **终极回退**：永久的 in-process 路径让 app 在无 host 可达时降级为自渲染。
- **code_editor（Plan 413）约束与反馈**：编辑器 core 产出按行稳定 id 键控的
  `EditorDrawList`（文本 run + quad + 行号文本 run），shaping 留 app 侧（光标/
  命中/换行需同步布局，不做 IPC shaping 服务）。启动 Stage 1 时：验收应纳入
  `examples/ui/041-auto-edit` 作为 golden 样例（最严苛的文本消费者）；协议需
  补三点——事件下行通道的 IME（preedit/commit/cursor rect，分离模式下 winit
  在 host 侧）、字体注册命令（app 自带字体的上传）、按行 CacheControl/
  DirtyRect。详见 Plan 413 §7。
- **可选加速（与 Plan 413 并行，不改变本计划启动条件）**：先做 editor-only 的
  `EditorDrawList` → RenderCommand golden lowering **薄切片**（纯函数、无
  transport、无 host、无 IPC，数天级工作量）——在 transport 开工前用最严苛的
  文本消费者（千级 glyph、按行高亮、IME）自下而上硬化文本协议。时序结论：
  不采用"RenderQueue 先行、auto-edit 后行"——分离架构是纯内存优化而非功能
  前置，editor 先行零丢弃（core 事件类型与 draw 契约已隔离，见 413 §3.1/§7）。

## 不在本计划范围

- COSMIC 组件复刻（cosmic-screenshot → cosmic-session → cosmic-monitor）—
  那是驱动 Host ② 真实实现的工作，与 RenderQueue 正交。
- in-process 架构的任何改动 — 那是 Plan 365（W1–W4 已完成）的范围。
- VM 后端 GUI — COSMIC 复刻是 a2r-only（Plan 364 约束）。

## 关联

- **Plan 365**：in-process 宿主架构（Host ①/②），W5 原指向本计划。
- **Design Doc 20**：分离架构的完整设计（AutoTree / RenderCommand /
  RenderQueue / Compositor）。
- **Plan 364**：a2r COSMIC 就绪——复刻 app 的语言能力前置。

## 执行步骤（Phase 1 / Stage 1：桌面协议 loopback，2026-08-29 复活执行）

前置自检（§0 表）：462 ✅ + Design 25 §7 蓝图 ✅ + I1 ✅。施工图 =
`docs/design/autoui/autoshell.md` §7/§7.1。执行 worktree =
`.worktrees/plan-386-dev`（分支 `plan-386-dev`，整计划一个）。
模块代码全部 `#[cfg(feature = "ui-iced")]`；验证一律 `--features ui-iced`
（该 feature 非 default，`cargo tf` 不覆盖本模块）。

- [✅ 已完成] **S1 协议模块骨架**（`ui/desktop_protocol/` 六文件 + mod 登记 + `PROTOCOL_VERSION=1`；`cargo check -p auto-lang --features ui-iced` 绿）：新建 `crates/auto-lang/src/ui/desktop_protocol/`
  （`mod.rs` 定义 `PROTOCOL_VERSION: u32 = 1` + 五通道总览文档），
  在 `crates/auto-lang/src/ui/mod.rs` 以
  `#[cfg(feature = "ui-iced")] pub mod desktop_protocol;` 登记。
  验证: `cargo check -p auto-lang --features ui-iced`
- [✅ 已完成] **S2 五通道消息 + 二进制编解码（TDD）**（message.rs 全变体 round-trip 5 测试 + 每通道 golden bytes + 坏 magic/版本/未知 tag/DrawOp tag 拒收；nextest 27 测试全绿）：`message.rs`（孵化握手含
  字体注册；帧 = DrawList + damage + CacheControl；输入含 IME 三变体，
  (Wid, event) 编码；控制双向含 DesktopBus 载荷；观测最小集）+
  `codec.rs`（信封 = magic "APDL" + u16 version + u8 channel + u32 len；
  逐消息 encode/decode，零新依赖）。测试：全变体 round-trip、每通道
  1 条 golden bytes、坏 magic / 版本不符 / 未知 tag 拒收。
  验证: `cargo t desktop_protocol --features ui-iced`
- [✅ 已完成] **S3 双端状态机 + loopback（TDD）**（AppEndpoint Detached→Handshaking→Active→Closing→Detached / HostEndpoint Listening⇄Active 全迁移测试 + 非法迁移 WrongState/VersionMismatch/NotActive 拒收；loopback FIFO + 线上破坏 BadMagic）：`endpoint.rs`（AppEndpoint:
  Detached→Handshaking→Active→Closing→Detached；HostEndpoint:
  Listening→Active；非法迁移返回 ProtocolError）+ `loopback.rs`
  （双向字节管道：send 侧编码成字节过线，recv 侧解码——编解码真实过线）。
  验证: 同 S2
- [✅ 已完成] **S4 HostEndpoint 绑定真实 462 DesktopSession**（incubation 测试断言 AppSession/VWinState/焦点/表面真实就位；pointer_down 走 WmState::hit_test→本地坐标；reclaim 测试断言窗/App/表面同清 + BufferRelease 通知）：`host.rs` ——
  Hello→`allocate_app` + `wm_add_win` + SurfaceStore（双缓冲槽模拟共享
  纹理）分配→Welcome；输入→`WmState::hit_test`→(Wid, InputMsg)；
  控制 Close/ExitRequest→`wm_remove_win` + App 回收（462 Close 语义）；
  FrameAck 槽位归还。
  验证: 同 S2
- [✅ 已完成] **S5 loopback demo（行为与直挂无差）**（counter_loopback_demo_parity_with_direct_mount：握手→帧 0→3 次协议点击→帧 3；state_parity/frame_parity/reclaimed 全真；零击退化路径同绿）：`demo.rs` —— 内联计数器 .at
  （`build_dynamic_component`）走协议全链：握手→帧 0（"count: 0"）→
  宿主 `hit_test` 命中→(Wid, click) 注入→VM handler 执行→帧 1
  （"count: 1"）合成进虚拟窗 surface；等价断言 = 直挂孪生组件同输入后
  `read_state` 相等 + 帧内容相等；控制 Close→虚拟窗回收；观测 Log 到达宿主。
  验证: 同 S2
- [✅ 已完成] **S6 041-auto-edit golden（413 §7 三点落位）**
  （`editor_frame.rs`：lower 纯函数 + EditorFrameSource；golden =
  打字落盘随帧 text runs 过线、IME preedit 上帧→commit 落盘推版本、
  (revision,fold_hidden) 缓存键随帧产出且版本化、协议 CharTyped 与 core
  直喂 EditorInput 无差；驱动对象 = 编辑器 core（413 §7 薄切片语义，
  041 的 .at E2E 需 live iced，归 Stage 2））：`editor_frame.rs` ——
  `EditorDrawList`→帧载荷纯函数 lowering（quads / text runs / gutter /
  caret / preedit，`revision` 随帧）；golden：编辑器 core 打字→帧含预期
  text runs；IME preedit/commit 输入事件→帧含 preedit 覆盖 / 文本落盘；
  CacheControl 以 revision + fold_hidden 为缓存键。
  验证: 同 S2
- [✅ 已完成] **S7 协议规范文档（版本化）**
  （`docs/design/autoui/desktop-protocol-v1.md`：版本表/wire format/五通道
  消息表/状态机/Stage 2 换 transport 面/偏差记录三条；登记
  `docs/design/autoui/README.md` 索引。scoped 验证 = desktop_protocol 31 绿
  + session 25 绿）：`docs/design/autoui/desktop-protocol-v1.md`
  （版本表 / 五通道消息表 / wire format / 状态机 / Stage 2 换 transport
  面 / 413 三点落位 / DrawList v1 偏差记录）+ 登记
  `docs/design/00-intro.md`；阶段 scoped 验证全绿。
  验证: `cargo check -p auto-lang --features ui-iced` &&
  `cargo t desktop_protocol --features ui-iced`

**Phase 2 立项（2026-08-29，用户裁定开工）**：前置核实通过——463/464
均已归档（`docs/plans/archive/`），常驻 shell/launcher 接线实证
（renderer 启动挂 shell；SummonLauncher 懒挂载 + Ctrl+Space 消费）。
开工依据 = §0 Stage 2 行 + autoshell §7.1 入口裁决三步。

**Phase 1 结果（2026-08-29）**：S1–S7 全部完成；Stage 1 交付物 =
协议模块 `ui/desktop_protocol/`（7 文件）+ 版本化规范文档 + loopback demo
（`counter_loopback_demo_parity_with_direct_mount`：帧/输入/控制/观测经协议
通路渲染进 462 虚拟窗口，state/frame parity 与直挂无差，Close 回收落地）。
Stage 2 前置仍待：463/464 合入（常驻 shell/launcher 就位）。

**Pre-fold 门禁（折入 master 前，流程性）**：`cargo tf` 全绿 +
`cargo t desktop_protocol --features ui-iced` 显式绿（ui-iced 非 default，
tf 不编译本模块，二者缺一不可）。

- [✅ 已完成] **S8 传输层抽象 + Windows 命名管道**
  （`transport.rs`：Transport trait（send/try_recv/pending/is_eof/recv_wait）
  + loopback 收编 + tokio named_pipe 实现（split 读写半 + 读线程
  select!{read,shutdown} + 多线程 runtime 常驻驱动 reactor）；5 测试：
  FIFO/双向往返/残帧等完整帧/对端 drop→EOF/loopback trait 同语义，
  5 连跑全绿。平台教训已沉淀于模块头注：同步句柄阻塞 ReadFile 的
  跨句柄唤醒、PeekNamedPipe 空管阻塞、他线程 Cancel 皆不可依赖——
  tokio 异步读 + select 取消是唯一可靠路径）：`transport.rs` —— 通道 trait
  （send/try_recv/pending，与 loopback 同语义；u32 长度前缀分帧）；
  loopback 收编为首个实现；named_pipe 实现 = 复用 tokio
  `net::windows::named_pipe`（autovm_daemon Plan 269 同款，零新依赖；
  `#[cfg(windows)]`，非 Windows 保持 loopback-only 可编译）。
  验证: `cargo t desktop_protocol --features ui-iced`
- [✅ 已完成] **S11 L2 detach/attach 协议消息**（先于 S9/S10 执行：纯协议
  层先行。ControlMsg 追加 L2Detach/L2Detached/L2AttachRequest（tag 8/9/10，
  演进纪律只追加）；AppState 增 Standalone；Active→L2Detach→Standalone→
  L2Detached→宿主 ReclaimWindow→connect()（Standalone 同入口）→新
  wid/surface→Active。测试断言 revision 往返不归零 = "状态未动"协议级
  证据。37 测试全绿）：`shm.rs` —— Windows FFI
  CreateFileMappingW/MapViewOfFile 双槽帧缓冲（`#[cfg(windows)]`）+
  `FrameMsg::FrameReadyShared{offset,len}` 追加（演进纪律：只追加不改义；
  大帧载荷走共享内存、控制面消息照走管道）。验证同上
- [✅ 已完成] **S10 broker + 入口裁决**
  （`broker.rs`：`adjudicate()` 三步 + `Broker::serve_once`/
  `request_incubation`（DesktopBus 同形 `incubate` 记录，per-app 管道名
  分配 + 先行 listen + 转连；空连接 ping 吞弃）。全链路测试：broker
  孵化 → ProtocolHost 绑真实 462 会话（AppSession/VWinState 落地）→
  协议点击 → VM 状态变化。transport 泛型 PipeEnd 以
  `Box<dyn Transport + Send>` 类型擦除统一签名。42 测试全绿 ×3 连跑）：`broker.rs` —— `adjudicate()` 三步
  （① `--autodesk-client=<pipe>` 孵化标记 ② 探测
  `\.\pipeutodesk-broker` ③ standalone）+ broker 侦听与孵化交接
  （分配 per-app 管道名回传）。验证同上
- [ ] **S11 L2 detach/attach 协议消息**：endpoint 追加 `L2Detach`
  （host→app：Active→Standalone，表面释放、VM 状态不动——路线 B 的
  核心红利）+ `L2AttachRequest`（app→host：重孵化握手续用既有状态）；
  协议文档记录 L2 语义。验证同上
- [ ] **S12 双模 exe + 两进程集成测试**：`examples/ui_client_demo`
  （三态裁决真 exe：client = 计数器协议循环；standalone = iced 窗，
  `ui_dual_app` 同款门控）+ 集成测试（re-exec 子进程模式：真管道 +
  共享内存 + 孵化/帧/输入/L2 全生命周期双端断言）。验证同上
- [ ] **S13 Phase 2 文档 + 收尾**：协议文档版本表追加（共享内存
  transport / broker / 新变体 / L2 语义）；scoped 验证全绿
  （desktop_protocol + session）。验证: cargo check + 上述模块测试
  ---

**Phase 2 本次增量折入（2026-08-29）**：S8/S9/S10/S11 四块（传输/
共享内存/broker/L2 协议，42 测试全绿 ×3 连跑）+ `cargo tf` 门禁通过后
折入 master；S12（双模 exe 集成）与 S13（文档升版）顺延下一 session，
折入时协议规范文档未含 S12 增量（下次补）。

**Pre-fold 门禁（Phase 2 折入 master 前）**：`cargo tf` 全绿 +
`cargo t desktop_protocol --features ui-iced` 显式绿。

## 待澄清事项

- ①（执行期决定，review 时请确认）：**帧通道载荷 v1 = `DrawList`**
  （quad + text run 最小显示列表；`EditorDrawList` 同型 lowering），
  **不含**全量 VTree→RenderCommand lowering——原"工作项 Stage 1"的
  逐 example golden lowering 已被 §0 复活重构（2026-08-28）取代；
  全量 lowering 归 Stage 2（载荷真跨进程时）。依据：`code_editor/draw.rs`
  头注"Plan 386 Stage 1 serializes it to quads and text runs"、§0
  "frame 用内存缓冲模拟共享纹理"、Design 25 §7 帧通道定位。
- ②（范围边界）：loopback demo 为 **headless 验证**（462 会话对象级，
  `DesktopSession`/`WmState` 真实参与）；live-iced 渲染器换接
  （`dynamic_view`→协议客户端）留 Stage 2 随真 transport 一起做——
  I1 评审已证零删除替换可达。
