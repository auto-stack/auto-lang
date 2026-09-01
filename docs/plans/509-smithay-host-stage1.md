---
plan_id: PLAN-509
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: smithay-host-stage1
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui, auto-cosmic]
current_step: 0
total_steps: 8
---

# [PLAN-509] Smithay 宿主 Stage 1——Linux 合成器线启动（原提案 457）

> **编号解析**：Design 23 M5 / 程序总览"457 Smithay 宿主"为提案号，实际
> 立项 **509**（程序台账计划一览随本计划同步改号）。

## 变更摘要

虚拟桌面程序的最后一块拼图（Design 23 目标："在 Linux 上是原生合成器宿主，
复用桌面 shell"）。Stage 1 三件事：

1. **启动条件兑现——路线评估定案**：三路线对比裁定——
   **A** auto-cosmic `host-libcosmic` 复活（libcosmic 工具套路线）；
   **B** Smithay 合成器 + **桌面协议宿主**（宿主消费 DrawList/Pixels——
   500/507 的 queue/independent 资产直接复用，宿主可不含 iced）；
   **C** 混合（Smithay 骨架 + libcosmic 画 shell 面）。裁定标准：桌面
   shell 资产复用最大化、维护面最小、与 iced 0.14 生态距离、Linux 验证
   环境适配。
2. **最小合成器骨架**：Smithay 会话（backend/udev 或 winit 后端起评裁定）
   + 单全屏面合成，**shell 首帧上屏**（静态即可）。
3. **I1 纪律核对**：WM/session/投影代码**零分叉**——Linux 臂只是新增
   "宿主合成"后端（diff 证明或配置差异），对应 Design 23"一套窗口管理
   代码"的总目标。

**Linux 环境是硬前置**：本仓主开发环境为 Windows——执行会话必须具备
Linux 图形验证环境（WSL2+Wayland / 物理 Linux / CI Linux runner 之一，
待澄清①定）。后续 Stage（本计划只登记不实施）：xdg_shell 原生客户端窗口
管理（Linux 上原生 app 是一等公民——"原生互通"问题在 Linux 形态下自然
消解）、输入/IME（S8 shell IME UI 挂此线）、多 App/工作区、与双模 exe
合流。

## 目标

- **G1 评估定案**：三路线对比报告（复用矩阵/依赖距离/风险/环境）+ 裁定
  与理由成文（`docs/plans/reports/509-smithay-route-verdict.md`），兑现
  程序台账登记的启动条件②。
- **G2 骨架实跑**：按裁定路线落地最小合成器，Linux 环境编译 + 实跑，
  会话建立 + 合成一帧。
- **G3 shell 首帧**：既有桌面 shell 面（.at 资产）以裁定路线渲染上屏
  （静态首帧即可，无交互要求）。
- **G4 I1 核对**：`crates/auto-lang` 的 WM/session/投影/事件路由在 Linux
  臂引入下零改动（diff 证据）；shell 资产（.at）零改动复用。
- **G5 台账回写**：程序总览 457 行改号 509 + 状态更新；overview 的
  auto-cosmic 条目状态翻转（experimental → active-线内）。
- **非目标**：xdg_shell 客户端管理/输入设备/IME（Stage 2+）；性能；X11
  兼容（Wayland-first）；S8 IME UI；发布打包。

## 架构方案

```
路线 B 形态（评估若裁定之——当前倾向，因 500 已落地）:
  Smithay compositor (Linux)
    ├─ 合成循环: wl_surface 合成（含 shell 面与未来的原生客户端）
    └─ 桌面协议宿主（ui/desktop_protocol 既有 host 端复用）
         ├─ queue 臂: DrawList → 宿主栅格化（500 生产化产物）
         └─ shell 作为 attach App（双模入口既有，--autodesk-client 链）
路线 A 形态: host-libcosmic 复活——libcosmic(iced 系) 直渲 shell，
  Smithay 仅做窗口管理外挂或暂不引入
```

- **落点随裁定**：A → `crates/auto-cosmic/host-libcosmic/` 复活扩展；
  B → 新 `crates/auto-cosmic/host-smithay/`（或 ports 层旁挂）；C → 组合。
- **ports 层价值**：`ports`/`ports-linux`（D-Bus 适配带 Windows mock 回退）
  为跨平台开发保底——评估须核对两路线与 ports 层的衔接。

## 技术栈

`smithay` 与/或 `libcosmic`（**本线新三方大依赖**——评估报告承担选型论证，
入 lock 前经用户过目）；其余既有。

## 需求分析与背景调查

（取材 docs/specs/overview.md §外围实验 + Design 23/程序台账 + 现场核验 2026-08-31）

- **启动条件**（程序台账"457 启动条件"）：① 454+455 完成（=462/463，✅
  早已满足）；② auto-cosmic 宿主复活评估（libcosmic 依赖决策，Linux
  环境）——未做，**本计划 T1 兑现**。
- **auto-cosmic 现状**（核验）：四子 crate——`ports/`（抽象层）、
  `ports-linux/`（zbus D-Bus，Windows 下 fallback mock 保跨平台编译）、
  `host-libcosmic/`（宿主骨架，依赖 auto-lang ui+ui-headless，**libcosmic
  未入锁**——骨架态）、`demo/`。experimental、无消费者。
- **路线 B 的新可能性**：500 落地后宿主已能消费 DrawList 命令帧（queue
  臂生产化栅格化）与 Pixels 帧——Smithay 宿主可以**不含 iced**，把 shell
  当 attach App（桌面协议五通道 + 双模入口既有）。这使"一套 WM 代码 +
  可插宿主"从设计宣言变为机制现实，是评估的最大变量。
