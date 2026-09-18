---
plan_id: PLAN-645
status: reviewed               # drafting → executing → execution_done → reviewed → archived
feature_name: toolchain-parity-debt（bps fn 转译缺口 + with_defaults 扫描根）
author: [zhaopuming]
created_at: 2026-09-18
updated_at: 2026-09-18
plan_revision: 1

# /auto-plan:review 结束时填写：
supersedes_spec_components: []
new_spec_components:
  - docs/specs/blueprint/contract.md（增补：bps 扫描 fn 转译契约行 + 扫描根解析序行）
touched_goals: ["GOAL-007: AutoUI 跨端视觉一致（parity 三层机制的 L2 前置）", "GOAL-011: Blueprint 一等公民生态"]

affects: [blueprint, auto-gen]
current_step: 4
total_steps: 4
---

# [PLAN-645] toolchain-parity-debt——bps fn 转译缺口与扫描根修正

> 背景设计：docs/design/30-autoui-parity-three-layer.md（三层 parity 机制）。
> 本计划偿还其中标记的两笔 auto-lang 工具链债，是三层机制 **Layer 2
> （Blueprint gallery parity）的第零任务**。

## 0. 变更摘要

偿还 DEBTS 070 登记的两笔工具链限制（均为 PLAN-070 执行期实勘发现）：

1. **vue 轨 bps 扫描不转译跨文件 fn 导入**：bp reference 含
   `use <mod>: fn`（bare 或 bps 限定）时，生成的 SFC 只有调用无定义
   （`Cannot find name 'flatten_tree'`，046-bp-import 构建断裂复现）——
   组合形态 bp 无法上 vue 轨。修复 = bps 扫描发射路径补 plan522 式
   helper 转译（按 base_dir/parent 解析模块源并内联被引符号）。
2. **BlueprintRegistry::with_defaults 扫描根为编译期 CARGO_MANIFEST_DIR**：
   `auto bp list/show/add/check` 恒扫构建机主检出 `<repo>/blueprints`，
   worktree/外部检出内的包不可见。修复 = 扫描根改为运行时解析
   （git toplevel 优先，`AUTO_BLUEPRINTS_ROOT` env 覆盖，回落编译期值）。

## 1. 目标

1. 组合形态 bp 在 vue 轨可构建：bp reference 的跨文件 fn 依赖被内联转译，
   生成的 SFC 自包含（filetree 组合形态回归启用即本目标的验收样本）。
2. `auto bp` 命令族在 worktree/任意检出内可见本仓 blueprints 包；
   `AUTO_BLUEPRINTS_ROOT` 可覆盖。
3. 既有面零回归：app 壳 ui_config 合成（PLAN-639 T-05）、046 语料、
   jade front/auto 构建。

**非目标**：VM 轨增量（本计划纯 vue/CLI 面）；bp 组合语义变更；多版本包。

## 2. 架构方案

- **fn 转译**：bps 扫描点（ui_gen/vue.rs 的 bp reference 构建路径）接入
  plan522 的 helper 转译臂——解析 `use` 语句目标模块源码，将被引用符号的
  fn 体按依赖序内联进 SFC script（与 `use { fn: ... from "*.ts" }` 的 ext
  内联同层，区别在源是 .at 模块）。解析复用 resolve_module_path
  （base_dir→parent→deps 探测，PLAN-635 门控沿用）。
- **扫描根**：with_defaults 改为 `AUTO_BLUEPRINTS_ROOT` env →
  cwd 向上找 pac.at/blueprints 的 git toplevel → 编译期值兜底（三级解析，
  与 AGENTS 跨仓解析序同构）。
- 回归护栏：046-bp-import（app 壳合成路径）+ filetree 组合形态启用样本
  （本计划新增，恢复自 af8c72a84 的 reference/default.at 并适配转译语义）
  + jade front/auto 构建。

## 3. 需求分析与背景调查

**授权记录**：2026-09-18 用户裁定（Q-8 后续：三层 parity 机制的 L2 前置
在本计划偿还，不另立计划）。仅起草；执行未授权。预算未指定。

**证据**：DEBTS 070 两行；PLAN-070 T-01/T-03 勾记（046 断裂复现、
with_defaults 实勘）；PLAN-522（use fn 双端同源转译先例）、PLAN-635
（声明门控）。

## 4. 详细设计

### 规范增量

