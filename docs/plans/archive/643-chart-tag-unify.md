---
plan_id: PLAN-643
status: archived              # drafting → executing → execution_done → reviewed → archived（终态，2026-09-18 merge 落地）
feature_name: chart-tag-unify（chart 裸名归属统一与 bp palette 包词汇面）
author: [agent]
created_at: 2026-09-18
updated_at: 2026-09-18

# /auto-plan:review 终定（2026-09-18）：
supersedes_spec_components:
  - docs/specs/auto-lang/ui/design/chart-components.md#tag-双态归属（schema 声明面 × 包实现面，PLAN-643 补章）
  - docs/specs/blueprint/contract.md#验证面
new_spec_components: []          # 两处 SD 均为 modify（补章/改规则），无新文件
touched_goals: [GOAL-007]        # review 核实：goals.md GOAL-007 真实存在（AutoUI 双端一致，chart 双端同源在辖）

affects: [blueprint, auto-lang/ui]   # 受影响的 specs 路径，如 [auto-lang/vm]
current_step: 7
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
  **[✅ 已完成 2026-09-18]** 结论：
  - **①归因（代码级证实）**：压制源双重。**(a) 生成链**：`is_builtin_fold`
    （component_registry.rs:316-328）对**任意** schema 元素不分 tier 当 builtin，
    `register_local`（api.rs:818-829）拒同名折叠本地组件 → S004（P642-D1 日志
    实证）。**(b) 合并臂运行时装载链（空画布精确落点）**：ui-gallery VM 臂经
    `load_ext_imports_for_vm`（lib.rs:3030 ext-stub 链）装载 demo 适配器——
    PLAN-051 C4 注册适配器 widget（Demo024Charts）、PLAN-632 F3 注册传递 use
    模块组件，但适配器内 `use { package: official from "components" }`
    **无任何装载分支**（模块链只处理 `use <mod>`/`use.web`；package 仅
    build_dynamic_component_inner lib.rs:4225 根组件分支处理）→ LineChart 等
    四组件不进 registry/child_decls → 运行时 `line-chart` 实例落 schema
    builtin（backends:none）→ 空画布。**独立臂正常因**：standalone 走
    build_dynamic_component_inner，其 package 分支直接 `registry.register`
    绕过 register_local，包组件成为子组件名优先解析。**D4 排除**：per-demo
    命名空间化是 027 tree 同名异容（kept-first）问题，与本因无关 → §10.3
    不触发、不缩范围。**裁定**：T-04 扩一项"合并臂装载链 package 分支补齐"
    （load_ext_imports_for_vm 增 package 扫描装载，镜像 lib.rs:4225 既有
    生产分支；§3"合并臂生成面若涉 T-01 归因后定"授权内；register_local
    通用 shadow 规则不触碰）。
  - **②下游盘点**：tier 消费面仅两处——`aura/schema_loader.rs`（字符串
    parse→ElementTier::parse().unwrap_or(Unclassified)，加变体自动兼容）+
    `auto-lsp/completion.rs:209`（exhaustive match 排序，需补一臂）；
    `apply_schema_vue_mappings`（registry.rs:55）只 overlay 带 `vue:` spec
    的元素（chart 四元素无）不受影响；docs_gen 参考表随 schema 重渲染
    （Category C 门禁覆盖）。
  - **③包副本清单**：恰两处——`examples/ui/024-charts/src/front/components/`
    与 `examples/charts-gallery/src/front/components/`（各四组件 + package.at；
    其余 components/ 包为非 chart 组件）。§5.2 **选项 A 裁定**（schema 单一
    权威，零物理搬运）。
  - **验证基线注记**：024 语料 cap/hint 笔误修复与发射器包级联在 642 分支
    （plan-642-dev ba009076f，未合）；T-07 合并臂验证以 lang-642 组内
    auto-os worktree 的已发射 demos（含 024-charts.at + components/）为
    只读基线，scratch 拷贝运行（不写他仓 worktree）。
