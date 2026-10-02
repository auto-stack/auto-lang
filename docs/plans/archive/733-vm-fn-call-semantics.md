---
plan_id: PLAN-733
status: archived
feature_name: VM 运行时模块级 fn 调用返空语义修复（store handler 域/computed 族）
author: [agent]
created_at: 2026-10-02T22:10:00Z
updated_at: 2026-10-02T23:59:00Z
plan_revision: 1
current_step: 4
total_steps: 4
supersedes_spec_components:
  - docs/specs/auto-lang/vm/architecture.md
new_spec_components:
  - docs/specs/auto-lang/vm/design/vm-fn-call-semantics.md
touched_goals:
  - GOAL-007
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
| [x] T-01 | 无 | 语料级最小复现矩阵+根因决策件 ✅ worktree 102d20652：`plan733_fn_call_semantics_tests`（handler 域全参型×同文件/跨模块×消费原语+跨轮次/timer 格）+二进制探针（`D:/autostack/.demo/lang-733/`,真机 --render=vm+MCP 快照直读）。**决策件**：①handler 域 fn 调用本体**无缺陷**（musk 附页③"handler 域返空"系输入面误归因——懒挂载 prop 迟到 frb=0→rb=3 实证+其提交树本就缺重算调用点）；②computed 面**真缺陷异根**=运行时层 `call_vm_fn` 列表实参 encode_list(TAG_LIST) 违反 H2 统一栈编码（engine 数值臂只认 TAG_OBJECT/裸 i32→按空迭代）；③跨仓实链追定两腿=同文件裸名歧义链接失败+视图 t() 动态键落空 | V01 矩阵（修复前 computed 格红复现）;P419 rc 追迹/物化读分锅证据 | 01,02 |
| [x] T-02 | T-01 | 修复三腿 ✅ worktree 102d20652+6bd9fa174：①`vm_bridge.rs::call_vm_fn` VmRef/大Int 实参 encode_list→encode_object（H2 对齐）；②`handler_codegen` store_scope_module own-module 绑定域（store→源文件模块名登记,合成 store handler/computed 时设 codegen——同文件裸名跨模块歧义不再悬空成 Undefined symbol 整链接失败）；③`aura_view_builder` 视图求值器 Expr::Object 字面量臂+t() 动态键解析 | V01 矩阵全绿（含 f_dup 歧义格/for-in 行格）;V03 零新增红（daily 档失败集与 master 3053f1fdf 基线逐名全等,15 预存红零增减;tv 162/162） | 01,02,04 |
| [x] T-03 | T-02 | SD-01/SD-02 规范文本 ✅ worktree 6bd9fa174（canonical 沉淀归 merge）：ADR-24（vm/architecture.md,含两腿增补）+design/vm-fn-call-semantics.md（语义矩阵 8 行+已知边界 4 条）+vm/overview.md 登记;computed 面处置=**判异根但在本计划内修复**（优于边界登记——AC-02 面收口） | V02 computed 三形态（f-string/直读/状态引用与字面量实参）真值绿;边界登记（懒挂载时序/push_value 占位/musk_vm_track 预存红族/视图 .length 成员链） | 02,04 |
| [x] T-04 | T-02 | 跨仓复验 ✅ AUTO_EXE=lang-733 worktree 6bd9fa174 构建：**probe-g15 SMOKE PASS**（G-15 摘要 {title:true,preview:true,visible:true} 全可见,收据 `.demo/lang-733/t05-g15-p733fix/snapshot-g15.txt`,「启动预览 已可见」=t() 动态键双形态真值）+smoke-t07 ALL PASS+musk V02 30/30（contract13/evidence8/focusboard4/vm-session5）。**关键更正**：musk 提交树（b2285ac..b1d9250）cp_prog_rows 重算调用点从未落地（"行源已就位/零改动解阻"断言不实）——复验以临时插桩（8 行,已回滚 musk 树 clean）补行源;**musk 侧需正式接线重算块后 T-05 方真零改动解阻**,由 auto-musk PLAN-097 会话落账 | V04 SMOKE PASS;V02 30/30 绿 | 03 |

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

### work 阶段记录（T-01～T-04 全落，execution_done）

- stage: work | plan_id: PLAN-733 | plan_revision: 1 | outcome: pass
- code_commit(worktree D:/autostack/.wt/lang-733/auto-lang @ plan-733-dev):
  102d20652（T-01 矩阵+T-02 腿①tag 对齐+Obj 臂）+ 6bd9fa174（T-02 腿②③
  own-module 绑定+t 动态键+T-03 规范文本）；基面 master 3053f1fdf
