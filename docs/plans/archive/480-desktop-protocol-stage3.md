---
plan_id: PLAN-480
status: archived               # drafting → executing → execution_done → reviewed → archived（终态）
feature_name: 桌面协议 Stage 3——多 App 共享 host 压测 / L1·L3 形态迁移 / 内存实测验收
author: [zcode]
created_at: 2026-08-29
updated_at: 2026-08-29

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - "docs/design/autoui/desktop-protocol-v1.md: 升版 v1.1→v1.2——§1.2 Stage 3 增量节（client runtime/broker 桌面接入+多 App 驻留宿主/压测与内存报告/弹性重连/L1 换窗/L3 v2a 快照/shm 撞名修复）+ §5-§7 注记更新；merge 回写现状段"
  - "docs/specs/auto-lang/ui/overview.md: 桌面协议段 v1.1→v1.2 现状回写（真桌面壳孵化通道/多 App 共享 host/L1·L3 形态迁移/内存基线结论）"
  - "crates/auto-lang/src/ui/desktop_protocol/host.rs + session.rs: shm 段名加 pid 前缀（跨进程同名段撞名修复——Windows CreateFileMappingW 同名即打开既有段）"
new_spec_components:
  - "crates/auto-lang/src/ui/desktop_protocol/client_runtime.rs: AppProjector 投影器 v1（AuraNode→DrawList text/button+线性堆叠+命中区）+ ClientPump 协议泵（step/run 双形态）+ ReconnectPolicy 弹性重连 + L3 快照编解码"
  - "crates/auto-lang/src/ui/desktop_protocol/stage3.rs: BrokerClient 驻留多 client 宿主 + N=3/5 压测 harness（re-exec 真协议主循环）+ K32GetProcessMemoryInfo 采样 + fused_state_snapshot"
  - "crates/auto/src/cmd_autodesk.rs + crates/auto/tests/autodesk_client_smoke.rs: auto 二进制双模入口（--autodesk-client/--autodesk-incubate/--app386）+ 真二进制双进程 smoke"
  - "docs/plans/reports/480-memory-baseline.md: N=1/3/5 边际增量内存基线（Private 口径 4.81MiB/App 临界达标；WorkingSet 口径 23.17MiB/App 未达标——度量+判定形态）"
touched_goals:
  - "GOAL-009: 虚拟桌面与桌面 Shell——路线 B 桌面协议 Stage 3 收官（真桌面壳孵化通道/多 App 共享 host 压测/L1·L3 形态迁移/内存实测验收）"

affects: [auto-lang/ui]       # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 10
total_steps: 10
---

# [PLAN-480] desktop-protocol-stage3 —— 多 App 共享 host 压测 / L1·L3 形态迁移 / 内存实测验收

> **来源**：Plan 386（已归档）§0 Stage 梯次的第三段承接计划。Stage 1
> （v1.0 loopback）与 Stage 2（v1.1 两进程：命名管道/共享内存/broker/L2）
> 已在 386 内完成并归档（S1–S13 全 ✅，复审 `cargo tf` 3255/3255 绿）。
> **前置**：Stage 2 ✅（协议模块 `ui/desktop_protocol/` 接口就绪）；
> 462/463/464/465 已归档（常驻 shell/launcher 就位）。
> **与 shell-track 边界**：本计划只做**孵化通道**（broker/spawn 接入
> `auto run` 桌面模式 + client 运行时）；Shell UI 表面（dock/switcher/
> 通知中心）归 shell-track（472/478/479），不交叉。

## 变更摘要

路线 B 的收官阶段：把 Stage 2 验证过的两进程协议机制接入真桌面运行时
（`auto run -r vm` 桌面模式），完成三件事——① **多 App 共享 host 压测**
（N 个独立 app 进程经 broker 孵化进同一桌面会话）；② **内存实测验收**
（对比 1-5MB/App 目标，度量口径：增量 WorkingSet，非硬达标、出判定
报告）；③ **形态迁移 L1/L3**（同进程虚拟窗↔独立 OS 窗换窗；融合态↔
进程态的 AutoVM 快照迁移 v2a）。

## 目标

1. **真桌面壳孵化通道**：`auto run -r vm` 桌面模式作为协议宿主——broker
   线程常驻受理孵化、child 进程（`auto --autodesk-client` 双模入口）装载
   examples/ui 的 .at App 并以协议 client 循环运行。
