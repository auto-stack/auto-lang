---
plan_id: PLAN-733
status: drafting
feature_name: VM 运行时模块级 fn 调用返空语义修复（store handler 域/computed 族）
author: [agent]
created_at: 2026-10-02T22:10:00Z
updated_at: 2026-10-02T22:10:00Z
plan_revision: 1
current_step: 0
total_steps: 4
supersedes_spec_components:
  - docs/specs/auto-lang/vm/architecture.md
new_spec_components: []
touched_goals:
  - goal-vm-parity
---

# PLAN-733 — VM 运行时模块级 fn 调用返空语义修复（store handler 域/computed 族）

## 0. 变更摘要

修复 VM 渲染目标（`--render=vm`）运行时的模块级 fn 调用返空缺陷族：
store handler 域调用模块级 fn（同文件内联与跨模块 import 同症；观测形态
为多参含列表首参）**不抛错、返回空列表**；视图 computed 内 fn 调用恒返空
（use.web.fn 与 use.web 两形态等同）。真实后果：消费方（auto-musk）在
VM 轨无法使用任何"helper fn + store 域组装"的正规写法（G-15 进度摘要、
PLAN-097 进度卡行内容等）。修复后 auto-musk PLAN-097 T-05 **零改动解阻**
（其行源/门/渲染已全部就位）。

## 1. 目标

1. store handler 域调用模块级 fn（同文件与跨模块 import；多参含列表参）
   返回真实计算值，参数逐位绑定，不抛错不返空。
2. 视图 computed 内 fn 调用返空面：T-01 有界调查判定与目标 1 是否同根
   ——同根则本计划一并修复；异根则明确登记边界与后续计划指针。
3. 跨仓解阻验证：auto-musk `probe-g15.mjs`（PLAN-097）在 musk 侧零改动
   前提下生成进度摘要及行内容可见。
4. 既有 .at 语料金样与测试面零新增红。

### 非目标

- 不改 web/vue 轨行为（trans/javascript 生成面不动；回归门覆盖）。
- 不改 fn 调用语法或 .at 语言面。
- 不修 P706-D1（release back-bridge 调用挂起——独立预存缺陷，载体不同）。
- 不动 AutoCache/项目解析（PLAN-097 附页②已证与本案无关）。

## 2. 架构方案

```text
最小复现收敛（bounded investigation, 语料级）
  → 定位层：compile/synthesis 丢符号 vs 运行时 CALL 约定(参数/返回错绑)
  → 修复（按定位结果落 crates/auto-lang 相应层）
  → 语料金样 + 既有回归门
  → 跨仓复验（auto-musk probe-g15, 锁当日构建）
```

观测约束（既有实证,调查须全部解释）：①无抛错、返回空列表（try 界定过）；
②同文件内联与跨模块 import 同症；③已证可用的调用形态 = handler 域
1~2 参（obj/Value/str/int 参数）跨模块调用（canvasPickedProjection 族、
canvas 面板在真机工作）；④视图 computed 内 fn 调用恒空与 import 形态
无关；⑤参数含 `.length` 成员链读在模板/视图侧返空（表现层,与本案
运行时返空需分层归因）。

## 3. 技术栈

- Rust（crates/auto-lang VM 运行时/编译链接层；crates/auto-man UI 装载链）。
- 既有测试设施：cargo nextest 语料金样分层（.cargo/config.toml aliases）。
- 跨仓复验：auto-musk worktree（PLAN-097 b2285ac）`probe-g15.mjs` +
  `smoke-t07.mjs`，AUTO_EXE 锁本仓当日构建。

## 4. 需求分析与背景调查

### 授权与边界

用户在 auto-musk PLAN-097 会话内明确授权："请给 auto-lang 立项
/auto-plan:new"（2026-10-02），范围 = 修复本缺陷族。无预算/自动续跑
限额授权；work 期遇超界按技能规则回报。

### 缺陷证据包（auto-musk PLAN-097）

- 观测矩阵与三轮更正记录：auto-musk `docs/reports/097-vm-demo-baseline.md`
  附页①（10 形态 VM 表达式语义矩阵）/②（依赖缺口更正）/③（729 稳定点
  复验）。
- 最小复现：auto-musk worktree `plan-097-dev` @ b2285ac——
  `src/front/canvas_store.at`（StatusBackfill 域内联 fn 调用 + try 界定 +
  cp_prog_has/err 落账）+ `tmp/plan097-t01/probe-g15.mjs`（真机探针：
  私有端口起 serve+VM、切 studio、快照直读判言）。
- 复验时点：auto-lang master 7d50989f7（PLAN-729 merged）构建下依旧；
  auto-musk worktree 干净、musk 侧终版设计已就位（52b7733/b2285ac）。
- 已排除：项目解析/AutoCache（附页②：worktree 解析正确）；prop 通道、
  handler 域少量参调用、for-in 成员链（均工作,见观测矩阵）。

### 规格现状

`docs/specs/auto-lang/vm/architecture.md`（call_fn_by_name 段驱动族、
已知坑节）与 `vm/overview.md` 未登记本缺陷——本计划补 fn 调用语义契约
（见规范增量）。KNOWN-DEBT-AND-RISKS 中 P706-D1/P708-R2 为不同面。

## 5. 详细设计

### T-01 有界调查（根因定位,产出决策件）