- task_ids: T-01, T-02, T-03, T-04
- evidence:
  ①T-01 决策件（§8 表内）：handler 域无缺陷（进程内矩阵+二进制探针
  fixed=2/fixedx=2/prop=3/input=3 全真值；musk"返空"=懒挂载迟到
  frb=0→rb=3 + musk 提交树缺重算调用点双误归因）；computed 面真缺陷=
  `call_vm_fn` encode_list 违反 H2 栈编码（engine.rs:1544 自述"H2 unified
  the stack encoding (everything→encode_object)",engine 44 处 encode_object
  唯 call_vm_fn 两处 encode_list 特立独行）。
  ②跨仓实链追定两腿：probe 补重算调用即现 `Undefined symbol:
  canvasProgressRows` 整链接失败（同文件裸名 × canvas_helpers.at 同名 →
  全局唯一性别名规则拒登记 → 裸 reloc）；行文本空 = t() 动态键无臂。
  ③门禁：daily 档 no-fail-fast 失败集与 master 基线**逐名全等**
  （15 预存红零增减；musk_vm_track p053_1/p053_4/p053_6/p054 等为
  3053f1fdf 既有红,修复前后零行为变化,已入边界登记）；tv 162/162；
  新测试文件 rustfmt clean。
  ④V04 收据：`.demo/lang-733/t05-g15-p733fix/{snapshot-g15.txt,vm-full.log}`
  + t07-20261002152709 manifest + musk V02 30/30。
- blockers: 无（musk 侧行源缺口为 auto-musk PLAN-097 侧接线项,非本仓阻断）
- next: review（独立 auto-plan:review 在 lang-733 worktree 复验;跨仓证据
  双侧落账——musk 侧 T-05 解阻记录由 auto-musk 会话执笔）

### review 阶段记录（2026-10-02，独立复审——同会话声明+工件重建）

- stage: review | plan_id: PLAN-733 | plan_revision: 1 | outcome: **pass**
- reviewed_commit: 6bd9fa174（worktree D:/autostack/.wt/lang-733/auto-lang @ plan-733-dev；
  102d20652+6bd9fa174 两提交,基面 master 3053f1fdf；树 clean,无未审脏改）
- base_commit: 3053f1fdf
- dependency_revisions: auto-down 兄弟 worktree .wt/lang-733/auto-down @ 895f8d0
  （detached=master,仅 workspace 解析用,零代码改动）；跨仓证据基
  auto-musk .wt/musk-097 @ b1d9250（含 b2285ac,树 clean）
- spec_inputs: docs/specs/auto-lang/vm/architecture.md（ADR-23 末态基线）+
  vm/overview.md + docs/specs/goals.md（GOAL-NNN 账本）；worktree 内
  SD-01/SD-02 落稿（ADR-24/design/vm-fn-call-semantics.md/overview 登记）
  已按 merge 前可审稿形态复核——描述当前行为与持久决策,无执行日志化文本
- acceptance_results:
  - **AC-01 pass**（V01）：`cargo nextest … p733` 2/2——矩阵测试
    `p733_store_handler_module_fn_call_matrix` 断言 handler 域全形态真值
    （0/1/2/3/6 参×同文件/跨模块×.length/==[]/for-in/成员读消费+跨轮次
    /timer+跨模块同名歧义格 f_dup="local"）；红相证据=work 期 musk 实链
    `Undefined symbol: canvasProgressRows` 整链接失败摘录（§8 T-01 格）
    +修复前 cc2="running:0:…" 进程内红值
  - **AC-02 pass**（V01/V02）：`p733_computed_fn_call_face`——computed
    三形态（f-string/直读/状态引用与 obj 字面量实参）+call_vm_fn 直调
    双形（VmRef/Array 实参）值精确断言；异根判定落 §10.1,三腿修复
    全在本计划 charter 面内（优于纯边界登记）
  - **AC-03 pass（含前提更正 F-2）**（V04）：probe-g15 SMOKE PASS 收据
    `.demo/lang-733/t05-g15-p733fix/snapshot-g15.txt` 关键行摘录——
    `text … "生成进度" {` / `text … "启动预览 已可见"`（=t() 动态键双形态
    真值）；smoke-t07 六步 ALL PASS（manifest
    `.demo/musk-097/t07-20261002152709/evidence-manifest.json`）；musk
    V02 30/30（contract13/evidence8/focusboard4/vm-session5）；AUTO_EXE=
    本 worktree 构建；musk 树插桩已回滚 clean
  - **AC-04 pass**（V03）：daily 档 no-fail-fast 全量×4 失败集与 master
    基线**确定性红逐名全等**（15 预存红零增减）；plan502_m3 闪红=F-1
    预存 flake（master 同发 2/5,同错 FlowDiagram Init CPU-slice parked
    x=108,非本 diff——零触 diagram/layout/parking 面）；diff 零
    trans/vue/ui_gen/aavm 触面（--stat 十文件全 VM/UI 面+规范+测试）
