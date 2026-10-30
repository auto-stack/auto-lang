---
plan_id: PLAN-896
status: drafting
feature_name: 简化模块选测流程与统一并行执行
author: [Codex]
created_at: 2026-10-30
updated_at: 2026-10-31

plan_revision: 3
supersedes_spec_components: [docs/specs/auto-test-case/module-selection.md, docs/specs/auto-test-case/test-purpose-and-admission.md, docs/specs/auto-test-case/governance-and-inventory.md, docs/specs/aavm/project.md]
new_spec_components: []
touched_goals: []

affects: [auto-test-case, test-infrastructure]
depends_on: [PLAN-888, PLAN-895]
current_step: 0
total_steps: 12
---

# [PLAN-896] 简化模块选测流程与统一并行执行

## 0. 变更摘要

纠正“先全目录发现/构建、再自动扩大闭包、最后逐叶串行”的昂贵流程：日常入口直接指定职责模块或提供变更清单，先定相关范围，只发现/构建所需目标，再由原nextest统一并行调度。细叶/义务用于内部归属和结果核对，不成为使用者的选测前置工作或串行执行单元。保留真实消费者、原oracle、资源约束及完整目录审计能力，不再让全局审计成为每次局部验证的前置。局部发现/构建由在途897持有，896复用其接口，不另造框架。

用户已同意895建议的48个模块作为日常候选池，随后要求顶级模块内兼容用例像旧cargo t一样统一并行调度，并裁定AAVM/AA2R实验仅留代码、测试全部默认ignore。后者是明确批准的启用条件例外，不是按用途标签过滤。日常有效候选相应为47个，`lang/vm-aavm`保留历史导航/手动验证归属；精确身份仍须核定。本计划不启用固定默认包，不改变周期频率。r3回应用户对890“773模块/5569义务串行运行”流程的质疑；当前仅起草，待用户确认本r3后实施。

## 1. 目标

- G1：顶级模块全部适用用例按兼容执行合同统一并行调度，支持跨叶及跨已选顶级模块合并；不以叶为串行运行边界。除G5明确批准的ignore外，身份、测试义务及oracle不减。
- G2：对历史48候选模块的闭包给出可重建路径与用途分布，逐项审核895记录的37个跨顶级扩张及当前新增/消失项。修正有证据的过宽映射，保留真实消费者和证据不足的保守边。
- G3：选择、执行、计量及旧入口均消费相同新合同，覆盖正常、拒绝、失败及清理路径；首次红与未执行状态不被聚合绿覆盖。
- G4：给后续默认决策提供真实成本及闭包结论，不以本计划替代默认包准入决策。
- G5：AAVM/AA2R实验测试全部默认ignore，保留实现、测试源码/夹具/期望及显式手动重放；不波及普通VM、Rust实现A2R或真实产品消费者。
- G6：无需先运行list或手工分析叶；显式职责模块/变更清单直接通向相关目标发现与统一执行，无关源码漂移不触发全recipe发现/编译。完整全仓盘点独立于普通局部命令；复用897而不重复实施其目录失效/构建域机制。

非目标：删除代码/测试，或关闭G5范围外测试；按special/research/pending裁剪；批量注册895未注册入口；裁定P-050用途；修复旧红和ACE产品性能；推广888 B/C worker批量试点；改变正式样本/压力规模/金样/截止/资源组；引入第二套runner或裁判；修改UI/Man外仓；启用固定日常默认包或改周期频率。

## 2. 架构方案

复用`module_command.py → module_catalog.select/verify_selection → run_selected.py`的选择/执行职能，并按§2.1调整全局刷新前置。先按职责和真实影响定范围，在必要目标完成真实发现/完整性核验，再做执行分组：组是调度对象，不是新义务/新叶，不改变选择并集。优先合并nextest同recipe兼容叶；Python和case-suite先保留独立调用，除非T-02证明原session/副作用/oracle/收据逐项等价并给负控。未证明兼容的形态显式独立执行，不静默漏测。

