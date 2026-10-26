---
plan_id: PLAN-826
status: executing
feature_name: AV紧凑ABC格式与字节码体积
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
  - docs/specs/auto-av/abc-core-i32.md
  - docs/specs/auto-av/runtime-benchmark-method.md
new_spec_components:
  - docs/specs/auto-av/compact-abc-format.md
touched_goals: []
affects: [auto-av, auto-mir, auto-test-case]
---

# [PLAN-826] AV紧凑ABC格式与字节码体积

## 0. 变更摘要

承接[Design43 §12](../design/43-ac-av-typed-mir-and-abc.md#12-abc评审采纳与实施责任)、[Design44](../design/44-av-data-layout-place-and-ownership.md)及[ABC评审](reports/798-abc-design-review-2026-10-20.md)。本计划是后续性能草案，**不阻塞798交付，不成为820–824功能链硬依赖**。T-01/T-10已于唯一实施树完成独立codec切片并定向验证；T-02起正式集成尚未启动。

## 1. 目标

在D798正确性和D825装载层基础上，交付新版本紧凑ABC格式、明确的语义/debug分区及有界解码验证，以WASM/V8口径报告code/metadata/raw/压缩/loaded体积和装载成本。

仅D798既有标量语义profile的wire版本与loader适配；不新增语言类型/算术、不引入lazy trust/JIT，不承诺公共稳定ABI、不强制全局slot分配。若820–824已交付，保留其旧合法codec/执行支持或在独立新版本明确路由，不能将不支持的新profile误解为标量。

成功为实际受检执行/恢复和反控闭环、确定的优化/密度目标、可重放分阶段报告；不以未来优化预期或相同实现自证代替独立结果。

## 2. 架构方案

798提供语义MIR/portable ABC和受检执行合同，825提供私有装载执行层，826提供紧凑wire。正式功能集成链为820→821→822→{823，824}，优化正式集成825→826；各Plan T-10私有同源模块可按§5并行，**公共定义/装配与实际派发仍按单写窗口串行交接**，不是整计划全部串行，也不是允许同时改同一公共合同。

**D-825**=上游独立最终review、冻结门禁、合入、Spec/merge收据齐全，不消费WIP。只解除独立codec对D825的等待；T-02首例、正式版本路由/准入→LoadedProgram→AV与最终集成仍要求D825/D828和核心窗口。825的私有帧表示不是wire编码算法的输入。 T-01只读准备可先；T-10在D798/D828及私有写面登记后可做真实独立模块开发，不占公共核心窗口。实际全路径构建T-02起遵守本阶段D门/窗口与原授权。具体规则见§5并行合同。

## 3. 技术栈

Rust与D-798共享的独立workspace；当前已合入源码为 `experimental/ac-av-probe/`，实施时在本Plan唯一树读取；原798实施树是历史位置，不作为依赖。code owner是其 `core/src/{mir,mir_verify,abc,abc_verify,av,contract}.rs` 与 `probe/src/{cache,driver,worker_restore}.rs` 相关符号，T-01以实际交付树定位，不从本轮主检出不存在的源码推断。适配D825已交付LoadedProgram接口，codec/profile版本在T-01冻结。

拟新增 `scripts/verify-av-826.ps1`，Scope=first/targeted/compat/bench/acceptance，测试前缀 `av_wire_826`、fixtures `test/cases/av-abc/826/`、AutoDown recipe `av-abc-826`。这些未来入口由T-02/06建立，当前不存在、未运行；不得把建议命令写成已验证证据。

唯一实施树 `D:/autostack/.wt/lang-826/auto-lang`、分支 `plan-826-dev` 已建立并保留给复审/merge。根workspace成员/产品manifest、旧VM/UI/产品SDK不在范围；正式recipe在当前真实owner处的最小注册/父侧判定接线为明确例外，按实际根Rust触面验证。正式recipe注册位置服从814–816实际owner交接，不复制协议/runner。

## 4. 需求分析与背景调查

2026-10-26用户授权审计/修订823–826并行边界，本轮仅改计划与战略，不启动实现或其他会话。取材初始821 archived r6、822 executing 6/9；期间822 R1 needs_fix重开至4/9，两次均非D822（他方进度原样保留）；实际输入/代码接缝与指纹见[依赖审计](reports/ace-parallel-dependency-audit-2026-10-26.md)及[inputs](attachments/ace-parallel-20261026/inputs.json)。旧“整计划串行/仅只读准备”排期按下文两阶段合同更新，原目标/非目标/预算和已有进度保留。

用户2026-10-21确认：828之后优先推进817/818、827与820功能线，825/826择机插入；全链路增量、D828门和原硬依赖保留。授权本轮更新计划/战略及单写交接规则，不启动实施、不改变已有进度/预算/支持面；当前合同以§5–8及共同排期为准，旧排期/待裁定记录为历史。

用户于2026-10-20认可ABC评审建议，要求补入Design43/44与实施计划，区分798正确性和后续性能，**先不写代码**。仅授权文档修订/计划起草；没有新模型、代理、安装工具或性能百分比授权，不自动进入work。已有798工作授权及R10finding保持，不能由本计划替其宣告交付。

取材：[Spec overview](../specs/overview.md)、[HIR阶段](../specs/auto-hir/stage-contract.md)、[语义身份](../specs/auto-semantics/project.md)、[产物合同](../specs/auto-ac/artifact-contract.md)、[测试协议](../specs/auto-test-case/architecture.md)。auto-av/auto-mir本轮尚未正式交付，目标Spec路径为proposed，不回写现状。来源提交/SHA和采纳映射见[inputs](attachments/av-abc-adoption-20261020/inputs.json.txt)、[decisions](attachments/av-abc-adoption-20261020/decisions.md)和[评审基线](attachments/av-abc-review-20261020/baseline.json.txt)。

调查时798树为f24bd46a6，主检出91a1b622d；该树的LoadedModule持CheckedMir、AV逐步调用block_base累加前块，CALL分配参数/槽，wire多为u32；它们是待优化观察，不是合入事实。798 r4的kind正确性及资源证据必须先收口，不能围绕已知错误分类设计新格式。WASM/V8一手资料及使用限制沿Design43 §12.4，不把外部机制当AV已有能力。

## 5. 详细设计

### 5.1 编码决策与首例

T-01在 attachments/826/contract-decision.md 比较ULEB与窄操作数+Wide至少两种候选，按固定标量语料的code/metadata、解码工作/峰值选择一个实施；冻结字节序、opcode/操作数、整数最大位宽/字节数、最短编码政策和profile cap。Wasm允许部分非最短LEB，AV自己的canonical政策不冒称继承Wasm要求。

新wire version不得重解释旧版本；kind/entry/imports规则消费D798已交付合同，不再用函数数猜类别。旧版真实编号以D依赖为准，T-01冻结保留独立decoder或显式拒绝/缓存重建策略；版本/profile/producer/query key同步，不加载歧义旧cache。源码语义和AV逻辑fuel不因wire变小改变。

T-02首例：旧profile同一个含调用/短路的完整模块→新ABC→decode/verify→D825私有LoadedProgram→AV，返回42/trace独立expected一致；store持久化→新worker恢复重验，旧版路径按策略判定。新旧版本均不恢复trusted。失败先回规划，不用改结果/跳过检查补绿。

### 5.2 分区与有界验证

明确module header、section目录/长度、函数body界限；类型/常量/函数签名/控制目标、kind/entry/imports/能力要求、必要owner/cleanup是语义，source/name/debug可选。现有D798 driver已有spanless code+sidecar，本计划提升完整格式分区，并沿用其位置重绑定。

每段checked length/offset/累计预算；重复/缺必需section、越界/重叠、超长varint、无效目标/opcode受控拒绝。未知语义section/opcode/required feature拒绝；未知可选debug只在已冻结封装允许跳过时按长度跳过，不能藏语义。可读目录不等于可执行；所有适用验证完成才能产生LoadedProgram，不因section独立而跳过全模块规则，不实施懒授信。

### 5.3 密度目标与兼容

小索引（各操作数0..127）body编码目标：BIN≤5B、COPY≤3B、RETURN≤2B（含opcode/子操作，不含模块头、表和debug）。T-01验证目标可行后冻结测试字节；大索引有显式扩展，不截断。短立即数/常量池选择、参数/槽元数据压缩若采用须在T-01冻结语义/空间收益反控；未采用项留在决策表，不伪称本计划实现slot复用/融合。

密度目标只说明指令body，微程序整体可能由header/表主导；报告same-semantics模块的raw总长、分段与压缩长，不用body比例冒充总体收益。解码成D825私有表示后执行，不能在每次分派重新解释整个变长wire。派生索引和解码暂存计入原资源额，不因紧凑磁盘放松内存/验证预算。

### 5.4 对标与资源

固定与825共同负载，另含大量小函数/短常量、较大合法索引和有/无debug组；手审结果/trace，AV数值与Wasm/JS语义先对齐。Wasm binary的code/types/imports/export/custom等与AV对应分项，V8 Ignition只报告有真实采集方法的bytecode/metadata，不将引擎可执行文件或JS源码长度冒充bytecode；无可比压缩流则标不可比，不填0。

raw/同算法同参数压缩/loaded分列，记录compression工具版本、引擎tier/flags、源码/二进制身份。报告encode、decode/verify/load、冷/热执行、编辑P95和真实峰值；维持D依赖原预算与P95，不许为更小wire接受不受控解码。WASM/V8必需对照输入真实执行及体积采集，缺工具不能报总体完成。比较方法沿Design43 §12.4，结论仅覆盖固定样本。

### 5.5 共用资源与证据纪律

沿D依赖冻结的适用cap交集，不放宽旧profile：当前拟沿输入4MiB、表上限65536、验证200万步/5s、执行100万步/1024帧/2s及值/槽合计8MiB、受管32MiB（hot8/staging16/execution8）、worker增量128MiB；T-01必须核对单位/实际更小限制，冲突回规划。表、索引、解码/装载暂存、参考/优化双表示均计量，不各领一份预算；父侧采样进程真实峰值。实验政策不是永久语言上限。

invalid/unsupported、语言trap、resource/cancel、infra分态，错误在规定门发生；分配前cap、失败后真实回收，不能仅timeout充负例。沿798固定编辑负载端到端P95<1s；新bench单实例串行，延迟/峰值含噪声与原样本，不许删坏样本补绿。

### 5.6 文档复审补充合同（r2）

硬依赖和版本路由按Design43 §12.5及当前排期：825等D798/D828，826等D825/D828；820–824不是反向前置。820指令表尚未交付时消费D798真实opcode/type/fuel合同，T-01制作已有操作覆盖清单，待820接手合并唯一表，不提前要求820新API。若功能profile已交付则其既有受检路径/codec必须保留并实际回归；未知profile不静默放入标量执行层。只在满足依赖后申请公共写入窗口，不持窗口等待owner。

准入非法dispatch=0；已准入执行的trap/fuel/cancel允许合法前缀，要求危险目标操作/提交/目标callee为0及实际frame/slot/资源清理正确。标量面不要求未交付owner token。模块类型+常量+导入+函数合计≤65536，每函数槽+块+指令合计≤65536，不是各表各领；32MiB受管分区和128MiB进程增量分别计量，不从worker实际峰值扣前端；T-01核798已交付同口径证据，不能用历史目标未达归因充依赖PASS。

编码域不是准入域。16/32位或LEB可表达65535/65536不代表存在合法同号slot；构造65536个slot还要加block/instruction，会先超过每函数总额65536。T-01分别冻结codec字段宽度反控和满足模块/函数族预算的完整工件ref反控；若选择窄+Wide则按其扩展/保留规则测试，不强制不存在的varint。

正式测量T-01冻结四组必需入口：AV参考/新执行路径、固定Ignition的V8 JS、V8 Wasm baseline、V8 Wasm优化组；后两组的tier/flags必须由实际被测版本证明。WASM与V8共用同一引擎可行，但每组独立样本/层证据与等价结果齐全；缺工具/无法证明固定层留gap并使相应必需门非PASS，不能用unknown/0/skip假充完成。工具准备/首次构建与采样分开计时，不新增安装授权。

case清单由T-01冻结，T-02建立首例，T-03/04覆盖算法/codec与安全反控，T-05覆盖恢复/消费者，T-06统一运行。清单含每个V及子case、Scope、实际入口、独立oracle、阶段/码、必需计数和数据文件身份；acceptance必须覆盖全集合，不能用重复执行一个family凑数。脚本first只验首例子集，不能代表整个V通过。

### 全链路增量兼容与828优先排期（r3）

798实际交付后优先828，本计划在原依赖和D828均满足后取得共享写窗口。D828=当前revision最终review/冻结门禁/实际合入/Spec与merge收据齐全，不消费WIP；只读调研/合同准备可先，已有准备保留。共同合同见[ACE顺序与兼容](reports/ace-execution-order-and-incremental-compatibility-2026-10-20.md)与[828](828-av-function-incremental-session.md)。

专项：紧凑wire保持函数寻址/上下文和828安装接口；热编辑只decode/verify变更单元，其他句柄保留；格式迁移走单独版本路径。

T-09 是work必做项：接入D828公共编译/装载/snapshot接口，保留稳定函数身份、私有凭证与实际依赖。纯body编辑、签名/实际依赖/profile/配置/授权不变时，未改HIR/MIR/代码对象和代际不变，未改生产阶段及读盘/decode/verify/load为0。元数据更新单列，不能用全source hash/总revision使所有函数失效。新工件仍走适用准入，恢复不可信。

四必需子case：1)D828真实source edit基线273575B/115函数，至少30不同修订，结构与稳定会话P95<1s双门；2)本计划新增能力真实支持入口两函数局部body更新，未改单元保留且语义/清理正确；3)实际接口/layout/helper body/授权事实变化的精确失效/拒绝正反控；4)强制whole前端/lower/重装载或只保hash却换对象/代际，正式gate必非零。仅手工HIR/MIR支持的新类型不能冒领该类型完整源码能力，但scalar全链仍须重放。由T-09 建立真实入口计数/身份/IO/反控，当前未有新增证据，review不得豁免或推给merge。

