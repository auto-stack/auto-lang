---
plan_id: PLAN-507
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: desktop-protocol-stage5-coverage
author: [zhaopuming]
created_at: 2026-08-31
updated_at: 2026-08-31

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [auto-lang/ui]
current_step: 0
total_steps: 10
---

# [PLAN-507] 桌面协议 Stage 5——RenderQueue 全 widget 覆盖爬坡 + parity 日常门禁

## 变更摘要

承接 500（Stage 4，已归档：载荷二态/三态开关 `desktop_render:`/AppProjector
块流布局引擎/001–005 子集端到端/parity_001 三臂金样，77/77 绿）。本期两件事：

1. **覆盖爬坡**：`schema/aura.at` 全量 **388 个 element** → 分级降维——
   **Tier 1**（display/form/layout 常用族，~40–60 kind）+ **Tier 2**
   （typography/语义容器，块流简单映射）全量覆盖；**Tier 3**（长尾与复合
   组件：编辑器/chart/diagram 家族）显式 not-yet 表 + auto 降级长期共存，
   复合组件是否永久 not-yet 逐项裁定。
2. **parity 门禁进日常档**：金样从 parity_001 单条扩为**覆盖表驱动的抽样
   矩阵**；并把 `cargo t desktop_protocol --features ui-iced` 纳入日常档
   （500 复审实证 `cargo tf` 不带 ui-iced——门禁盲区家族，本期收口）；
   **覆盖表漂移检测**：registry/aura.at 变更 ↔ Coverage 表同步告警——
   新 widget 默认 not-yet 必须显式登记，防"第三端 parity 债"烂尾。

## 目标

- **G1 分级清单**：388 element 三级清单成文（T1 产出，按使用频率×家族依赖
  排序），Tier1/Tier2 覆盖目标集、Tier3 not-yet 政策逐项注记。
- **G2 Tier1 覆盖**：常用 display/form/layout 族在 queue 模式渲染正确
  （含交互命中区/焦点态/禁用态），抽样示例 re-exec 端到端绿。
- **G3 Tier2 覆盖**：typography（h1–h6/b/em/i/code 等）与语义容器
  （header/footer/article/aside/nav/main 等）块流映射覆盖。
- **G4 Tier3 政策**：not-yet 显式表 + `auto` 降级链路复核（覆盖不足 →
  independent + 观测行）；复合组件（code_editor/autodown_editor/markdown/
  chart/diagram 家族）逐项裁定"永久 not-yet / 后续专项"。
- **G5 parity 门禁日常化**：金样矩阵按覆盖表抽样扩容；desktop_protocol
  （ui-iced 特性）进 `cargo t` 日常档；覆盖表漂移检测入 schema 三件套
  同款治理位。
- **非目标**：Stage 6（默认策略/远程端消费）；复合组件的实现级 lowering
  （仅裁定政策）；性能调优专项；vue 端行为对齐（I4' 登记义务不变，行为
  对齐仍在 vue 侧自身线）。

## 架构方案

```
aura.at(388) ──T1 分级──▶ Tier1(~50) / Tier2(~80) / Tier3(not-yet 表)
AppProjector(client_runtime.rs, 500 块流引擎) ──爬坡──▶ Tier1+2 臂+命中区
Coverage 表 ──驱动──▶ auto 降级判定(既有) + parity 金样抽样矩阵(T4)
门禁: cargo t 日常档 ⊕ desktop_protocol(--features ui-iced) ⊕ 漂移检测
```

- **爬坡机制**：每 widget 臂 = 投影 lowering + 命中区推导 + 覆盖表登记 +
  金样（家族抽样而非逐 widget 全量金样——矩阵规模可控）。
- **防烂尾刹车**：漂移检测 = aura.at element 集 vs Coverage 表 diff 测试
  （新增未登记 → 红；登记 not-yet → 过）——parity 债从此有自动闸门，
  这是"第三端纪律"的机制化落地。

## 技术栈

既有 AppProjector/协议栈/金样体系。零新三方依赖。

## 需求分析与背景调查

（取材 docs/specs/overview.md §ui + 500 归档复审记录 + 现场核验 2026-08-31）