顶级模块内兼容nextest用例汇总为一次调度，沿用原并发额度、资源组及HTTP/性能门必要串行约束；不是同时无限启动所有测试。多个已选模块及消费者的兼容用例也可共享调度。拆调用必须说明配置、argv或隔离原因，不能只减少少数启动却继续以叶串行作为默认。AAVM/AA2R先按原子身份显式ignore，再重新建立目录分区，不用整顶级模块黑名单代替源码标记。

闭包审计沿`impact_graph.build_graph → module_closure → select → verify_selection`，解释直接来源及传递边，区分源码变化影响、构建依赖、运行时/helper/集成依赖和保守回退。Cargo构建依赖不自动等于可删除的执行边；共享产品源变更继续拉起必要消费者。图修正须在生产端和消费端一致生效。

### 2.1 简化后的日常路径与897分工

使用者直接执行现有`cargo tm <顶级模块ID>`或`cargo tm --changes <路径清单>`；不要求先list，更不要求理解1207个叶。显式选择模块仍包含必要真实消费者，changes路径按真实影响选择；包级依赖不机械等价为“某个测试一变就运行依赖包的所有测试”。范围不确定须具名解释扩大原因，不能静默省略真实消费者，也不能把缺元数据默认转为所有ACE工程编译。

`--list`只读可用的声明/目录元数据；元数据过期显示状态和明确刷新入口，不暗中全量构建。执行所选目标时用真实发现核完整适用义务，保留新增/删除/改名测试的发现与消费端守恒；依赖完整性不靠接受旧目录假装成立。新树冷态、局部源码漂移、共享接口与新目标这些边界必须在最小端到端链验证后再迁移。

897 r3持有按相关输入的分片失效、先选后发现/构建、非ACE构建域隔离、target/profile/缓存存储；896持有兼容组并行执行、结果归属、消费者执行边审核与AAVM精确ignore。共享`module_command.py/module_catalog.py/impact_graph.py/prepare_tools.py`按实际符号串行交接，记录版本/接口/验证，不让两计划各自重写同一机制。897不必全计划先交付，但相关接口必须已具备并在T-12真实集成；未具备时该验收仍pending，不能沿用全局重发现就宣称r3完成。

全量目录发现、分区守恒、历史迁移对账在明确治理入口及必要周期执行；不要求每次局部验证重新发现全部测试。叶元数据继续可审计，但不构造“先跑773次再完成Web门”的工作流；审核773的来源，实际执行次数由兼容目标数和必要隔离决定，不由叶数决定。不增加另一套选择DSL、守护服务或裁判。

## 3. 技术栈

现有Python治理工具、Cargo/nextest及真实枚举/结果协议、Windows原生Job计量、execution_trace包装内部收据、TestCase原比较器。沿用LF目录序列化、内容哈希分片和源码/依赖/环境指纹，无需新增产品运行时协议。

## 4. 需求分析与背景调查

### 4.1 授权、状态与依赖

用户2026-10-30同意48模块作为日常候选池，并要求“新建一个计划，先做合并调用和闭包审核，保留真实消费者及原测试义务，不直接过滤专项或调研测试；计划写好后给我确认”。当时仅授权规划及簿记，r1未进入实施；现行合同为下述r2。无另行时间/自动续行预算。主检出他方WIP不纳入本计划。

2026-10-31补充授权：AAVM和AA2R为计划由ACE替代的历史尝试，仅保留代码，其测试全部默认ignore；此裁定覆盖此前AGENTS AAVM触面必测与本r1“忽略条件不变”在该精确实验范围内的要求。ACE替代是路线意图，不据此声称已完成等价替代或代码无人消费。本r2连同统一并行调度一起交确认，尚未开始实施。