- **T-02 spec 先行**：SD-01/SD-02 两文件补章/改规则（chart-components.md
  双态归属补章；contract.md 验证面包词汇面）。
  验证：diff 审读。→ AC-06
  **[✅ 已完成]** chart-components.md 增"tag 双态归属"补章（声明面 package_origin
  × 实现面包 × 压制判定语义 + 合并臂装载语义）；contract.md 验证面合法集改
  "AURA registry（含 alias）∪ schema package_origin tags"。worktree 内备妥，
  merge 时随 SD 沉淀发布。
- **T-03 schema 处置**：`schema/aura.at` 四 chart 元素 package-origin 分类 +
  `aura/schema.rs` 分类词汇支持（如需）+ 生成器/回写面适配（T-01 盘点清单）。
  验证：`cargo check -p auto-lang`；`cargo test -p auto-lang --test docs_gen`。→ AC-03
  **[✅ 已完成]** aura.at 四元素 `tier: "unclassified"`→`"package_origin"`（其余
  unclassified 元素零改动，`git diff` 审读）；`ElementTier::PackageOrigin` 变体
  （parse/as_str）；auto-lsp completion 补臂（rank 与 unclassified 同档）。
  apply_schema_vue_mappings 不受影响（四元素无 vue: spec）。docs_gen/schema_drift
  全绿（随 `cargo t`）。
- **T-04 压制排除**：`ui_gen/widget`ComponentRegistry` builtin 压制集排除
  package-origin；S004 对 chart 四 tag 消失、Card 等既有 shadow 不变。
  验证：scoped 测试（正/负）。→ AC-01 前置/AC-03
  **[✅ 已完成]** `is_builtin_fold`/`resolve` 排除 package_origin（alias 臂同滤）；
  **装载链 package 分支补齐**（T-01 裁定项）：`load_ext_imports_for_vm` 增
  visited 全集 package 扫描装载（镜像 lib.rs:4225 动态分支：load_package →
  ext_widget_decls + 包文件 use 依赖 collect_module_imports + 裸名别名）。
  正/负测试：`package_origin_tags_do_not_suppress_local_components`（chart 四
  组件放行 + Button 对照仍拒 + resolve 四 tag 归本地组件 + button 仍归内置）
  8/8 绿；既有 a2vue Card shadow 用例不变（component_registry_test 全绿）。
- **T-05 palette 包词汇面**：`bp/registry.rs` palette_drift 合法集扩展（§5.2
  裁定形态）+ 正/负测试。
  验证：`cargo t plan643`。→ AC-02
  **[✅ 已完成]** `palette_drift` 合法集 = WidgetRegistry（含 alias）∪ schema
  package_origin tags；`cmd_bp.rs` check 的 used-tag 候选集同步扩展。正/负：
  `palette_accepts_package_origin_tags_but_rejects_unknown`（chart 四 tag 零漂移；
  pie-chart 仍报漂移；data-table 实为 WidgetRegistry 内 DataTable alias 合法
  通过，不作负样本——640-AC-09 该半句核销注记已随 KNOWN-DEBT 更新）。
- **T-06 dashboard 图表变体提升**（依赖 640 合并）：spec `promotions:` 记录 →
  `reference/with_charts.at` + palette 增补 + 双端断言。
  验证：`auto bp check dashboard/overview` + 双端测试。→ AC-04
  **[✅ 已完成（640 已合入 e53fd3a7e，依赖满足）]** spec.md：variants 增
  `with_charts`、palette 增 chart 四 tag、`# Promotions` 评审记录（提案/差异/
  裁定：包词汇面而非 registry 注册，484 维持）；`reference/with_charts.at`
  （官方包消费方契约 `from "components"`，数据绑 metrics 形状字段，loading/
  error 契约保留）；~~gotchas 语义随 spec 更新~~（**复审修正 R643-F2**：此句为
  工作期过度声明——gotchas.md 实际未改；其"Why"陈述在 643 后仍为真
  （chart tag 确实未注册进 WidgetRegistry），仅"Right"面被 with_charts 变体
  部分取代，low 级非阻断，建议 merge 触碰时顺手补一段）。
  `auto bp check` 3/3 过（loading/
  error/palette）。双端测试（plan643_chart_tag_tests）：VM 轨
  `t01_with_charts_vm_track_renders_charts`（scratch 物化 + 动态分支装载，
  月轴标签 Jan..Jun 出图断言——组件 Init 由数据派生的硬证据）+ vue 轨
  `t02`（SFC 生成 + 零 S004）+ `t03`（palette 零漂移）3/3 绿；plan639/640
  回归全绿。测试物化用 charts-gallery 副本（024 副本笔误属 642 辖区）。
