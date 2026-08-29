# AutoOS 虚拟桌面程序总览（Program Tracker）

> **性质**：活账（living tracker）。每次计划状态头变更时同步本文件（已写入各计划
> finish-plan 步骤）。详细状态只住在各计划自己的状态头里，这里只放指针级一行。
> **架构依据**：`docs/design/autoui/virtual-desktop.md`（下称 Design 23）。
> **本文拥有的东西**：依赖图、入口条件仪表盘、裁定登记簿。计划内部进度不在此抄。

## 目标

一套窗口管理代码（WM 是 AutoUI App），在 Win/Mac 上是单 OS 窗口的虚拟桌面，
在 Linux 上是原生合成器宿主，在 Web/移动端是同构嵌入——设计级一致，
App 一次编写处处原生。里程碑 M0-M6 详见 Design 23 §6。

## 计划一览

452 已立项（2026-08-26）；453+ 编号以立项时实际分配为准。

> **当前状态**：452/453/459 已完成归档（2026-08-28，M1 收口：多 App 会话 + daemon
> 多窗口 + panic 隔离）。2026-08-28 立项 M2–M4 计划族（Design 24：
> `docs/design/autoui/desktop-shell-and-launcher.md`）——462/463/464/465；
> **462 已完成**（单 OS 窗口多虚拟窗口 + WM 最小集，实机验收 + 3222 测试全绿），
> 463/464/465 未开工。

**编号解析规则**：本表是"里程碑 ↔ 实际计划号"的**唯一事实源**。Design 23 与
各计划正文中的 453–457 字样均为**提案编号**，一律经本表解析为实际编号。
立项时若实际编号顺延（如 453 被其他工作占用、里程碑 M1 实际立项为 458），
只需三步：① 本表"计划"列改实际编号并在单元格内注记原提案号；② 本节追加
一行映射说明；③ 之后的新计划引用实际编号。**不逐处改正文里的提案编号**。

| 计划 | 里程碑 | 范围一行 | 状态 | 依赖 |
|---|---|---|---|---|
| 452 设计+裁定翻转+IME spike | M0 | 正式收编 Design 23；执行 §5 同步清单；跑 §9 spike | ✅ 完成 2026-08-26（报告 reports/452-ime-spike.md，已归档） | 无 |
| 453 多 App 会话运行时 | M1 | AppSession/DesktopSession；(AppId,·) 扇出；panic 边界；多 OS 窗口验证 | ✅ 完成 2026-08-28（随 459 验收一并归档） | 452 ✅ |
| 459 DesktopSession 多窗口化 | M1 收口 | iced daemon 迁移；AppId 递增分配与打标；双 AppSession 双窗口 demo；panic 隔离验证 | ✅ 完成 2026-08-28（C0–C4 + 实机验收，已归档） | 453 ✅ |
| 462（原提案 454）VirtualWindow + WM | M2 | 路线 A：`virtual_window` 注册 widget、chrome、事件/焦点分区、桌面宿主入口、R4 接缝 v1（`462-virtual-window-wm.md`） | ✅ 完成 2026-08-28（实机双窗 demo 全交互 + `cargo t` 3222 绿 + I2 五套 desktop_mcp 全绿；IME 两项残留转 463） | 459 ✅ |
| 463（原提案 455）桌面 shell | M3 | 全屏 borderless、shell pack+任务栏、layout 三模式+snap、`desktop.*` 命令接缝、workspace 驱动模型、桌面热键、pac.at 注册表（`463-desktop-shell-auto-arrange.md`；Design 25 §3/§4.1/§6 对齐） | ✅ 完成 2026-08-28（复审通过 + 合入归档 + specs 沉淀 P463-1..4） | 462 ✅ |
| 464 Launcher App（吸收 441） | M3 组成 | `examples/ui/028-launcher`：palette+网格双形态、模糊搜索、键盘流、真注册表+`LaunchApp`（`464-launcher-app.md`） | 已立项 2026-08-28，未开工（M1 子阶段可先行） | 462+463 |
| 465（原提案 456）Vue 虚拟桌面 | M4 | DOM 嵌入 + 多挂载宿主（`createApp`/虚拟窗）+ registry 构建期生成 + tauri 全屏壳 + E1/E2 接缝语义（`465-vue-virtual-desktop.md`） | 已立项 2026-08-28，未开工 | 462 契约（与 463/464 并行） |
| 472 shell-track M1 | M3+ | **AutoShell 地基**：状态投影协议 v1（`__wm_*` 族 schema 化）+ workspace 驱动模型补课（463 §3.6 未实施）+ dock 升级（图标/pinned/配置）（`472-shell-track-m1-projection-dock.md`） | ✅ execution_done 2026-08-29（T1–T6 全落；cargo t 3234/3234 绿；协议合同 schema/projection-protocol-v1.md；复审以 /auto-plan:review 承载） | 463 ✅ + 464 ✅ |
| shell-track M2（478） | M3+ | **switcher overlay + workspace pager**：Ctrl+Tab 召唤 MRU 面板（464 overlay 同型第二槽）+ dock 升格 pager（1 基标签/当前高亮/增删分区）+ send_to 跨区发送 + 协议 v1.1（`478-shell-track-m2-switcher-pager.md`） | 🔄 execution_done 2026-08-29（T1–T7 全落；cargo t 3236/3236 + ui-iced 档 3879/3879 绿；I2 五套 desktop_mcp 14/11/11/19/26 与 472 基线同数；T6 键流实机项注入通道受阻按 472 先例 headless 指针成文；复审以 /auto-plan:review 承载；缩略挂 386、shell IME 挂 457 不变） | 472 ✅ + 464 ✅ |
| 457 Smithay 宿主 | M5 | Linux 原生合成器宿主，复用桌面 shell | 提案中 | 462+463 |
| 386（**已复活**） | M6 | 路线 B：桌面协议五通道 + 双模 exe + 形态迁移，Stage 1-3 重构见 386 文件 §0 | 🔄 复活 2026-08-28（Stage 1 即刻可开工，前置仅 462✅+蓝图；Stage 2 待 463/464，Stage 3 含内存实测验收） | Stage1: 462✅；Stage2: +463/464 |