r3新增需求：用户指出890会话目录7263义务/1207叶、Web闭包773模块/5569义务，并要求重新考虑复杂流程。本轮已核实际入口：`module_command.main`在list分支前先对全局`source_identity`判过期；`refresh`遍历全部recipes；`discover_modules.py`对各recipe真实列测试并可能构建。确认机制风险；890数字来自用户所给该会话状态，不冒称已在本主检出重建。新主检出已存在`897-experimental-build-storage-lifecycle.md` r3，局部发现与ACE构建隔离有明确owner，本r3整合分工；不擅自修改890/897合同或替其豁免冻结门。

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
| `docs/specs/aavm/project.md`、`crates/auto-lang/src/tests/aavm*.rs`、`test/parity/crates/auto-parity/src/aavm.rs` | 自举实验VM及Auto写的Rust转译器AA2R，不是普通Rust实现A2R；parity含显式五方矩阵 | T-11按真实源绑定审核root和目录外测试/入口，不按名字子串批量ignore |
| `docs/plans/897-experimental-build-storage-lifecycle.md` r3 | 在途draft：先选后发现、相关分片身份、ACE构建域隔离及存储生命周期 | 复用局部发现接口；构建与缓存工作不在896重复设计/实施 |

### 4.2 已观察与待证实

已观察：逐叶重复发现与调用；`AUTO_TEST_TRACE_MODULE`目前每次仅绑定一个叶；结果与资源按叶汇总；图含保守helper关联与case-suite未知引用回退。895开销/闭包数字为历史证据。

待证实：哪些叶可共享调用、nextest逐义务机器结果与包装trace如何双重绑定、哪些边确属过宽、合并后当前成本。不能预先宣称任一边错误或所有组都会低于60秒。T-02先用最小真实链关闭可行性，再扩面。

## 5. 详细设计

### 5.1 候选池边界

T-01冻结895历史48个ID及源哈希，复核20个普通lang、10个core、8个ace、5个case、4个infra，加历史条件`lang/vm-aavm`。按G5将AAVM/AA2R实验转默认忽略，候选池有效日常项47个；历史48及37闭包对账仍保留以解释差集。其他A2C/ignored/平台/里程碑条件原样保留。

本计划在规范/审计摘要登记批准事实，历史suggestion字段不成为执行开关；不新增无参数隐式全跑、不改legacy默认、不新增按用途过滤入口。

### 5.2 合并兼容键与逐项结果

1. T-02在现有runner内确定最小分组及审计收据（可新增`invocation_groups.py`，拟新增路径）。兼容键至少绑定workspace、recipe、runner、features/profile、nextest配置/过滤/ignored策略、有效环境/工具、适用条件、HTTP串行/资源组、截止与session/输出隔离约束。同名不同recipe不误去重；超Windows argv预算确定性拆组。
2. 对已选完整叶生成精确义务并集，真实发现集合必须与预期适用集合相等，拒绝漏项、重复和额外执行。未选义务NOT_RUN，条件排除NOT_APPLICABLE。发现仅在本次冻结身份内复用；源/依赖/配置/环境漂移失效。
3. nextest逐测试终态来自真实结构化输出/可靠原协议，不以整组exit0推断每叶PASS。包装输入/check仍由`execution_trace.py`实际child绑定校验；调用上下文必须无歧义映射回原叶/义务，错源/未知/重复/缺终态拒绝。Python/TestCase原比较器和case_artifacts沿用。
4. 失败、崩溃、超时、截断、取消保留已完成项真实结果，其余NOT_RUN/基础设施失败，整门非零。保首次尝试及独立重放argv，不能以重跑绿覆盖首红。不扩大原截止；不能证明原单项/组边界仍受约束的组合保持独立。60秒是成本目标，不是强杀时限。
5. 组计真实墙钟/CPU/Job峰值/启动数，叶记真实终态及组引用；共享成本不虚构为每叶独立实测，不把组CPU/墙钟重复累计。分摊值必须标估算，不用于宣称叶/顶级模块独立低于60秒。顶级成本以完整选面真实单轮为准，闭包额外成本另列。
6. nextest原进程隔离/配置/资源组、HTTP串行、heavy_gate单实例、Job/输出预算、源码前后身份与清理保持。同一nextest不自动证明资源语义等价。只回收本次明确所有权的进程/目录，不按映像名停止他方进程。