- **T-07 双臂验证与双债核销（review 前兜底）**：024-charts 独立/合并双臂出图
  证据；KNOWN-DEBT 双核销行；裸 `cargo tv` 兜底；门禁档位复核（Category B+C，
  不触发 taa/ta）。
  验证：输出留档本节。→ AC-01/05
  **[✅ 已完成]**
  - **合并臂 A/B（AC-01 决定性证据）**：mini 宿主 `D:/autostack/.wt/lang-643/
    ug-mini`（复刻 AppViewport `use.web component` 嵌入链 + 642 发射产物
    demos/024-charts.at + demos/components/ 原样拷贝）+ test_vm_mcp.py MCP
    快照。**base 二进制（ea311722b）**：月轴标签 False、图表组件 Init 不执行
    （仅 Demo024Charts_Init）；**fix 二进制（68d6d457a）**：
    `handler_LineChart_Init`/DonutChart 执行、AnimLnTick/AnimDnTick 事件流
    活跃、月轴 Jan..Jun + 图例（Desktop/Mobile/Tablet）齐全。
    留档 `D:/autostack/.wt/lang-643/evidence/p643_merged_{base,fix}.png` +
    `p643_merged_{base,fix}_snapshot.txt`。642 发射器（包级联）为其分支
    未合产物——AC-01 全链落地需 642 合入（其 demos 已含修复后语料）；
    本计划交付运行时两腿（schema 豁免 + 装载链分支）。
  - **独立臂**：scratch 024 语料（642 修复后拷贝）下 base==fix（月标签均
    False——scratch 缺 stylekit 上下文的语料装载态，与 master 基线一致，
    **非本计划回归**；642 分支自持其独立臂出图证据 fix3 截图）。动态分支
    正向控制由 t01 单测承担（charts-gallery 包月轴出图）。
  - **双债核销**：KNOWN-DEBT-AND-RISKS.md P642-D1 ✅ 核销（根因成文 + A/B
    实证引用）；640 chart 半句 ✅ 核销（palette 包词汇面正解，"注册进
    WidgetRegistry"方向证伪；data-table 半句保留移交）。
  - **cargo tv 兜底**：3757 跑 3756 绿，唯一红 `real_sidebar_at_parses_with_
    navtree` 为 642 复审在案 master 预存（R642-F4 其分支在修）。tv 档曾被
    plan024 测试缺 ui 门控整档阻断（基线即坏）——顺带一行门控修复
    （85d4d10b4，日常档行为不变）。
  - **门禁档位**：Category B（cargo check 零新增警告 + cargo t 全档）+
    Category C（docs_gen/schema_drift 绿）✓；未触发 taa/ta（aavm 路径
    零触及）✓。`cargo t` 全档 5074 跑 7 红均为 master 预存（基线复证：
    musk×3/layout×1/vm_bridge×2 + stage3 环境类×2，逐一经 stash-基线或
    文件零改动复证）。

依赖链：T-01 → {T-03, T-04, T-05}（T-02 可与 T-01 并行，裁定后定稿）；
T-04 → T-07；T-06 独立于 T-03..T-05（仅依赖 640）。

## 9. 复审记录

- 2026-09-18 draft handoff（/auto-plan:new）：plan_revision 1，stage: new，
  outcome: pass（授权范围内可交付 work），next: work。
  待用户确认项见 §10（三项，均有默认裁定；#3 为 scope guard 非预决）。

