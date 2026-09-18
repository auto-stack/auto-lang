---
plan_id: PLAN-643
status: drafting               # drafting → executing → execution_done → reviewed → archived
feature_name: chart-tag-unify（chart 裸名归属统一与 bp palette 包词汇面）
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 结束时填写：
supersedes_spec_components:
  - docs/specs/auto-lang/ui/design/chart-components.md#归属（chart 契约 v2 补章：tag 双态归属与合并臂裁定）
new_spec_components:
  - docs/specs/blueprint/contract.md#验证面-palette-包词汇面
touched_goals: [GOAL-007]      # 引用 docs/specs/goals.md 的 GOAL-NNN（review 终定）

affects: [blueprint, auto-lang/ui]   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 0
total_steps: 7
---

# [PLAN-643] chart-tag-unify —— chart 裸名归属统一与 bp palette 包词汇面

## 0. 变更摘要

chart 四裸名 tag（`area-chart`/`bar-chart`/`line-chart`/`donut-chart`）的归属被两处
历史决定夹住了：PLAN-484 裁定 chart 走 **official 组件包**（Plan 435 P4
`namespace: "auto"`）、不回引擎内置（shadcn chart 七注册已退役删除）；但
`schema/aura.at:2422-2440` 里四个 `backends: none` 的 unclassified 空壳元素让这些
tag 仍是"builtin tag"——在 ui-gallery 合并 VM 臂把 example 自带的官方包组件压成
builtin 桩（"builtin wins"，`ui_gen/api.rs:824`），画布空（**P642-D1**）；同时
Blueprint 的 `palette_drift` 只认 WidgetRegistry（`bp/registry.rs:129-146`），
palette 无法引用 chart tag（**PLAN-640 AC-09 DEBT**）。

本计划三件事：
1. **aura 空壳处置**：四 chart 元素改"package-origin"分类（或等价机制），不再作为
   builtin 压制源 → 核销 P642-D1（024-charts 合并 VM 臂出真图）。
2. **palette 包词汇面**：`palette_drift` 接受官方组件包导出 tag（chart 四 tag 首批）
   ——这是 640-AC-09 DEBT 的正解：**不是**把 chart 注册回 WidgetRegistry（会违背
   484 "不引引擎内置 chart 控件"），而是让 bp 词汇面认识"包即官方组件"这一层。
3. **dashboard 图表变体提升**（依赖 PLAN-640 合并）：`dashboard/overview` 经
   变体提升评审（promotions）加 chart 变体——L3/promotions 通道的首个实战案例。

Plan 408/435 的 "builtin wins" 通用规则**不动**（a2vue Card 等合法 shadow 生态
保护，api.rs:817-823 既有注释）。

## 1. 目标

- **G1 合并臂出图**：`examples/ui/024-charts` 在 ui-gallery 合并 VM 臂渲染真图
  （与独立 `auto run -r vm` 臂同源），独立臂不回归。
- **G2 palette 包词汇面**：blueprint palette 可声明 chart 四 tag 且
  `palette_drift` 零漂移；未知名仍拒绝（负断言）。规则沉淀进
  `docs/specs/blueprint/contract.md` 验证面。
- **G3 chart tag 双态归属成文**：schema 元素（声明面）= package-origin 非压制性
  builtin；实现面 = official 组件包（484 裁定维持）。沉淀进
  `docs/specs/auto-lang/ui/design/chart-components.md` 契约补章。
- **G4 dashboard 图表变体**：`dashboard/overview` 增加 chart 变体（props 面接
  dataSource，palette 含 chart tag），双端绿，promotions 评审记录留痕。

### 非目标

- 不改 Plan 408/435 "builtin wins" 规则本体与 S003/S004 告警语义。
- 不把 chart 注册回 `WidgetRegistry` 内置（484 裁定维持；640-AC-09 DEBT 草稿中
  "注册 chart tag 进 WidgetRegistry"的方向由本计划证伪并核销）。
- 不做 P642-D4 per-demo 包命名空间化——若 T-01 归因认定它才是 P642-D1 主因，
  本计划缩范围移交（scope guard，见 §10.3）。
- 不做新图类型/交互（crosshair、pie/scatter 等 484 后置项不变）。
- 不收拢 example 内 chart 组件副本的物理位置（除非 T-01 裁定选项 B，见 §5.2）。

## 2. 架构方案

**双态归属模型**（本计划的核心裁定）：