2. **多 App 压测**：N=3/5 个 child App 同时挂同一桌面会话，可点击、帧
   随点击递增、全存活。
3. **内存实测验收**：N=1/3/5 边际增量 WorkingSet 采样报告 + 对 1-5MB/App
   目标的判定结论（度量与判定，非硬达标——Rust runtime 底噪如实呈现）。
4. **弹性**：host 断连后 child 存活等待重连，重连后 VM 状态不丢。
5. **形态迁移 L1**：同一 App 虚拟窗 ↔ 独立 OS 窗往返换窗，不重启、状态
   连续（459 多窗 daemon + 462 WM 机件拼装）。
6. **形态迁移 L3 v2a**：融合态 App → AutoVM 快照 → 孵化 child 注入恢复
   （count 类状态保留）。

## 架构方案

- **孵化通道**（Phase A）：`auto run -r vm` 桌面模式 boot 时起 broker
  serve 循环（线程；管道名 `autodesk-broker`）→ `request_incubation` 的
  child → per-app 管道 → `ProtocolHost` 绑既有 `DesktopSession`
  （resolver = 既有 app registry/编译管线）。复用 S8–S10 全部机件。
- **client 运行时**（Phase A）：`ui/desktop_protocol/client_runtime.rs` ——
  child 进程侧装载 .at App（`build_dynamic_component`）+ **通用最小投影
  器 v1**（AuraNode view → DrawList：text/button + 线性堆叠布局、button
  命中区推导）+ 协议主循环（select 输入 → VM handler → shm 产帧）。
  投影保真 v1：像素级等价非本计划目标（那是 live-iced 换接的事）。
- **双模入口**（Phase A）：`auto` 二进制三态裁决接入（adjudicate）：
  ① `--autodesk-client=<pipe>` → 协议 client 循环；③ 无标记 → 现行
  `auto run` 行为零改动（459 既有多窗能力即独立形态）。
- **内存采样**（Phase B）：Windows `GetProcessMemoryInfo` FFI（零新依赖），
  采 WorkingSet/PrivateBytes；口径 = **边际增量**（N=1→3→5 每增一个
  child 的均摊增量），排除底噪误判。
- **形态迁移**（Phase C）：L1 = `DesktopSession` 表面迁移 API（虚拟窗
  `wm_remove_win` + 新 iced 窗 `register_window`，反程同理；App/VM 不动）；
  L3 v2a = 446 已证的 AutoVM snapshot 序列化 + 孵化注入恢复（协议消息
  承载快照，机制执行期在 Hello 扩展与新消息间定夺）。

## 需求分析与背景调查

- 协议模块与全部机件就绪：`ui/desktop_protocol/` 12 文件（386 S1–S13，
  44 测试），`broker::request_incubation`/`adjudicate`/`produce_frame_shared`
  均有测试背书；规范 `docs/design/autoui/desktop-protocol-v1.md`（v1.1）。
- 依赖台账：462 虚拟窗/463 布局/464 launcher/465 vue 桌面均已归档；
  shell-track（472/478/479）持有 Shell UI 表面，与本计划边界互斥。
- Stage 3 验收原文（386 §0）：**内存实测验收**（对比 1-5MB/App 目标，
  原复活条件转为此处度量）+ 多 App 桌面压测 + L1/L3 形态迁移。
- 已知边界：① 通用投影器 v1 只保真 text/button 线性布局（像素级等价
  非目标）；② 1-5MB/App 目标受 Rust runtime 底噪影响，验收形态为
  "实测数字 + 判定结论"；③ 融合态→child 的通用产帧器保真度同 ①。

## 详细设计

### Phase A 真桌面壳孵化通道

- `DesktopSession::enable_broker(&mut self, stop: Arc<AtomicBool>)` —— 起
  serve 线程（spawn 循环 `Broker::serve_once`），孵化动作 = 复用
  `ProtocolHost` 的 ResolveAndAttach 路径（resolver = 桌面既有 app
  registry）；桌面模式 boot（renderer desktop 分支）调用。
