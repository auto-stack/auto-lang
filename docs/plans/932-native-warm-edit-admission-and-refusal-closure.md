---
plan_id: PLAN-932
status: reviewed
feature_name: Native暖编辑准入与拒绝分类闭环
author: [Codex]
created_at: 2026-11-04
updated_at: 2026-11-04
plan_revision: 2
supersedes_spec_components: [docs/specs/auto-ac/function-incremental-source.md, docs/specs/auto-ac/native-source-diagnostics.md, docs/specs/auto-ac/mir-native-cache.md, docs/specs/auto-test-case/profile-v2-native.md, docs/specs/auto-test-case/m1-order-layout-observations.md]
new_spec_components: [docs/specs/auto-test-case/native-warm-edit-admission.md]
touched_goals: [GOAL-002]
affects: [auto-ac, auto-av, auto-test-case]
depends_on: [PLAN-818, PLAN-828, PLAN-911, PLAN-914]
current_step: 10
total_steps: 12
---

# [PLAN-932] Native暖编辑准入与拒绝分类闭环

## 0. 变更摘要

复核D-927-2的合法body准入、typed拒绝保真及父判/旧测试合同漂移。原收据不能证明合法body与故意签名拒绝同因；914已批准有效typed策略拒绝作为diagnostic verdict（覆盖所有增量mn recipe），不得恢复为一律ERROR。保合法编辑真函数增量；真签名继续拒、旧态完整；有效策略拒绝按原声明预期PASS/FAIL，缺失/未知/矛盾typed事实及协议/资源/infra才ERROR。先重建实际输入再最小修复，不预设必须改core。

依据[下一波调查与写面](reports/ace-next-wave-correctness-and-admission-2026-11-04.md)。drafting r2 0/12；只规划，不施工/建树/运行测试。927的review pass不自动关闭此基座债，928/929原职责不变。

## 1. 目标

- 真实scalar函数body编辑按现有签名/声明/模式/依赖不变量准入，不能用位置索引/拼写漂移制造假接口变化。
- 错误发生处输出typed原因，拒绝/语言诊断/资源/infra在worker、协议、父侧保持事实与来源，不能读message/last-error猜分类。
- 拒绝候选不发布，旧执行态/版本/缓存保持；后续合法修复可继续。
- 真正函数级增量、恢复重验、连续父观察、原资源/时间门与两处t11及914已批准策略判面一致。

非目标：不增加签名编辑快路径或新语法，不把929的type/enum增量迁移塞入本包，不改变Auto类型/参数规则/NA/expected，不重写codegen/runtime或抬P90。

## 2. 架构方案

真实链：core retained CheckedEnv候选→AV scalar session prepare/commit→native单元cache/link/真实PE→mn-worker→ac_mn/strict父侧→TestCase状态。准入比较当前受检语义接口与其绑定，不以源码文本/节点号替代名义/模式/签名事实；哪个比较或分类造成原红由T-02实际定位，未知原因不预填修复。

`source.incremental`代码本身不自动证明全部是signature drift；如存在多种拒绝原因，采用显式typed原因/原操作快照传递或等价能区分机制，兼容旧人类消息面不反向驱动决策。911原诊断事实边界继续生效，资源态不得变语言错误。

## 3. 技术栈

现core/source retained环境、av/session、native/incremental/source_diagnostics/worker、auto-test-case/语言ac_mn桥及ACE conformance host/performance固定组。复用现受控PE/Job执行，不新造runner/计量器。

## 4. 需求分析与背景调查

**r2复核（2026-11-04）**：本轮用户只授权复核/修订规划，不施工、派发或运行验证。初读基面 `ffc59e505`，终检 `50555f3d3`（928由3/12推进4/12，源码调查身份未漂移）；D927=`5d5054d5d` 已交付归档并清树，928 executing r2 4/12，929 executing r2 1/14且按用户裁定挂起等待D928；930 drafting r1 0/9。下列 aa6791649/927未合入等为初订调查历史，由本条与[本次复核](reports/ace-931-933-plan-review-and-m1-distance-2026-11-04.md)取代其状态/写窗描述；不改在途计划进度/授权，未知D不以分支代替。

用户授权规划下一步，未授权施工/派发。初订main取材`aa6791649`及927已审树`35ad072b2`；最新取材为本节r2复核，D927已交付，不再等待927。已读auto-ac/function-incremental-source/native-source-diagnostics/mir-native-cache、auto-syntax/snapshot-cst与正式TestCase/性能合同；r2补核auto-test-case/m1-order-layout-observations §5/7及914归档复审（N3：refusal覆盖全部增量mn recipe）。profile-v2-native的911旧句与host旧Error断言未同步，属于本包必须具名消除的合同漂移，不是重新请求用户裁定。