### 5.3 闭包逐项审核与最小修正

T-03交付历史48模块在当前身份下逐模块直接叶/义务、全部闭包叶/义务、额外顶级及用途分布，以及扩张路径、边类型、来源文件/符号/哈希、原规则和结论。895的37跨顶级模块逐一对账；循环/强连通分量、未知回退、多来源边不漏；路径可归并但边覆盖可重建。

结论三类：保留（真实消费者有证据）、修正（反例证明原映射过宽，正控证明真实消费者保留）、保守保留待证（具名原因和下一事实，不视作可删）。用途标签只解释报告；闭包缩小不是验收指标。

修正基于源/构建/运行时边界，在`impact_graph.py/impact_rules.json`及必要目录构建/消费处一致落地。不以文本“没引用”证明动态消费者不存在。保global_inputs、未知路径扩大、删除/重命名、manifest、optional/dev/platform、动态worker/fixture/helper与传递集成。每条删除/收窄边列原边、新规则、选择差集、正反控；证据不足维持旧行为。全部原义务仍登记且可达，oracle/profile/条件不变。

上段“条件不变”仅G5实验ignore为已批准例外；其真实消费边仍登记，默认选择可出现已忽略实验的NOT_APPLICABLE，不得通过删边隐藏。普通产品消费者继续执行。

### 5.3.1 AAVM/AA2R精确默认忽略范围

T-11枚举root的`aavm_runner_tests/aavm_at_mode_tests/aavm2_*`、其生成器/宏注册及`test/parity`等目录外真实AAVM/AA2R测试，给出逐ID源码归属、原ignore/条件、新原因和手动入口。当前目录`lang/vm-aavm`共23叶/138义务，120条已ignored；其中还混入`plan492_m2_tests::m2_primary_shorthand_bracket`与`plan492_m3_tests::m3_vue_text_content_arms`，不能对顶级模块无差别施加ignore。精确实验数量由T-11复核，包括未注册入口，不把parity全部459测试视为AAVM。

Rust测试在源码/生成源使用具原因的`#[ignore]`，不删`#[test]`/mod/feature及源码，不以feature默认关闭冒充ignore；自动任务不使用include-ignored/run-ignored绕过该决定。非Rust或手动矩阵保持显式手工入口，从自动调度中按精确身份登记禁用，非静默exit0。手动运行必须主动指定ignored实验及原feature/平台/资源约束，仍保原oracle/样本；原T3条件不自动解除。正常只选择这些已忽略项应报告未执行/NOT_APPLICABLE，不认证通过。

同步目录、冻结合同/分区、legacy导航及周期驱动中的适用性记录，逐条公开授权差集；保持原义务全集，仅启用状态变化。宏/生成测试从源头覆盖，重生成不能恢复默认启用。不修改普通VM/A2R及共享产品消费者测试；`packages/aavm/lib-legacy`等存在真实载荷消费者时保代码和相应产品验证。更新AGENTS中冲突AAVM触面要求和所属Specs，不能一边默认ignore一边仍宣称旧必测门已通过。

### 5.4 成本与交接

对895标记的24开销组逐组给出新分组/调用数变化/不可合并原因；对可合并组完成完整单轮成本，复用同身份有效证据。旧/新相同选面、冻结源码/依赖/环境下选小组、中组及接近argv预算组对照，分列准备/冷编译、发现、执行、比较/清理、整命令、CPU/峰值与背景负载。至少一个真实多叶组发现/执行启动次数严格减少且逐义务结果相等；超60秒或收益不显著具名归因，不以小样本宣称P90或改原收益门。

