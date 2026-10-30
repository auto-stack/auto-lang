---
plan_id: PLAN-896
status: drafting
feature_name: 测试合并调用与消费者闭包审核
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-30

plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/test-purpose-and-admission.md, docs/specs/auto-test-case/governance-and-inventory.md]
new_spec_components: []
touched_goals: []

affects: [auto-test-case, test-infrastructure]
depends_on: [PLAN-888, PLAN-895]
current_step: 0
total_steps: 10
---

# [PLAN-896] 测试合并调用与消费者闭包审核

## 0. 变更摘要

在888唯一义务/叶模块和895顶级职责模块之上，减少兼容叶的重复发现与运行器启动，逐条审核消费者闭包扩张的来源及必要性。以相同输入的义务守恒、逐项真实终态、生命周期/资源约束和真实消费者不漏为交付标准；用途标签不参与执行过滤。

用户已同意895建议的48个模块作为日常候选池。本计划仅登记该决定、完成合并调用及闭包审核；不启用一个固定的48模块默认包，不改变现有周期触发、注册或忽略条件。当前仅起草，待用户确认本r1后实施。

## 1. 目标

- G1：合并同执行合同的兼容叶，降低逐叶发现/启动开销；顶级/叶/原子身份不变，原测试实际执行及oracle不减。
- G2：对全部48候选模块的闭包给出可重建路径与用途分布，逐项审核895记录的37个跨顶级扩张及当前新增/消失项。修正有证据的过宽映射，保留真实消费者和证据不足的保守边。
- G3：选择、执行、计量及旧入口均消费相同新合同，覆盖正常、拒绝、失败及清理路径；首次红与未执行状态不被聚合绿覆盖。
- G4：给后续默认决策提供真实成本及闭包结论，不以本计划替代默认包准入决策。

非目标：删除/关闭/ignore测试；按special/research/pending裁剪；批量注册895未注册入口；裁定P-050用途；修复旧红和ACE产品性能；推广888 B/C worker批量试点；改变正式样本/压力规模/金样/截止/资源组；引入第二套runner或裁判；修改UI/Man外仓；启用日常默认包或改周期频率。

## 2. 架构方案

继续使用`module_command.py → module_catalog.select/verify_selection → run_selected.py`真实链。选择合同先闭合，再做执行分组：组是调度对象，不是新义务/新叶，不改变选择并集。优先合并nextest同recipe兼容叶；Python和case-suite先保留独立调用，除非T-02证明原session/副作用/oracle/收据逐项等价并给负控。未证明兼容的形态显式独立执行，不静默漏测。

闭包审计沿`impact_graph.build_graph → module_closure → select → verify_selection`，解释直接来源及传递边，区分源码变化影响、构建依赖、运行时/helper/集成依赖和保守回退。Cargo构建依赖不自动等于可删除的执行边；共享产品源变更继续拉起必要消费者。图修正须在生产端和消费端一致生效。

## 3. 技术栈

现有Python治理工具、Cargo/nextest及真实枚举/结果协议、Windows原生Job计量、execution_trace包装内部收据、TestCase原比较器。沿用LF目录序列化、内容哈希分片和源码/依赖/环境指纹，无需新增产品运行时协议。

## 4. 需求分析与背景调查

### 4.1 授权、状态与依赖

用户2026-10-30同意48模块作为日常候选池，并要求“新建一个计划，先做合并调用和闭包审核，保留真实消费者及原测试义务，不直接过滤专项或调研测试；计划写好后给我确认”。本轮仅规划及计划簿记，本r1实施尚待确认。无另行时间/自动续行预算。主检出他方WIP不纳入本计划。

已读`docs/specs/overview.md`；其总览含早期快照，测试细则以当前模块Specs及实际源码为准。规划主检出HEAD为`e2b037cba8f248239d5e15da99ce052124370d2a`，work须重新绑定当时源/依赖/配置，不把规划SHA当最终门身份。