原支持面/资源/原生等性能门不放宽；cold/restart/版本迁移或真实广泛失效另报，不掩盖正常body退化。旧帧/值/drop引用期间代码与布局保活，失败/取消/迟到不安装，退出后回收；不兼容布局按实际支持策略拒绝/扩大失效，不擅自迁移活值。

### 并行模块与集成两阶段合同（2026-10-26）

只解除独立codec对D825的等待；T-02首例、正式版本路由/准入→LoadedProgram→AV与最终集成仍要求D825/D828和核心窗口。825的私有帧表示不是wire编码算法的输入。

**T-01→T-10独立模块门**：依D798/D828及已合入公开接口开展消费已交付标量MirModule/指令合同的新版字节编解码、section/函数目录、密度/非法长度与整数边界；decode只产未授信MIR并走真实verify_mir，不构造LoadedModule。单一生产源为`core/src/compact_abc.rs`（拟新增），测试壳拟为`experimental/ac-av-probe/parallel/826/`独立workspace/lock，仅用同树已合入auto-av-core。T-01冻结可达符号、具体路径/owner、支持子集和预算；T-10先证明最小可编译接缝，再实现真实模块与故障oracle。可由harness重导出已交付core公开模块并路径编译同一源，生产阶段只加薄adapter；不复制值/IR/store/VM，不改core/lib.rs/父members，不产生依赖环。若需要pub(crate)私有符号，不绕过权限，登记具体缺口并申请独立验收合入的小接缝；不能从上游WIP取代码。