| delta_id | add/modify/retire | docs/specs/ 目标 | before/after 规则 | rationale | acceptance |
| --- | --- | --- | --- | --- | --- |
| SD-01 | modify | docs/specs/blueprint/contract.md | L1/组合形态"暂缓"节 → 组合形态启用；增补"bps 扫描 fn 转译"契约行（跨文件 fn 依赖内联，GENERATED 语义不变） | L2 parity 的前置能力 | AC-01 |
| SD-02 | modify | docs/specs/blueprint/contract.md（或 project.md） | 扫描根=编译期常量 → env/toplevel 运行时解析三级序 | worktree/外部检出包可见性 | AC-02 |

## 5. 测试设计

- 新增组件级测试（plan639_bp_tests 谱系）：bp reference 跨文件 fn 导入 →
  SFC 含内联 fn 体（正）；未导入符号仍报错（负）。
- with_defaults 三级序测试（env 设置/未设置 × git toplevel/编译期值）。
- 046-bp-import 全量构建回归；filetree 组合形态样本构建回归。
- jade front/auto `pnpm build` 回归（消费方零影响）。

## 6. 验收标准

| ID | 可观察行为 | 验证方法 |
| --- | --- | --- |
| AC-01 | 组合形态 bp（含跨文件 fn 依赖）vue 轨构建绿，SFC 自包含 | filetree 组合形态回归样本 `auto build` + vue-tsc 零该类错误；DEBTS 070 第二行销号 |
| AC-02 | `auto bp list` 在 worktree 内可见本仓包；env 覆盖生效 | worktree 内 `auto bp list` 含 navigation/filetree；`AUTO_BLUEPRINTS_ROOT` 指向他处时列举跟随 |
| AC-03 | 零回归：046 + jade front/auto + 既有 plan639 测试 | 三面全绿 |

## 7. 执行步骤

> 执行 worktree：`D:/autostack/.wt/lang-645/auto-lang`（plan-645-dev）。
> 注意：jade 侧消费样本在 auto-down 仓（组 worktree
> `.wt/lang-645/{auto-lang,auto-down}` 或临时借用 jade front 构建面）。

- **T-01** [x] [改] with_defaults 三级解析根 + 测试。依赖：无。→ AC-02
  [✅ 已完成 2026-09-18] `resolve_blueprints_root` 纯函数（AUTO_BLUEPRINTS_ROOT
  env → cwd 向上找 blueprints/ → 编译期 CARGO_MANIFEST_DIR 兜底）+ with_defaults
  改造（registry.rs）；`registry::root_resolution_tests` 4/4 绿 + 既有 registry
  115 测绿。提交 afc79e53d。AC-02 实证：worktree 内 `auto bp list` 含
  navigation/filetree（master 二进制同 cwd 对照 = 主检出包库 variants=[]，
  worktree 二进制 = worktree 包库 variants=["default"]——扫描根跟随运行位置）；
  env 指向空目录 → 列举为空，指向主检出 blueprints → 列举跟随。
- **T-02** [x] [改] bps 扫描 fn 转译臂 + 正/负测试。依赖：无（可与 T-01 并行）。→ AC-01
  [✅ 已完成 2026-09-18] 实勘修正：发射路径 = auto-man vue.rs Plan 475 dep 通道
  （collect_dep_front_dirs 对 bps 库包原目录直推→逐 .at 编译 SFC——LoginForm/
  FileTree.vue 的实际生产点；计划草案所写 "ui_gen/vue.rs bp reference 构建路径"
  定位不准，语义不变）。修复 = 通道内挂 `collect_use_module_fns` 池（与
  components/bps 通道 Plan 522 臂同款——首遍编译已在 api.rs 挂池但逐 widget
  重生成需重挂）+ library 形态 dep（无 src/front、无 front/）strict 软化为告警
  （证据见 T-04 基线红）。提交 13fa93205。Q-1 裁定 = 默认案：仅被引符号闭包
  （负断言 filter_tree/collect_ids 不入 SFC；闭包到不动点正断言
  has_id/ext_icon 入）。
  [🔶 review r1 修正 2026-09-18] 首版 library 判别式在编译循环对 dep_front
  再 join（应用形态恒误判为库，软化越权覆盖所有 dep）+ Phase 1c 增量腿缺
  软化——r1 needs_fix 后修复（49bd351da）：旗标随 collect_dep_front_dirs
  形状裁定携带 + handle_compile_error_with_dep_shape；双向负测实证
  （应用形 S003 硬炸保持/库形 warn-only）。