| 已核实来源 | 版本/事实 | 本计划使用方式 |
|---|---|---|
| `docs/specs/auto-test-case/module-selection.md` | SHA256 `d7bd3c2188a84ab7cd1936ed1b98db0bc76f2210c4662e538a74d8e034357418` | 完整叶/消费者并集、消费端复核、资源和条件不变 |
| `docs/specs/auto-test-case/test-purpose-and-admission.md` | SHA256 `4cf0d7beb2a5dff572f08e428d4e598963fec19a603ec5c8f934480a8b06dad3` | 侧表不进入筛选；候选池与默认启用分离 |
| `docs/specs/auto-test-case/batch-execution.md` | 888 A/B/C合同 | 本计划是外层调用合并，不改变worker试点合同 |
| `scripts/test-governance/run_selected.py` | SHA256 `84aa1efe9936c94265a5433af3b9d16c99802a2468e6442780a1097030cdab41` | 已观察逐叶循环、发现/调用、trace绑定和资源汇总，存在Windows argv上限 |
| `scripts/test-governance/impact_graph.py` | SHA256 `5da9e79705dd72a9a4c928851c3499231a82dfbb35c8946a4e6c9f6fc32de022` | Cargo dev/optional/platform依赖、helper引用、runtime、case-suite保守回退共同建图 |
| `scripts/test-governance/module_catalog.py` | SHA256 `78d0e104d4a3c608091f16aef2ae9ceffdcc8dcc34b77765d471b8932de394c7` | `module_closure/select/verify_selection`是生产与消费关口 |
| `scripts/test-governance/{top-level-modules,daily-admission-proposal,test-purpose-policy}.json`、895归档 | 历史66顶级/1181叶/7165义务；recommend 48；37跨顶级扩张；24组开销主导超60秒 | T-01重核当前分母；历史数不是新上限或全目录重跑理由 |
| `docs/plans/893-ace-test-framework-and-slow-case-attribution.md` | 在途draft r2：ACE接入/13慢原子归因 | 不接管其产品阶段归因/新入口；共享选择/计量接口需核对，不覆盖他方WIP |

### 4.2 已观察与待证实

已观察：逐叶重复发现与调用；`AUTO_TEST_TRACE_MODULE`目前每次仅绑定一个叶；结果与资源按叶汇总；图含保守helper关联与case-suite未知引用回退。895开销/闭包数字为历史证据。

待证实：哪些叶可共享调用、nextest逐义务机器结果与包装trace如何双重绑定、哪些边确属过宽、合并后当前成本。不能预先宣称任一边错误或所有组都会低于60秒。T-02先用最小真实链关闭可行性，再扩面。

## 5. 详细设计

### 5.1 候选池边界

T-01冻结895 `daily_admission=recommend`完整48个ID及源哈希，复核20个普通lang、10个core、8个ace、5个case、4个infra，加1个条件`lang/vm-aavm`。候选池不是每次必跑清单；AAVM按AGENTS真实触面规则，A2C/ignored/平台/里程碑条件原样保留。

本计划在规范/审计摘要登记批准事实，历史suggestion字段不成为执行开关；不新增无参数隐式全跑、不改legacy默认、不新增按用途过滤入口。

### 5.2 合并兼容键与逐项结果

1. T-02在现有runner内确定最小分组及审计收据（可新增`invocation_groups.py`，拟新增路径）。兼容键至少绑定workspace、recipe、runner、features/profile、nextest配置/过滤/ignored策略、有效环境/工具、适用条件、HTTP串行/资源组、截止与session/输出隔离约束。同名不同recipe不误去重；超Windows argv预算确定性拆组。
2. 对已选完整叶生成精确义务并集，真实发现集合必须与预期适用集合相等，拒绝漏项、重复和额外执行。未选义务NOT_RUN，条件排除NOT_APPLICABLE。发现仅在本次冻结身份内复用；源/依赖/配置/环境漂移失效。
3. nextest逐测试终态来自真实结构化输出/可靠原协议，不以整组exit0推断每叶PASS。包装输入/check仍由`execution_trace.py`实际child绑定校验；调用上下文必须无歧义映射回原叶/义务，错源/未知/重复/缺终态拒绝。Python/TestCase原比较器和case_artifacts沿用。
4. 失败、崩溃、超时、截断、取消保留已完成项真实结果，其余NOT_RUN/基础设施失败，整门非零。保首次尝试及独立重放argv，不能以重跑绿覆盖首红。不扩大原截止；不能证明原单项/组边界仍受约束的组合保持独立。60秒是成本目标，不是强杀时限。
5. 组计真实墙钟/CPU/Job峰值/启动数，叶记真实终态及组引用；共享成本不虚构为每叶独立实测，不把组CPU/墙钟重复累计。分摊值必须标估算，不用于宣称叶/顶级模块独立低于60秒。顶级成本以完整选面真实单轮为准，闭包额外成本另列。
6. nextest原进程隔离/配置/资源组、HTTP串行、heavy_gate单实例、Job/输出预算、源码前后身份与清理保持。同一nextest不自动证明资源语义等价。只回收本次明确所有权的进程/目录，不按映像名停止他方进程。