**T-02集成门**：T-10通过不等于profile/Plan交付。满足原D门后同步最新master，刷新接口/版本/预算，取得共享核心/注册窗口，完成原首例真实verify→ABC→AV/store/new-worker/AutoDown和trace/资源；继续T-03..06/T-09及最终review。所有模块独立通过不能代替原全路径、精确增量、P95/内存或反控。T-10测试命令为`cargo test --manifest-path experimental/ac-av-probe/parallel/826/Cargo.toml`（入口已建立，9 selected/9 passed/0 ignored @8190551fd），正式脚本必须汇总新V-13，不是独立壳绿就全计划绿。

**写面与交接**：本计划只写上述私有新增源、parallel/826及自身fixture/报告；现有mir/contract/ownership/place/layout/instr_spec、codec/verifier、av派发/装载/snapshot、公共lib/manifest和共用driver/recipe注册默认一个核心窗口。基面/版本/文件符号、前后owner、兼容证据及接受/释放需记录；公共tag/opcode/version由核心owner统一登记。复审期间其他Plan可以继续无公共写面的T-10，不能拿未审公共定义开工或持窗等上游。最后集成者重核已交付消费者并集，复审独立、整档仍merge-owned，测量单实例。详见[共同审计/窗口表](reports/ace-parallel-dependency-audit-2026-10-26.md)。