- **T-03** [x] [新] filetree 组合形态回归样本（恢复 reference/default.at，
  适配转译语义；`use` 形态按 T-02 实现定 bare/bps 限定）。依赖：T-02。→ AC-01
  [✅ 已完成 2026-09-18] reference/default.at 按 af8c72a84 原样恢复（bare
  `use tree_util:`/`use tree_icon:`——父目录解析即达包内支撑件，无需改写）；
  spec.md variants=["default"]、暂缓节撤除（palette=[] 维持，icon 债 P643-D1
  不变）。新夹具 examples/capability-tests/047-bp-compose（pac.at dep bps +
  bps 限定组合导入）+ plan645_bp_tests 3/3 绿。提交 13fa93205。
- **T-04** [x] [改] 回归三面（046/jade/既有测试）+ DEBTS 两行销号 +
  contract.md SD 落笔。依赖：T-01..T-03。→ AC-03
  [✅ 已完成 2026-09-18] **基线红发现与修复**：046 全量 `auto build` 在 master
  （1fc4772d9）即红——PLAN-643 with_charts.at `use { package: official from
  "components" }`（消费方契约导入）在包内 standalone strict 编译 S003 硬炸
  （643 门禁不覆盖全量构建故漏检）→ 库形态 dep strict 降为告警（Plan 041a
  fn_only 先例同向延伸；应用形态 dep 门禁不变）。回归证据：①046 gen-only
  （0 失败告警外全绿）+ vue-tsc+vite build 绿；②047 同全绿；③jade front/auto
  codegen 30 组件 0 失败 + **双工具链 gen 产物逐字节一致**（master vs plan645
  二进制同源双跑 diff = 空——零影响的最强形态）；④bp 谱系测试 plan639/640/
  645 全绿 + registry 115 绿。DEBTS 两行销号 + P645-D1（data-table R006 语料
  债新登记）+ P645-D2（musk p053_4 预存红基点对照归因）落 KNOWN-DEBT 增补四；
  contract.md SD-01/SD-02 契约行 + 库包 dep 扫描纪律行落笔。提交 0c46b23bc。

## 8. 复审记录

- 2026-09-18 draft handoff：`stage: new | plan_id: PLAN-645 | plan_revision: 1 |
  outcome: pass（起草完成；执行未授权） | next: review → work`。
- 2026-09-18 work handoff：`stage: work | plan_id: PLAN-645 | plan_revision: 1 |
  outcome: pass | code_commit: 0c46b23bc（worktree plan-645-dev；
  afc79e53d T-01 → 13fa93205 T-02+T-03 → 0c46b23bc T-04 docs） |
  task_ids: T-01,T-02,T-03,T-04 | evidence: 见 §7 各任务勾记（046/047
  auto build+vue-tsc+vite 绿、jade 双工具链 gen 逐字节一致、
  plan645 3/3+registry 115+plan639/640 绿、AC-02 双二进制对照、
  全档对照零新增红：基点 1fc4772d9 no-fail-fast 41 failed（unique 36）vs
  本计划 40 failed（unique 35，净新增=空集；少的 1 个=iced renderer
  external_config_poll flaky）+7 新测试全绿 |
  blockers: 无 | next: review`。