当前源码：`av/src/session.rs`在prepare_candidate_document返回任一`source.incremental`时归SignatureOrContextDrift；`native/src/incremental.rs::edit`/close_exit已有失败计数窗口；`tests/conformance/tests/host/mir_native_protocol.rs::mir_native_incremental_formal_runner_gate`包含115fn合法body编辑、cold分离、故意增加fn_leaf_1参数的拒绝及缺失edit literal控制。债务收据报合法body拒绝和t11.refused分类红；本轮只读不复跑，不假定两现象同一根因或已经定位。

实施树`D:/autostack/.wt/lang-932/auto-lang`，分支`plan-932-dev`，基面master@84c894b92（2026-11-04经/auto-plan-goal目标授权开工）；开工材料仅`D:/autostack/plan-artifacts/plan-932/`登记，merge摘要/全输出路径对账后删除，guard清树。无指定模型/预算，不另起会话。

## 5. 详细设计

### 5.1 原输入与分类合同

T-01/02冻结原rev0/rev1（301,387B级、115fn负载以实际producer为准）、literal替换位置/结果、profile/caps/entry与每层typed记录；另冻结914 approved Spec/复审、911旧Spec和t11原断言冲突证据，核同一源码新CST/旧测试参照路径、canonical签名/参数名绑定/mode/result/名义owner和context差异。只改body/位置但语义接口未变应走单体候选；真参数个数/名字绑定/类型/mode/返回或声明集变化不得宽松视为body。若根因仅父侧分类，则不为凑修复而改core语义。

| 事实 | worker/父侧必须如何判 |
|---|---|
| 合法body更新且PE返回独立新结果 | 真实提交/执行；按expected正常PASS/FAIL，不能误拒或运行旧结果冒成功 |
| 真签名/声明context变化、原合同非快路径，typed事实完整有效 | 当前根不变；按914为stage verify、code mn.incremental.refused.<kind>的diagnostic completion，按原声明预期PASS/FAIL；不运行旧PE满足新源结果 |
| 实际语言诊断/运行语义 | 保raw stage/code/span/message/顺序与source身份，按既有负例/结果期望判PASS/FAIL，不洗成infra |
| 有效受控InvalidEdit/SessionOrCandidate策略拒绝；或无有效事实的错源/协议/工具/资源故障 | 前者按914声明的策略verdict；找不到替换literal等父请求错误、假source/坏typed、资源/infra为ERROR。不能仅因含stale/range字样分类，不能伪造语言拒绝或Default-0收据 |

故意签名控制的`expect completion=success/exit=exp0`与源保持；完整有效typed拒绝不满足success，因此该case应FAIL（控制测试则通过），另有明确diagnostic预期的I09应PASS。若原回包只有untyped/缺字段，则应ERROR；不能用typed/untyped都判FAIL掩盖真实缺事实。T-02核实际回包后将旧host/performance的“一律Error”断言按914最小修订并断言精确typed码/阶段，保留旧断言及冲突依据于§9；这不是改行为expected凑绿。成功修复不等于开放全声明编辑，929仍承担其统一入口/依赖扩展原合同。

### 5.2 真实生产、恢复与事务

从父侧prepare/拷源/定位/IPC→candidate→verify/lower/cache→commit→link/run→admission→原始行落盘保持连续观察。实际producer attempt在执行前计数，失败早退也有；未改函数parse/project/resolve/typecheck/HIR/lower/object-write0、whole/fallback0，改函新产恰一、其余resident hit按原115负载为114。R重验与M目录/源hash/位置及必要artifact I/O分账，不把全源重建/遍历藏M。

fresh/hit/restart/body-edit、失败暖编辑→修复、source变长/前缀漂移、历史body回退至少覆盖原支持路径；restore必要深验不免，首编辑不预热冒0。通用恢复后ensure_env及全profile迁移仍由929，发现相同路径时按稳定接口交接，不能在本包宣称SYN全部关闭。

拒绝/取消/失配后当前source/revision/单元wire/object/manifest及diagnostic归属不被错误提交，旧合法程序可继续，下一合法body编辑成功。source-v1诊断操作级快照绑定候选源，不读全局last-error或解析字符串补cause。