增加旧t同义务选面的单次nextest调度与逐叶调度对照，区分框架成本、实际慢例、覆盖增加与闭包成本；不预先承诺恢复50秒。并行验收以真实跨叶调度观察为证：至少一个无串行限制的多叶组观察到不同叶测试执行区间重叠；有资源约束组仍按原限制运行。AAVM/AA2R默认ignore带来的工作量减少单列，不计作合并/并行提速；同选面比较以相同适用性为前提，不为成本测量恢复已批准默认关闭的重实验。

闭包全量元数据审计，实际只测本计划触面和代表性真实消费者，不为横切runner重跑7165全目录，不隐含批准A/B/C完整矩阵或并发重型任务。

拟唯一实施树`D:/autostack/.wt/lang-896/auto-lang`、分支`plan-896-dev`；按分组脚本补必要auto-down兄弟，不建symlink/junction。当前T-01依赖用户确认，无实施项已完成。work先核HEAD/路径/893接口及目录身份；review在同一权威树独立上下文核验，期间停实施写入。用户未指定模型/代理分工，不自动创建其他chat/代理。

T-02最小链必须先证明“显式职责模块→只发现必要目标→统一并行→逐义务结果”；相关897接口未就绪时先闭合接口设计/可行性，不在旧全局刷新路径上全面扩建。T-12覆盖普通无ACE模块与真实必要ACE消费者两个方向，既不能无关全编，也不能伪造legacy标签漏掉真正ACE影响。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/module-selection.md | 全局刷新后逐叶执行→先选职责/真实影响、相关目标发现、兼容统一并行；全局审计与局部执行分离；原身份/真实消费者不漏 | 复用897局部发现，不再围绕细叶增加日常步骤 | AC-02/03/04/06/10 |
| SD-02 | modify | docs/specs/auto-test-case/test-purpose-and-admission.md | 895尚未人审建议→历史48已批准、AAVM实验默认ignore后有效日常47、固定默认包未启用；用途不筛选 | 沉淀本轮决定 | AC-01/04 |
| SD-03 | modify | docs/specs/auto-test-case/governance-and-inventory.md | 补分组守恒、图边变更证据及消费端反控 | 防聚合漏报与漏消费者 | AC-02/03/04/05 |
| SD-04 | modify | docs/specs/aavm/project.md | 实验按触面自动门→保代码/测试与手动入口、全部默认ignore；普通VM/A2R产品消费者保持 | 用户2026-10-31明确裁定；同步AGENTS对应要求 | AC-09 |

## 6. 测试设计

以下均拟执行，非已跑证据。work负责实现与反例，review独立验收全部AC/规范增量，merge持冻结模块并集门。