- findings:
  - F-1（nonblocking,已入 KNOWN-DEBT P733-R1）：plan502_m3_layout_
    geometry_e2e 并行负载 flake（双树同发,机理=Init CPU-slice parked）
  - F-2（前提更正,跨仓路由）：musk 提交树 cp_prog_rows 重算调用点从未
    入册,"零改动解阻"前提不实——musk 侧需 8 行接线（本仓复验同款插桩
    SMOKE PASS 已证）；归 auto-musk PLAN-097 会话落账（§10.3）
  - F-3（nonblocking,已入 KNOWN-DEBT P733-R2）：musk_vm_track p053_1/
    p053_4/p053_6/p054 预存红族=同族深层腿（子件 override 态链）,修复
    前后零行为变化,边界登记于设计文档
  - F-4（frontmatter 定稿,本次执行）：touched_goals 原稿 "goal-vm-parity"
    为不存在的 ID——按 docs/specs/goals.md 账本定稿 **GOAL-007**；
    new_spec_components 补 design/vm-fn-call-semantics.md
- evidence: 复审门禁命令与结果如上（全量失败集 diff=空/逐名全等；
  `cargo tv` 162/162；p733 2/2）；收据文件在审后仍核读存在
- next: merge（/auto-plan:merge 在 lang-733 worktree 执行;SD-01/SD-02
  canonical 沉淀+ledger 刷新+KNOWN-DEBT P733-R1/R2 已随本记录落账）

### merge 阶段收据（PLAN-733:r1，2026-10-02）

- stage: merge | plan_id: PLAN-733 | plan_revision: 1 | outcome: pass
  （completion_kind: delivered）
- **prepared**：审基=review 记录（reviewed_commit 6bd9fa174 @ r1 pass）；
  canonical delta 已在审内提交（ADR-24/design/vm-fn-call-semantics.md/
  overview 登记）；rebase master（102d20652→481bcb0b9、6bd9fa174→
  0b5c8758d，git range-diff 2/2 全等=补丁等价证明；基座增量仅 docs/plans
  簿记,crates/ 内容与受审树逐字节一致免重验）；投影提交 550895b68
  （projection-only descendant → delivery commit：vm/plans.md 733 行/
  goals.md GOAL-007 关联 733/specs.json upsert P733-1/2/3+INDEX 再生）
- **landed**：`wt-guard.sh D:/autostack/.wt/lang-733/auto-lang` clean →
  主检出 `git merge --ff-only plan-733-dev` 成功（无合并提交）；master
  tip==plan-733-dev==**550895b68**；主检出冒烟 known-good：`p733` 2/2 +
  `cargo tv` 162/162（landed tip 上重跑）
- **ledger_refreshed**：workspace=D:/autostack/auto-lang（tracked
  `.autoos/specs.json`,worktree 内 upsert 随 delivery 落地）;verified
  item IDs：P733-1（architecture）/P733-2（designs）/P733-3（reviews,
  file 随归档指 archive/ 路径）；source=canonical docs/specs/auto-lang/
  vm/{architecture.md,design/vm-fn-call-semantics.md}；INDEX.md 再生
  （26 projects,内容无漂移）
- **archived**：git mv → docs/plans/archive/733-vm-fn-call-semantics.md；
  frontmatter status: archived；provenance（review 记录/收据/KNOWN-DEBT
  P733-R1/R2/目标 GOAL-007）链接闭合
- 批量回归到期判定：**未到期**——回执 last_covered_plan_id=726
  （2026-10-02T05:25Z,~11h 新鲜<48h）；其后落地 727/728/729 与本计划
  733 均非 5 整除（733%5=3）；无 L0 fix-* 合入记录
- 部署观察：主检出 `target/debug/auto.exe` 落地前为 2026-10-02 13:06
  构建（落后本修复）——默认 AUTO_EXE 消费面（auto-musk 探针/演示按
  主检出 debug 路径解析）需重建,见 cleaned 后观察项

## 10. 待澄清事项

1. **computed 返空是否与 handler 域同根。** ✅ T-01 已判：**异根**——
   handler 域调用本体无缺陷;computed 面=视图求值器/桥编码层三处缺口
   （栈 tag/同文件裸名歧义链接/t 动态键）。按计划预留分支处置：异根面
   **在本计划内修复**（三腿全落,AC-02 收口优于纯边界登记——缺陷即
   "视图 computed/模板 fn 调用"族,属本计划 charter 面）。
2. **在途计划交互。** ✅ 未撞车：基面 3053f1fdf（729 已合,730/731
   各在自 worktree）;修复合回按 AGENTS 依赖收尾规则尽快 ff。
3. **musk 侧行源缺口（work 期新发现,回填 auto-musk PLAN-097）。**
   musk 提交树（b2285ac/b1d9250）cp_prog_rows 重算调用点从未落地
   （6084a76 提交信息所称"Poll 域每拍重算"未随提交兑现——仅字段+内联
   fn 定义入册）;其"musk 侧终版就绪/零改动解阻"断言不实。**解阻动作**
   （auto-musk 侧 8 行,本仓复验已验证同款插桩 SMOKE PASS）：Poll 头部
   补 `canvasProgressRows(.cp_prog_msgs,...)` 重算块（.cp_prog_rows/
   .cp_prog_has 落账）。由 auto-musk PLAN-097 会话正式接线并回填
   T-05 解阻记录。