- **声明面（schema）**：chart 四 tag 在 `schema/aura.at` 的元素从 "unclassified
  空壳 / implicit fallback" 改为显式 **package-origin 分类**——语义是"此 tag 的
  实现由 official 组件包提供，schema 只登记名与契约，不参与 builtin 压制"。
- **实现面（包）**：官方 chart 组件（AreaChart/BarChart/LineChart/DonutChart，
  kebab tag 四裸名）维持 Plan 435 P4 组件包形态（现状活于
  `examples/ui/024-charts/src/front/components/*.at` 与 charts-gallery 同构包）。
- **palette 面（bp）**：`palette_drift` 的合法集从 "WidgetRegistry tags" 扩为
  "WidgetRegistry tags ∪ schema package-origin tags"——schema 本就是单一权威，
  不需要物理收拢组件文件（§5.2 选项 A）。

**修复臂**：只动 chart 四元素的分类与压制判定，不触碰 `register_local` 的
通用 shadow 规则（api.rs:807-830）。

**验证臂**：024-charts 双臂（独立 + gallery 合并）出图断言 + bp palette
正/负断言 + dashboard 变体双端验证。

## 3. 技术栈

- Rust：`crates/auto-lang` 的 schema 解析（`aura/schema.rs` 元素分类字段）、
  `ui_gen/widget/ComponentRegistry`（builtin 压制集）、`ui_gen/bp/registry.rs`
  （palette_drift）；合并臂生成面若涉 `auto-man`（T-01 归因后定）。
- Schema：`schema/aura.at`（chart 四元素改分类）。
- 验证：`cargo check -p auto-lang`、scoped `cargo t`、schema 改动触发
  `cargo test -p auto-lang --test docs_gen`（Category C）、review 前 `cargo tv`
  兜底（builtin 集变化可能触及 VM golden 面的 S003/S004 告警消失）、
  autoui-verifier 双端 + gallery 合并臂截图。

## 4. 需求分析与背景调查

### 授权记录

- 2026-09-18 会话：用户确认按"640 后续排序表"起草 ①（chart 面统一）为
  PLAN-643；排序表其余项的归组结论同会话给出（③锁面裁定、②组装样板随后各自
  成计划）。仓库=auto-lang 单仓；worktree `D:/autostack/.wt/lang-643/auto-lang`。
- 前置依赖：PLAN-640（executing，worktree lang-640 在途）——仅 T-06
  （dashboard 变体）依赖其合并；T-01..T-05 无依赖可先行。

### 证据（路径实勘，2026-09-18）

- **压制点**：`crates/auto-lang/src/ui_gen/api.rs:824` —— `register_local` 拒绝
  与 builtin tag 同名的本地组件，"builtin wins (Plan 408/435)"，规则在解析层强制，
  告警 Info 级（S004）。
- **空壳**：`schema/aura.at:2422-2440` —— `area-chart`/`bar-chart` 元素
  `backends: { web: "none", iced: "none" }`、`category: "unknown"`、
  `tier: "unclassified"`，description 自述 "used in widgets-gallery but
  unregistered in any production table (implicit fallback/ext path); P2 review"；
  全 schema unclassified 共 148 处（`grep -c`）。line/donut 同构（T-01 全列）。
- **484 裁定**（`docs/plans/archive/484-declarative-charts.md` frontmatter）：
  shadcn 七元素（areachart/barchart 等驼峰）+ registry 七注册 + unovis 依赖
  全部退役；裸名 tag 让位 official 组件包（namespace "auto"）；
  "引擎给笔，Auto 持笔"。
- **官方包现状**：`examples/ui/024-charts/src/front/components/`
  `{area_chart,bar_chart,line_chart,donut_chart,chart_geom}.at` + `package.at`
  （`namespace: "auto"`）；`examples/charts-gallery/src/front/components/package.at`
  同构——**无单一物理权威位置**，副本分布（T-01 盘点全名单）。
- **palette 面**：`crates/auto-lang/src/ui_gen/bp/registry.rs:129-146`
  `palette_drift` 只查 `WidgetRegistry::all_widgets()`；Rust registry 无 chart
  tag（grep 实勘；`apply_schema_vue_mappings` registry.rs:88 只 overlay 不新增）。
- **债**：P642-D1（KNOWN-DEBT 2026-09-18 增补节：合并臂 builtin 桩、独立臂正常、
  候选方向两条）；PLAN-640 AC-09（chart 四 tag 与 `data-table` 未入
  WidgetRegistry 的 DEBT——本计划核销 chart 半句，`data-table` 半句移交后续）。