不得把新wire填成旧DecodedAbc版本绕过入口；正式新wire准入在D825后接真实loader，T-10字节/密度通过不能声称V-01全路径/V-08 loaded体积或执行性能通过。已交付data/owned codec路由保留。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-av/compact-abc-format.md | 无本Plan合同→§5已验证的版本化紧凑wire/section/密度、拒绝及资源规则；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则；记录独立模块/正式集成支持边界、同源消费与准入责任 | 设计与实物一致 | AC-01/02/03/04 |
| SD-02 | modify | docs/specs/auto-av/cache-and-conformance.md | 已交付profile→本Plan恢复/版本/当前能力准入及测量反控；r2纳入§5.6/V-11版本依赖/阶段计数与预算规则 | 不恢复trusted或PASS | AC-04/05/06 |
| SD-03 | modify | docs/specs/auto-av/project.md | 已交付能力保持→登记本Plan实际优化/边界与未实现面 | 不冒领data/JIT | AC-06/08 |
| SD-04 | modify | docs/specs/auto-test-case/architecture.md | 保留既有runner→独立av-abc-826 recipe/父侧结果/统计反控 | 真实路径验收 | AC-01/06 |
| SD-05 | modify | docs/specs/auto-av/abc-core-i32.md | 旧profile单一wire→同一语义profile的版本路由/旧缓存策略与必需元数据 | 不重解释旧版本 | AC-02/04/06 |
| SD-06 | modify | docs/specs/auto-av/runtime-benchmark-method.md | D825测量合同→追加新wire分段/压缩/loaded体积及编码域与准入域区分、可比边界 | 体积口径唯一来源 | AC-02/05/06 |
| SD-07 | modify | docs/specs/auto-av/resident-function-install.md | 继承D828函数IR/代码复用、受检安装/依赖/寿命；新增能力不退回整模块热重编/装载 | 用户明确的顺序与增量机制保留 | AC-09 |