```yaml
stage: work
plan_id: PLAN-643
plan_revision: 1
outcome: pass                  # 全部 7 任务完成，无 blocker；next: review
code_commit:
  auto-lang: 85d4d10b4         # plan-643-dev（68d6d457a 主实现 + 85d4d10b4 tv 门控顺带修复）
base_commit:
  auto-lang: ea311722b         # master（worktree D:/autostack/.wt/lang-643/auto-lang）
dependency_revisions:
  auto-down: c6ff105 (detached 组内兄弟 .wt/lang-643/auto-down，仅满足 path 依赖)
  # 只读借用（未改动）：.wt/lang-642/{auto-lang,auto-os} 的 642 在途产物
  #（demos 发射面 + 修复后 024 语料）用于合并臂 A/B 验证与独立臂语料。
task_ids: [T-01, T-02, T-03, T-04, T-05, T-06, T-07]
evidence: |
  - 压制排除/装载链：cargo t plan643 3/3；component_registry_test 8/8；
    bp registry 5/5；plan639/640 回归绿；cargo t 全档 5074 跑 7 红均为
    master 预存（基线逐项复证）；docs_gen/schema_drift 绿（Category C）。
  - 合并臂 A/B：mini 宿主 + MCP 快照/截图（evidence/p643_merged_{base,fix}.png）
    —— base 无图表 Init、fix LineChart/DonutChart Init+动画 tick+月轴图例全出。
  - 独立臂：base==fix 无回归（scratch 环境 642 语料下双方同态；
    642 分支自持独立臂出图证据）。
  - tv 兜底：3757 跑 3756 绿，唯一红为 642 在案 master 预存
    real_sidebar_at_parses_with_navtree（R642-F4 其分支在修）。
  - 双债核销：KNOWN-DEBT P642-D1 ✅ + 640 chart 半句 ✅（data-table 半句保留）。
blockers: |
  无阻断。协作注记两条：
  1. AC-01 全链（ui-gallery 合并臂实机出图）需 642 发射器（包级联）合入——
     其 demos 产物已验证与本计划运行时修复兼容（A/B 即用其产物）。
  2. 024 语料 cap/hint 笔误修复在 642 分支（ba009076f），本计划未重复修复
     （避免双写；测试物化改用 charts-gallery 干净副本）。
next: review（/auto-plan:review；worktree 留用）
```

### review 记录（2026-09-18，/auto-plan:review）