- child 循环 = Stage 2 `dual_mode_child_body` 的产品化：装载目标 App 源
  （examples/ui 或注册表）→ `SpawnCounterSource` 通用化为
  `client_runtime::AppProjector`（AuraNode 最小投影）→ 主循环
  （输入→handler→shm 产帧→L2 处理）。

### Phase B 压测与内存实测

- `stage3.rs` harness：spawn N child（re-exec 或 `auto --autodesk-client`
  真入口）→ N App 同时 Active → 每 App 点击 → 帧递增断言 → 全存活。
- 采样：child 侧自报（Observe 通道 Metric 已有）+ 父侧
  `GetProcessMemoryInfo` FFI 双口径；报告落
  `docs/plans/reports/480-memory-baseline.md`。

### Phase C 形态迁移

- L1：`DesktopSession::detach_surface_to_os_window(app)` /
  `attach_surface_back(app)`——表面迁移不改 App/VM；459 的多窗
  `run_dynamic_iced_multi` 路径承接独立 OS 窗渲染。
- L3 v2a：snapshot 载荷协议化（`Hello` 扩展字段或独立消息，T8 定），
  child 端 `VmBridge` 恢复后按 revision 续跑。

## 测试设计

- 孵化通道集成：broker（真桌面模式线程）+ `request_incubation` 双端
  连通 + 真实 462 会话落地（S10 测试模式扩展至 `auto run` 形态）。
- 投影器：AuraNode→DrawList 快照测试（button/text/嵌套线性布局）。
- 压测：N=3/5 child 全 Active、逐 App 点击帧递增、30s 稳定存活。
- 内存：采样函数单测（合理区间）+ N=1/3/5 报告数字生成。
- 弹性：杀 host 端 → child 等待重连不退出 → 重连后 count 连续。
- L1：换窗往返后 `read_state("count")` 连续 + 窗登记翻转断言。
- L3：迁移前后 `count` 一致 + revision 延续。

## 验收标准

1. `auto run -r vm` 桌面模式下 broker 孵化通道集成测试绿（T3）。
2. N=3/5 多 App child 压测稳定（全 Active、逐 App 帧递增、全存活，T4）。
3. 内存实测报告入库（T6）：N=1/3/5 边际增量 WorkingSet 数字 + 对
   1-5MB/App 目标的明示判定结论。
4. L1 换窗往返状态连续（T8）。
5. L3 v2a 快照迁移状态保留（T9）。
6. 全量门禁：`cargo tf` 绿 + `cargo nextest … desktop_protocol
   --features ui-iced` 全绿（折入前）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加
[✅ 已完成] 一行证据。执行 worktree：`.worktrees/plan-480-dev`，分支
`plan-480-dev`；全部代码 `#[cfg(feature = "ui-iced")]`，验证一律
`--features ui-iced`。）

- [✅ 已完成] S1 通用 client 运行时：新建
  `crates/auto-lang/src/ui/desktop_protocol/client_runtime.rs` ——
  `AppProjector`（AuraNode view → DrawList：text/button + 线性堆叠，
  button 命中区推导）+ `ClientPump` 主循环（输入→`on_with_input`→
  shm 产帧→L2 处理；Stage 2 `dual_mode_child_body` 产品化）。
  验证: `cargo nextest run -p auto-lang --lib --features ui-iced
  desktop_protocol::client_runtime`（投影快照 + 循环单测）——5/5 绿
  （cdb3ce86a；投影 prop text/label+FStr 插值；泵形态 step/run 双轨）
- [✅ 已完成] S2 双模入口接入 `auto`：`crates/auto/src/main.rs` Run 分支加
  `--autodesk-client=<pipe>` + `--app386=<name>`（装载
  `examples/ui/<name>/src/front/app.at`）→ 走 `client_runtime` 循环；
  ③ 无标记行为零改动。验证: 双进程 smoke（spawn `auto
  --autodesk-client` + 测试侧 host）——`cargo nextest -p auto --test
  autodesk_client_smoke` 绿（孵化/点击帧递增/L2Detach 出口/宿主回收；
  顺带修复 ClientPump::run recv_wait 弹出丢 Welcome 缺陷）；`--app386`
  之外另接 `--autodesk-incubate`② broker 孵化入口（S4 用）