## 6. 测试设计

排期交接沿本节既有兼容性入口与独立review核查：对照记录中的实际基面/API/版本/消费者验证结果；公共写面无交接、依赖未交付或旧证据失配时不得判就绪。不新增整档测试；原增量、AutoDown、P95/内存及反控门保持。

T-01已冻结实际输入与独立expected，T-10模块子集已定向运行；其余fixture/正式测量仍未实现。T-02建立真实首例及recipe，T-03/04完成补充反例，T-05覆盖cache/兼容，T-06冻结原始测量脚本并汇总；核心补充矩阵如下。

| Case | AC / owner | 输入/状态 | 真实入口 | 结果/区分错误实现的断言 |
|---|---|---|---|---|
| V-01 | AC-01；T-02 | 同一完整模块、unit、旧新wire、单函数未走自调用分支 | encode/decode/verify→LoadedProgram→AV | 合法42/trace一致；kind保真，旧版按策略拒绝/重建/独立解码 |
| V-02 | AC-02；T-03/04 | 索引0/1/127的BIN/COPY/RETURN | encoder字节+独立手工decoder期望 | body分别≤5/3/2B，模块总长与分段合计一致 |
| V-03 | AC-02；T-03/04 | 127/128、255/256的可准入小表；65535/65536数值域编码与同索引非法ref；按选定格式的溢出/非规范编码 | 真实公开decoder/verifier | codec数值域可表达65535/65536不截断，不宣称相同索引可准入；ref超实际表长为invalid，表族超额为resource；选定格式的非法编码有界拒绝 |
| V-04 | AC-03；T-03/04 | 截断/尾随/重复/缺必需、越界/重叠长度、未知required/debug | 手工变异实际bytes→真实loader | 未知必需拒绝；可选debug仅按冻结政策跳过；非法dispatch=0 |
| V-05 | AC-03；T-03/04 | strip前后entry/权限/活跃状态语义、错类型/槽/控制目标 | 新codec verifier/AV | 语义一致，debug可选；好hash非法内层仍拒绝，无授信跳过 |
| V-06 | AC-04；T-05 | fresh/hit/new worker、旧producer/wire、错能力、位置变化 | driver/cache实际恢复 | 重建受检执行层、当前授权必检、诊断位置新绑定 |
| V-07 | AC-05；T-05 | 大但合法表、恶意长度、取消/超时/fuel两侧 | decoder/loader/execute+父侧峰值 | 分配前cap，全部暂存/索引计峰值，fuel/取消同原语义 |
| V-08 | AC-06；T-06 | 825共同负载、小函数/常量/大索引、有无debug | 真实编码/压缩采集与引擎运行 | 结果一致；分段raw/压缩/loaded齐全，tier/版本固定 |
| V-09 | AC-06；T-06 | 实际已交付817/818/data、D798原编辑负载 | 其真实支持面recipe+同机测量 | 不扩/退化原语义，原P95及预算成立 |
| V-10 | AC-06；T-06 | 错长度/结果/expected、漏section计数、skip、伪采集0 | 验证脚本/AutoDown父侧 | 正控PASS；各错误使总体非零/ERROR，不缓存PASS |
| V-11 | AC-02/04/05/06；T-01/03/04/05/06 | 编码数值与可准入ref分离；旧/新wire路由及已交付data codec；漏子case/重复冒计/缺tier证据 | 实际D依赖核心/公开loader、当前consumer及验证脚本父侧清单 | 无反向依赖820；已交付支持矩阵保留；准入/运行计数分开，必需清单及资源口径按§5.6，无漏例假PASS | 不扩标量支持，未交付data只测unknown-profile拒绝，不造token/消费者 |