### 风险

- schema 元素分类调整的下游消费面（生成器回写、a2ts/vue 轨 tag 解析、docs_gen
  参考表）未知——T-01 全链盘点。
- 合并臂（ui-gallery，auto-man 生成面）与独立臂解析差异的归因未定——T-01 首任务。

## 5. 详细设计

### 5.1 双态归属裁定（G1/G3 的机制面）

chart 四元素在 schema 增 package-origin 标记（具体字段形态沿 schema 既有分类
词汇，T-01 对齐 `aura/schema.rs` 数据结构）；`ComponentRegistry` 的 builtin
压制集排除 package-origin 元素 → 本地/包组件同名合法接管（S004 消息对 chart
四 tag 不再出现）。其余 147 个 unclassified **不动**。

### 5.2 palette 包词汇面（G2 机制面，两选项）

- **选项 A（推荐）**：palette 合法集 = WidgetRegistry ∪ schema package-origin
  tags。schema 是单一权威，零物理搬运；chart 组件副本仍归 examples。
- 选项 B：官方 chart 组件收拢为单一权威包目录（如 `packages/chart-at/`），
  registry 扫包导出 tag。物理统一但动 024/charts-gallery 两处消费方，面大。
- 默认 A；若 T-01 发现 palette 需要包级 props/事件契约（不止 tag 名），升级 B。

### 5.3 dashboard 图表变体提升（G4，promotions 首例）

`blueprints/dashboard/overview`（640 产物）spec 增 `promotions:` 记录 → 评审通过
后正式声明 chart 变体：palette += 四 chart tag、`charts` extension_point 升为
变体物化样本（`reference/with_charts.at`），dataSource `metrics()` 复用。走
contract.md 变体提升评审流程留痕。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/ui/design/chart-components.md | before：契约 v2 只定包组件契约，未定 tag 在 schema 的归属分类与压制语义；after：补章"tag 双态归属"（schema 元素=package-origin 非压制 builtin；实现=组件包；压制判定排除）+ 合并臂裁定记录 | P642-D1 根因成文，防止 schema 生成器再产空壳压制源 | AC-01/03 |
| SD-02 | modify | docs/specs/blueprint/contract.md（验证面节） | before："palette 声明的每个 widget 必须在 AURA registry"；after：合法集 = AURA registry ∪ schema package-origin tags（official 组件包词汇面），未知名仍拒绝 | 484 裁定下 chart 只能以包形态存在，palette 需要对应词汇面；640-AC-09 DEBT 正解 | AC-02 |

（无 retire 项；KNOWN-DEBT 双核销为簿记非 spec。）

## 6. 测试设计

- **palette 正/负断言**（`bp/registry.rs` 测试）：含 chart 四 tag 的 palette
  零漂移；未知名（如 `pie-chart`）仍报漂移；data-table 仍报漂移（移交项不误放）。
- **压制排除断言**（`ui_gen` scoped 测试）：chart 四 tag 的本地/包组件不再被
  `register_local` 拒绝；既有合法 shadow 用例（a2vue Card）行为不变。
- **024-charts 双臂**：独立臂 `auto run -r vm` 回归不破；合并臂出图断言
  （DOM/snapshot 或截图留档，沿 P642 修复验证口径 `fix3_024-charts.png` 对照）。
- **dashboard 变体双端**（依赖 640）：vue 轨 SFC 发射 + VM 轨 view 结构断言
  （仿 plan640 测试形态，变体面追加）。
- **门禁**：`cargo check -p auto-lang` 零警告；schema 改动跑
  `cargo test -p auto-lang --test docs_gen`；review 前裸 `cargo tv`
  （builtin 集/告警面变化触及 VM golden 的兜底）；`cargo t plan643` scoped。

## 7. 验收标准

- **AC-01 合并臂出图**：024-charts 在 ui-gallery 合并 VM 臂渲染真图（断言 +
  截图留档），独立臂不回归。验证：双臂运行证据 + 对照 P642-D1 修复前截图。
- **AC-02 palette 包词汇面**：chart 四 tag 进 palette 零漂移；未知名负断言仍红。
  验证：`cargo t plan643`（palette 正/负测试）。
- **AC-03 schema 处置**：aura.at 四 chart 元素 package-origin 分类落地，压制
  判定排除；其余 147 unclassified 不动。验证：schema diff 审读 + scoped 测试。