### 5.3 资源、时间与写面

沿原native/AV/cache准备/装载/IPC/PE期限、内存/磁盘/历史与最后owner预算；old/current/候选同时存活纳峰值。正式P90≤2000ms及原样本、115fn/≥256KiB负载、平台/ignore/里程碑条件保持。原注释P95作历史，不能改为当前门或缩负载/把拒绝控制变成功控制。正式采样错峰，只在冻结提交按确需职责并集运行，不叠全仓。

N/private：native incremental/source_diagnostics/worker与host协议反例；S仅实际必要av/session/core retained guard。D927=`5d5054d5d`已正式交付，消费该基线；928/929若改同一CST投影/目录/typed结果，按符号与SHA单写，本包只修原scalar支持与原因，不并写grammar/闭包子体复用。H为ac_mn、contract/父decoder、tests/conformance、固定组；与931/933/他方930按完整文件单写，记录所有producer/reader/construct-match、schema/STALE/pending/验证与释放。私有调查不等928整包，公共写必须绑定原owner释放的实际稳定接口；最终证据钉D927及后续实际已交接版本，接口/grammar漂移则标STALE重验，不额外制造整包语义依赖；原在途计划不扩包。

### 5.4 r2严格准入与首操作证据（T-02..09，AC-01..09）

**既有批准优先级**：914归档复审N3及现Spec明确typed refusal为verdict、覆盖全部增量recipe，优先于911未同步旧句。T-05/10消除旧Spec冲突的差额，review/merge再沉淀，不在规划期改current Spec。

1. typed分类必须从真实失败原因产生：core `resolve.rs::recheck_swap`当前多个分支共用`source.incremental`，分别涉及槽位/owner、签名及上下文；T-02列完各return，不把全部该码视为signature，也不解析message补类型。签名重建内部真实类型诊断与真实signature drift分开，保合法近邻与原负例；若需新typed字段，core→AV→native→worker→`crates/auto-lang-core/src/test_cases/ac_mn.rs`及strict decoder全部construct/match/版本一起列窗。
2. 收据是本次操作事实。refusal kind/phase/diagnostic presence/source hash/revision/stage计数互相一致；缺kind、未知值、重复/矛盾字段、假阶段/旧source snapshot均ERROR，不降成普通FAIL；有效策略拒绝则按914可判PASS/FAIL。语言诊断精确原字段/顺序且绑定候选源，不借旧current快照；资源与infra按原判面，不能把合法语言负例改成ERROR以“保守拒绝”。格式无变时保原wire；需要变更则版本化或按原严格策略显式拒旧，不默填default。
3. 拒绝后不得继续用旧PE满足本次新源expected；序列必须在拒绝处停止并按有效策略/语言verdict或invalid事实ERROR分类，后续独立run旧current合法但明确旧revision。至少反控旧结果恰好等于expected、伪typed signature遮住语言错、假refusal放行成cold/fallback、旧snapshot+新revision；原签名控制expect不动。
4. `SyntaxNotConverged`按911原规则仍是唯一准许fallback的种类；932不开放signature/语言/resource fallback。D-911-3无自然合法触发样本仍单列既有缺口：可用受控真实路由反控验证分类，但不可把注入样本写成已补自然E2E，最终总验收须给该债适用性/关闭裁定，不静默豁免。
5. 生产与I/O计数在restore/prepare之前安装，失败attempt也实录；每未改函数parse/project/resolve/typecheck/HIR/plan/bind/lower/encode/object持久化0。恢复R可以deep-verify/decode，编辑中未改单元payload额外读写0；源hash/目录重锚及整体link/manifest发布另计M/I/O并进父总窗口，不把重扫body/深clone改名metadata。合法body的oracle是独立exp1，拒绝控制oracle是914继承的typed diagnostic completion/预期判面与旧态不变，二者不共享一条“runner没崩”判据。
6. 新进程恢复→立即首个有效body编辑，以及恢复→首个坏编辑→修复→有效编辑必须由实际支持的source恢复接口执行；普通`[run]`走whole-request不能冒P0恢复。T-06在929或等价已释放接缝上联验，通用来源状态实施仍归929；接缝尚无则AC-06该腿pending，可先交稳定修复供929接入，不增加D929整包才可开工的前置，也不能以原cold门关闭新首编辑断言。错源/版本/缺required/损坏object各有恢复拒绝对照，合法历史A→B→A不得借source相同跳过凭证重验。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-ac/function-incremental-source.md | 原支持body准入/typed拒绝/真实失败计数澄清并接实际consumer | 保原快路径 | AC-01/02/03/05 |
| SD-02 | modify | docs/specs/auto-ac/native-source-diagnostics.md | typed refusal/诊断/infra分态与操作身份 | 不读message猜原因 | AC-03/04 |
| SD-03 | modify | docs/specs/auto-ac/mir-native-cache.md | 修复后事务/恢复/正式证据实况 | 不冒假增量 | AC-05/06/07 |
| SD-04 | add | docs/specs/auto-test-case/native-warm-edit-admission.md | t11合法/真签名/缺literal按914继承的父判与反控 | 关闭D-927-2且不回退批准 | AC-04/08/09/10 |
| SD-05 | modify | docs/specs/auto-test-case/profile-v2-native.md | 同步911旧句：有效typed策略拒绝是verdict，缺/坏事实才ERROR | 消除与914冲突 | AC-03/04/10 |
| SD-06 | modify | docs/specs/auto-test-case/m1-order-layout-observations.md | 保914全增量recipe拒绝判面，链接932精确有效性门/反控 | 防把旧测试当新裁定 | AC-04/08/10 |

