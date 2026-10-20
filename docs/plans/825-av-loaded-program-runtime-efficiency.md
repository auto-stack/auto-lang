---
plan_id: PLAN-825
status: drafting
feature_name: AV装载执行表示与运行效率
plan_revision: 3
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-20
current_step: 0
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-av/resident-function-install.md
  - docs/specs/auto-av/project.md
  - docs/specs/auto-av/cache-and-conformance.md
  - docs/specs/auto-test-case/architecture.md
new_spec_components:
  - docs/specs/auto-av/loaded-program-runtime.md
  - docs/specs/auto-av/runtime-benchmark-method.md
touched_goals: []
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-825] AV装载执行表示与运行效率

## 0. 变更摘要

承接[Design43 §12](../design/43-ac-av-typed-mir-and-abc.md#12-abc评审采纳与实施责任)、[Design44](../design/44-av-data-layout-place-and-ownership.md)及[ABC评审](reports/798-abc-design-review-2026-10-20.md)。本计划是后续性能草案，**不阻塞798交付，不成为820–824功能链硬依赖**。当前只写文档，未开树或执行测试。

## 1. 目标

在已交付的798标量profile上，建立私有LoadedProgram执行层，去掉每指令扫描前置块的确定性开销，并通过实测选择安全的调用/帧分配策略；交付公平、可重放的WASM/V8执行成本对标。

只处理D-798已交付i32/bool、控制流、调用及trap；不改wire语义、不扩source/类型/CTE/native、不加JIT/unsafe/NaN-boxing。820–824不是本计划依赖；若它们已交付，必须保留其受检参考路径和全部已承诺支持面，不能让优化成为降级。

成功为实际受检执行/恢复和反控闭环、确定的优化/密度目标、可重放分阶段报告；不以未来优化预期或相同实现自证代替独立结果。

## 2. 架构方案

798提供正确的语义MIR/portable ABC和受检执行合同，825提供私有装载执行层，826提供新版本紧凑wire。磁盘和执行表示独立，能力、资源和所有权语义保持。功能链D798→820→821→822→823→824和可选优化链D798→825→826共享核心，**实现按实际单写窗口串行交接**，不是可并发改同一公共定义。

**D-798**=该计划最终独立review、冻结门禁通过、实际合入、Spec与merge收据齐全；不能只看状态或复制在途分支。T-01可先只读准备，T-02及依赖构建等D和本计划实施授权。可将825/826安排在820前，也可等一个功能Plan实际交付后接续；不添加环或把未交付data变成依赖。

## 3. 技术栈

Rust与D-798共享的独立workspace；观察位置为 `D:/autostack/.wt/lang-798/auto-lang/experimental/ac-av-probe/`。code owner是其 `core/src/{mir,mir_verify,abc,abc_verify,av,contract}.rs` 与 `probe/src/{cache,driver,worker_restore}.rs` 相关符号，T-01以实际交付树定位，不从本轮主检出不存在的源码推断。拟新增 core/src/loaded_program.rs，模块注册服从当前owner。

拟新增 `scripts/verify-av-825.ps1`，Scope=first/targeted/compat/bench/acceptance，测试前缀 `av_runtime_825`、fixtures `test/cases/av-abc/825/`、AutoDown recipe `av-abc-825`。这些未来入口由T-02/06建立，当前不存在、未运行；不得把建议命令写成已验证证据。

唯一拟实施树 `D:/autostack/.wt/lang-825/auto-lang`、分支 `plan-825-dev`；本轮未创建。根workspace成员/产品manifest、旧VM/UI/产品SDK不在范围；正式recipe在当前真实owner处的最小注册/父侧判定接线为明确例外，按实际根Rust触面验证。正式recipe注册位置服从814–816实际owner交接，不复制协议/runner。

## 4. 需求分析与背景调查

用户2026-10-20确认798完成后优先828，所有ACE扩展保留全链路增量。D828为共享核心实施/集成排期门；只读准备可先，既有准备/进度保留。原其他依赖仍有效，无对798/828的反向依赖。旧revision/排期记录仅为历史，当前以本轮§5–8为准。

用户于2026-10-20认可ABC评审建议，要求补入Design43/44与实施计划，区分798正确性和后续性能，**先不写代码**。仅授权文档修订/计划起草；没有新模型、代理、安装工具或性能百分比授权，不自动进入work。已有798工作授权及R10finding保持，不能由本计划替其宣告交付。

取材：[Spec overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[语义身份](../specs/auto-semantics/project.md)、[产物合同](../specs/auto-ac/artifact-contract.md)、[测试协议](../specs/auto-test-case/architecture.md)。auto-av/auto-mir本轮尚未正式交付，目标Spec路径为proposed，不回写现状。来源提交/SHA和采纳映射见[inputs](attachments/av-abc-adoption-20261020/inputs.json.txt)、[decisions](attachments/av-abc-adoption-20261020/decisions.md)和[评审基线](attachments/av-abc-review-20261020/baseline.json.txt)。

调查时798树为f24bd46a6，主检出91a1b622d；该树的LoadedModule持CheckedMir、AV逐步调用block_base累加前块，CALL分配参数/槽，wire多为u32；它们是待优化观察，不是合入事实。798 r4的kind正确性及资源证据必须先收口，不能围绕已知错误分类设计新格式。WASM/V8一手资料及使用限制沿Design43 §12.4，不把外部机制当AV已有能力。

## 5. 详细设计

### 5.1 装载表示与首例

Typed MIR→portable ABC→verify→私有LoadedProgram→AV。装载表示只保存当前版本已验证的执行信息，不是新语义IR；不能公开随意构造、恢复trusted标记或把宿主地址持久化。MIR/ABC仍是共同语义/portable合同。原受检参考解释路径保留作迁移对照，不作为唯一oracle。

T-01在 attachments/825/contract-decision.md 冻结：D798修复后的实际profile/版本与unit/module规则、私有接口/源码owner、块PC方案、参考路径边界、资源/逻辑fuel/来源映射、调用策略候选和基准口径。T-02首例为真实HIR→MIR→ABC→LoadedProgram执行含调用与未走trap分支的程序，返回42且trace独立expected一致；同产物进store，经新worker恢复重验执行。失败先带反例回规划，不能跳验证改走旧VM。

### 5.2 PC与调用优化

为每函数预计算块起点/长度或平坦PC；block/instruction→逻辑PC和分派取指O(1)，不在每步累加前置块长度。块跳转、终结位置和诊断source保持正确；表大小/索引checked，装载成本独立计时。用递增B个短块链和长块对照，统计前置块遍历次数为0、PC索引构建工作O(B+I)、运行每指令定位工作有固定上界，不能只靠抖动时间证明复杂度。

分别测arg Vec、frame Vec和全槽typed_zero；T-01以相同guest工作量冻结至少一种安全策略（如预computed参数映射/复用有界帧空间/消除一次临时参数拷贝）的首例与计数目标，T-04落实。不得使用未初始化Rust值或借位重解释；本计划在标量面验证帧区间、slot初始化和错误后回收，不引入未交付的borrow/owned token作为假测试。若已有data profile，保留其受检参考路径，不能用标量复用策略绕过其owner规则。若候选不能改善目标计数，带数据显式修订，不能留下“以后优化”却勾完成。

### 5.3 语义、预算与兼容

继承D798已冻结profile、cap、溢出/trap、求值顺序、逻辑fuel及当前host grants；预解码/融合不得把多条逻辑工作少计或推迟取消越界。root/call frame在分配前申请，LoadedProgram、映射表、暂存/共享或重复MIR占用纳入真实峰值；不为新层额外领取内存。新进程恢复portable输入后重建执行层，不读取序列化LoadedProgram。

817/818及data owner仍在改公共定义时等待交接，T-01记录消费图。API适配不扩其类型面。不得修改798历史收据/冒称替它关闭finding；D798已正确交付为前门。

### 5.4 测量与对标

固定标量算术、分支短块、长块、直接调用/递归四类工作量及独立结果/trace。冻结同机二进制身份、输入、样本数、预热和方差报告，依次测reference/new路径的装载、首执行、热执行、分配次数/复制字节、受管/进程峰值与端到端编辑。算法计数/分配目标按T-01冻结，速度不预许百分比；回退/无收益如实报告。

Wasm以公共不溢出子集或显式等价checked算术对齐AV，MIN%-1/除法和JS Number差异单独解释。V8 JS Ignition与V8 Wasm baseline/优化tier分表，记录编译/热身/flags/引擎版本；工具无相应tier证据则标未知，不伪称纯解释对比。报告差异及限制，不承诺胜率。WASM与V8各至少一条真实执行的同义路径；工具缺失属于验收缺口，不用空输出PASS，也不擅自安装。详细口径沿Design43 §12.4。

### 5.5 共用资源与证据纪律

沿D依赖冻结的适用cap交集，不放宽旧profile：当前拟沿输入4MiB、表上限65536、验证200万步/5s、执行100万步/1024帧/2s及值/槽合计8MiB、受管32MiB（hot8/staging16/execution8）、worker增量128MiB；T-01必须核对单位/实际更小限制，冲突回规划。表、索引、解码/装载暂存、参考/优化双表示均计量，不各领一份预算；父侧采样进程真实峰值。实验政策不是永久语言上限。

invalid/unsupported、语言trap、resource/cancel、infra分态，错误在规定门发生；分配前cap、失败后真实回收，不能仅timeout充负例。沿798固定编辑负载端到端P95<1s；新bench单实例串行，延迟/峰值含噪声与原样本，不许删坏样本补绿。

### 5.6 文档复审补充合同（r2）

硬依赖和版本路由按Design43 §12.5：825只等D798，826等D825；820–824不是反向前置。820指令表尚未交付时消费D798真实opcode/type/fuel合同，T-01制作已有操作覆盖清单，待820接手合并唯一表，不提前要求820新API。若功能profile已交付则其既有受检路径/codec必须保留并实际回归；未知profile不静默放入标量执行层。只在满足依赖后申请公共写入窗口，不持窗口等待owner。

准入非法dispatch=0；已准入执行的trap/fuel/cancel允许合法前缀，要求危险目标操作/提交/目标callee为0及实际frame/slot/资源清理正确。标量面不要求未交付owner token。模块类型+常量+导入+函数合计≤65536，每函数槽+块+指令合计≤65536，不是各表各领；32MiB受管分区和128MiB进程增量分别计量，不从worker实际峰值扣前端；T-01核798已交付同口径证据，不能用历史目标未达归因充依赖PASS。

O(B+I)仅指已验证结构的PC索引构建；完整decode/CFG验证/load可能受块×槽、参数/常量表和worklist影响，必须分别计步/测峰值并受D798预算，不宣称整体线性。调用候选决策基于D798已交付后的有界首例/计数对照，T-01的提前只读草案不等于完成该冻结；T-02依D门跑真实首例，T-04按冻结目标实施，失败带数据回规划。

正式测量T-01冻结四组必需入口：AV参考/新执行路径、固定Ignition的V8 JS、V8 Wasm baseline、V8 Wasm优化组；后两组的tier/flags必须由实际被测版本证明。WASM与V8共用同一引擎可行，但每组独立样本/层证据与等价结果齐全；缺工具/无法证明固定层留gap并使相应必需门非PASS，不能用unknown/0/skip假充完成。工具准备/首次构建与采样分开计时，不新增安装授权。

case清单由T-01冻结，T-02建立首例，T-03/04覆盖算法/codec与安全反控，T-05覆盖恢复/消费者，T-06统一运行。清单含每个V及子case、Scope、实际入口、独立oracle、阶段/码、必需计数和数据文件身份；acceptance必须覆盖全集合，不能用重复执行一个family凑数。脚本first只验首例子集，不能代表整个V通过。

### 全链路增量兼容与828优先排期（r3）

798实际交付后优先828，本计划在原依赖和D828均满足后取得共享写窗口。D828=当前revision最终review/冻结门禁/实际合入/Spec与merge收据齐全，不消费WIP；只读调研/合同准备可先，已有准备保留。共同合同见[ACE顺序与兼容](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)与[828](828-av-function-incremental-session.md)。

专项：PC/调用缓存和执行表示按函数驻留，单函数变化不重建全Program索引/其他LoadedFunction；结构/速度双门。

T-09 是work必做项：接入D828公共编译/装载/snapshot接口，保留稳定函数身份、私有凭证与实际依赖。纯body编辑、签名/实际依赖/profile/配置/授权不变时，未改HIR/MIR/代码对象和代际不变，未改生产阶段及读盘/decode/verify/load为0。元数据更新单列，不能用全source hash/总revision使所有函数失效。新工件仍走适用准入，恢复不可信。

四必需子case：1)D828真实source edit基线273575B/115函数，至少30不同修订，结构与稳定会话P95<1s双门；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-09 建立真实入口计数/身份/IO/反控，当前未有新增证据，review不得豁免或推给merge。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/loaded-program-runtime.md | 无本Plan合同→§5已验证的私有装载/PC/调用策略、拒绝及资源规则；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则 | 设计与实物一致 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-av/cache-and-conformance.md | 已交付profile→本Plan恢复/版本/当前能力准入及测量反控；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则 | 不恢复trusted或PASS | AC-04/05/06 |
| SD-03 | modify | docs/specs/auto-av/project.md | 已交付能力保持→登记本Plan实际优化/边界与未实现面 | 不冒领data/JIT | AC-06/08 |
| SD-04 | modify | docs/specs/auto-test-case/architecture.md | 保留既有runner→独立av-abc-825 recipe/父侧结果/统计反控 | 真实路径验收 | AC-01/06 |
| SD-05 | add | docs/specs/auto-av/runtime-benchmark-method.md | 无固定执行测量合同→同义负载/引擎tier/阶段/计数/噪声/工具缺口政策 | 不以跨层速度误导 | AC-02/04/06 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |

SD-05的runtime-benchmark-method.md为拟新增Spec，由本计划唯一拥有；826消费并补充其体积数据，不另立冲突规则。

## 6. 测试设计

当前均为待实现fixture/观测要求，未运行。T-01冻结实际输入、独立expected和原始测量脚本，T-02建立真实首例及recipe，T-03/04完成补充反例，T-05覆盖cache/兼容，T-06汇总；核心补充矩阵如下。

| Case | AC / owner | 输入/状态 | 真实入口 | 结果/区分错误实现的断言 |
|---|---|---|---|---|
| V-01 | AC-01；T-02 | 完整单函数42、unit/import未闭合、好hash坏类型/entry | 真实verify/loader/execute、手工DecodedAbc | 合法42；非法dispatch=0，无绕过旧kind修复 |
| V-02 | AC-02；T-03/04 | 不同B短块链/长块、最后块、跳转/终结/source | loader与AV计数 | 前置块遍历0；定位固定上界，PC索引构建O(B+I)；source精确 |
| V-03 | AC-02；T-03/04 | 零/多参、重排参数、递归、不同槽数 | 参考/优化AV+手审结果/trace | 与独立expected一致，T-01冻结目标计数改善，帧无串值 |
| V-04 | AC-03；T-03/04 | 溢出、除零、MIN边界、短路、trace、等fuel边界 | 真实AV与native适用对照 | 同结果/trap/顺序及逻辑预算，无优化绕过 |
| V-05 | AC-03；T-03/04 | 根/调用帧预算两侧、取消/耗尽后下一标量请求复用帧区间、不同槽数 | 执行/预算hook与外部峰值采样 | 预算前拒该申请attempt=0；已入执行故障按目标操作计0，实际frame/slot台账回基线，下一请求无串值，峰值真值；不要求未交付token |
| V-06 | AC-04；T-05 | fresh/hit/新worker restart、错版本/能力/坏内层 | driver/store/contained worker | 恢复重验且仍真实执行；错误受控拒绝，撤销授权不执行 |
| V-07 | AC-05；T-05 | D798旧profile及实际已交付817/818/data消费 | 实际支持面定向recipe | 无语义扩面/退化，未知profile不进无验证fallback |
| V-08 | AC-06；T-06 | 四类负载、公共子集+边界差异对照 | 实际AV、Wasm、V8 JS/Wasm runner | 结果先一致；分tier/阶段/样本/峰值，缺必需工具非绿 |
| V-09 | AC-06；T-06 | 固定798编辑负载+新loaded表/双表示暂存 | driver span、分配事件、父侧峰值采样 | 原P95<1s及原预算不放宽，全部暂存计入 |
| V-10 | AC-06；T-06 | 错expected/trace、漏例/skip、伪tier、抹掉峰值/失败 | 本计划验证脚本/AutoDown父侧 | 合法控制PASS，每项实际让总体非零/ERROR |
| V-11 | AC-02/04/05/06；T-01/03/04/05/06 | 未交付820表的优化先行；已交付data参考路径；PC专用构建计数与整个verify工作分开；漏子case/重复冒计/缺tier证据 | 实际D依赖核心/公开loader、当前consumer及验证脚本父侧清单 | 无反向依赖820；已交付支持矩阵保留；准入/运行计数分开，必需清单及资源口径按§5.6，无漏例假PASS | 不扩标量支持，未交付data只测unknown-profile拒绝，不造token/消费者 |

各V以对应§5不变量为oracle，不只看测试family绿；语义适用面之外明确unsupported。好hash坏内层、手工输入和真实persist/restart均需验证。时间/数量由T-01固定可重放输入，不能用稀释工作量赢基准。

实现/内部检查/独立最终review在同一唯一树顺序进行，未指定人/模型，不创建代理。未来work仅定向测试/fmt/check；例如 `powershell -NoProfile -File scripts/verify-av-825.ps1 -Scope first` 建立后应返回0且断言真实执行42/新worker恢复。固定Scope覆盖：first为T-02首例；targeted负责V-01..05；compat负责V-06/07及V-11的消费者/路由子例；bench负责V-08/09和V-11的计量子例；acceptance负责V-01..11全体以及V-10父侧反控。V-09的旧consumer语义由compat先执行、其计量由bench完成；清单区分子case，重复执行不重复计完成。未来入口由T-02/06创建，本轮未运行。

### 本轮新增的增量兼容验收（work-owned）

| Case ID | AC / owner | 输入 | 实际入口 | 区分性断言 | 边界 |
|---|---|---|---|---|---|
| V-12 | AC-09 / T-09 | D828 source30修订+新增能力两函数+依赖变化+强制退化四子case | 本计划真实consumer/正式recipe与D828 edit接口，由T-09 接线 | 未改对象/代际相同、未改函数生产阶段/产物IO=0，结果/清理正确，P95<1s；强制退化非零 | 当前profile真实起点，不冒领未交付源码/迁移能力，四子case无skip |

## 7. 验收标准

| ID | owner_stage | 可观察结果/证据 |
|---|---|---|
| AC-01 | work | T-02首例真实各阶段、store/新worker恢复及父侧独立expected成立；V-01通过 |
| AC-02 | work | §5具体装载表示/PC复杂度与冻结调用计数目标成立，V-02/03逐案可复现；§5.6/V-11逐项通过（不把运行期故障当准入零执行） |
| AC-03 | work | 数值/顺序/逻辑fuel、非法输入拒绝及取消/cleanup责任保持；V-04/05及适用V-07通过 |
| AC-04 | work | 版本/cache/current grants及恢复重验成立，准入非法dispatch=0；V-01/06及适用V-04通过；§5.6/V-11逐项通过（不把运行期故障当准入零执行） |
| AC-05 | work | 所有新表/暂存真实峰值受预算，旧消费者无扩面或退化；V-05/07/09适用路径通过；§5.6/V-11逐项通过（不把运行期故障当准入零执行） |
| AC-06 | work | V-08..10真实WASM/V8分层对标、原编辑负载和父侧反控齐全，0例/skip/缺工具不报PASS；报告结论可由原始数据重算；§5.6/V-11逐项通过（不把运行期故障当准入零执行） |
| AC-07 | review | 独立核对所有AC/V/SD和遗漏/延后/绕行，绑定实现及依赖hash、无未处理findings；显式记merge-owned门禁pending |
| AC-08 | merge | 冻结门禁过检、SD沉淀、索引/实际合入归档/收据齐全；wt-guard clean后清唯一树，批量回归到期按仓规 |
| AC-09 | work | D828与新增能力保持全链路增量，V-12 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |

## 8. 执行步骤

| Task | owner_stage | 依赖 | 文件/操作/输出 | 验证与AC |
|---|---|---|---|---|
| [ ] T-01 | work | 本Plan实施授权；只读准备可早于D-798 | 核对§3实际源码/owner/Spec/依赖身份；新增attachments/825/contract-decision.md，冻结§5全部决策/反例/基准及预算边界 | 决策表无无主问题，V/AC/SD完整；AC-01..06；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-02 | work | T-01、D-798、共享核心单写交接、D828排期门 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； §3真实核心/adapter、fixtures/脚本与正式recipe首例，persist→新worker恢复 | first scope，42/trace/真实阶段与独立oracle；AC-01/04 |
| [ ] T-03 | work | T-02 | core/loaded_program、av/mir相关：实现O(1)PC与私有映射；保留参考路径/版本边界 | targeted scope V-02/03及合法反控；AC-02；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-04 | work | T-03 | av调用/帧策略计数改善及所有§5安全/资源反例；先完成findings | targeted V-04/05/07；AC-02/03/05；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-05 | work | T-04 | probe cache/worker/兼容consumer最小适配，错版本/授权/代际/损坏及真实restart | compat V-01/06/07/09；AC-04/05；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-06 | work | T-05 | scripts/verify-av-825.ps1汇总、WASM/V8真实bench、父侧反控、SD草案及证据；fmt和独立workspace scoped check | acceptance V-01..10非0例/漏例/skip；AC-01..06；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-09 | work | D828及本计划其他work实现完成 | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-12 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档 |
| [ ] T-07 | review | T-01..06完成、T-09 完成 | 同树独立验收、复核SD/遗漏和峰值/测量原始证据，记录绑定版本review | AC-07，merge-owned门禁pending；不提前跑整档 |
| [ ] T-08 | merge | T-07 pass | 冻结提交执行独立workspace完整有限合同测试及按实际触面裸cargo t/必要tv；SD沉淀、merge/archive/ledger、到期回归及清树 | AC-08；门禁失败归因，代码改变先受影响复审，清树前wt-guard |

worktree及build/test仅在本Plan树，簿记主检出。work/review不运行整档t/tv/tt等；定向验收必做，最终review后的merge统一冻结门禁。tf只按到期在主检出单实例。未改Rust的本轮严格不跑cargo/docs_gen。

## 9. 复审记录

2026-10-20 stage:new | PLAN-825 r2→r3 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-09 / V-12四子case / SD-06 / T-09；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 8→9，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

### 文档合同复审与r2修订（2026-10-20）

stage:review；review_scope:文档合同定向复核（非实现final）；reviewer_identity:同一聊天Codex，曾起草本文件，不声称上下文隔离的外部独立审稿。入审r1结论needs_replan，依用户授权直接r1→r2修订§5.6、V-11、AC/T/SD及Scope映射，drafting 0/8保持。出审文档检查见[报告](reports/av-architecture-contract-review-2026-10-20.md)，没有代码/运行层验收pass，AC-07/08及依赖D门仍pending。

2026-10-20，stage:new；outcome:pass（合同静态预检，非实现验收）。中央分配器在互斥/独占allocation.lock下分配825，未覆盖同号；r1 drafting，0/8，T-01..08均pending。本轮文档授权已记录；D-798未实际满足、实施未启动，不因起草通过宣称work就绪/实现pass。采纳与来源见[记录](attachments/av-abc-adoption-20261020/decisions.md)。

## 10. 待澄清事项

没有本轮必须询问用户的问题。T-01负责实际合入版本、核心owner窗口、PC/安全调用策略与计数目标、可用WASM/V8工具及精确输入的有界决策；材料不足则带实际冲突回new，不擅自改支持面、提高资源上限或删除必需oracle。未知工具不视为用户授权安装。


**当前排期与验收**：798→828→本计划（原前置仍需满足）；T-09 / AC-09 / V-12 未完成，旧revision/测试总数不覆盖新增义务。用户已确认该顺序/目标，不重复请求同范围授权。