- **Stage 4 落点**（爬坡起点）：pac.at `desktop_render:` 字段定名；
  AppProjector 已重写为**块流布局引擎**（NodeStyle 复用 `ui/style`——D2
  定案为"自有块流+样式系统复用"）；T3 五真源 App 同宿主 re-exec；T4 三臂
  基线（queue 臂自动档+实机档）；复审门禁事实 `cargo tf` 不带 ui-iced、
  套件须 `cargo t desktop_protocol --features ui-iced`（77/77）。
- **目标集现状**：`schema/aura.at` 388 个 element（含别名/分层
  tier/builtin_widget 与 web_component 混合）——T1 分级需同时吃 registry
  的 tier/类别元数据（WidgetCategory 已有 Display/Form 分类基础）。
- **I4' 纪律依据**：设计文档 §1.3 三臂 parity 条款（覆盖表 not-yet 显式、
  禁止静默错绘）；I4 双端金样体系为形态先例。
- **排程**：503（reviewed 待合并）/505（execution_done 待复审）/506
  （drafting）在途或待领——本期改动面 = `ui/desktop_protocol/
  client_runtime.rs`（爬坡）+ Coverage/金样/门禁配置，与 503 视觉面、
  505 事件泵段、506 示例批**零交叠**；499（charts v2）若动 svgdoc 渲染
  臂与 projector 无共享文件。可并行。

## 详细设计

### 1. 分级清单（T1 产出，回写本计划）

- 依据：registry WidgetCategory + examples/实际使用频率扫描 + 家族依赖
  （如 button 被 card/dialog 族引用）；
- 预期形态（T1 定稿）：Tier1 = text/button/input/checkbox/switch/image/
  icon/badge/avatar/divider/container/column-row/scrollable/card/list/
  tooltip/menu 族 ≈ 40–60；Tier2 = typography + 语义容器 ≈ 60–90；
  Tier3 = 复合编辑器/markdown/chart/diagram/nav 系/menubar 等 ≈ 其余。

### 2. 投影器爬坡（client_runtime.rs）

- 每 widget：NodeStyle 映射扩展 + lowering 臂 + 交互命中区（form 族：
  焦点/禁用态的命令差分）+ Coverage 登记；
- 块流引擎不动结构——只加臂与样式映射（500 的 NodeStyle 管线扩展点）。

### 3. 覆盖表与漂移检测

- Coverage 表（500 已有结构）扩为 element 级全集视图：covered | not-yet
  (reason) | n/a（web_component 类不由 queue 臂消费的）；
- 漂移测试：`aura.at element 集 ⊆ Coverage 表`（未登记即红）——落
  `crates/auto-lang/tests/` 既有 schema 三件套同族位置。

### 4. parity 金样矩阵

- 抽样规则：Tier1 每家族 ≥1 条全交互金样、Tier2 每语义组 1 条、not-yet
  不产金样（覆盖表即证据）；矩阵由覆盖表生成清单（防漏）。

### 5. 门禁日常化

- `cargo t` 日常档纳入 desktop_protocol 套件（`.cargo/config.toml` 别名
  增补 `--features ui-iced` 的 scoped 用法或独立 gate 行——以 nextest
  profile 实际能力定，T6 定案）；复审档 `cargo tf` 的 ui-iced 缺口以
  文档注记 + 日常档覆盖达成（不动 tf 语义）。

## 测试设计

1. **T1 清单成文**：分级表回写本计划（复审对照物）。
2. **T2 爬坡单测**：每 widget lowering 臂的 round-trip/渲染快照断言
   （家族参数矩阵：状态×prop 关键集）。
3. **T3 re-exec 集成**：Tier1/Tier2 抽样示例（001–005 + 新构造覆盖示例）
   queue 模式端到端；auto 降级链路（构造 not-yet app → independent +
   观测行）。
4. **T4 parity 矩阵**：金样抽样全绿（三臂对拍）。
5. **T5 漂移检测**：表与 aura.at 同步（红→登记→绿演练一次）。
6. **T6 门禁演练**：`cargo t` 日常档一跑即含 desktop_protocol 结果；
   覆盖率数字（covered/total）输出可读。