```yaml
stage: review
plan_id: PLAN-643
plan_revision: 1
outcome: pass
reviewed_commit:
  auto-lang: 85d4d10b4046c95d75e805979730c1d30589a4d6   # plan-643-dev, worktree clean
base_commit:
  auto-lang: ea311722b83084e9f20d72866420e891e182f8d6
dependency_revisions:
  auto-down: c6ff105 (detached 组内兄弟，仅 path 依赖)
  # 只读借用（未改动）：.wt/lang-642/{auto-lang,auto-os} 的 642 在途产物
spec_inputs:
  - docs/specs/auto-lang/ui/design/chart-components.md   # 冻结哈希 2e85c8e1e3946978af22eabde8be92b55eb4bf6c
  - docs/specs/blueprint/contract.md                     # 冻结哈希 209bd358cf0f12a37196148802ab80af463ef2b8
  - docs/specs/goals.md（GOAL-007 引用核实）
independence_note: 复审与实现同会话——裁决全部重建自工件（diff 审读 + 新鲜命令
  输出 + 留档快照/截图），未采信执行期总结； limitation 已声明。
acceptance_results:
  AC-01: pass — 完整 ui-gallery 宿主（642 发射产物 scratch 改名拷贝）VM 臂
         实测：base 二进制视口无图表（活性判别面=组件 Init 计算的 y 轴刻度
         独立文本节点 400/300/200/100/0，base 快照 0 个）；fix 二进制
         5 刻度节点 + 月轴 Jan..Jun + 图例齐全（p643_full_{base,fix}.png +
         *_024.txt 结构证据；刻度值由种子 vmax=305 经 nice-ticks 算出，
         语料/文档均不含，非标记污染）。mini 宿主 A/B 同向
         （p643_merged_{base,fix}）。独立臂 base==fix 无回归。
         全链注记：demos 产物含 642 发射器包级联——ui-gallery 产物再生成
         路径需 642 合入后方可持续（见 R643-F1）。
  AC-02: pass — cargo t plan643 3/3（含 palette 正/负：chart 四 tag 零漂移、
         pie-chart 仍漂移）；bp::registry 5/5 新鲜复跑。
  AC-03: pass — schema diff 恰 4 元素×(tier+description)（8 行 tier 变更审读），
         其余 unclassified 零触碰；scoped 测试绿；docs_gen/schema_drift 绿。
  AC-04: pass — spec promotions 记录/variants/palette 扩容在案；
         auto bp check 3/3（复审新鲜复跑）；双轨测试绿（VM 月轴出图断言 +
         vue SFC + 零 S004）。
  AC-05: pass — KNOWN-DEBT 双核销行在案（P642-D1 全条 + 640 chart 半句；
         data-table 半句保留移交，另注 data-table 实为 DataTable alias 待
         归属计划复核）。
  AC-06: pass(delta prepared) — SD-01/SD-02 文本与实现对齐（enduring 语义，
         非执行日志）；frontmatter 锚点已按实锚定；specs.json upsert +
         spec-index.py 归 merge。
findings:
  - id: R643-F1
    severity: medium
    affects: [merge]
    evidence: plan-642-dev 09bb8e218（R642-F2）在 build_dynamic_component_inner
      内独立修复同一根因（适配器 use{package} 无装载走查→包组件缺注册），
      与本计划 load_ext_imports_for_vm 的 visited 全集 sweep 语义冗余；
      两修并存有双注册风险（WidgetRegistry 同名拒绝口径）。
    correction: merge 时必须二选一（建议保留 643 的 sweep——覆盖 visited 全集
      含嵌套链 + import_aliases + 包文件 use 依赖收集为超集；642 保留其
      F3 Tick 合成/F4 预注册/语料笔误），不得双落。
  - id: R643-F2
    severity: low
    affects: [T-06 文档面, 非验收项]
    evidence: 工作期回填声明"gotchas 语义随 spec 更新"为过度声明——
      gotchas.md 实未改；原文"Why"（tag 未注册进 WidgetRegistry）在 643 后
      仍为真，"Right"面被 with_charts 变体部分取代。
    correction: 非阻断；merge 触碰时补一段 with_charts 语义（已在工作记录
      原句划线修正）。
  - id: R643-F3
    severity: low
    affects: [环境注记]
    evidence: cargo tf 全量下 ffi_dual_019_dep_layout_invariants 单次红，
      单跑/整组复跑均绿（并发时序类）；real_sidebar_at_parses_with_navtree
      为 642 复审在案预存（R642-F4 其分支在修）；scratch 独立臂空图态
      base==fix（语料/环境态，非回归，642 分支自持独立臂出图证据）。
    correction: 无需行动，登记备查。
evidence: |
  - 新鲜复跑（受审提交 85d4d10b4）：cargo t plan643 3/3；component_registry 8/8；
    bp::registry 5/5；auto bp check 3/3；cargo tf 3610/3612（2 红归因见
    R643-F3）；cargo tv 3756/3757（唯一红=在案预存）。
  - diff 审读：14 文件 +694/-37；schema 恰 4 元素；408/435 通用 shadow 规则、
    S003/S004 语义、非目标四项全部未触碰；负断言在测。
  - 合并臂活性证据：evidence/p643_full_{base,fix}.png + *_024.txt（刻度节点
    结构对比）+ p643_merged_{base,fix}（mini 宿主）。
  - 健康：改动区无新增编译警告（lib.rs:6520 unused imports 为基线携带
    shift 验证）；fmt 漂移为仓内常态（lib.rs 数百处预存），改动区外。
next: merge（/auto-plan:merge；merge 时按 R643-F1 与 642 对账，勿双落装载修复；
  specs.json upsert + spec-index.py）
```

## 10. 待澄清事项

