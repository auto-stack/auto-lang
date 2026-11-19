# ASE M0 交互原型（双应用会话/关闭代次/增量执行已修复）现状

> **2026-11-20 修复交付（PLAN-1106）**：`AceStateEngine` 已重接 AV 增量
> 事件会话——固定源一次编译、每实例独立 cell 状态根、每个被接纳事件恰
> 一次目标 handler 真实派发、读取/切换零派发、卸载释放状态根。执行层
> oracle（`SessionAudit`/`EngineExecutionAudit`，真实调用边界计量）与
> 同 owner 隐蔽重放/重建/getter 派发负控已闭合 M0-SESSION 无重放子门
> （`D-ASE-1023-REPLAY` closed）。能力合同见
> [组件事件会话](../ace-migration/component-event-sessions.md)。
> 下文 2026-11-19 更正保留为历史（其对旧观察口盲区的描述在旧道上成立）。

> **2026-11-19 验收更正（历史保留）**：源码核查确认`AceStateEngine::dispatch_impl→refresh`仍汇总历史ops并重算整个场景。撤回本页原先将对象常驻/外层计数解释为“每个新事件只执行一次、无历史重放”的结论；[PLAN-1106](../../plans/archive/1106-ase-incremental-event-sessions-and-replay-audit-repair.md)承接，`D-ASE-1023-REPLAY`为OPEN。1023仍archived，旧行为/窗口证据保留；M0-SESSION无重放子门与严格完整M0完成声明恢复pending。本次只纠正已观察事实，未实现或运行修复。

> 来源：PLAN-1023（2026-11-19 交付，复审/交付身份见计划 §9，归档于
> `docs/plans/archive/1023-ase-m0-interactive-sessions-and-acceptance.md`）；
> 增量修复来源：PLAN-1106（2026-11-20 交付，归档于
> `docs/plans/archive/1106-ase-incremental-event-sessions-and-replay-audit-repair.md`）。
> 本文件只记录**已验证**能力与边界；未交付面一律不视为已支持。UI 侧消费
> 合同见 auto-ui
> [ase-m0-frame-consumption](../../../../auto-ui/docs/specs/runtime/ase-m0-frame-consumption.md)。

## 支持域（已验）

单原生宿主支持**两个存活应用/引擎对象**：每个对象在ASE外层独立装载，
持有自己的 `AceStateEngine`（1106 起：一次编译的事件程序 + 每实例独立
增量会话——真实程序/状态根驻留）。
UI 显示车道只为**选中**会话打开（切换=关出向车道+开目标车道，显示从引擎
真值重提交——980 已交reopen语义）。已验切换不重做ASE外层装载、不重编/
重初始化/不重放历史事件、Counter A×3/B×7经typed队列得到正确值、切回显示
各原值、同app行为隔离，关闭重开新代次为0。

- 接线（冻结，两策略比较裁定）：`ui_apply_*` 宿主钩子定向进程全局
  current session（唯一 setter `ui_open`），双开显示车道会串写——**被拒**；
  采用“选中车道+引擎对象保留”（每会话一个`NodeDesktopBridge`/engine）。
- 事件：无参 `Inc/Dec/Reset`（981 事件 A unit 面）由普通 Node on-handler
  经受检执行服务执行，**每事件恰一次 handler 真实派发**（1106 修复）；
  M0 fixture的非法带参/载荷近邻在边界具名拒（零状态/UI
  效果）；按钮目标越界具名拒。Rust 不代算 count、不手写翻译 handler。
- 状态与显示：引擎`read_all`读回会话真值（cell 纯观察零派发）；每事件只
  对变化槽做差量 `ui_apply_*` 提交（单事件=恰一个 int 槽 apply，目标/值
  与显示对应）；显示模型是只读镜像。
- 代次/上限/关闭：generation=该 app 的会话序号（每次 open 递增；每会话
  init 恰一次）；最多两个活会话，第三个具名 `session.limit` 拒（现有
  不动）；ASE 级关闭撤显示车道并**释放引擎 owner 与状态根**；迟到事件
  定向旧车道 late_dropped，对关闭/重开后的新模型零业务/UI 效果；20 轮
  开/切/关受管资源收敛（loads/dispatches/lane_opens 精确对账 + 会话
  init/链接/编译计数恒定，无存活泄漏）。
- 负控覆盖（1106 升级）：外部重新装载/重新提交历史事件仍由外层计数识别
  （1023 旧负控保留）；同 owner 内隐蔽重放/每事件重建/getter 派发由
  **执行层 oracle** 识别（`EngineExecutionAudit` 不变量 events==handlers、
  replay/rebuild 计数）——数字/外层计数相同不能蒙混。

## 观察口与入口

外层观察口（`loads`/`ui_lane_opens`/`clicks_offered`/`dispatches`）保留
为装载/车道/注入边界计量；**执行层观察口**（1106）：会话
`SessionAudit`（handler_entries/entry_seqs=执行器进程序号/init_runs/
links/compile_count）与桥级 `EngineExecutionAudit`（含跨 unmount 折入，
增量不变量生产道全程可对账）。
现有入口：auto-ui
`cargo test -p auto-ui-runtime --features ace-bridge --test ase_m0_sessions
-- --test-threads=1`（CPU 恒跑）＋排窗档 `AUTO_UI_ASE_M0_E2E=1`（真窗+
OS 级 resize/焦点注入，单实例）；增量会话/负控目标
`--test component_event_sessions_1106`；原生验证 exe
`examples/ase_m0_sessions_window`。fixtures：本仓
`experimental/ase-m0/fixtures/counter-{a,b}/`。

## 边界

真鼠标/键盘注入与真人走查档 pending（与自动注入分账，1022 同界）；
IME/DPI 完整面、网络沙箱、A2R/Native 发行、Clock/后台任务、checkpoint、
Todo/服务/持久化不在 M0 域。多车道并发 apply 的公共切换口属 980 owner
后继（current热路径限制已记录）。1023 AC-D 的“M0 正式完成”记录仅保留
历史；M0-SESSION 无重放子门已由 1106 执行层证据恢复，完整 M0 声明以
路线图（ase-m0-delivery-roadmap）当前状态为准。旧对象/显示车道关闭证据
保留；状态根/资源释放现由会话 close/Drop 计数与 20 轮对账支持。