以语料级最小 .at 复现收敛：store + 模块级 fn（同文件/跨模块 × 参数形态
0/1/2/N 参 × 列表/obj/str 参型）矩阵,在 `--render=vm` 目标下逐格断言
返回值。定位层二选一（或并列）：
- compile/synthesis 层：store 模块编译时跨文件/同文件 fn 符号未编入
  VM 模块（链接期静默降级为空）——观测"不抛错"与该层一致;
- 运行时 CALL 约定层：多参/列表参错绑或返回值序列化丢失。
决策件 = 根因层 + 修复落点 + computed 同根判定,入本计划修订或直接进
T-02（同计划内由调查结论选择,不另开计划）。

### T-02 修复

按 T-01 落点修复（预期面：VM 模块编译的 store 域 fn 符号编入,或 CALL
执行核的参数/返回约定）。不改 .at 语法、不改 web 轨。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-lang/vm/architecture.md | before: store handler 域/视图 computed 的模块级 fn 调用语义未定义(实测返空);after: VM 渲染目标下模块级 fn 调用返回真实值,参数逐位绑定,失败显式报错不静默返空 | 固定调用语义契约,消费方(前端 helper+store 组装写法)依赖 | AC-01,02 |
| SD-02 | add | docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md | 新增:fn 调用语义矩阵(handler/computed × 同文件/跨模块 × 参数形态)与已知边界 | 把 10 形态观测矩阵沉淀为契约面,后续回归锚 | AC-01,02,04 |

## 6. 测试设计

| 验证 | 命令/方法 | 期望与必要反例 |
|---|---|---|
| V01 语料最小复现 | 新增 `tests/<vm 语料位>/fn_call_semantics*`（T-01 定位后定路径）：store handler 调同文件/跨模块 fn（列表首参多参）断言返回非空 | 修复前红（返空）,修复后绿；反例格（错误参数绑定预期）按 T-01 矩阵 |
| V02 computed 面 | 同语料:视图 computed fn 调用断言 | T-01 判同根:修复后绿;判异根:显式跳过并登记边界 |
| V03 既有回归 | cargo t（.cargo aliases 默认层）+ 语料金样 | 零新增红 |
| V04 跨仓解阻 | auto-musk worktree b2285ac `probe-g15.mjs`（AUTO_EXE 锁本仓当日构建） | 生成进度标题+启动预览/验证行可见,SMOKE PASS |

## 7. 验收标准

| ID | 可观察结果 | 验证 |
|---|---|---|
| AC-01 | VM 渲染目标下,store handler 域调用模块级 fn（同文件+跨模块,多参含列表首参）返回真实值,参数逐位绑定;失败显式报错 | V01 |
| AC-02 | 视图 computed 内 fn 调用返回真实值,或 T-01 判异根且边界/后续指针已登记 | V01/V02 |
| AC-03 | auto-musk PLAN-097 T-05 零改动解阻:probe-g15 SMOKE PASS(生成进度+行内容可见) | V04 |
| AC-04 | 既有语料金样与测试面零新增红;web/vue 轨行为不变 | V03 |

## 8. 执行步骤

| 完成/任务 | 依赖 | 文件/符号与产出 | 验证与预期 | AC |
|---|---|---|---|---|
| [ ] T-01 | 无 | 语料级最小复现矩阵（新增 .at 语料+断言）+ 根因决策件（compile vs 运行时层;computed 同根判定）;落点与修复方案写入本计划修订或直接执行 | V01 矩阵红格可复现;根因层证据（符号表/CALL 执行迹） | 01,02 |
| [ ] T-02 | T-01 | 修复（crates/auto-lang 相应层;或 crates/auto-man 装载链） | V01 矩阵绿;V03 零新增红 | 01,02,04 |
| [ ] T-03 | T-02 | SD-01/SD-02 规范文本（可审稿;canonical 沉淀归 merge）+ computed 面处置（修复或边界登记） | V02 按判定结果绿/显式登记 | 02,04 |
| [ ] T-04 | T-02 | 跨仓复验:auto-musk worktree（PLAN-097 b2285ac）probe-g15 + smoke-t07,锁本仓当日构建;结果回填 auto-musk PLAN-097（T-05 解阻记录,由 auto-musk 侧会话落账） | V04 SMOKE PASS;auto-musk V02 全套 30/30 仍绿 | 03 |

## 9. 复审记录

### new 阶段交接（草稿准备完成）

- stage: new
- plan_id: PLAN-733
- plan_revision: 1
- outcome: pass
- next: work；T-01 有界调查先行（根因定位决策件）；证据包与最小复现
  已锚定（auto-musk PLAN-097 baseline 附页①③ + b2285ac worktree）
- changed_tasks: T-01～T-04
- changed_acceptance: AC-01～AC-04
- spec_delta: SD-01～SD-02
- evidence: §4 证据包;授权=用户在 auto-musk PLAN-097 会话明确指令立项
- review_authority: 独立 auto-plan:review 在本仓 lang-733 worktree 复验;
  跨仓复验(V04)证据由两侧会话分别落账

## 10. 待澄清事项

1. **computed 返空是否与 handler 域同根。** owner=T-01 有界调查;同根
   一并修（AC-02）,异根登记边界与后续计划指针,不扩本计划范围。
2. **在途计划交互。** master 正被 PLAN-728～732 活跃开发;work 期
   rebase 对齐,修复合回按本仓 AGENTS 依赖收尾规则尽快 ff。
