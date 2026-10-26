---
plan_id: PLAN-825
status: executing
feature_name: AV装载执行表示与运行效率
plan_revision: 5
author: [Codex]
created_at: 2026-10-20
updated_at: 2026-10-26
current_step: 2
total_steps: 10
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

承接[Design43 §12](../design/43-ac-av-typed-mir-and-abc.md#12-abc评审采纳与实施责任)、[Design44](../design/44-av-data-layout-place-and-ownership.md)及[ABC评审](reports/798-abc-design-review-2026-10-20.md)。本计划是后续性能草案，**不阻塞798交付，不成为820–824功能链硬依赖**。2026-10-26 已获用户实施授权并完成 T-01/T-10 私有模块，提交 `22a770c48`、定向 11/11；整体 executing 2/10。真实执行/恢复/性能/增量集成未实施，T-02 等待 822 交付与共享核心窗口释放/接受。

## 1. 目标

在已交付的798标量profile上，建立私有LoadedProgram执行层，去掉每指令扫描前置块的确定性开销，并通过实测选择安全的调用/帧分配策略；交付公平、可重放的WASM/V8执行成本对标。

只处理D-798已交付i32/bool、控制流、调用及trap；不改wire语义、不扩source/类型/CTE/native、不加JIT/unsafe/NaN-boxing。820–824不是本计划依赖；若它们已交付，必须保留其受检参考路径和全部已承诺支持面，不能让优化成为降级。

成功为实际受检执行/恢复和反控闭环、确定的优化/密度目标、可重放分阶段报告；不以未来优化预期或相同实现自证代替独立结果。

## 2. 架构方案

798提供语义MIR/portable ABC和受检执行合同，825提供私有装载执行层，826提供紧凑wire。正式功能集成链为820→821→822→{823，824}，优化正式集成825→826；各Plan T-10私有同源模块可按§5并行，**公共定义/装配与实际派发仍按单写窗口串行交接**，不是整计划全部串行，也不是允许同时改同一公共合同。

**D-798**=上游独立最终review、冻结门禁、合入、Spec/merge收据齐全，不消费WIP。不等822/823/824；T-02及真实loader/三执行入口派发适配仍等已交付基面与共享核心窗口。av.rs::ModuleView是pub(crate)、Frame私有，并不存在已交付通用插件executor API。 T-01只读准备可先；T-10在D798/D828及私有写面登记后可做真实独立模块开发，不占公共核心窗口。实际全路径构建T-02起遵守本阶段D门/窗口与原授权。具体规则见§5并行合同。

## 3. 技术栈

Rust与D-798共享的独立workspace；当前已合入源码为 `experimental/ac-av-probe/`，实施时在本Plan唯一树读取；原798实施树是历史位置，不作为依赖。code owner是其 `core/src/{mir,mir_verify,abc,abc_verify,av,contract}.rs` 与 `probe/src/{cache,driver,worker_restore}.rs` 相关符号，T-01以实际交付树定位，不从本轮主检出不存在的源码推断。拟新增 core/src/loaded_program.rs，模块注册服从当前owner。

拟新增 `scripts/verify-av-825.ps1`，Scope=first/targeted/compat/bench/acceptance，测试前缀 `av_runtime_825`、fixtures `test/cases/av-abc/825/`、AutoDown recipe `av-abc-825`。这些未来入口由T-02/06建立，当前不存在、未运行；不得把建议命令写成已验证证据。

唯一实施树 `D:/autostack/.wt/lang-825/auto-lang`、分支 `plan-825-dev`；2026-10-26 从已提交 master `9c7a7436f` 创建，当前私有模块提交 `22a770c48`。根workspace成员/产品manifest、旧VM/UI/产品SDK不在范围；正式recipe在当前真实owner处的最小注册/父侧判定接线为明确例外，按实际根Rust触面验证。正式recipe注册位置服从814–816实际owner交接，不复制协议/runner。

## 4. 需求分析与背景调查

2026-10-26用户授权审计/修订823–826并行边界，本轮仅改计划与战略，不启动实现或其他会话。取材初始821 archived r6、822 executing 6/9；期间822 R1 needs_fix重开至4/9，两次均非D822（他方进度原样保留）；实际输入/代码接缝与指纹见[依赖审计](reports/ace-parallel-dependency-audit-2026-10-26.md)及[inputs](attachments/ace-parallel-20261026/inputs.json)。旧“整计划串行/仅只读准备”排期按下文两阶段合同更新，原目标/非目标/预算和已有进度保留。

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

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

硬依赖和版本路由按Design43 §12.5及当前排期：825等D798/D828，826等D825/D828；820–824不是反向前置。820指令表尚未交付时消费D798真实opcode/type/fuel合同，T-01制作已有操作覆盖清单，待820接手合并唯一表，不提前要求820新API。若功能profile已交付则其既有受检路径/codec必须保留并实际回归；未知profile不静默放入标量执行层。只在满足依赖后申请公共写入窗口，不持窗口等待owner。

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

### 并行模块与集成两阶段合同（2026-10-26）

不等822/823/824；T-02及真实loader/三执行入口派发适配仍等已交付基面与共享核心窗口。av.rs::ModuleView是pub(crate)、Frame私有，并不存在已交付通用插件executor API。

**T-01→T-10独立模块门**：依D798/D828及已合入公开接口开展真实CheckedMir/受检函数单元的PC/参数映射派生表，O(B+I)构建、预算与未改函数身份保留；不复制Frame/dispatch或模拟执行。单一生产源为`core/src/loaded_program.rs`（拟新增），测试壳拟为`experimental/ac-av-probe/parallel/825/`独立workspace/lock，仅用同树已合入auto-av-core。T-01冻结可达符号、具体路径/owner、支持子集和预算；T-10先证明最小可编译接缝，再实现真实模块与故障oracle。可由harness重导出已交付core公开模块并路径编译同一源，生产阶段只加薄adapter；不复制值/IR/store/VM，不改core/lib.rs/父members，不产生依赖环。若需要pub(crate)私有符号，不绕过权限，登记具体缺口并申请独立验收合入的小接缝；不能从上游WIP取代码。

**T-02集成门**：T-10通过不等于profile/Plan交付。满足原D门后同步最新master，刷新接口/版本/预算，取得共享核心/注册窗口，完成原首例真实verify→ABC→AV/store/new-worker/AutoDown和trace/资源；继续T-03..06/T-09及最终review。所有模块独立通过不能代替原全路径、精确增量、P95/内存或反控。T-10测试命令拟为`cargo test --manifest-path experimental/ac-av-probe/parallel/825/Cargo.toml`（入口由T-10建立，当前未运行，selected>0），正式脚本必须汇总新V-13，不是独立壳绿就全计划绿。

**写面与交接**：本计划只写上述私有新增源、parallel/825及自身fixture/报告；现有mir/contract/ownership/place/layout/instr_spec、codec/verifier、av派发/装载/snapshot、公共lib/manifest和共用driver/recipe注册默认一个核心窗口。基面/版本/文件符号、前后owner、兼容证据及接受/释放需记录；公共tag/opcode/version由核心owner统一登记。复审期间其他Plan可以继续无公共写面的T-10，不能拿未审公共定义开工或持窗等上游。最后集成者重核已交付消费者并集，复审独立、整档仍merge-owned，测量单实例。详见[共同审计/窗口表](reports/ace-parallel-dependency-audit-2026-10-26.md)。

旧data/owned或其他已合入profile保留具名受检参考路径并实际回归；优化不能因新增Value变体导致错误fallback，实际装载/派发收益须T-02..06的生产路径计数，不以T-10表构建时间替代。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/loaded-program-runtime.md | 无本Plan合同→§5已验证的私有装载/PC/调用策略、拒绝及资源规则；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则；记录独立模块/正式集成支持边界、同源消费与准入责任 | 设计与实物一致 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-av/cache-and-conformance.md | 已交付profile→本Plan恢复/版本/当前能力准入及测量反控；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则 | 不恢复trusted或PASS | AC-04/05/06 |
| SD-03 | modify | docs/specs/auto-av/project.md | 已交付能力保持→登记本Plan实际优化/边界与未实现面 | 不冒领data/JIT | AC-06/08 |
| SD-04 | modify | docs/specs/auto-test-case/architecture.md | 保留既有runner→独立av-abc-825 recipe/父侧结果/统计反控 | 真实路径验收 | AC-01/06 |
| SD-05 | add | docs/specs/auto-av/runtime-benchmark-method.md | 无固定执行测量合同→同义负载/引擎tier/阶段/计数/噪声/工具缺口政策 | 不以跨层速度误导 | AC-02/04/06 |
| SD-06 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |

SD-05的runtime-benchmark-method.md为拟新增Spec，由本计划唯一拥有；826消费并补充其体积数据，不另立冲突规则。

## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、P95/内存及反控门保持。

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

### 并行同源与依赖边界验收（新增work-owned）

| Case ID | AC / owner | 输入与真实入口 | 区分性oracle | 覆盖边界 |
|---|---|---|---|---|
| V-13 | AC-10；T-10/T-03/T-05 | 同一拟新增生产源由parallel/825构建并在核心窗口后由真实生产adapter消费；坏输入/失败分配/版本或窗口负控 | 真实verify_mir的合法短块/末块PC映射等于手算，坏MIR不产生可执行凭证，索引预算两侧分账；接入生产后前置块遍历0且未改函数表/代码身份保留，无第二dispatch；同源hash/函数映射一致，越写公共面、WIP依赖或适用必需组合缺例不得就绪 | T-10只证明已交付子集模块；T-02..06补正式profile/cache/AutoDown，未知/未交付类别诚实拒绝，不冒领全路径 |

## 7. 验收标准

| ID | owner_stage | 可观察结果/证据 |
|---|---|---|
| AC-01 | work | T-02首例真实各阶段、store/新worker恢复及父侧独立expected成立；V-01通过 |
| AC-02 | work | §5具体装载表示/PC复杂度与冻结调用计数目标成立，V-02/03逐案可复现；§5.6/V-11逐项通过（不把运行期故障当准入零执行） |
| AC-03 | work | 数值/顺序/逻辑fuel、非法输入拒绝及取消/cleanup责任保持；V-04/05及适用V-07通过 |
| AC-04 | work | 版本/cache/current grants及恢复重验成立，准入非法dispatch=0；V-01/06及适用V-04通过；§5.6/V-11逐项通过（不把运行期故障当准入零执行） |
| AC-05 | work | 所有新表/暂存真实峰值受预算，旧消费者无扩面或退化；V-05/07/09适用路径通过；§5.6/V-11逐项通过（不把运行期故障当准入零执行） |
| AC-06 | work | V-08..10真实WASM/V8分层对标、原编辑负载和父侧反控齐全，0例/skip/缺工具不报PASS；报告结论可由原始数据重算；§5.6/V-11逐项通过（不把运行期故障当准入零执行） |
| AC-07 | review | 独立核对所有AC/V/SD和遗漏/延后/绕行，绑定实现及依赖hash、无未处理findings；显式记merge-owned门禁pending ；核AC-10及V-13的真实模块/集成边界 |
| AC-08 | merge | 冻结门禁过检、SD沉淀、索引/实际合入归档/收据齐全；wt-guard clean后清唯一树，批量回归到期按仓规 |
| AC-09 | work | D828与新增能力保持全链路增量，V-12 四子case及结构/身份/资源/时间/反控raw齐全；未改函数不重编/重装载，时间绿不替代粒度绿 |
| AC-10 | work | V-13的T-10独立同源模块与T-03/T-05真实集成证据齐全，D门/公共写面接受释放/版本与消费者矩阵可核；不以模块绿替代原全路径，适用拒绝/失败控制与原增量/资源门不减 |

## 8. 执行步骤

| Task | owner_stage | 依赖 | 文件/操作/输出 | 验证与AC |
|---|---|---|---|---|
| [x] T-01 | work | 用户实施授权；只读准备可先 | 核对§3实际源码/owner/Spec/依赖身份；新增attachments/825/contract-decision.md，冻结§5全部决策/反例/基准及预算边界；冻结§5私有同源模块/harness可编译接缝、T-10子集与集成门、实际公开API和窗口登记 | 决策表无无主问题，V/AC/SD完整；AC-01..06；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [x] T-10 | work | T-01、D798、D828、独立私有写面登记 | 新`core/src/loaded_program.rs`与parallel/825独立harness/lock；最小接缝先编译，再完成真实CheckedMir/受检函数单元的PC/参数映射派生表，O(B+I)构建、预算与未改函数身份保留；不复制Frame/dispatch或模拟执行；不改公共装配 | V-13模块子集；AC-10；未建入口/0例不能绿，完整case待T-03/T-05生产同源证明 |
| [ ] T-02 | work | T-01/T-10、D798、D828、核心/注册窗口接受 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； §3真实核心/adapter、fixtures/脚本与正式recipe首例，persist→新worker恢复；记录已合入基面/API版本/写面owner与接受交接，无公共写面则明确登记；只消费T-10同源模块，不另实现；刷新已交付基面/版本、记录接受 | first scope，42/trace/真实阶段与独立oracle；AC-01/04 |
| [ ] T-03 | work | T-02 | core/loaded_program、av/mir相关：实现O(1)PC与私有映射；保留参考路径/版本边界；生产直接消费T-10同源，完成适用V-13集成轴 | targeted scope V-02/03及合法反控；AC-02；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-04 | work | T-03 | av调用/帧策略计数改善及所有§5安全/资源反例；先完成findings；按§5核已交付消费者并集/类别拒绝及同源，完成V-13适用轴 | targeted V-04/05/07；AC-02/03/05；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-05 | work | T-04 | probe cache/worker/兼容consumer最小适配，错版本/授权/代际/损坏及真实restart；按§5核已交付消费者并集/类别拒绝及同源，完成V-13适用轴 | compat V-01/06/07/09；AC-04/05；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-06 | work | T-05 | scripts/verify-av-825.ps1汇总、WASM/V8真实bench、父侧反控、SD草案及证据；fmt和独立workspace scoped check；正式脚本汇总V-13/AC-10，无模块通过冒领集成或缺适用必需例 | acceptance V-01..10非0例/漏例/skip；AC-01..06；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-09 | work | D828及本计划其他work实现完成 | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-12 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档 |
| [ ] T-07 | review | T-01..06完成、T-09 完成、T-10及AC-10完成 | 同树独立验收、复核SD/遗漏和峰值/测量原始证据，记录绑定版本review；独立核依赖/同源与窗口并集及V-13 | AC-07，merge-owned门禁pending；不提前跑整档 |
| [ ] T-08 | merge | T-07 pass | 冻结提交执行独立workspace完整有限合同测试及按实际触面裸cargo t/必要tv；SD沉淀、merge/archive/ledger、到期回归及清树 | AC-08；门禁失败归因，代码改变先受影响复审，清树前wt-guard |

worktree及build/test仅在本Plan树，簿记主检出。work/review不运行整档t/tv/tt等；定向验收必做，最终review后的merge统一冻结门禁。tf只按到期在主检出单实例。未改Rust的本轮严格不跑cargo/docs_gen。

## 9. 复审记录

2026-10-26 stage:work | PLAN-825 r5 | outcome:blocked（仅 T-02 共享核心窗口；T-01/T-10 私有模块完成，非整体实施/验收 pass） | authority:用户“计划825…实施”，后续明确要求自行检查未知锁文件 | worktree_path:D:/autostack/.wt/lang-825/auto-lang | branch:plan-825-dev | base_commit:9c7a7436f97799b3fb593a62e64e7afe22fc1de5 | code_commit:22a770c482f8df2cc92f637e43c5a8a83bc94701 | dependency_revisions:同树已合入 auto-av-core@base；D798 archived r7/11of11、D828 archived r3/11of11(delivery944403b50为base祖先)；auto-down=c3464d0a0d36e4f8a7988b0c0c26448fc52361b4仅记录未消费 | task_ids:T-01/T-10 completed，current_step=2,total_steps=10 | evidence:[决策/接口/资源/候选/必需case清单](attachments/825/contract-decision.md)、[检查报告及raw](attachments/825/work-r5-20261026/report.md)；独立harness同源构建、定向av_runtime_825 **11/11**、check/clippy(-D warnings)/fmt/diff-check通过；私有新源+壳/lock共5文件，树clean；未改公共装配/dispatch/codec/规范/ledger。模块证明PC直接索引/参数映射、预算两侧/分配故障回收、未改对象/代际保留与错误表/重建反控；数据/owned仅局部builder受控拒，原受检执行未改。候选微例真实生产param_slots调用3份/48bytes容量→驻留表查询0份，首次表账100bytes，非AV加速结论。主检crates/test六未跟踪锁均Cargo生成且无tracked源码diff，原样保留，不复制到实施树。blockers:822当前execution_done7/9、next独立review；无公共窗口释放，不能改av/lib/snapshot/driver/recipe或取822 WIP。T-02..06/T-09、AC01..06/09/10及V13生产轴全pending；WASM/V8只探测工具版本/flags，未实跑tier/benchmark。merge-owned冻结整档门禁pending；本轮未跑整档或docs_gen、未送整体final review/合入/清树。next_responsible_role:822独立review/merge完成交付释放→825 work刷新基面/API/profile/预算/已交付消费者并集并接受窗口，继续T-02；无需用户重批原范围。

2026-10-26 stage:new | PLAN-825 r4→r5 | outcome:pass（依赖/并行合同可交work，非实现通过或D门自动解锁） | authority:用户明确要求实际依赖审计与独立模块并行 | changed:§4–8两阶段门、T-10/AC-10/V-13、SD-01和当前入口；total9→10，原0进度/drafting及预算保持 | next:T-01后依D798/D828和私有写面登记做T-10，T-02仍等D798/D828与核心窗；不代启实施 | evidence:[审计](reports/ace-parallel-dependency-audit-2026-10-26.md)、[预检](attachments/ace-parallel-20261026/preflight.md)。

2026-10-21 stage:new | PLAN-825 r3→r4 | outcome:pass（排期/交接合同静态修订，非实现验收或依赖解锁） | authority:用户确认828后优先817/818、827和820功能线，825/826择机插入 | changed:§4授权、§5排期/共享单写、§6交接核查、T-02及§10当前入口；T/AC/V/SD ID、数量、状态和原进度保留 | next:只读准备可先，D828及本计划原D门/单写交接满足后work；不自动启动 | evidence:[本轮预检](attachments/ace-order-20261021/preflight.md)。

2026-10-20 stage:new | PLAN-825 r2→r3 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-09 / V-12四子case / SD-06 / T-09；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 8→9，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

### 文档合同复审与r2修订（2026-10-20）

stage:review；review_scope:文档合同定向复核（非实现final）；reviewer_identity:同一聊天Codex，曾起草本文件，不声称上下文隔离的外部独立审稿。入审r1结论needs_replan，依用户授权直接r1→r2修订§5.6、V-11、AC/T/SD及Scope映射，drafting 0/8保持。出审文档检查见[报告](reports/av-architecture-contract-review-2026-10-20.md)，没有代码/运行层验收pass，AC-07/08及依赖D门仍pending。

2026-10-20，stage:new；outcome:pass（合同静态预检，非实现验收）。中央分配器在互斥/独占allocation.lock下分配825，未覆盖同号；r1 drafting，0/8，T-01..08均pending。本轮文档授权已记录；D-798未实际满足、实施未启动，不因起草通过宣称work就绪/实现pass。采纳与来源见[记录](attachments/av-abc-adoption-20261020/decisions.md)。

## 10. 待澄清事项

用户已授权本次依赖/并行合同修订，无需重新批准同范围方向。当前先T-01冻结真实公开API/独立harness最小接缝与适用矩阵，再在D798/D828及私有写面登记后做T-10；T-02正式首例与集成须D798/D828及核心/注册窗口。不等822/823/824；T-02及真实loader/三执行入口派发适配仍等已交付基面与共享核心窗口。av.rs::ModuleView是pub(crate)、Frame私有，并不存在已交付通用插件executor API。

旧data/owned或其他已合入profile保留具名受检参考路径并实际回归；优化不能因新增Value变体导致错误fallback，实际装载/派发收益须T-02..06的生产路径计数，不以T-10表构建时间替代。 若实际接口无法兑现，应在T-10最小接缝阶段给具体输入/符号/预算反例并回new，不静默复制WIP、弱化验证/资源门或退回第二实现。当前所有任务未实施，不因审计/模块准备改变状态或宣布Plan完成；旧§9串行排期为历史。

2026-10-26 当前执行交接：T-01/T-10 完成 2/10；私有模块冻结在22a770c48，唯一树保留且clean。公共核心仍由822执行/复审/合入链持有，未取得释放/接受收据；T-02前刷新全部实际基面/版本/消费者与窗口合同。已完成的是V-13模块子集，不是AC-10全部生产证据；其余work-owned门不得豁免。详见本计划§9最新记录及attachments/825/contract-decision.md。此前“当前所有任务未实施”等句为修订时历史状态，由本记录更新；无新增待用户裁定事项。