| case_id / owner_stage | AC / 不变量 | 输入/状态与实际入口 | 可观察oracle与区别性断言 | 边界/启用任务 |
|---|---|---|---|---|
| V-01 / work | AC-02 统一并行 | `run_selected.py`顶级模块多叶同recipe、顶级+叶混选、同名跨binary/recipe、长argv | 并集等于原路径且恰一次；兼容组一次统一调度、跨叶执行区间真实重叠；不兼容/argv拆组具名，非逐叶串行 | T-02/05夹具+真实nextest小组 |
| V-02 / work | AC-03 真终态 | 同组一绿一红、崩溃/超时、缺/重复/额外终态、包装child错源/漏check | 真实PASS/FAIL保留，未完NOT_RUN，整门非零；删child收据不能exit0冒绿 | `execution_trace.py`及消费端，T-05/06 |
| V-03 / work | AC-02/03/05 条件隔离 | 不同env/profile/ignored/milestone/session；HTTP、Job清理失败、身份漂移 | 不兼容不合并；原NOT_APPLICABLE不充PASS；资源串行/截止不弱化；漂移/残留失败 | nextest/Job真实代表组；Python/TestCase保原路，T-05/06 |
| V-04 / work | AC-04 真消费者 | `select/verify_selection`链/环/多路径、动态worker/helper、共享源、dev/optional/platform、global_inputs、删改名/未知路径 | 真消费者逐ID必选；删必要边/篡改选择在消费端失败；未知保守扩大 | 小图正反控+真实owner/fixture，T-03/04/06 |
| V-05 / work | AC-01/04 用途不删 | 同一真实special/research消费者换用途标签；有证据的过宽边 | 标签变化选择集合不变；修正差集可核，原义务可达；无可证修正则保留并报告 | `test_module_selection.py/test_test_purpose_policy.py`，T-04/06 |
| V-06 / work | AC-05 入口兼容 | tm显式/changes/legacy带滤串/schedule；`measure_modules.py`消费新结果 | 身份符合原合同或具名获证图修正；组/叶计数不混、资源不重记、首红不覆盖 | 选择全量对账+代表真实调用，不全跑旧档；T-06 |
| V-07 / work | AC-06 成本 | 冻结身份新旧多叶组与24开销组 | 启动/墙钟/CPU/峰值/阶段可核；完整义务不变；冷准备/未测/共享估算明确 | T-07，无统一P90/三warm/全局矩阵 |
| V-08 / work | AC-09 精确ignore | root与目录外实验逐ID/宏重生成、tm/legacy/periodic条件、普通VM/A2R及plan492正控、显式手动入口 | 已批准实验真实发现为ignored，自动入口不执行且不冒PASS；重生成不复活；原源码/oracle保留；普通消费者不受影响；显式ignored小例可运行 | T-11提供清单/消费端负控；不全跑自举矩阵 |
| V-09 / work | AC-10 局部直达 | 显式职责模块无需list，新树冷态、无关源码漂移、相关新增/删除/改名测试、真实共享接口影响ACE、只list/选择 | 实际子进程/构建argv证明仅发现/构建必要目标；list不构建；范围内新增测试不漏、错目录/漏消费者拒绝；未知范围具名扩大，真正ACE影响正确启用；不全recipe刷新 | T-02/12与897接口集成；不以伪造干净缓存或mock代替真实边界 |

治理入口：`python -m unittest discover -s scripts/tests -p test_module_selection.py`，对应`test_test_purpose_policy.py`及确有触面的`test_native_meter.py`。若新增分组专用测试文件，T-05接入现有治理recipe/消费者，不留下未受管唯一验证。

模块入口：896 worktree内用`cargo tm infra/selection infra/meter --select-only --out <local>`审实际闭包（meter仅真实触及时纳入），及`cargo tm --changes <changed-paths-file> --select-only --out <local>`核实际并集；去掉select-only才执行。包含真实消费者/集成/必要慢模块，不机械扩AAVM/UI/Man。目录守恒用实际枚举与负控，入口迁移用代表性执行/局部并集；冻结前明确最终选面及必要慢项，不仅凭治理绿认证全部recipe。

原始日志/截图/探针/转储本地保留不入库；长期摘要记命令、退出码、首次失败、数字、源身份、本地路径/SHA256。遵守14天保留、hold、预算与合并收尾先预览后执行的现行规则；未知owner不删。纯规划本轮不运行Rust测试/docs_gen。

## 7. 验收标准