- 2026-09-18 review r1（实现会话内复审——基于工件重建，非采信执行摘要）：
  `stage: review | plan_id: PLAN-645 | plan_revision: 1 | outcome: needs_fix |
  reviewed_commit: 0c46b23bc | base_commit: 1fc4772d9 |
  dependency_revisions: auto-down fae21d9（clean，未改动） |
  spec_inputs: docs/specs/blueprint/contract.md @0c46b23bc |
  acceptance_results: AC-01 pass（发射面+构建门实证）、AC-02 pass（双二进制
  对照+env 跟随）、AC-03 partial（见 F-R1 门禁越权与 F-R2 run 腿缺口） |
  findings:
  - **F-R1（high，T-02/contract 第三行）**：auto-man vue.rs 库形态判别式
    `library_dep = !dep_front.join("src/front") && !dep_front.join("front")`
    判错对象——collect_dep_front_dirs 对应用形态 dep 推入的 dep_front 本身就
    是 front 目录，join 结果恒假 → library_dep 恒真 → 软化覆盖**所有 dep**，
    Plan 475 对应用形态 dep 的 strict 门禁被无声削弱，与代码注释及
    contract.md"应用形态 dep（有 front 布局）strict 门禁不变"声明矛盾。
    修正：库形态旗标在 collect_dep_front_dirs 做形状裁定时随元组携带。
  - **F-R2（medium，T-02/AC-01 目标面）**：Phase 1c（Plan 475 通道的
    `auto run` 增量腿，vue.rs handle_compile_error 硬升级臂）未随 F-R1 修复
    一并软化——消费组合形态 bp 的项目 `auto run` 仍会在 with_charts.at
    S003 上硬炸，dev 环路不可用。修正：dep 腿错误处理按库形态旗标软化，
    Phase 1b（项目自身组件）保持 strict。
  evidence: diff 逐行复审（0c46b23bc vs 1fc4772d9）；vue.rs:2986-3003/
    4936-4944/5526；collect_dep_front_dirs 形状裁定 vue.rs:2271-2319 |
  next: work（F-R1/F-R2 修复后重审；重开 T-02）`。
- 2026-09-18 review r2（r1 findings 修复后重审）：
  `stage: review | plan_id: PLAN-645 | plan_revision: 1 | outcome: pass |
  reviewed_commit: 49bd351da（r1 修复：F-R1 库形态旗标随
  collect_dep_front_dirs 形状裁定携带 + F-R2 Phase 1c 增量腿
  handle_compile_error_with_dep_shape 软化） | base_commit: 1fc4772d9 |
  dependency_revisions: auto-down fae21d9（clean） |
  spec_inputs: docs/specs/blueprint/contract.md @49bd351da（SD-01/SD-02 +
  库包 dep 扫描纪律行——修复后代码与"应用形态 dep strict 门禁不变"声明一致） |
  acceptance_results: AC-01 pass（修复后复验：047 gen-only+vue-tsc+vite 绿、
  FileTree.vue fn 闭包内联+TreeIcon 导入、负断言 filter_tree/collect_ids
  不入 SFC）；AC-02 pass（双二进制对照+env 跟随+registry::root_resolution_tests
  4/4）；AC-03 pass（046 gen-only 0 硬错、bp 谱系 plan639/640/645 绿、
  auto-man 298/298、cargo tf 3631/3632——唯一红 plan367 navtree 为基点
  1fc4772d9 即红的预存项（r1 对照集在案）、jade 双工具链 gen 逐字节一致）|
  findings: F-R1/F-R2 已修复并双向实证（应用形 dep S003 strict 硬炸
  EXIT=1 保持 / 库形 dep warn-only EXIT=0）——判别式语义与 contract
  声明对齐；无新增 findings |
  evidence: diff 复审 0c46b23bc..49bd351da；负测/对照 fixture 实测记录于
  review r1/r2 条目；持久工件=plan645_bp_tests（仓内）+ root_resolution_tests
  （仓内）+ 047 夹具（仓内） | next: merge`。
- r2 复审范围声明：本复审在实现会话内完成（独立会话不可用）——结论由
  工件重建（逐行 diff、可复现命令、双向负测对照、基点对照集），非采信
  执行摘要；局限已如实记录。
- 执行摘要：两债均按三级序/plan522 先例偿还；实勘修正一处（bps 扫描
  发射路径 = auto-man Plan 475 dep 通道，非草案所写 ui_gen/vue.rs）；
  附带修复 PLAN-643 引入的 046 全量构建基线红（with_charts S003 strict，
  库形态 dep 软化）；新增 P645-D1/P645-D2 两条环境/语料债登记。

## 9. 待澄清事项

| # | 事项 | 影响 | owner/下一步 |
| --- | --- | --- | --- |
| Q-1 | ~~fn 内联的体积上限~~ **已裁定（T-02 实现取默认案）**：仅被引符号闭包（imported 项 + 调用闭包到不动点），未导入符号不入 SFC——plan522 既有纪律，plan645 负断言锁定（filter_tree/collect_ids 不入 FileTree.vue） | 已闭环 | — |
| Q-2 | filetree 组合形态启用后是否回填 jade（web file_tree 谱系仍独立，Q-7=A 不变） | 范围 | 维持 Q-7=A；组合形态仅作为 bp 能力样本与未来消费方基建 |