## 6. 测试设计

新目录`crates/auto-lang/test/cases/ace-m1/native-warm-admission-932/`由T-08使能，原host/performance输入与expected保留。新测试均未执行。

| case | AC | 输入/实际入口/work任务 | oracle/可区分反控 |
|---|---|---|---|
| V-01 | AC-01/02 | T-01/02/03，原rev0→rev1只改body，经真实mn runner | 独立新退出值，改函1/未改0、114hit、whole0；原红原因逐层查证，允许core或parent根因不同 |
| V-02 | AC-02/03/04 | T-02/03/04/05，原fn_leaf_1新增参数签名控制 | 继续拒绝，typed SignatureOrContextDrift与原success预期下FAIL（控制测试通过）、旧状态不变；明确diagnostic预期另应PASS，缺/坏typed为ERROR；放开门/改expected/旧run误PASS必红 |
| V-03 | AC-03/04 | T-04/05/08，真实语法/类型负例、bad literal/stale/range/错协议/资源各配合法近邻 | 语言与infra/资源区分，source-v1事实全字段/顺序保真，错误源不被伪造；未知kind严格拒 |
| V-04 | AC-05/06 | T-03/06，fresh/hit/restart/body-edit与失败→修复/历史回退/变长 | 真P/M/R/counter/来源重验，无隐藏whole/预热，拒绝不提交；下一真实编辑与fresh等价 |
| V-05 | AC-06/07 | T-06/07，旧链/current/候选、取消/预算/最后pin、原负载 | 原准备/执行/资源上限及先预留保持，无半发布/泄漏；缺计数/Default-0/spike/放宽预算反控 |
| V-06 | AC-08/09 | T-08/09，原host mir_native_incremental_formal_runner_gate与正式mn.t11.refused | 合法body原期望保持；故意签名case的source/expect保持，旧Error测试断言按914显式纠偏；有效拒绝FAIL/diagnostic匹配PASS、坏事实ERROR对照齐 |
| V-07 | AC-09/10 | T-09/10，原ace-performance campaign/consumer/SD | 原60项样本/适用条件一项不删，既有唯一红应关闭；新增case单列，其他新红具名不可冒整个campaign绿 |
| V-08 | AC-03/04/08 | T-04/05/08，实际父decoder/worker，缺typed/矛盾phase/错候选source/未知kind/旧结果等于expected | 错收据ERROR且序列不以旧run变绿；真实语言负例及有效策略拒绝保PASS/FAIL判面，缺typed原控制必须ERROR |
| V-09 | AC-05/06/07 | T-06/07/08，929或等价稳定新进程来源恢复→首有效编辑、首坏编辑→修复及坏source/version/required/object | R深验真实、无预热或whole伪装、未改P和单元payload读写0；旧根不变，修复新值等于fresh；尚无真实接缝则pending |

职责组`cargo t ace-native ace-native-worker ace-av ace-source ace-host ace-adapters auto-test-case infra-fixed`，触shared lower/IR补`ace-lowering ace-a2x ace-semantics`；必要集成`case-ace-m1-order-layout-914 case-ace-m1-native-diags-911 case-ace-m1-native-collections-889 case-ace-m1-native-resource-923 case-ace-m1-native-closures-924 case-ace-m1-closures-recovery-913`。拟新增`case-ace-native-warm-admission-932`由T-08注册。正式`cargo t ace-performance`在原条件/里程碑档单实例进行，T-09冻结实际命名样本/选择数/原排除与报告，保原60项义务及新样本，不能只重跑绿子集称60/60。本轮不执行全仓批次。