**编号映射（2026-08-28 立项）**：M2→**462**、M3→**463**、M4→**465**（原提案
454/455/456，正文历史提案号不回改）；新增 **464**（launcher，吸收 Plan 441，
其 palette 原语化降为 464 可选任务、vm 焦点原语改由 462 承载）。
2026-08-28 增补：**shell-track** 立项（`docs/design/25-autoshell-dsl-unified-shell.md`
——shell 表面统一为 AutoUI DSL：投影协议 + dock/switcher/pager/通知/settings/
桌面本体，I7–I9），立项时分配实际编号；463 同步转正 workspace 驱动模型与
`desktop.*` builtin 命令接缝（Design 25 §3/§6）。

## 入口条件仪表盘

定期（建议每次状态审计时）更新：

**386 复活条件（2026-08-28 复活时重构为 Stage 依赖，见 386 文件 §0）**：
- [x] I1 接缝评审 —— ✅ `reports/462-i1-seam-review.md`（view 侧零删除替换；E1/E2 扩展点挂 465 消费）
- [ ] 常驻 App ≥ 3 → **转为 Stage 2 前置**（463/464 合入即满足）
- [ ] 内存实测 → **转为 Stage 3 验收度量**（对比 1-5MB/App 目标；建议 463 合入后先测一次留基线）

**未完成计划执行顺序（2026-08-28 排定）**：
```
463 收尾合入（执行中）──┬─▶ 464 launcher ──┬─▶ shell-track M1+（立项）
386 Stage 1（即刻可开工）┤                  └─▶ 386 Stage 2 ─▶ 386 Stage 3
465 vue 虚拟桌面（并行）─┘
```

**457 启动条件**：
- [ ] 454+455 完成 —— 当前：未开始
- [ ] auto-cosmic 宿主复活评估（libcosmic 依赖决策，Linux 环境）—— 当前：未做

## 裁定登记簿

程序最大的风险不是丢任务，是旧裁定在新代码/新文档里悄悄复活。每次翻转裁定记一行，
核销所有受影响文档的同步状态。

| # | 裁定 | 旧内容（出处） | 新内容 | 变更载体 | 同步状态 |
|---|---|---|---|---|---|
| 1 | Win/Mac 宿主窗口拓扑 | 宿主非合成器，每 App 一 OS 窗口（Plan 365 archive） | 单 OS 窗口虚拟桌面 | Design 23 R2 / 452 | doc20 ✅ 386 ✅ 365注记 ✅（2026-08-26，T2–T4；提交号随本批提交后补） |
| 2 | chrome/窗口管理归属 | 宿主拥有窗口管理（doc 20 §6.1） | 特权桌面 App 拥有窗口语义，宿主只管合成 | Design 23 R1 / 452 | doc20 ✅（2026-08-26，T2） |
| 3 | RenderQueue 定位 | 独立的分离渲染内存优化（Plan 386） | R4 接缝的 RenderCommand 后端（路线 B） | Design 23 R7 / 452 | 386 ✅（2026-08-26，T4） |
| 4 | 独立窗口模式地位 | （隐含）默认且唯一 | 永久一等公民 + 退化桌面构造（R3/R6） | Design 23 / 452 | — |
| 5 | shell 组件归属 + 排布/注册表/启动/热键（**追加 R8–R12**，非翻转） | — | launcher/任务栏=特权 AutoUI App；排布=WM 纯函数策略（free/grid/master-stack+snap）；注册表=pac.at 清单；启动=会话挂载非进程孵化；桌面级热键路由优先于 App 分发 | Design 24 / 462-465 立项 | — |
| 6 | shell 层工程化（**追加 I7–I9 + DesktopBus 定案**，非翻转） | 463 T1 命令接缝两候选待定 | `desktop.*` builtin 命名空间定案（候选 A）；驱动=内核/shell=用户态（投影唯一事实 I9、shell 无几何操作 I7、表面双端同源 I8）；workspace 驱动模型转正入 463 | Design 25 / shell-track 立项 | — |

同步动作完成时把 ⬜ 改 ✅ 并附提交号。归档计划（365）不改正文，只加状态注记。

## 防腐规则

1. 单一事实源：计划详细状态只在各计划状态头；本文件只放一行指针。
2. 本文件只拥有任何单个计划都不拥有的东西：依赖图、仪表盘、裁定登记簿。
3. 452 收编 Design 23 后，架构问题先查 Design 23，本文件不重复架构论述。
4. 周期审计（plans-status-audit 体系）引用本文件，不重新推导程序状态。