### 5.3 闭包逐项审核与最小修正

T-03交付当前48模块逐模块直接叶/义务、全部闭包叶/义务、额外顶级及用途分布，以及扩张路径、边类型、来源文件/符号/哈希、原规则和结论。895的37跨顶级模块逐一对账；循环/强连通分量、未知回退、多来源边不漏；路径可归并但边覆盖可重建。

结论三类：保留（真实消费者有证据）、修正（反例证明原映射过宽，正控证明真实消费者保留）、保守保留待证（具名原因和下一事实，不视作可删）。用途标签只解释报告；闭包缩小不是验收指标。

修正基于源/构建/运行时边界，在`impact_graph.py/impact_rules.json`及必要目录构建/消费处一致落地。不以文本“没引用”证明动态消费者不存在。保global_inputs、未知路径扩大、删除/重命名、manifest、optional/dev/platform、动态worker/fixture/helper与传递集成。每条删除/收窄边列原边、新规则、选择差集、正反控；证据不足维持旧行为。全部原义务仍登记且可达，oracle/profile/条件不变。

### 5.4 成本与交接

对895标记的24开销组逐组给出新分组/调用数变化/不可合并原因；对可合并组完成完整单轮成本，复用同身份有效证据。旧/新相同选面、冻结源码/依赖/环境下选小组、中组及接近argv预算组对照，分列准备/冷编译、发现、执行、比较/清理、整命令、CPU/峰值与背景负载。至少一个真实多叶组发现/执行启动次数严格减少且逐义务结果相等；超60秒或收益不显著具名归因，不以小样本宣称P90或改原收益门。

闭包全量元数据审计，实际只测本计划触面和代表性真实消费者，不为横切runner重跑7165全目录，不隐含批准A/B/C完整矩阵或并发重型任务。

拟唯一实施树`D:/autostack/.wt/lang-896/auto-lang`、分支`plan-896-dev`；按分组脚本补必要auto-down兄弟，不建symlink/junction。当前T-01依赖用户确认，无实施项已完成。work先核HEAD/路径/893接口及目录身份；review在同一权威树独立上下文核验，期间停实施写入。用户未指定模型/代理分工，不自动创建其他chat/代理。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/module-selection.md | 逐叶执行→兼容调用组、逐义务终态、成本归属与闭包证据；原身份/消费者/条件不减 | 降开销不降认证粒度 | AC-02/03/04/06 |
| SD-02 | modify | docs/specs/auto-test-case/test-purpose-and-admission.md | 895尚未人审建议→48候选池已批准、固定默认包未启用；用途不筛选 | 沉淀本轮决定 | AC-01/04 |
| SD-03 | modify | docs/specs/auto-test-case/governance-and-inventory.md | 补分组守恒、图边变更证据及消费端反控 | 防聚合漏报与漏消费者 | AC-02/03/04/05 |

## 6. 测试设计

以下均拟执行，非已跑证据。work负责实现与反例，review独立验收全部AC/规范增量，merge持冻结模块并集门。