各V以对应§5不变量为oracle，不只看测试family绿；语义适用面之外明确unsupported。好hash坏内层、手工输入和真实persist/restart均需验证。时间/数量由T-01固定可重放输入，不能用稀释工作量赢基准。

实现/内部检查/独立最终review在同一唯一树顺序进行，未指定人/模型，不创建代理。未来work仅定向测试/fmt/check；例如 `powershell -NoProfile -File scripts/verify-av-826.ps1 -Scope first` 建立后应返回0且断言真实执行42/新worker恢复。固定Scope覆盖：first为T-02首例；targeted负责V-01..05/07；compat负责V-06/07及V-11的消费者/路由子例；bench负责V-08/09和V-11的计量子例；acceptance负责V-01..11全体以及V-10父侧反控。V-09的旧consumer语义由compat先执行、其计量由bench完成；清单区分子case，重复执行不重复计完成。未来入口由T-02/06创建，本轮未运行。

### 本轮新增的增量兼容验收（work-owned）

| Case ID | AC / owner | 输入 | 实际入口 | 区分性断言 | 边界 |
|---|---|---|---|---|---|
| V-12 | AC-09 / T-09 | D828 source30修订+新增能力两函数+依赖变化+强制退化四子case | 本计划真实consumer/正式recipe与D828 edit接口，由T-09 接线 | 未改对象/代际相同、未改函数生产阶段/产物IO=0，结果/清理正确，P95<1s；强制退化非零 | 当前profile真实起点，不冒领未交付源码/迁移能力，四子case无skip |

### 并行同源与依赖边界验收（新增work-owned）

| Case ID | AC / owner | 输入与真实入口 | 区分性oracle | 覆盖边界 |
|---|---|---|---|---|
| V-13 | AC-10；T-10/T-03/T-05 | 同一拟新增生产源由parallel/826构建并在核心窗口后由真实生产adapter消费；坏输入/失败分配/版本或窗口负控 | 真实MIR同源codec往返，0/127/128/255/256及溢出/截断/重复section有手算oracle与分配前预算；decode不能直接执行，D825后生产同源codec经真实准入/AV/store恢复，禁止旧version伪装；同源hash/函数映射一致，越写公共面、WIP依赖或适用必需组合缺例不得就绪 | T-10只证明已交付子集模块；T-02..06补正式profile/cache/AutoDown，未知/未交付类别诚实拒绝，不冒领全路径 |

## 7. 验收标准