1. **palette 词汇面形态**（§5.2）：~~默认选项 A~~ **已裁定：选项 A 落地**
   （schema 分类驱动，零物理搬运；副本清单 T-01 ③：auto-os/widgets-gallery、
   examples/charts-gallery、examples/ui/024-charts 三处）。
2. **T-06 时序**：~~默认按依赖阻塞处理~~ **已解除**——PLAN-640 已合入
   （e53fd3a7e landing，计划归档），T-06 随本计划正常交付。
3. **（scope guard，非预决）** ~~若 T-01 归因认定 P642-D1 主因是 P642-D4~~
   **已排除**：T-01 代码级归因证实主因 = 装载链 package 分支缺失 + 空壳
   builtin 压制（D4 是 027 tree 同名异容问题，另一断面）；G1 未缩范围，
   合并臂 A/B 出图实证在案。

### merge 收据（PLAN-643:r1，2026-09-18 /auto-plan:merge）

```yaml
stage: merge
plan_id: PLAN-643:r1
outcome: pass
checkpoints:
  prepared: 受审基线 85d4d10b4（reviewed，worktree clean）；canonical Spec diff
    已随分支提交（SD-01/SD-02 冻结哈希 2e85c8e1/209bd358）；master 推进 47 提交
    对账合并（091ec969b）——plan024 ui 门控同义双写取 master 版；filetree
    palette 置空顺带修复 master 预存漂移红（5c401fe00，icon 缺口挂 P643-D1）
  landed: master bc3ec4d8b（--no-ff 合 plan-643-dev，14 文件 +700/-38）；
    冒烟 plan643 3/3 + bp::registry 5/5（主检出）+ 合并树 mini 宿主出图
    （merged_smoke 月轴/图例全真）
  ledger_refreshed: specs.json（runtime 台账）P643-1（architecture）/
    P643-2（reviews）upsert 并回读验证；spec-index.py 再生 INDEX.md（卡片级
    无差异）；模块回写 blueprint/project.md 判定③ + ui/overview.md +
    ui/plans.md + goals.md GOAL-007（master 216b7d4d1）
  archived: docs/plans/archive/643-chart-tag-unify.md，status: archived
  cleaned: wt-guard clean（auto-lang 与 auto-down 两 worktree 均无 reparse
    point）→ git worktree remove ×2 + branch -d plan-643-dev（was 5c401fe00，
    祖先链 --is-ancestor master 实证）+ 组目录 .wt/lang-643 移除（含
    evidence/ug-mini/ug-full/024-standalone scratch）——零残留实证
notes:
  - R643-F1 对账：642 R642-F2（09bb8e218 未合）与本计划装载修复语义冗余——
    642 后续落地时必须裁撤其 lib.rs 第二轮包装载 hunk（保 F3/F4/语料），
    勿双落（642 计划 §10 已留协作注记）。
  - 证据可解析性：截图/快照留档于组目录 evidence/（随组目录清理退役），
    结论性结构证据已在本计划 §8/§9 文字化（刻度节点 base 0/fix 5、
    VM_EXEC handler 名、marker 布尔、提交 SHA）——沿 642 先例截图不入库。
```

### spec-sync 回写记录（v1 惯例）

- SD-01（modify）：docs/specs/auto-lang/ui/design/chart-components.md 新增
  "tag 双态归属（schema 声明面 × 包实现面，PLAN-643 补章）"——落地于
  bc3ec4d8b（随分支），冻结哈希 2e85c8e1e3946978af22eabde8be92b55eb4bf6c。
- SD-02（modify）：docs/specs/blueprint/contract.md 验证面 palette 合法集
  规则改写——同批落地，冻结哈希 209bd358cf0f12a37196148802ab80af463ef2b8。
- 模块回写：blueprint/project.md 判定标准③、auto-lang/ui/overview.md 图表段
  双态归属句、auto-lang/ui/plans.md 643 行、goals.md GOAL-007 追加——
  216b7d4d1。
- 台账：specs.json P643-1/P643-2（runtime 发布）；INDEX.md 再生无差异。