- [✅ 已完成] S3 桌面模式 broker 接入：`ui/session.rs` 增
  `DesktopSession::enable_broker()`（serve 线程 + 孵化落
  `ProtocolHost` 路径）；`ui/iced/renderer.rs` desktop boot 分支调用。
  验证: 集成单测（`enable_broker` + `request_incubation` 双端连通 +
  462 会话落地）：`cargo nextest … desktop_protocol` ——50/50 绿
  （c18252b8a；serve 线程搬运端点、属主线程 attach 落地；
  ServiceTick 周期消费孵化排队）
- [✅ 已完成] S4 多 App 压测 harness：新建
  `crates/auto-lang/src/ui/desktop_protocol/stage3.rs` —— N=3/5 child
  spawn → broker 孵化 → 全 Active → 逐 App 点击帧递增 → 全存活。
  验证: `cargo nextest run … desktop_protocol::stage3`（N=3 先绿，
  N=5 压测）——两者均 30.3s 绿（7ee391875；附赠 BrokerClient
  驻留多 client 宿主——S2 预留的"多 App 并发"兑现；53/53 desktop_protocol
  全绿）
- [✅ 已完成] S5 内存采样 instrumentation：`stage3.rs` 增
  `GetProcessMemoryInfo` FFI（WorkingSet/PrivateBytes，零新依赖）+
  harness 集成（N=1/3/5 每阶段采样）。验证: 采样单测（数值 >0 且
  N=5 > N=1）——`stage3_memory_baseline_n1_3_5` 绿（8f14f1d3e；
  实测边际增量 WS 23.1MiB/App、Private 4.8MiB/App）
- [✅ 已完成] S6 内存实测报告：新建
  `docs/plans/reports/480-memory-baseline.md` —— N=1/3/5 边际增量
  WorkingSet 表 + 对 1-5MB/App 目标的明示判定结论（达标/未达标 +
  底噪归因）。验证: 文档在库，数字与 S5 输出一致
  （复跑差 <1%；Private 口径 4.81MiB/App 临界达标，WS 口径 23.17MiB/App
  未达标 + 三点归因）
- [✅ 已完成] S7 弹性重连：child 循环增 host 断连（EOF）→ 等待重连（状态
  保持，不退出）→ 重连后 VM 状态继续。验证: 单测（server drop →
  child 存活 → 重建管道 → count 连续）：`cargo nextest …
  desktop_protocol::stage3` —— `stage3_reconnect_state_continuous` 绿
  （82aa1db65；循环机件为 S1 ClientPump 的 ReconnectPolicy，本步落
  验收单测：EOF 存活 50 拍 → 同名管道重连 → count=2/rev=3 连续）
- [✅ 已完成] S8 L1 同进程换窗：`ui/session.rs` 增
  `detach_surface_to_os_window(app)` / `attach_surface_back(app)`
  （`wm_remove_win` ↔ 新 iced 窗 `register_window`；App/VM 不动；
  459 `run_dynamic_iced_multi` 路径承接）。验证: 单测（往返后
  `read_state("count")` 连续 + 窗登记翻转）：`cargo nextest …
  desktop_protocol` + `ui::session` ——l1_surface_transfer 绿
  （1699ff410；39/39 + 55/55）
- [✅ 已完成] S9 L3 v2a 快照迁移：snapshot 载荷协议化（实现期定 =
  `ControlMsg::StateSnapshot` 追加式新消息，tag 11）+ 融合态→child 注入
  恢复（read_all_state/write_state 状态机件复用）。验证: 单测（迁移前后
  `count` 一致 + revision 延续）——`stage3_l3_v2a_snapshot_migration` 绿
  （993d7e288；composed count:42 注入同步 + 点击 43 + rev 41+1=42；
  附赠修复 shm 段名跨进程撞名——`autodesk-shm-{surface}` 加 pid 前缀，
  压测暴露的真多宿主缺陷；56/56 两轮全绿）
- [✅ 已完成] S10 收尾：协议文档升版 v1.2（Stage 3 增量：client runtime/
  broker 接入/内存报告/L1/L3）+ scoped 全绿（`cargo nextest …
  desktop_protocol --features ui-iced` + `ui::session`）。
  验证: 文档在库 + 上述命令全绿
  （56/56 desktop_protocol + 39/39 ui::session；版本表/§1.2 增量节/
  §5-§7 注记更新）

## 复审记录

