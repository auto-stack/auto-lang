# ASE M0 交互会话（两活会话/真实保留/关闭代次）已验现状

> 来源：PLAN-1023（2026-11-19 交付，复审/交付身份见计划 §9，归档于
> `docs/plans/archive/1023-ase-m0-interactive-sessions-and-acceptance.md`）。
> 本文件只记录**已验证**能力与边界；未交付面一律不视为已支持。UI 侧消费
> 合同见 auto-ui
> [ase-m0-frame-consumption](../../../../auto-ui/docs/specs/runtime/ase-m0-frame-consumption.md)。

## 支持域（已验）

单原生宿主支持**两个存活应用会话**：每会话一次独立装载（真实编译+引擎挂载）
持有自己的 `AceStateEngine` 为**业务状态权威**，随会话整个生命周期常驻；
UI 显示车道只为**选中**会话打开（切换=关出向车道+开目标车道，显示从引擎
真值重提交——980 已交 reopen 语义；切换零编译、零业务初始化、零旧 handler
重复执行）。Counter A×3/B×7 经真实 typed 队列按钮事件达成，切回各保留
原值；同 app 双会话模型独立；关闭重开为新代次种子 0。

- 接线（冻结，两策略比较裁定）：`ui_apply_*` 宿主钩子定向进程全局
  current session（唯一 setter `ui_open`），双开显示车道会串写——**被拒**；
  采用"选中车道+引擎常驻"（每会话一个 `NodeDesktopBridge`/engine）。
- 事件：无参 `Inc/Dec/Reset`（981 事件 A unit 面）由普通 Node on-handler
  经受检执行服务执行，宿主只路由；带参/载荷近邻在边界具名拒（零状态/UI
  效果）；按钮目标越界具名拒。Rust 不代算 count、不手写翻译 handler。
- 状态与显示：引擎 `read_all` 为权威读回；每事件只对变化槽做差量
  `ui_apply_*` 提交（单事件=恰一个 int 槽 apply，目标/值与显示对应）；
  显示模型是只读镜像。
- 代次/上限/关闭：generation=该 app 的会话序号（每次 open 递增）；最多
  两个活会话，第三个具名 `session.limit` 拒（现有不动）；ASE 级关闭撤显示
  车道并**释放引擎 owner**；迟到事件定向旧车道 late_dropped，对关闭/重开
  后的新模型零业务/UI 效果；20 轮开/切/关受管资源收敛（loads/dispatches/
  lane_opens 精确对账，无存活泄漏）。
- 判别重放（负控已验）：最终数字相同但机制错误的路径（切回重放历史事件、
  切回重建模型+种子补数）在真实调用边界观察口（loads/ui_lane_opens/
  clicks/dispatch_log）下**判红**；数字相等不代驻留证明。

## 观察口与入口

观察口在真实调用边界计量：`loads`（装载=编译+引擎挂载）、`ui_lane_opens`
（显示车道 open）、`clicks_offered`（队列注入）、`dispatches`（step 边界
累计 + 引擎 `dispatch_log` 同源）。入口：auto-ui
`cargo test -p auto-ui-runtime --features ace-bridge --test ase_m0_sessions
-- --test-threads=1`（CPU 恒跑）＋排窗档 `AUTO_UI_ASE_M0_E2E=1`（真窗+
OS 级 resize/焦点注入，单实例）；原生验证 exe
`examples/ase_m0_sessions_window`。fixtures：本仓
`experimental/ase-m0/fixtures/counter-{a,b}/`。

## 边界

真鼠标/键盘注入与真人走查档 pending（与自动注入分账，1022 同界）；
IME/DPI 完整面、网络沙箱、A2R/Native 发行、Clock/后台任务、checkpoint、
Todo/服务/持久化不在 M0 域。多车道并发 apply 的公共切换口属 980 owner
后继（current 热路径限制已记录）。M0 正式完成以归档计划 AC-D 收口为准，
不因 A 出口自动达成。