- **Design 23 语义**：R1/R2——WM 是特权 AutoUI App，宿主只管合成；Linux
  臂 = 合成宿主多一个后端。S8（shell IME UI）挂本线后续 Stage。
- **排程**：与 507/508（同协议线后续）的交叠 = 评估若走 B 将消费其产物
  ——**开工前置建议 = 507 合入**（覆盖集就绪）；骨架编译类任务可先行。
  503/505/506 无交叠。

## 详细设计

### 1. 路线评估（T1，报告成文）

- 对比维度（表格式）：桌面 shell 资产复用度（.at/投影/协议/Dock 等）、
  依赖引入量与 iced 0.14 生态距离、Linux 图形栈适配（Wayland session/
  udev/drm）、跨平台编译影响（Windows 主仓 dev 流不能红）、维护面、
  Stage 2+ 演进空间（xdg 客户端管理是 Smithay 强项）；
- 产出：裁定 + 依赖清单（smithay/libcosmic 版本选型）+ Stage 2+ 路线图
  初稿；**新依赖入 lock 前经用户确认**（大依赖门槛，待澄清②）。

### 2. 最小骨架（按裁定）

- Smithay 路线：session（libseat/udev 或开发期 winit 后端起手）+ 单
  全屏 surface + 渲染循环（借宿主栅格化产物画 DrawList，或 libcosmic
  直渲——随路线）；
- 跨平台保底：骨架 crate 在非 Linux 目标 cfg 隔离（auto-cosmic 既有
  fallback 模式延续），主仓 `cargo check`/`cargo t` 零影响。

### 3. shell 首帧

- 以 attach 形态拉起 shell 面（`--autodesk-client` 既有入口，桌面
  protocol loopback/pipe 均可）或直渲（A 路线），输出静态首帧截图；
- 首帧内容 = shell.at dock + 桌面背景（496 桌面本体资产直接复用）。

### 4. I1 核对

- diff 证据：`crates/auto-lang/src/ui/{session,iced,virtual_window,
  native_dock}` 与投影/事件路由零改动（或仅 cfg 配置差异行）；
- shell .at 资产零改动（复用证明）。

## 测试设计

1. **T1 评估报告**：三路线矩阵 + 裁定（复审对照物）。
2. **T2 跨平台编译**：Windows 主仓 `cargo check`/`cargo t` 不回归 +
  Linux 目标骨架编译绿（交叉或 Linux 环境）。
3. **T3 首帧验证**：Linux 环境（WSL2/Wayland 或物理）实跑截图留痕
   （像素级：dock 可见 + 背景非空）。
4. **T4 I1 diff**：核对零分叉证据链。
5. **T5 环境记录**：验证环境形态（WSL2 版本/内核/Wayland 合成方式或
   物理机）成文——后续 Stage 的环境基线。

## 验收标准

1. T1 报告成文且裁定经用户确认（大依赖门槛）。
2. T2 双平台绿；T3 首帧截图留痕。
3. T4 零分叉证据成文（auto-lang 主仓 diff 干净）。
4. T5 环境基线记录；程序台账 457→509 改号回写。
5. `cargo check -p auto-lang` 零警告；主仓日常档不回归。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **路线评估**：三路线对比矩阵 + 裁定 +
   `docs/plans/reports/509-smithay-route-verdict.md` 成文（含依赖清单，
   入 lock 前置用户确认）。
   验证：报告成文 + 用户确认记录。
2. **环境就绪**：Linux 验证环境搭建/确认（WSL2+Wayland 或物理）+
   T5 基线记录。
   验证：环境内 `rustc --version`/合成环境冒烟记录。
3. **依赖与骨架 crate**：按裁定建/复活宿主 crate（cfg 隔离保 Windows
   编译）+ 依赖引入（经确认）+ T2 双平台编译。
   验证：`cargo check -p auto-lang`（Win）+ Linux 侧骨架编译绿。
4. **Smithay 会话/合成循环**（B/C 路线）：最小 compositor（session +
   单 surface + 渲染循环）Linux 实跑一帧。
   验证：实跑日志 + 单帧合成证据。
5. **shell 首帧上屏**：attach/直渲（随路线）拉起 shell 面 → 全屏首帧
   截图留痕。
   验证：截图（dock+背景可见）。
6. **I1 diff 核对**：auto-lang 主仓改动清单 = 仅新增/配置差异；shell
   资产零改动证明。
   验证：diff 证据贴本计划。
7. **台账回写**：`docs/plans/autos-desktop-program.md` 457 行改号 509 +
   状态；overview auto-cosmic 条目注记。
   验证：台账 diff。
8. **收尾**：健康检查；状态翻 execution_done。
   验证：`cargo check -p auto-lang && cargo t`（主仓日常档）。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **① Linux 验证环境（硬前置）**：WSL2（Wayland 合成可行性随内核/WSLg
  版本）vs 物理 Linux vs CI Linux runner——执行前由用户指定；环境记录
  为 Stage 2+ 基线。**无 Linux 环境则本计划不可开工**。
- **② 大依赖门槛**：smithay/libcosmic 为重量级新三方——评估报告的依赖
  清单**入 lock 前经用户确认**（对齐仓库对重大依赖的谨慎惯例）。
- **③ 路线倾向声明**：B（Smithay+桌面协议宿主）因 500 资产复用最大化而
  是当前倾向，但**评估不得倒果为因**——A（libcosmic 直渲）在"shell 交互
  保真"上可能有短期优势，矩阵如实打分。
- **④ 排程**：建议 507 合入后开工（queue 臂覆盖集就绪）；骨架/评估类
  任务（T1/T2）可先行。
- **⑤ smithay 版本策略**：smithay 生态版本迭代快——评估锁定具体版本并
  注明升级策略（线内 pin + 升级计划）。