唯一树中实现→内部检查→独立review顺序交接，用户未指定人员/模型，不创建agent/chat。输入不变有效证据复用，实际依赖改变标STALE；首次红/环境重放/未执行分别记录。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | V-01原输入/版本/链路与错误根因可复核，合法body和故意签名红分列，非猜测修复 |
| AC-02 | work | V-01/02合法body真快路径成功（若旧现象只是故意签名控制，证实澄清而非强造core修复），原参数/名义/mode/接口拒绝不放宽 |
| AC-03 | work | V-02/03 typed原因从发生处传递，源码/资源/infra保真，不从消息反推 |
| AC-04 | work | V-02/03/08父侧按914有效策略/语言PASS-FAIL与坏typed/infra ERROR分态，原源/行为预期不改，旧状态测试断言纠偏有批准证据 |
| AC-05 | work | V-04真实未改函数生产0/whole0，失败attempt、R/M与I/O完整计量 |
| AC-06 | work | V-04/05/09失败候选保旧态、恢复深验→无预热首编辑/坏编辑修复与提交事务正确，接缝未交明确pending |
| AC-07 | work | V-05原时间/资源/负载/cleanup门保持，不能cold或出窗预热冒增量 |
| AC-08 | work | V-06/08两个t11门与I09/typed有效性补例实际父路径成立，旧断言按914纠偏而非放松typed准入 |
| AC-09 | work | V-06/07原正式campaign义务与冻结必要consumer证据齐，新红/条件/未验不冒绿 |
| AC-10 | work | D-927-2/SD/真实窗口与后继余账准确，健康无本包新警告/debug |
| AC-11 | review | 独立逐AC/SD/遗漏/原门/反控复核revision-bound pass |
| AC-12 | merge | Spec读回/交付/归档、全输出目录材料摘要与删除、guard清树完成，不冒M1签章 |

## 8. 执行步骤