| ID | owner_stage | 可观察完成条件 / 验证 |
|---|---|---|
| AC-01 | work | 固化48候选ID及批准事实/条件例外；无固定默认启用、用途过滤、注册或周期调整；配置diff和V-05核验 |
| AC-02 | work | 顶级内及多个已选模块的兼容用例统一并行调度，叶不作为串行边界；拆调用有合同/argv依据；真实跨叶重叠且调用数减少；原隔离/资源组保持；V-01/03 |
| AC-03 | work | 真终态与原包装收据闭合，缺项/崩溃/超时/残留不冒绿；独立重放与首红可追；V-02/03 |
| AC-04 | work | 48完整闭包与37历史项逐一对账；逐项来源路径和三类结论；图修正正反控/差集可核，真实/未知消费者保留，标签不影响选择；V-04/05 |
| AC-05 | work | tm显式/changes/legacy/schedule和计量等直接消费者兼容；生产/消费拒篡改，条件/资源/身份门保持；V-03/06 |
| AC-06 | work | 24组新分组/不可合并原因、可合并组完整单轮；代表组同身份新旧对照和资源/阶段统计，不伪造共享叶成本；V-07 |
| AC-07 | review | 独立核全部AC、遗漏/延后/workaround、warning/格式/debug、SD-01..04；债务候选入KNOWN-DEBT-AND-RISKS，不掩盖合同缺口；revision-bound pass |
| AC-08 | merge | 最终审查后冻结受影响完整模块/消费闭包门；规范及仓规沉淀、archive、留存预算、fresh guard clean后清理；周期仅到期触发 |
| AC-09 | work | AAVM/AA2R全范围逐ID对账，所有该实验测试默认ignore且自动入口不绕过，原源码/夹具/期望与显式重放保留；普通VM/Rust A2R/真实消费者和混入plan492测试不误关；合同/目录/候选状态与AGENTS/Specs一致，已忽略不充绿；V-08 |
| AC-10 | work | 职责模块/changes直接执行，无list/手工叶选择前置；局部命令无全recipe重发现/无关构建；范围内真实发现及消费端完整性保持；897相关接口身份与分工清晰且V-09真实通过，不新造缓存/runner/DSL |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 文件/操作、交付与验证 | AC |
|---|---|---|---|---|---|
| [ ] | T-01 | work | 用户确认r3 | 建唯一lang-896组树，绑HEAD/依赖/配置、893及897接口/写面；冻结历史48ID、当前分母、24/37映射及原合同，单列G5差集；核现有守恒审计 | AC-01/04/09/10 |
| [ ] | T-11 | work | T-01 | `crates/auto-lang/src/tests/aavm*.rs`及宏/生成源、`test/parity`等实验入口逐ID审核和ignore；同步`scripts/test-governance`目录/冻结合同/周期适用性和候选状态；拟新增`docs/plans/attachments/896/aavm-ignore-scope.md`记录边界/授权差集/手动入口；V-08，更新AGENTS和SD-04 | AC-09 |
| [ ] | T-02 | work | T-01 | 先核897局部发现接口，以`module_command.py`直达必要目标，再接`run_selected.py/recipe_runtime.py/execution_trace.py`双叶并行真链和终态/trace/截止/成本；V-01/02/09最小红绿。不能兼容者独立；关键链不可行则needs_replan，不在全局刷新上继续堆方案 | AC-02/03/10 |
| [ ] | T-03 | work | T-01 | `impact_graph.py/module_catalog.py/impact_rules.json/build_modules.py`闭包审核；拟新增`docs/plans/attachments/896/closure-audit.md`，记48/37对账/边来源/三类结论；机器路径本地 | AC-04 |
| [ ] | T-04 | work | T-03,T-11 | 实施有证据的最小图修正或保留，生产/消费同步、目录哈希及生成建议；V-04/05；拟新增`docs/plans/attachments/896/closure-deltas.md`逐边差集/正反控 | AC-01/04/05 |
| [ ] | T-05 | work | T-02 | 扩nextest兼容合并，按需新增`invocation_groups.py`；`run_selected.py/execution_trace.py/module_observations.py`终态/trace/资源；不兼容独立，V-01..03和unittest | AC-02/03/05 |
| [ ] | T-06 | work | T-04,T-05,T-11 | `module_command.py/measure_modules.py/audit_test_purposes.py`等直接消费者兼容；`scripts/tests/test_module_selection.py/test_test_purpose_policy.py`等反控；V-04..06/08及代表真实tm执行 | AC-01..05/09 |
| [ ] | T-12 | work | T-06及897相关接口可集成 | 按§2.1集成并验证局部职责入口；记录接口源/版本与实际子进程证据，V-09正反控；不在896重复实现897缓存/存储/构建域工作；未就绪不冒完成 | AC-10 |
| [ ] | T-07 | work | T-06,T-12 | 按5.4用`measure_modules.py/module_observations.py`完成旧t同选面对照、24组处置/局部成本与V-07；拟新增`docs/plans/attachments/896/cost-summary.md`；不全目录/全矩阵重跑 | AC-06/10 |
| [ ] | T-08 | work | T-07 | 完成SD-01..04源文档及执行摘要，逐AC（含AC-10）证据/未执行条件/旧红、G5精确授权差集、AGENTS冲突步骤同步、格式/warning/debug核验，实施提交冻结待审，status→execution_done | AC-01..06/09/10 |
| [ ] | T-09 | review | T-08 | 同权威树独立上下文auto-plan-review，核全部AC/V敏感性及SD/延期/债务，绑定修订/代码SHA；有finding回work，不直接修代码 | AC-07 |
| [ ] | T-10 | merge | T-09 pass | auto-plan-merge持冻结tm变更并集门、复用有效证据；按仓规规范/簿记沉淀、archive、留存预算/周期判断；guard clean后移除树/分支/空组 | AC-08 |