| case_id / owner_stage | AC / 不变量 | 输入/状态与实际入口 | 可观察oracle与区别性断言 | 边界/启用任务 |
|---|---|---|---|---|
| V-01 / work | AC-02 合并精确 | `run_selected.py`双叶同recipe、顶级+叶混选、同名跨binary/recipe、长argv | 实际枚举/执行并集等于原路径，恰一次；跨合同不合并，argv有界拆组；真实调用数下降 | T-02/05夹具+真实nextest小组 |
| V-02 / work | AC-03 真终态 | 同组一绿一红、崩溃/超时、缺/重复/额外终态、包装child错源/漏check | 真实PASS/FAIL保留，未完NOT_RUN，整门非零；删child收据不能exit0冒绿 | `execution_trace.py`及消费端，T-05/06 |
| V-03 / work | AC-02/03/05 条件隔离 | 不同env/profile/ignored/milestone/session；HTTP、Job清理失败、身份漂移 | 不兼容不合并；原NOT_APPLICABLE不充PASS；资源串行/截止不弱化；漂移/残留失败 | nextest/Job真实代表组；Python/TestCase保原路，T-05/06 |
| V-04 / work | AC-04 真消费者 | `select/verify_selection`链/环/多路径、动态worker/helper、共享源、dev/optional/platform、global_inputs、删改名/未知路径 | 真消费者逐ID必选；删必要边/篡改选择在消费端失败；未知保守扩大 | 小图正反控+真实owner/fixture，T-03/04/06 |
| V-05 / work | AC-01/04 用途不删 | 同一真实special/research消费者换用途标签；有证据的过宽边 | 标签变化选择集合不变；修正差集可核，原义务可达；无可证修正则保留并报告 | `test_module_selection.py/test_test_purpose_policy.py`，T-04/06 |
| V-06 / work | AC-05 入口兼容 | tm显式/changes/legacy带滤串/schedule；`measure_modules.py`消费新结果 | 身份符合原合同或具名获证图修正；组/叶计数不混、资源不重记、首红不覆盖 | 选择全量对账+代表真实调用，不全跑旧档；T-06 |
| V-07 / work | AC-06 成本 | 冻结身份新旧多叶组与24开销组 | 启动/墙钟/CPU/峰值/阶段可核；完整义务不变；冷准备/未测/共享估算明确 | T-07，无统一P90/三warm/全局矩阵 |

治理入口：`python -m unittest discover -s scripts/tests -p test_module_selection.py`，对应`test_test_purpose_policy.py`及确有触面的`test_native_meter.py`。若新增分组专用测试文件，T-05接入现有治理recipe/消费者，不留下未受管唯一验证。

模块入口：896 worktree内用`cargo tm infra/selection infra/meter --select-only --out <local>`审实际闭包（meter仅真实触及时纳入），及`cargo tm --changes <changed-paths-file> --select-only --out <local>`核实际并集；去掉select-only才执行。包含真实消费者/集成/必要慢模块，不机械扩AAVM/UI/Man。目录守恒用实际枚举与负控，入口迁移用代表性执行/局部并集；冻结前明确最终选面及必要慢项，不仅凭治理绿认证全部recipe。