| T | owner_stage | 前置/实际写面 | 产出/验证与AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 当前main/927已审版本只读取材/私有输入 | 原两个t11输入/源码/身份/914批准与911旧句冲突/矩阵/窗口冻结，AC-01/10 | [✅ 2026-11-04：冻结档 `D:/autostack/plan-artifacts/plan-932/T-01-T-02-freeze.md`（两t11门=同一测试函数`performance::mir_native_incremental_formal_runner_gate`经ace-host/ace-performance双入口；三关键输入SHA与next-wave报告全等无漂移@84c894b92；负载=workload.rs fn_041 a0+=1纯body编辑）] |
| T-02 | work | T-01，最小原链路探针 | 合法/签名控制逐层重建、914继承及过时断言裁定、root cause与typed接缝可行性，AC-01..04；不明先needs_replan | [✅ 2026-11-04：复现命令+红在`mir_native_protocol.rs:1573`（leg c：实际Fail vs 旧断言Error）；**leg(a)合法body编辑与leg(b)冷分离通过**——D-927-2"合法body被拒"现象不在本门复现，AC-02澄清分支成立，不造core准入修复；根因=t11用例default模式仅untyped refused→父降级verdict→Fail，与914批准的typed verdict判面及旧Error断言冲突；typed接缝=既有`diagnostics:"source-v1"`通道（I09先例绿），见冻结档§3/§5] |
| T-03 | work | T-02，core/av session必要S窗 | 修真实准入比较/候选语义且保真签名门，V-01/02/04，AC-02/05/06 | [✅ 2026-11-04：AC-02澄清分支——leg(a)合法body真快路径本就绿，无core准入bug；真实修复=守卫拆码与typed保真：`recheck_swap`槽位/名字失配→`source.incremental.slot`、owner缺失→`source.incremental.owner`（session映射SessionOrCandidate）、签名重建guard_diags原样透传为语言事实（不再折叠成签名漂移）、env安装身份族（incremental*/foreign/stale/closed）→SessionOrCandidate（不再伪typed语言拒绝）；`owned_session.rs`配套prefix匹配保持925线Fallback语义；闭包线`resolve_owner_in_env`未动（§5.3不并写）。验证：ace-source/ace-av绿、ace-native 222/223（唯一红924时序环境归因）] |
| T-04 | work | T-02/03，native worker/source_diagnostics | typed原因/操作快照与失败计数，V-02/03，AC-03 | [✅ 2026-11-04：typed通道核验为既有完备（`typed_refusal` kind/phase/snapshot绑定+`close_exit`失败计数+`body_work`捕获，SessionOrCandidate经From映射承接新slot/owner身份族）；无需新增字段，实证=ace-native 222/223绿（唯一红为924线P90时序，隔离重放PASS环境归因）+ace-native-worker绿+932套件语言/typed拒绝腿绿] |
| T-05 | work | T-04，ac_mn/contract/父H窗 | 914策略verdict/语言/资源-infra判面与严格字段，V-02/03，AC-04 | [✅ 2026-11-04：`refusal_verdict`严格化——有效typed策略kind（signature-or-context-drift/invalid-edit/session-or-candidate/lower-admit）=verdict按expect判PASS/FAIL；default模式untyped `refused`、缺/未知kind、untyped/resource kind、language/syntax-not-converged kind出现在refusal面（矛盾）=infra ERROR不降级FAIL；宿主门leg(c)按914 N3纠偏（旧Error断言→Fail+精确typed码断言，冲突依据见§9）+新增(c2)精确码/阶段Pass控制与(c3)untyped→Error负控；`refusal_tests`8单控绿；t11门全绿（含ace-host组内170/172中该门PASS）] |
| T-06 | work | T-03..05，retained/cache/历史事务 | 四态/真实恢复首编辑/坏编辑修复/来源与P-M-R边界，V-04/05/09，AC-05/06 | [✅ 2026-11-04：四态P90门（restart-warm/body-edit/signature-reject≤2000ms，`mn_sd911_source_v1_four_state_p90_30`）在里程碑campaign绿；fail→修复/历史A→B→A/状态存活=932套件绿腿；**V-09新进程恢复→首编辑腿pending**：worker rounds每请求冷建会话、`session::restore`未接协议，无929或等价已释放接缝（套件规范§4与§10登记，接缝就绪后由929联验，不造D929整包前置）] |
| T-07 | work | T-06，原资源/执行probe | 预算/取消/当前旧链/pin/最后回收，V-05，AC-07 | [✅ 2026-11-04：原资源/预算/回收门全数保持——`case-ace-m1-native-resource-923`/`resource-events`绿、campaign资源腿（resources_cold_set_churn_gc_convergence/transient_spike_sampler/32mib_product_random_read）绿、`probe_adapter_turnaround_ipc_hard_gate_p95`绿；本包零放宽/零预算改动（diff仅分类语义）] |
| T-08 | work | T-03..07，case/host/注册H | 使能V-01..09文件化独立oracle、typed矛盾负控与原t11及914 I09控制，AC-08 | [✅ 2026-11-04：新目录`native-warm-admission-932/`7 case全绿（body成功64/语言拒绝source.type-mismatch@typecheck/fail→修复85/签名拒绝typed verify码/状态存活43/多decl invalid-edit/历史A-B-A 43）+`cargo t case-ace-native-warm-admission-932`注册PASS+父侧`refusal_tests`8单控绿+宿主门c/c2/c3腿（typed→Fail+精确码、精确码/阶段Pass、untyped→Error）] |
| T-09 | work | T-08及恢复接缝稳定SHA，冻结提交/正式采样错峰 | 职责/必要consumer+原ace-performance义务复验，AC-01..09 | [✅ 2026-11-04：冻结提交`67c7ae61d`；职责组ace-native(222/223,1红环境归因隔离重放PASS)/ace-native-worker/ace-av/ace-source/auto-test-case/ace-adapters/infra-fixed全绿；集成组order-layout-914/native-diags-911/native-collections-889/native-resource-923/native-closures-924/closures-recovery-913/native-warm-admission-932全绿；ace-host 170/172（两红均基座既有，见§9）；**T3_MILESTONE=1 ace-performance：61执行/59绿，t11门绿(11.59s)——D-927-2既有唯一红关闭**；两红具名：six_scenarios（stash本包5源文件后基线84c894b92隔离仍红→基座既有）+concurrent_gc_30（SQLite锁竞争，当日ace-native批次绿+隔离重放PASS→环境归因）；61枚举冻结于plan-artifacts/T-09-*.txt（债务收据60项为他线增腿后的计数漂移）] |
| T-10 | work | T-09 | SD/债务/交接/健康核查与真实余账，AC-10 | [✅ 2026-11-04：SD-01..06已制备于worktree提交（含新spec native-warm-edit-admission.md与911旧句对齐）；债务更新=D-927-2关闭登记+新登记D-932-1（six_scenarios基座红，含stash-bisect证据）/D-932-2（AP-16清单行指向007已迁走路径，fresh worktree必红）；健康=触面文件rustfmt干净（仓内他人文件既有漂移不扩）、编译零新告警、无debug输出；余账=V-09 pending（929接缝）+D-911-2/3/4等按原owner保留] |
| T-11 | review | T-01..10完成，冻结实现 | 独立逐AC/SD/遗漏/反控复审，AC-11 | [ ] |
| T-12 | merge | T-11 pass及后续实施/合入授权 | canonical沉淀/交付/归档/全路径清理，AC-12 | [ ] |

