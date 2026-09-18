# 30 — AutoUI 跨端一致性三层机制（Widget → Blueprint → App）

> **状态**:设计文档（已裁定，执行计划见 auto-down PLAN-071/072/073 及 roadmap）
> **日期**:2026-09-18
> **关联**:GOAL-007（跨端视觉一致）、GOAL-011（Blueprint 生态）、Design 17（Blueprint
> tier）、PLAN-639（blueprint-tier）、PLAN-637（style recipe 滚动）、PLAN-070@auto-down
> （jade/auto-edit 平台化——本机制的首个 app 级应用）
> **目的**:把"vue/vm 对照到处都不一样、难以下手"的无界视觉差异问题，分解为
> **Widget → Blueprint → App 三层各有穷尽定义（DoD）与门的 parity 流水线**。

## 1. 背景与问题定义

jade-garden web 形态（vue 轨）功能进度领先 desktop（VM 轨），曾尝试以
"web 为基准整体对照 VM"的方式做统一，实际体感是**到处都不一样、无从下手**。
根因：app 级的视觉差异空间是无界的，没有可执行的完成定义。

一致性是**可组合的**：每个 widget 两端一致 + 每个 widget 组合（blueprint）一致
⇒ app 只剩布局组装与数据流层面的差异。因此 parity 工作按组合层级分解为三层，
每层有限单元集、独立的 gallery 隔离面、各自的门与完成定义。

**定位声明**：组件级乃至像素级的跨端一致不是"重复花费"，而是 AutoUI 抽象的
**保真度指标**——vue/vm 渲染不一致即 DSL 抽象泄漏，属工具链缺陷。本机制是
GOAL-007 在 app 级语料上的展开（widgets-gallery 证明逐 widget 可行，jade 证明
app 级可行）。

## 2. 三层模型

| 层 | parity 单元 | 隔离面 | 门 | DoD |
| --- | --- | --- | --- | --- |
| **L1 Widget** | AutoUI widget（.at 单文件件，含 jade 特有件） | 组件 gallery（双轨渲染：vue 页面 + VM 渲染） | 每单元结构快照 + 截图基线**双份** | 清单内全部单元 gate 绿；三桶分类清零 |
| **L2 Blueprint** | 可复用组合（bp 包，spec+reference+gotchas） | blueprints 包库 + bps-gallery | 每单元双端 gate | gallery 全单元绿；抽取判定规则执行完毕并有记录 |
| **L3 App** | 流程 + 布局组装 + 数据流 | jade 本体 | vm-smoke 双模 + 全量 playwright + 双端截图基线 | 全绿；剩余差异仅限布局组装级 |

## 3. Step 0：盘点与三桶分类（L1 的定价与范围依据）

枚举消费 app（jade-garden front/auto + desktop）全部 .at widget，逐个归入：

- **桶① 双端都有、可能有差异** → L1 parity 修复循环；
- **桶② 只有 vue 有** → VM 缺件，进"VM 实现待办"（lighthouse 流素材）；
- **桶③ 只有 vm 有** → 反向补 vue。

三桶清单即"难以下手"的解药：无界问题变为三张有限清单。清单同时定价冻结期
（每单元的修复成本可见）。

## 4. 第三方边界规则（L1 的单元资格）

L1 的 parity 单元是 **AutoUI 面**，不含第三方内核：

| web 侧实现 | parity 单元 | 说明 |
| --- | --- | --- |
| shadcn ui 模块（menubar/button 等） | AutoUI widget 发射面（如 `menubar {}`） | reka-ui 内核不比；宿主自备 ui 模块+reka-ui（PLAN-070 T-06 契约） |
| Tiptap（@autodown/engine AutoDownEditor） | **引擎对拍**：autodown-engine(TS) ↔ autodown-core(Rust) | parser parity/roundtrip 金标纪律的自然延伸，是三层中最深但对拍基建最熟的部分 |
| cytoscape（图谱） | 图谱视图面（graph_view widget） | VM 侧实现形态（native/canvas）单列待办 |

## 5. L1 内部节奏：配方先行，再锁 parity

字面 style 的组件跨端收敛最贵（VM Tailwind 任意值/暗色 token/字形差异，
PLAN-512/518/527 在案），recipe/token 化的组件几乎免费收敛。每单元顺序：

```
盘点 → （需要则）style recipe/token 化（PLAN-607/635/637 机制）
     → 双端 gate（结构快照+截图基线）
     → 修复 → 锁基线 → 下一单元
```

637 在 examples/ui 的 B1–B5 滚动收口即此流水的前例，对象从 examples/ui 换成
消费 app 的组件集。

## 6. L2 Blueprint 抽取判定规则

防过度抽象。升级为 bp 的条件（满足其一）：

1. **≥2 个真实使用位**（同 app 或跨 app）；
2. 明确的跨 app 复用预期（有具名候方）。

同名不同物（数据流/运行时惯用/功能面互异的三版组件，如 jade 的 web/desktop/
041 三版 status_bar）不硬抽"全量 bp"——抽**骨架 bp + 内容 slot**（分隔线+行
布局+slot 的骨架，三版内容做 slot 变体），或裁定不收敛并登记。

**已知前置债（L2 第零任务）**：vue 轨 bps 扫描不转译跨文件 fn 导入
（DEBTS 070 第二行）——组合形态 bp（widget 依赖包内支撑件 fn）在 vue 轨不可
构建。L2 开工前须偿还（发射器补 plan522 式 fn 转译），否则 bp gallery 的
vue 侧无法成立。

## 7. 执行形态与冻结的关系

- L1/L2 工作发生在 **gallery 隔离面**，不直接改 app；app 仅在"修复回写"时被
  触达，始终处于可发布状态。
- app 层新功能冻结令与三层流水并行：gallery 是工地，冻结只针对 app 特性面。
- 修复回写按"唯一源纪律"执行：行为/逻辑留在 `.at` 单源 + ui_config 声明，
  vue 专属只进 ext/视图样式层（ext 缝纪律）。
- VM 存量门（双模 vm-smoke/parity）冻结期间照跑：存量不烂，增量不进。

## 8. 与既有资产的关系

| 既有资产 | 在本机制中的角色 |
| --- | --- |
| widgets-gallery（auto-os） | L1 的跨 app 先例与基础设施参照；跨 app 通用 widget 的 canonical 家 |
| blueprints/ 包库 + bps-gallery（auto-lang，PLAN-639/070） | L2 的包库与 gallery 本体 |
| style recipe/token（PLAN-607/635/637） | L1 配方先行的机制层 |
| vm-smoke（双模）/playwright/截图基线 | L3 的门；基线更新流程（刻意见面变更→--update-snapshots→复跑） |
| 对拍审计台（常驻） | L1/L2 差异定位工具 |
| autodown engine 对拍纪律（parser parity/roundtrip） | 编辑器单元的引擎级对拍 |