- **复审人**：zcode（/auto-plan:review，2026-08-29）
- **基点**：分支 `plan-480-dev`（基 bcd597baa，10 提交 cdb3ce86a…07f7bf767，14 文件 +2889/−7，全部在计划范围内）

### 逐条验收（verify, don't trust——全部当场重跑）

| # | 验收标准 | 判定 | 证据 |
|---|---|---|---|
| 1 | broker 孵化通道集成测试绿 | ✅ | `broker::tests::enable_broker_incubates_into_real_session`（desktop_protocol 56/56 内） |
| 2 | N=3/5 压测稳定（全 Active/逐 App 帧递增/全存活） | ✅ | `stage3_multi_app_stress_n3`/`n5` 各 30.3s 绿（56/56 内，本次复审重跑） |
| 3 | 内存实测报告入库 + 数字与 S5 一致 | ✅ | `docs/plans/reports/480-memory-baseline.md` 在库；两次采样运行差 <1%，报告 §2 与 `stage3_memory_baseline_n1_3_5` 输出一致（Private 边际 4.81MiB/App、WS 23.17MiB/App） |
| 4 | L1 换窗往返状态连续 + 登记翻转 | ✅ | `ui::session::tests::l1_surface_transfer_round_trip_state_continuous`（39/39 内，重跑） |
| 5 | L3 v2a 快照迁移状态保留 + revision 延续 | ✅ | `stage3_l3_v2a_snapshot_migration`（56/56 内，重跑；线格式往返恒等 + count 42→43 + rev 41+1=42） |
| 6 | 全量门禁 | ✅ | **`cargo tf` 3255/3255 全绿**（本工作树，含 1M churn 档）；scoped：desktop_protocol 56/56、ui::session 39/39、`cargo nextest -p auto --test autodesk_client_smoke` 1/1；`cargo check`（auto-lang/auto）零 error |

### 遗漏 / 延后 / workaround 扫描

- **遗漏**：无——S1–S10 每步均有对应 diff 落点（14 文件逐一核对）；`--autodesk-incubate`②路径、`__test_open_desktop` 助手为计划内验证所需增补，均已记入步骤证据行。
- **延后**：三项均计划授权或报告明示——①投影器 v1 保真边界（text/button+线性堆叠；待澄清②原文）；②L3 载荷形态"实现期定"→选独立消息 `StateSnapshot`（tag 11 追加式）；③重连机件随 S1 循环一次成型、S7 落验收单测（行为与验证完整交付，S7 证据行注明）。无未经批准的缩水。
- **workaround**：无 TODO/FIXME/HACK 残留（diff 扫描零命中）。执行期发现并修复两个**真缺陷**（非 workaround）：`ClientPump::run` recv_wait 丢弃消息（S2 smoke 根因）；shm 段名跨进程撞名（并行宿主帧互覆，压测暴露）。

### 债候选（merge 时记 KNOWN-DEBT）

- **P480-R1**：`enable_broker` 停止旗标生产路径无调用点（boot 期进程级常驻为 v1 设计，代码注释已注明；显式停机接线归桌面退出流程后续）。
- **P480-R2**：内存基线数字来自 debug 测试宿主；release + strip 复核点已在报告 §3 明示（不影响"度量+判定"验收形态）。

### 漂移注记

- 分支基点后 master 已前进 5+ 提交（481/482/483 + 语义色主题，部分触及 `ui/` 同域）——本门禁验证的是"分支基点 + 本计划改动"的全量绿；与 master 的折入与二次验证归 `/auto-plan:merge`。

### 结论

**全部 6 项验收 pass，无阻断债 → `status: reviewed`**，可进入 `/auto-plan:merge`。

## 待澄清事项

- ① **1-5MB/App 度量口径**：增量 WorkingSet（N=1→3→5 均摊）执行期
  S5 定稿；Rust runtime 底噪可能使绝对值超目标——验收形态为"实测数字 +
  判定结论"，非硬达标（386 §0 原文即"度量"语义）。
- ② **通用投影器 v1 保真边界**：text/button + 线性堆叠；复杂布局
  （grid/scroll/chart）不在本计划——像素级等价归 live-iced 换接
  （shell-track/后续）。
- ③ **L3 注入机制**（Hello 扩展 vs 独立消息）：S9 实现期定，遵循协议
  追加式演进纪律。