下一入口T-01原例盘点可私有准备；T-03公共修改须同符号原owner释放与稳定SHA，不新增等待927/928整包的语义前置；D927以后实际输入漂移必须重验，不得并写。本轮不授权work。

## 9. 复审记录

2026-11-04 stage:review / review_scope:final / reviewer:同实施会话独立工件重建（无独立模型/会话——局限已声明；判定从diff/命令输出/冻结档重建，非执行者摘要）/ plan_id:PLAN-932 / plan_revision:2 / outcome:**pass** / worktree:`D:/autostack/.wt/lang-932/auto-lang` branch `plan-932-dev` / reviewed_commit:`67c7ae61d`（基线核对：HEAD=67c7ae61d、树净除2个D-911-2类测试残留未跟踪json）/ base_commit:`84c894b92` / dependency_revisions:无跨仓 / spec_inputs:docs/specs/auto-ac/{function-incremental-source,native-source-diagnostics,mir-native-cache}.md、auto-test-case/{profile-v2-native,m1-order-layout-observations}.md@67c7ae61d / acceptance_results:AC-01..10全pass（逐AC工件映射：AC-01/02=T-01/02冻结档+复现日志leg(a)(b)绿+套件/宿主腿证明拒绝门未放宽；AC-03/04=diff核验码从发生处映射无message解析+refusal_tests 8/8+I09组绿；AC-05=四态门+t11腿1unit/114hit断言+结构准入；AC-06=套件survive/repair/history绿+V-09两腿按AC原文显式pending；AC-07=campaign资源/预算腿绿+diff零时序/预算改动；AC-08=两t11门绿+宿主c/c2/c3+旧断言纠偏有914批准证据；AC-09=61/59两红具名不冒绿；AC-10=债务三笔+SD-01..06+健康）/ pending_closeout_gates:merge拥有——冻结受影响模块并集已在此基线跑过（组证据在fmt前语义等价树上，冻结提交冒烟三绿补绑：refusal_tests 8/8+932套件+t11门73.6s@67c7ae61d）/ findings:0 in-contract；非阻塞注记2条——(a)campaign 61vs60差额无法从927收据精确具名（其未枚举；候选=924 perf腿ba96b7762@2026-11-04 00:22；无删除证据：全族在场+原唯一红在场且绿）；(b)rustfmt连带归一ac_mn三个子模块（纯格式、H窗内）/ evidence:冻结档plan-artifacts/plan-932/{T-01-T-02-freeze.md,T-09-ace-performance-milestone-61.txt}+§8任务行内联+§9组运行台账 / next:merge（T-12）。

**SD增量复审**：SD-01..06全部核验与实现一致（守卫拆码/身份族映射/三态分判/缓存语义保持）；SD-04新规范native-warm-edit-admission.md准确描述现行为含pending边界（V-09/D-911-3）；SD-05消除911旧句与914冲突；frontmatter supersedes/new/touched_goals与SD表一致。规范文件已在67c7ae61d提交内（worktree制备、merge读回验证归merge）。

2026-11-04 stage:goal / plan_id:PLAN-932 / plan_revision:2 / round:2 / from:work / to:review / status:execution_done / outcome:pass / worktree:`D:/autostack/.wt/lang-932/auto-lang` | commit:`67c7ae61d` | next:review。

2026-11-04 stage:work / plan_id:PLAN-932 / plan_revision:2 / outcome:pass / worktree:`D:/autostack/.wt/lang-932/auto-lang` branch `plan-932-dev` / base_commit:`84c894b92` / code_commit:`67c7ae61d`（18文件+801/-130）/ dependency_revisions:无跨仓依赖（auto-down兄弟树仅作组内解析）/ task_ids:T-01..T-10 / evidence:见§8各任务行内联证据；冻结档`plan-artifacts/plan-932/{T-01-T-02-freeze.md,T-09-ace-performance-milestone-61.txt}` / blockers:无合同内阻塞；V-09恢复腿pending（929接缝，§10）/ next_responsible_role:review（T-11独立复审）。