原始日志/截图/探针/转储本地保留不入库；长期摘要记命令、退出码、首次失败、数字、源身份、本地路径/SHA256。遵守14天保留、hold、预算与合并收尾先预览后执行的现行规则；未知owner不删。纯规划本轮不运行Rust测试/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察完成条件 / 验证 |
|---|---|---|
| AC-01 | work | 固化48候选ID及批准事实/条件例外；无固定默认启用、用途过滤、注册或周期调整；配置diff和V-05核验 |
| AC-02 | work | 兼容键/拆组/共享成本明确；真实多叶调用数减少；义务/profile/oracle/条件/隔离守恒；V-01/03 |
| AC-03 | work | 真终态与原包装收据闭合，缺项/崩溃/超时/残留不冒绿；独立重放与首红可追；V-02/03 |
| AC-04 | work | 48完整闭包与37历史项逐一对账；逐项来源路径和三类结论；图修正正反控/差集可核，真实/未知消费者保留，标签不影响选择；V-04/05 |
| AC-05 | work | tm显式/changes/legacy/schedule和计量等直接消费者兼容；生产/消费拒篡改，条件/资源/身份门保持；V-03/06 |
| AC-06 | work | 24组新分组/不可合并原因、可合并组完整单轮；代表组同身份新旧对照和资源/阶段统计，不伪造共享叶成本；V-07 |
| AC-07 | review | 独立核全部AC、遗漏/延后/workaround、warning/格式/debug、SD-01..03；债务候选入KNOWN-DEBT-AND-RISKS，不掩盖合同缺口；revision-bound pass |
| AC-08 | merge | 最终审查后冻结受影响完整模块/消费闭包门；规范及仓规沉淀、archive、留存预算、fresh guard clean后清理；周期仅到期触发 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 文件/操作、交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 用户确认r1 | 建唯一lang-896组树，绑HEAD/依赖/配置、893接口；冻结48ID、当前分母、24/37历史映射与原合同；核`audit_test_purposes.py --help`后运行守恒审计 | AC-01/04 |
| [ ] | T-02 | work | T-01 | `run_selected.py/recipe_runtime.py/execution_trace.py/module_observations.py`最小双叶nextest真链、机器终态/trace/兼容键/截止/成本；V-01/02最小红绿。不能证明兼容者独立；连nextest多叶不可行则needs_replan，不空交报告 | AC-02/03 |
| [ ] | T-03 | work | T-01 | `impact_graph.py/module_catalog.py/impact_rules.json/build_modules.py`闭包审核；拟新增`docs/plans/attachments/896/closure-audit.md`，记48/37对账/边来源/三类结论；机器路径本地 | AC-04 |
| [ ] | T-04 | work | T-03 | 实施有证据的最小图修正或保留，生产/消费同步、目录哈希及生成建议；V-04/05；拟新增`docs/plans/attachments/896/closure-deltas.md`逐边差集/正反控 | AC-01/04/05 |
| [ ] | T-05 | work | T-02 | 扩nextest兼容合并，按需新增`invocation_groups.py`；`run_selected.py/execution_trace.py/module_observations.py`终态/trace/资源；不兼容独立，V-01..03和unittest | AC-02/03/05 |
| [ ] | T-06 | work | T-04,T-05 | `module_command.py/measure_modules.py/audit_test_purposes.py`等直接消费者兼容；`scripts/tests/test_module_selection.py/test_test_purpose_policy.py`等反控；V-04..06及代表真实tm执行 | AC-01..05 |
| [ ] | T-07 | work | T-06 | 按5.4用`measure_modules.py/module_observations.py`完成24组处置/局部成本与V-07；拟新增`docs/plans/attachments/896/cost-summary.md`；不全目录/全矩阵重跑 | AC-06 |
| [ ] | T-08 | work | T-07 | 完成SD-01..03源文档及执行摘要，逐AC证据/未执行条件/旧红、格式/warning/debug核验，实施提交冻结待审，status→execution_done | AC-01..06 |
| [ ] | T-09 | review | T-08 | 同权威树独立上下文auto-plan-review，核全部AC/V敏感性及SD/延期/债务，绑定修订/代码SHA；有finding回work，不直接修代码 | AC-07 |
| [ ] | T-10 | merge | T-09 pass | auto-plan-merge持冻结tm变更并集门、复用有效证据；按仓规规范/簿记沉淀、archive、留存预算/周期判断；guard clean后移除树/分支/空组 | AC-08 |

## 9. 复审记录

2026-10-30 / stage: new / plan_id: PLAN-896 / plan_revision: r1 / outcome: pass（起草交接自检，非实施复审） / authorization: 48候选池已批准；本r1实施待用户确认 / grounding: 当前Specs、实际runner/图/消费关口、895历史及893在途边界 / preflight: 10任务、8AC、7V、3SD覆盖；真实消费者/原义务不减，终态/成本有所有者，不确定机制先由T-02关闭 / next: 用户确认r1后work T-01；未建实施树、未运行测试、未修改产品/工具或规范源文件。

## 10. 待澄清事项

无需要用户另选的技术前置；兼容边界、终态协议、图修正由T-02/03依证据裁定，不得削弱合同。待确认本r1本身。固定默认包、P-050、未注册入口、ACE产品优化均在范围外，不能作为本合同漏项延期容器。