## 9. 复审记录

2026-10-30 / stage: new / plan_id: PLAN-896 / plan_revision: r1 / outcome: pass（起草交接自检，非实施复审） / authorization: 48候选池已批准；本r1实施待用户确认 / grounding: 当前Specs、实际runner/图/消费关口、895历史及893在途边界 / preflight: 10任务、8AC、7V、3SD覆盖；真实消费者/原义务不减，终态/成本有所有者，不确定机制先由T-02关闭 / next: 用户确认r1后work T-01；未建实施树、未运行测试、未修改产品/工具或规范源文件。

2026-10-31 / stage: new / plan_revision: r2 / outcome: pass（合同修订自检，非执行） / authorization: 用户新裁定AAVM/AA2R仅保代码、测试全部默认ignore；当前计划仍待确认 / changed: G1/G5、§2/5.1..5.4、AC-02/09、V-01/08、T-01/06/08/11、SD-04；统一跨叶并行不止减少调用；实验ignore为唯一新增启用例外，真实产品消费者保留 / evidence: 当前lang/vm-aavm 23叶/138义务/120已ignored且混入2条plan492，源码AA2R为Auto写的a2r，parity独立矩阵不是全部parity测试 / preflight: 11任务/9AC/8V/4SD，历史48→47有效日常候选另保历史导航，默认忽略节省不冒充调度提速 / next: 用户确认本r2后work；未改测试源码或默认状态。

2026-10-31 / stage: new / plan_revision: r3 / outcome: pass（规划自检） / trigger: 用户质疑890先全目录再773叶串行的复杂性 / changed: 摘要、G6、§2.1/4/5.4、SD-01、AC-10、V-09、T-01/02/07/12；日常直接职责选测，全局审计与局部执行分离，897持局部发现/构建，896持并行/执行闭包/精确ignore / evidence: 实际module_command全局身份失效→refresh遍历全部recipe，list在其后；897 r3已经认领该构建路径 / preflight: 12任务/10AC/9V/4SD，补真实冷态/失效/新增测试/真正ACE消费者控制，未改变890冻结门或忽略真实消费者 / next: 确认r3后work；本轮只改896计划。

## 10. 待澄清事项

待确认本r3；相关897接口尚未实施是具名工程依赖，不是同意本计划后再额外索取同范围批准的理由。兼容边界、终态、图修正、实验逐ID由T-02/03/11依证据裁定，不扩大ignore到普通产品测试。固定默认包、P-050、未注册入口新增注册、ACE产品优化在范围外；现存目录外AAVM/AA2R默认关闭属G5不能遗漏。新日常流程写入规范后也须同步本仓AGENTS中的冲突步骤；适用计划技能若有冲突列明维护交接，不在本计划扩展外仓写面，不另设运行前置。