- **AC-04 dashboard 图表变体**：`dashboard/overview` chart 变体双端绿，
  promotions 评审记录留痕。验证：变体双端断言 + spec `promotions:` 小节。
  **依赖 PLAN-640 合并**——若执行时未合并，本 AC 标 blocked 于复审裁决（不静默删）。
- **AC-05 双债核销**：KNOWN-DEBT-AND-RISKS.md 核销 P642-D1 与 640-AC-09 的
  chart 半句（`data-table` 半句明确移交后续计划）。验证：DEBT 文件 diff。
- **AC-06 spec 沉淀**：SD-01/SD-02 落地；merge 时 specs.json upsert +
  `python scripts/spec-index.py`。

## 8. 执行步骤

> worktree `D:/autostack/.wt/lang-643/auto-lang`（分支 `plan-643-dev`）；
> plan 簿记留主检出。T-06 依赖 PLAN-640 合并，其余可先行。

- **T-01 [有界调查] 三臂归因与下游盘点**（1h）
  无依赖。操作：①归因 024-charts 合并臂空画布的精确解析路径（stub 落点、
  standalone 与 merged 差异；排除/确认 P642-D4 命名空间因素）；②盘点 chart 四
  schema 元素的全部下游（生成器回写、a2ts/vue 轨解析、docs_gen 参考表、S004
  消费方）；③盘点官方 chart 组件包副本全名单。产出：归因结论 + §5.2 选项
  裁定 + 若主因属 D4 则触发 §10.3 缩范围。验证：结论记入本节。→ 全 AC 前置
- **T-02 spec 先行**：SD-01/SD-02 两文件补章/改规则（chart-components.md
  双态归属补章；contract.md 验证面包词汇面）。
  验证：diff 审读。→ AC-06
- **T-03 schema 处置**：`schema/aura.at` 四 chart 元素 package-origin 分类 +
  `aura/schema.rs` 分类词汇支持（如需）+ 生成器/回写面适配（T-01 盘点清单）。
  验证：`cargo check -p auto-lang`；`cargo test -p auto-lang --test docs_gen`。→ AC-03
- **T-04 压制排除**：`ui_gen/widget`ComponentRegistry` builtin 压制集排除
  package-origin；S004 对 chart 四 tag 消失、Card 等既有 shadow 不变。
  验证：scoped 测试（正/负）。→ AC-01 前置/AC-03
- **T-05 palette 包词汇面**：`bp/registry.rs` palette_drift 合法集扩展（§5.2
  裁定形态）+ 正/负测试。
  验证：`cargo t plan643`。→ AC-02
- **T-06 dashboard 图表变体提升**（依赖 640 合并）：spec `promotions:` 记录 →
  `reference/with_charts.at` + palette 增补 + 双端断言。
  验证：`auto bp check dashboard/overview` + 双端测试。→ AC-04
- **T-07 双臂验证与双债核销（review 前兜底）**：024-charts 独立/合并双臂出图
  证据；KNOWN-DEBT 双核销行；裸 `cargo tv` 兜底；门禁档位复核（Category B+C，
  不触发 taa/ta）。
  验证：输出留档本节。→ AC-01/05

依赖链：T-01 → {T-03, T-04, T-05}（T-02 可与 T-01 并行，裁定后定稿）；
T-04 → T-07；T-06 独立于 T-03..T-05（仅依赖 640）。

## 9. 复审记录

- 2026-09-18 draft handoff（/auto-plan:new）：plan_revision 1，stage: new，
  outcome: pass（授权范围内可交付 work），next: work。
  待用户确认项见 §10（三项，均有默认裁定；#3 为 scope guard 非预决）。

## 10. 待澄清事项

1. **palette 词汇面形态**（§5.2）：默认选项 A（schema 分类驱动，零物理搬运）；
   若希望官方组件物理收拢单一权威包目录（选项 B，动两处 example 消费方），
   请明示——影响 T-03/T-05 形态与工作量（B 约 +1 天）。
2. **T-06 时序**：默认按依赖阻塞处理（643 执行期 640 未合则 T-06 等待，其余
   任务不受阻）；若希望 T-06 强制随本计划交付，需协调 640 先合并。
3. **（scope guard，非预决）** 若 T-01 归因认定 P642-D1 主因是 P642-D4
   per-demo 包命名空间化（kept-first 冲突）而非 chart 空壳压制：本计划 G1 缩为
   "空壳处置 + palette 面"（AC-01 改为移交记录），P642-D1 本体移交 D4 归因计划，
   复审时按修订流程处理。