| ID | owner_stage | 可观察结果/证据 |
|---|---|---|
| AC-01 | work | T-02首例真实各阶段、store/新worker恢复及父侧独立expected成立；V-01通过 |
| AC-02 | work | §5具体新版本编码/分区及5/3/2B密度目标成立，V-02/03逐案可复现；§5.6/V-11逐项通过（不把运行期故障当准入零执行） |
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
| [x] T-01 | work | 用户实施授权；只读准备可先 | 核对§3实际源码/owner/Spec/依赖身份；新增attachments/826/contract-decision.md，冻结§5全部决策/反例/基准及预算边界；冻结§5私有同源模块/harness可编译接缝、T-10子集与集成门、实际公开API和窗口登记 | [✅ 已完成 @8190551fd] canonical ULEB/AVAC1、section/预算/完整V-AC-SD清单与候选25B/39B反控已冻结；详见contract-decision §1–8 |
| [x] T-10 | work | T-01、D798、D828、独立私有写面登记 | 新`core/src/compact_abc.rs`与parallel/826独立harness/lock；最小接缝先编译，再完成消费已交付标量MirModule/指令合同的新版字节编解码、section/函数目录、密度/非法长度与整数边界；decode只产未授信MIR并走真实verify_mir，不构造LoadedModule；不改公共装配 | [✅ 已完成 @8190551fd] 同源harness 9/9、0 ignored、fmt/diff-check过；真实MIR/5-3-2B/目录/边界/损坏/分配前预算与真实verify已验，仅V-13模块子集，生产同源/cache/恢复待T-03/05 |
| [ ] T-02 | work | T-01/T-10、D825、D828、核心/注册窗口接受 | D828后先刷新T-01接口/版本/预算合同，旧准备不冒领新依赖pass； §3真实核心/adapter、fixtures/脚本与正式recipe首例，persist→新worker恢复；记录已合入基面/API版本/写面owner与接受交接，无公共写面则明确登记；只消费T-10同源模块，不另实现；刷新已交付基面/版本、记录接受 | first scope，42/trace/真实阶段与独立oracle；AC-01/04 |
| [ ] T-03 | work | T-02 | core/abc、abc_verify及loader：实现紧凑wire/section与密度；保留参考路径/版本边界；生产直接消费T-10同源，完成适用V-13集成轴 | targeted scope V-02/03及合法反控；AC-02；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-04 | work | T-03 | codec异常长度/required section/语义必需信息及所有§5安全/资源反例；先完成findings；按§5核已交付消费者并集/类别拒绝及同源，完成V-13适用轴 | targeted V-04/05/07；AC-02/03/05；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-05 | work | T-04 | probe cache/worker/兼容consumer最小适配，错版本/授权/代际/损坏及真实restart；按§5核已交付消费者并集/类别拒绝及同源，完成V-13适用轴 | compat V-01/06/07/09；AC-04/05；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-06 | work | T-05 | scripts/verify-av-826.ps1汇总、WASM/V8真实bench、父侧反控、SD草案及证据；fmt和独立workspace scoped check；正式脚本汇总V-13/AC-10，无模块通过冒领集成或缺适用必需例 | acceptance V-01..10非0例/漏例/skip；AC-01..06；r2按§5.6负责V-11及清单对应决策/实施/反控/恢复/汇总 |
| [ ] T-09 | work | D828及本计划其他work实现完成 | 实际consumer接入828编译/装载/snapshot接口并落实专项；建立V-12 四子case生产观测/身份/依赖/失败控制及正式脚本接线，更新SD/coverage | AC-09；共同基线+新增能力兼容定向验收，未实测不勾；送审前不跑整档 |
| [ ] T-07 | review | T-01..06完成、T-09 完成、T-10及AC-10完成 | 同树独立验收、复核SD/遗漏和峰值/测量原始证据，记录绑定版本review；独立核依赖/同源与窗口并集及V-13 | AC-07，merge-owned门禁pending；不提前跑整档 |
| [ ] T-08 | merge | T-07 pass | 冻结提交执行独立workspace完整有限合同测试及按实际触面裸cargo t/必要tv；SD沉淀、merge/archive/ledger、到期回归及清树 | AC-08；门禁失败归因，代码改变先受影响复审，清树前wt-guard |

worktree及build/test仅在本Plan树，簿记主检出。work/review不运行整档t/tv/tt等；定向验收必做，最终review后的merge统一冻结门禁。tf只按到期在主检出单实例。未改Rust的本轮严格不跑cargo/docs_gen。

## 9. 复审记录

