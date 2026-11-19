# ASE M0 交互原型（双应用对象/关闭代次；增量执行待修复）现状

> **2026-11-19 验收更正**：源码核查确认`AceStateEngine::dispatch_impl→refresh`仍汇总历史ops并重算整个场景。撤回本页原先将对象常驻/外层计数解释为“每个新事件只执行一次、无历史重放”的结论；[PLAN-1106](../../plans/1106-ase-incremental-event-sessions-and-replay-audit-repair.md)承接，`D-ASE-1023-REPLAY`为OPEN。1023仍archived，旧行为/窗口证据保留；M0-SESSION无重放子门与严格完整M0完成声明恢复pending。本次只纠正已观察事实，未实现或运行修复。

> 来源：PLAN-1023（2026-11-19 交付，复审/交付身份见计划 §9，归档于
> `docs/plans/archive/1023-ase-m0-interactive-sessions-and-acceptance.md`）。
> 本文件只记录**已验证**能力与边界；未交付面一律不视为已支持。UI 侧消费
> 合同见 auto-ui
> [ase-m0-frame-consumption](../../../../auto-ui/docs/specs/runtime/ase-m0-frame-consumption.md)。

## 支持域（已验）

单原生宿主支持**两个存活应用/引擎对象**：每个对象在ASE外层独立装载，
持有自己的 `AceStateEngine`。该桥保存事件历史与最近AV场景重算结果；
对象生命周期常驻不证明底层已验程序/业务模型驻留。
UI 显示车道只为**选中**会话打开（切换=关出向车道+开目标车道，显示从引擎
最近状态重提交——980 已交reopen语义）。已验切换不重做ASE外层装载、
Counter A×3/B×7经typed队列得到正确值、切回显示各原值、同app行为隔离，
关闭重开新代次为0；这些外部行为不证明每次事件没有内部重编/初始化/重放。

- 接线（冻结，两策略比较裁定）：`ui_apply_*` 宿主钩子定向进程全局
  current session（唯一 setter `ui_open`），双开显示车道会串写——**被拒**；
  采用“选中车道+引擎对象保留”（每会话一个`NodeDesktopBridge`/engine）；真正增量执行由1106另验。
- 事件：无参 `Inc/Dec/Reset`（981 事件 A unit 面）由普通 Node on-handler
  经受检执行服务执行，**当前每次派发仍重跑累计历史场景**；M0 fixture的非法带参/载荷近邻在边界具名拒（零状态/UI
  效果）；按钮目标越界具名拒。Rust 不代算 count、不手写翻译 handler。
- 状态与显示：引擎`read_all`读回最近AV重算结果；每事件只对变化槽做差量
  `ui_apply_*` 提交（单事件=恰一个 int 槽 apply，目标/值与显示对应）；
  显示模型是只读镜像。
- 代次/上限/关闭：generation=该 app 的会话序号（每次 open 递增）；最多
  两个活会话，第三个具名 `session.limit` 拒（现有不动）；ASE 级关闭撤显示
  车道并**释放引擎 owner**；迟到事件定向旧车道 late_dropped，对关闭/重开
  后的新模型零业务/UI 效果；20 轮开/切/关受管资源收敛（loads/dispatches/
  lane_opens 精确对账，无存活泄漏）。
- 旧负控的实际覆盖：外部重新装载/重新提交历史事件、切回重建+补数可被
  `loads/ui_lane_opens/clicks/dispatch_log`识别；**同一个对象内refresh重放未被覆盖**。
  原“已证明无重放”判定不成立，不能把旧负控PASS作为1106的关闭证据。

## 观察口与入口

现有观察口在ASE/桥外层调用边界计量：`loads`（外层装载）、`ui_lane_opens`
（显示车道 open）、`clicks_offered`（队列注入）、`dispatches`（step 边界
累计＋桥`dispatch_log`同源）。它们不是实际编译/初始化/handler入口次数，
漏掉refresh内部工作；1106须在ACE实际执行层补区分同owner重放的oracle。
现有入口：auto-ui
`cargo test -p auto-ui-runtime --features ace-bridge --test ase_m0_sessions
-- --test-threads=1`（CPU 恒跑）＋排窗档 `AUTO_UI_ASE_M0_E2E=1`（真窗+
OS 级 resize/焦点注入，单实例）；原生验证 exe
`examples/ase_m0_sessions_window`。fixtures：本仓
`experimental/ase-m0/fixtures/counter-{a,b}/`。

## 边界

真鼠标/键盘注入与真人走查档 pending（与自动注入分账，1022 同界）；
IME/DPI 完整面、网络沙箱、A2R/Native 发行、Clock/后台任务、checkpoint、
Todo/服务/持久化不在 M0 域。多车道并发 apply 的公共切换口属 980 owner
后继（current热路径限制已记录）。1023 AC-D曾记M0正式完成，现仅保留历史，
不能再单靠该记录声明严格M0已闭；恢复无重放及真实模型释放子门须1106的
执行层正反控、独审与正式交付。旧对象/显示车道关闭证据保留，不自动扩展为
未来增量状态根/资源已释放的证据。