**旧断言与冲突依据（§5.1要求保留）**：宿主门leg(c)旧断言`assert_eq!(status, Error, "a refused warm edit is an infra non-verdict, never a green language result")`（mir_native_protocol.rs原1573-1577，pre-914信念"refused=infra非判面"）；冲突依据=914归档复审N3与m1-order-layout-observations §5/§7（typed refusal=diagnostic completion verdict，stage verify，覆盖全部增量mn recipe；缺字段/协议破损才是infra）+已绿I09先例（order-layout-914 declare source-v1+typed expect PASS）。本包按914最小修订：case声明`diagnostics:"source-v1"`，断言改为Fail并加精确typed码/阶段断言（c2）与untyped→Error负控（c3）——旧expect（success/exit exp0）与源保持不变，非改行为expected凑绿。

**组运行台账（T-09）**：ace-native 222/223（红=closures_native_924 P90 2425ms，隔离重放PASS，环境归因D-911-4类）；ace-native-worker ✅；ace-av ✅；ace-source ✅；auto-test-case ✅；ace-adapters ✅；infra-fixed ✅；7个case集成组 ✅；ace-host 170/172（红1=six_scenarios缓存状态断言**基座既有**：stash本包5源文件后基线84c894b92隔离复跑同红；红2=AP-16清单行scan-error，指向auto-ui PLAN-007已迁走的examples/ui/015-notes路径，主检出靠未跟踪残留假绿、fresh worktree必红——两红均新登记债务D-932-1/D-932-2，非本包引入不改判）；T3_MILESTONE=1 ace-performance 61/59（t11门绿11.59s，D-927-2唯一红关闭；six_scenarios=上述基座红；concurrent_gc_30=SQLite锁竞争环境归因，当日批次绿+隔离重放PASS）。四态P90门、924 campaign腿、全部资源/预算腿在campaign绿。首次红/环境归因/重放绿分别保留于本台账，条件排除不充PASS。

2026-11-04 stage:goal / plan_id:PLAN-932 / plan_revision:2 / round:1 / from:new / to:work / status:executing / outcome:pass（目标授权记录）/ 授权：`/goal 完成计划932`（auto-plan-goal整链：work→review→bounded repair→merge）/ worktree:`D:/autostack/.wt/lang-932/auto-lang` branch `plan-932-dev` base `84c894b92` / artifacts:`D:/autostack/plan-artifacts/plan-932/`（已登记.retention）/ next:work。主检出既有未跟踪代码路径（examples/ui/015-notes、scripts/tools/native-fixture/Cargo.lock、test/fixtures/reader-import-budget）经核在932写面之外，具名浮现不编辑，归属他会在途工作。

2026-11-04 stage:new / revision:2 / outcome:pass（规划合同静态复核，不是实现复审）/ next:work（后续实施指令及各任务实际写窗就绪）。修改见[复核报告](reports/ace-931-933-plan-review-and-m1-distance-2026-11-04.md)；保全部T/AC编号、总步骤及0进度，r1记录保留为历史；新增V与收口前置未执行，旧调查/历史运行不能为r2新增断言背书。

2026-11-04 stage:new / revision:1 / outcome:pass（规划静态核对）/ next:work（后续明确实施指令及公共窗释放）。未运行构建测试/性能/认证，任务全未勾，D-927-2仍open；现有59/60为历史收据，不是本轮实测。

## 10. 待澄清事项

继承914有效策略拒绝为verdict及911事实准入的批准，不需要用户重新裁定或放宽签名/语言/时间规则；真实根因由T-02有界重建决定。若原记录把某合法body与故意签名的现象混记，先拆清实际入口/expected再修，不把观察错误变新需求。929承担全profile语法/声明变化扩展；D-911-3等其他具名债按原owner保留，不冒本包已闭。

**pending（非阻塞，具名）**：V-09新进程会话恢复→首有效body编辑/首坏编辑→修复两腿——worker rounds协议每请求冷建会话，`session::restore`未接协议，尚无929或等价已释放接缝；按§5.4.6该腿pending（AC-06该腿不宣称关闭），929接缝就绪后联验，本包已交稳定修复（typed拒绝/状态保持/修复链）供其接入。