## 验收标准

1. Tier1+Tier2 目标集 covered（覆盖率 ≥ 计划定稿值，复审对照 T1 清单）；
   Tier3 not-yet 表逐项有裁定。
2. T2–T5 全绿；T6 门禁演练留痕（日常档含套件 + 覆盖率输出）。
3. 漂移检测红→绿演练成文；schema 三件套不回归。
4. `cargo t ui`、`cargo t desktop_protocol --features ui-iced` 不回归；
   `cargo check -p auto-lang` 零警告。
5. 既有 001–005 与 auto 降级行为零变化（500 行为回归）。

## 执行步骤

（原子任务：精确文件路径 + 确切操作 + 验证命令；每步完成后追加 [✅ 已完成] 一行证据）

1. **分级清单**：扫描 registry/aura.at + examples 使用频率 → Tier1/2/3
   清单回写本计划 + 覆盖基线数字。
   验证：清单成文（T1 对照物）。
2. **漂移检测**：Coverage 表扩 element 全集视图 +
   `crates/auto-lang/tests/` 漂移测试（未登记即红）。
   验证：`cargo test -p auto-lang --test schema_drift`（或新测试文件）绿 + 红演练。
3. **Tier1 爬坡（一）display 族**：`ui/desktop_protocol/client_runtime.rs`
   text/image/icon/badge/avatar/divider/container/scrollable 臂 + T2 单测。
   验证：`cargo t desktop_protocol --features ui-iced`。
4. **Tier1 爬坡（二）form/交互族**：input/checkbox/switch/button 族含
   焦点/禁用态差分与命中区 + T2。
   验证：`cargo t desktop_protocol --features ui-iced`。
5. **Tier1 爬坡（三）layout/复合容器**：column-row/card/list/tooltip/menu
   族 + T2。
   验证：`cargo t desktop_protocol --features ui-iced`。
6. **Tier2 爬坡**：typography + 语义容器批量臂（块流映射为主）+ T2。
   验证：`cargo t desktop_protocol --features ui-iced`。
7. **Tier3 政策**：not-yet 表逐项裁定（复合编辑器/chart/diagram/nav 系）
   + auto 降级链路复核用例。
   验证：`cargo t desktop_protocol --features ui-iced`（降级用例绿）。
8. **parity 矩阵**：金样抽样扩容（覆盖表生成清单）+ 三臂对拍。
   验证：金样套件绿（parity_001 同档）。
9. **门禁日常化**：`.cargo/config.toml` 或 nextest profile 调整 +
   覆盖率输出 + T6 演练。
   验证：`cargo t` 一跑含 desktop_protocol（留痕输出）。
10. **re-exec 全量 + 实机 + 收尾**：T3 抽样示例全绿；实机 queue 模式跑
    Tier1+2 示例集；健康检查；状态翻 execution_done。
    验证：`cargo t desktop_protocol --features ui-iced && cargo t ui`。

## 复审记录

（/auto-plan:review 填写）

## 待澄清事项

- **Tier 边界**：T1 清单定稿数值（预期 Tier1 40–60/Tier2 60–90）以扫描
  结果为准；若 Tier1 超 70 需再切半防单计划膨胀（可拆两批折叠）。
- **复合组件裁定基准**：chart/diagram 家族的 queue 化若被 499/502 的
  canvas/SVG v2 路线覆盖（独立渲染面），则永久 not-yet 合理化——T7 时
  与 499 落地形态对表一次。
- **门禁形态**：`cargo t` 别名能否 scoped 增 feature（nextest profile
  实际能力）——不能则退路 = 独立 gate 别名（`cargo td`？）+ 复审清单
  注记，T9 定案。
- **金样成本**：三臂对拍的 vue 臂依赖 vue 侧渲染稳定性——若 vue 臂抖动
  （字体/AA 差异）允许像素容差档位调整，沿用 a2vue 金样容差先例。
- 排程：与 503/505/506 零交叠核对在 T2 时复查一次（它们可能先合入移动
  行号——grep 重定位）。