2026-10-26 stage:work | plan_id:PLAN-826 | plan_revision:5 | outcome:pass（仅T-01/T-10授权切片完成，整Plan保持executing 2/10） | worktree_path:D:/autostack/.wt/lang-826/auto-lang | branch:plan-826-dev | base_commit:e417247a5ca51aa3ba305215be52b9d543071a19 | code_commit:8190551fd38cee179514ed7da609469430bd6d1e（树clean） | dependency_revisions:D798/D828已归档交付，消费同基面已合入auto-av-core；D825未消费 | task_ids:T-01,T-10 | evidence:[contract-decision](attachments/826/contract-decision.md) §1–9；独立harness `cargo test --manifest-path experimental/ac-av-probe/parallel/826/Cargo.toml` 9/9、0 ignored，fmt/diff-check过、无警告；canonical ULEB、真实MIR+verify、目录/整数/损坏/分配前预算通过 | blockers:T-02起等D825交付和公共核心/注册窗口；生产adapter同源、AV/store/new-worker/AutoDown、V-08测量及完整AC-10未做 | next_responsible_role:review（仅本切片）；merge-owned统一整档/触面门禁pending，未合入/未发布Spec/未清树。

2026-10-26 stage:new | PLAN-826 r4→r5 | outcome:pass（依赖/并行合同可交work，非实现通过或D门自动解锁） | authority:用户明确要求实际依赖审计与独立模块并行 | changed:§4–8两阶段门、T-10/AC-10/V-13、SD-01和当前入口；total9→10，原0进度/drafting及预算保持 | next:T-01后依D798/D828和私有写面登记做T-10，T-02仍等D825/D828与核心窗；不代启实施 | evidence:[审计](reports/ace-parallel-dependency-audit-2026-10-26.md)、[预检](attachments/ace-parallel-20261026/preflight.md)。

2026-10-21 stage:new | PLAN-826 r3→r4 | outcome:pass（排期/交接合同静态修订，非实现验收或依赖解锁） | authority:用户确认828后优先817/818、827和820功能线，825/826择机插入 | changed:§4授权、§5排期/共享单写、§6交接核查、T-02及§10当前入口；T/AC/V/SD ID、数量、状态和原进度保留 | next:只读准备可先，D828及本计划原D门/单写交接满足后work；不自动启动 | evidence:[本轮预检](attachments/ace-order-20261021/preflight.md)。

2026-10-20 stage:new | PLAN-826 r2→r3 | outcome:pass（规划修订，非实现通过） | authority: 用户指定798→828优先/后继保留全链路增量 | delta: AC-09 / V-12四子case / SD-07 / T-09；review依赖新work项，D828排期/集成门，原D门不减 | 原进度保留，total 8→9，新证据pending | next:只读准备可先，D828后串行实施；无代码/测试/Spec发布/合入。

### 文档合同复审与r2修订（2026-10-20）

stage:review；review_scope:文档合同定向复核（非实现final）；reviewer_identity:同一聊天Codex，曾起草本文件，不声称上下文隔离的外部独立审稿。入审r1结论needs_replan，依用户授权直接r1→r2修订§5.6、V-11、AC/T/SD及Scope映射，drafting 0/8保持。出审文档检查见[报告](reports/av-architecture-contract-review-2026-10-20.md)，没有代码/运行层验收pass，AC-07/08及依赖D门仍pending。

2026-10-20，stage:new；outcome:pass（合同静态预检，非实现验收）。中央分配器在互斥/独占allocation.lock下分配826，未覆盖同号；r1 drafting，0/8，T-01..08均pending。本轮文档授权已记录；D-825未实际满足、实施未启动，不因起草通过宣称work就绪/实现pass。采纳与来源见[记录](attachments/av-abc-adoption-20261020/decisions.md)。

## 10. 待澄清事项

用户已授权本次依赖/并行合同修订，无需重新批准同范围方向。T-01的真实公开API/独立harness接缝与适用矩阵、T-10独立codec均已完成；T-02正式首例与集成仍须D825/D828及核心/注册窗口。只解除独立codec对D825的等待；T-02首例、正式版本路由/准入→LoadedProgram→AV与最终集成仍要求D825/D828和核心窗口。825的私有帧表示不是wire编码算法的输入。

不得把新wire填成旧DecodedAbc版本绕过入口；正式新wire准入在D825后接真实loader，T-10字节/密度通过不能声称V-01全路径/V-08 loaded体积或执行性能通过。已交付data/owned codec路由保留。 若后续实际接口无法兑现，应给具体输入/符号/预算反例并回new，不静默复制WIP、弱化验证/资源门或退回第二实现。当前executing 2/10，仅有T-01/T-10完成，不宣布Plan完成；旧§9串行排期为历史。
