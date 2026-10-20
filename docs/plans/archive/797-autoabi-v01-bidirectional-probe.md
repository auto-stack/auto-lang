---
plan_id: PLAN-797
status: archived
feature_name: 首版 AutoABI 合同与双向互调探针
plan_revision: 1
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-20
current_step: 9
total_steps: 9
supersedes_spec_components:
  - docs/specs/auto-ac/project.md
  - docs/specs/auto-ac/data-resource-contract.md
new_spec_components:
  - docs/specs/auto-abi/project.md
  - docs/specs/auto-abi/windows-x64-v01.md
  - docs/specs/auto-abi/conformance-cache.md
touched_goals: []
affects: [auto-abi, auto-ac, auto-semantics, auto-test-case]
---

# [PLAN-797] 首版 AutoABI 合同与双向互调探针

## 0. 变更摘要

承接[ABI RFC](../design/strategy/auto-native-abi-rfc.md)与[Design41](../design/41-interface-metadata-and-foreign-abi.md)，冻结首个有界Windows x64 AutoABI候选profile，验证独立编译的AutoABI↔AutoABI、AutoABI↔C双向互调。目标是可消费的公共边界及conformance证据，不是扩大现有内部ABI的名字。

与[796语义计划](796-ac-semantic-metadata-inference-probe.md)并行：前五项工作可用有版本/hash的输入快照独立开展；最终类型描述/AutoDown接入等待796/788合入。共享边界见[并行合同](attachments/ac-foundations-20261016/parallel-contract.md)。

本计划从规范声明描述生成Cranelift原生模块，不能称Auto源码编译器已经支持全部ABI类型/独立模块；将其接回完整Auto前端另立计划。v0.1是候选标签，独立review/门禁/merge后才登记“本target有限profile冻结”，不宣布所有平台/未来类型ABI稳定。

## 1. 目标

1. 明确ABI域/版本、target、公开符号/descriptor、类型布局、调用/返回、ownership、错误及runtime最小服务。
2. 独立编译双方真实COFF/静态模块，验证AutoABI↔AutoABI、AutoABI调用C、C调用AutoABI三条腿；不是一个exe里的同编译单元自测。
3. 冻结有限资源/回调/失败支持面与不支持项，原方释放、失败不提交/部分输出回收可观察。
4. 首例纳入统一cache/增量输入、真实失效、AutoDown文件与阶段/端到端性能观测。
5. 接入796合入的语义类型描述，保留target layout为ABI侧派生，不能双方各造TypeStore。

范围：新独立 `experimental/auto-abi-probe/` workspace、C参考源/头文件/原生fixtures、脚本与报告；消费已合入ac-core link工具、783 store、D-796后的contract与probe adapter。只允许D-796后追加本recipe登记/独立模块，不重写公共decoder。禁止修改现有ac-core/native/source、runtime/text、旧774 ABI、783/787 store、crates生产FFI/诊断/包管理/生成器。

非目标：完整AutoABI所有类型/平台、C header importer、Rust shim产品、variadic/bitfield/任意union、C++ ABI、DLL loader/热重载事务、跨线程/异步保存callback、异常跨边界、MIR/优化、自举后端桥。拒绝这些形态是有限profile支持表，不是永久删去战略。

## 2. 架构方案

逻辑声明/语义TypeRef→target ABI layout与函数签名→独立模块emit→平台link→真实native conformance；C侧由独立C编译器生成obj，不能用Rust extern C或同一个Cranelift emitter代替独立C证据。

core contract库不依赖auto-lang/UI/auto-man/AVM；probe驱动可依赖Cranelift/ac-core/783，元信息和类型从796消费。AutoABI版本、平台C ABI、接口revision、descriptor Schema、runtime服务版本分别存在。Auto内部表示不被迫变成C结构，C bridge显式转换。

模型/Schema与性能从首例出现；早期用只读语义ref快照验证物理布局，D-796后必须切至真实合入contract校验并废止重复临时模型，不能留下第二套长期语义规范。

## 3. 技术栈

Rust独立workspace/Cargo.lock、同现有ac-core锁定的Cranelift 0.126栈、ObjectModule COFF；复用 `link::{find_rust_lld,find_sdk_um_dir,run_deadlined_split}` 等实际合入接口。真实C侧用可定位的MSVC cl.exe或clang/clang-cl，版本/路径/flags/SDK指纹钉死；不能误用PATH的coreutils link.exe。

T-01确认C工具和最小三腿；工具缺失是明确infra阻断，不能改用“Rust repr C足够了”伪通过。不得自动下载/安装工具或操作旧仓。所有编译/链接/运行沿用contained设施，原始bytes/退出code/阶段与输出限制使用788合入版本，未合入前只用master已交付能力，预算缺口需隔离包装/拒绝。

每次probe请求build≤60s、run≤5s、total≤70s；stdout/stderr各≤1MiB、IPC元数据≤4MiB、单实际产物≤64MiB/每请求合计≤128MiB。上限是拟议probe运行策略，准确可配置字段和超限控制由T-01冻结，不靠截断成功或提高预算过关；早期执行器若不支持强制限额须显式补齐隔离包装或报告不可行，不继承“已有deadline所以所有资源安全”的推断。

## 4. 需求分析与背景调查

当前依据[overview](../specs/overview.md)、[AC](../specs/auto-ac/project.md)、[774数据/资源合同](../specs/auto-ac/data-resource-contract.md)、[产物合同](../specs/auto-ac/artifact-contract.md)、[缓存](../specs/auto-cache/project.md)、[TestCase](../specs/auto-test-case/project.md)；Design41/42及ABI RFC是目标，不当成已发布ABI。

调查基面 `9c773601abfdb25b80c08d06922d0ad50edc653a`；逐文件hash见[planning-inputs.tsv](attachments/797/planning-inputs.tsv)。774 `experimental/ac-core/probes/data-resources/{src/lib.rs,shim}`已有Cranelift caller→显式ABI Rust shim路径，内部ABI-A–E冻结供770/771/782；**非生产AutoABI、非独立C前端对拍**。本计划不回溯更改其接受面或资源表示。

具体接缝为ac-core::link工具/执行器，现有774独立workspace/COFF范式，783 ManagedStore/identity公开API。796拥有语义TypeRef及规范描述；本计划只拥有target布局/调用/边界资源。793/794/795是诊断/宿主scroll/包锁拆分，不改其生产触面，不以独立crate名义重复抽取其基础库。

用户本轮只授权起草两计划、并行边界与cache/AutoDown/周转要求，未授权实施/开树/代理/工具安装/发布。资源阈值与候选v0.1均为本计划提出，非已测或用户指定值。

D-796：796实际独立review/门禁/合入/Spec齐全，方接真实semantic contract和正式probe recipe；D-788：788实际交付后才消费v2 worker/观察协议。796的probe adapter接入本身等D-788，因此最终T-06需二者；T-01–05不依赖787/788/796全完成，不复制其在途实现。不存在相互等待环。

## 5. 详细设计

### 5.1 首批候选profile与冻结条件

首平台Windows x86_64-pc-windows-msvc，调用约定沿平台Windows x64；公开AutoABI函数使用规范符号、显式context、i32 status及显式out参数，避免首批按值聚合/sret差异。普通scalar计算与需失败通道的转换在descriptor中显式，不能靠调用者猜隐藏参数。

拟支持：unit、i32/u32/i64/u64、f64、bool（公开u8且仅0/1）、char（u32合法Unicode标量）、借用byte/UTF-8 buffer、声明序无环record的const/mut view与caller-owned out、opaque owned handle及原方destroy、同步per-call callback+context。具体记录头/函数C原型/服务表在T-01冻结。f64用于真正GPR/XMM混合参数对拍，不宣称Auto源码已支持浮点。

buffer/record/handle不按值暴露Rust String/Vec/enum；长度固定u64字节数（文本另标UTF-8），size/align/offset checked计算，越界/溢出/非法char/bool在合法边界受控拒绝。聚合按显式pointer/out协议，不复用旧text内部12/8布局。不同逻辑名义类型即使布局相同不自动互换。

raw C指针的实际有效性/可访问内存仍是caller契约，不能承诺任意伪造地址都能安全验证。描述符预检约束适用于本计划受控link/调用入口；任意外部C程序直接跳过该入口不因有AutoABI header就获得沙箱或内存安全保证。句柄身份/代际和可检验的长度/布局错误须由实际入口拒绝。

descriptor包含ABI域/target/profile、接口revision/指纹、类型/布局/ownership/效果、符号及runtime服务要求；符号发现/metadata格式与公开调用域分版本。未知版本/错target/packing/符号/服务能力在调用前拒绝，native被调次数0。本计划不把版本不兼容自动转发。

### 5.2 独立原生三腿与独立oracle

每个producer独立命令/输入目录生成obj/lib与descriptor；consumer独立构建，链接时读取公开边界，不在同一次codegen中偷偷共享签名指针。

必验：AutoABI caller→AutoABI callee；AutoABI caller→独立C callee；独立C caller→AutoABI callee。早期AutoABI模块由本计划受限声明IR→Cranelift emitter生成，报告明确“不经Auto源码parser”；所有腿最终运行真实PE。

至少覆盖混合整数/f64/指针参数、超过四个位置参数、嵌套调用/返回、字段顺序/对齐/有padding record、空/非空buffer、out提交。C侧使用独立审定的header/sizeof/offsetof与运行结果oracle，不能双方都从同一错误layout生成器取预期。选择精确可表示的浮点输入或冻结明确误差准则；不以日志文本“call ok”判定成功。

descriptor错配测试在调用前拒绝，不执行已知不兼容签名制造UB。native语义测试采用异质非对称参数和可观察计数/校验值，互换参数或漏调用必失败。保留真实obj/头文件/命令/工具身份/返回观察；PE时间戳等非确定字节不充语义oracle。

### 5.3 所有权、错误与callback

分配由provider拥有并提供destroy；使用真实分配与独立live/alloc/free账本（不得只画计数模型），跨边界不让另一方调用自己的allocator释放。handle包含可验证身份/代际，double destroy、错误provider、借用逃逸和不支持的持久callback受控拒绝。

status/out：成功才提交已初始化输出；失败清理本次部分分配并保持caller旧输出不变。明确null/empty/borrow/mut排他、错误对象生命周期及销毁；无panic/exception穿边界，provider将测试故障映射到声明的status，进程崩溃永远不是expected-language pass。

callback仅同步per-call、不保存、不跨线程/代际；显式ctx、调用约定、错误/重入策略与资源责任。实际正例及拒绝保存/异步能力控制，不能宣称loader能在线卸载。未来热重载需pin callback/handle/deleter所依赖代码，这些字段/能力位从合同开始预留，实际装载/安全点留后续。

### 5.4 cache从首个layout/对象开始

复用783 store，namespace独立；layout/签名、producer/C对象、wrapper/header、链接manifest分别有recipe/content身份。key含语义类型版本、字段顺序/representation、target/ABI/算法、实际body、runtime/服务表、C编译器/链接器路径+文件/版本、flags/宏/SDK及受管include闭包。未知依赖不能当纯函数缓存。

T-01最小layout查询和首个obj即真实持久化；跨进程/工程命中须禁调producer/编译器反控并仍真实link/run、重新比较oracle，不缓存PASS。改ABI/布局/目标/flags/toolchain/函数体按层失效，源码不变不够。良好封装但错地址/owner/descriptor/hash不能成功调用。

早期由ABI probe合法opaque页adapter使用现有store，不改783/787底层注册或GC；保活必要descriptor/obj，失效/provenance不永久pin历史。512次接口/实现修订后无live超额时受管总量按继承预算自动收敛；句柄/实际运行中资源不是普通cache项，失败/取消释放lease且不删交付物。

### 5.5 AutoDown与性能从首例

T-01即新增 `test/cases/autoabi/*.case.ad`、独立C/reference oracle与来源映射，同时通过Rust定向native测试实际执行。D-796/D-788之前标AutoDown execution pending；不能以文件存在或自造runner当通过。T-06用796注册的受限probe/run公共合同，追加本recipe/worker绑定，不修改共享decoder或偷偷接受未知字段。

各阶段trace从一开始记录descriptor校验/layout/cache/emit或C compile/link/run/比较发布、等待/实际命中与producer计数/I/O/内存。命中计数与禁调控制一致；runtime微基准不代替编译周转。

T-01冻结参考机/样本/工具。六组各30样本：fresh、layout热点、对象热点、重启磁盘热、body-only edit、ABI/layout edit（再加GC并发资源观察）。**硬门**：代表单函数AutoABI producer body edit、复用不变C consumer的“请求→正确PE执行结果”P95<1000ms；只改descriptor后应拒绝的请求→正确诊断P95<1000ms。包含本次必要校验、cache/emit/link/IPC与发布，不扣掉真实用户等待；工具准备单列。全冷C工具构建单报，不冒称SDK首次构建都亚秒。

该路径在D-796后通过正式AutoDown再测，阶段P50/P95/P99、peak memory/bytes与错误样本保留；不允许根据结果改总阈值。受管cache热点≤8MiB、布局/查询暂存和索引另≤8MiB；layout深链/大字段数（至少4096）与坏图全路径资源/拒绝控制，超预算是resource状态而非ABI兼容。native子进程内存另测，不冒称8MiB覆盖C编译器。

### 5.6 与796的最终绑定和发布边界

796 T-01输出语义ref envelope/Schema快照；797 T-01记录所读版本/hash，不改796文件。前期ABI fixtures仅保存所需语义引用与物理签名形态，不重建通用类型推导/CT库。D-796后使用其真实contract解码、校验nominal/版本/实例与layout输入，重跑全部三腿；临时快照adapter退出有收据。

freeze proposal附支持矩阵、canonical声明/头文件/descriptor、布局/符号/ownership/错误/服务版本、工具链、全部正反oracle和旧内部ABI对照。最终review通过与merge门禁后仅冻结本target/范围；修正公开兼容字段要升级候选profile，不修改历史已交付内部ABI或声称所有模块可互操作。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-abi/project.md | 无公共有限profile→候选/冻结状态、工具与支持边界 | 防冒称完整ABI | AC-01–06 |
| SD-02 | add | docs/specs/auto-abi/windows-x64-v01.md | 无首版合同→经实证布局/调用/status/out/资源/callback与拒绝 | 可独立消费 | AC-01–03 |
| SD-03 | add | docs/specs/auto-abi/conformance-cache.md | 无互调基准→三腿/独立oracle、key/依赖/trace/性能/资源 | 可重验 | AC-02–05 |
| SD-04 | modify | docs/specs/auto-ac/project.md | 补ABI probe导航，生产前端/旧profile仍原状 | 不扩大源码支持 | AC-06 |
| SD-05 | modify | docs/specs/auto-ac/data-resource-contract.md | 774内部ABI与新公共profile分域，禁止误用兼容声明 | 旧承诺保持 | AC-01、06 |

## 6. 测试设计

新入口由T-01–07创建：独立manifest的 `abi_contract_`、`abi_layout_`、`abi_calls_`、`abi_resource_`、`abi_cache_`、`abi_turnaround_`、`abi_admission_`；新 `scripts/verify-autoabi-797.ps1 -Scope acceptance|closeout`。目前是未来入口，不算执行证据。

| case | AC / 输入/不变式 | owner / 入口 | 区分性oracle及边界 |
|---|---|---|---|
| V-01 | AC-01 域/target/版本/符号/服务/packing错配 | T-02 / abi_contract_、abi_admission_ | 校验入口拒绝且native计数0，合法兄弟通过 |
| V-02 | AC-02 独立AutoABI双方、AutoABI→C、C→AutoABI | T-03 / abi_calls_ | 独立工具/obj，异质参数/返回/嵌套实际结果；不以同编译单元或Rust C代替 |
| V-03 | AC-01、02 padding/字段顺序/混合f64、>4参数 | T-02、03 / abi_layout_、abi_calls_ | C offsetof/sizeof与结果独立oracle，换参数/错offset反控失败 |
| V-04 | AC-03 空/非空buffer、owned handle/原方destroy、失败第N分配 | T-04 / abi_resource_ | 真资源账本、成功唯一释放、失败回滚/旧out不变；无跨allocator释放 |
| V-05 | AC-03 callback+ctx、错误status、拒持久/线程能力 | T-04 / abi_resource_ | 真回调计数/数据正确，unsupported不执行；无unwind穿界 |
| V-06 | AC-04 首次存盘/重启/跨工程，body/layout/ABI/flags/toolchain改 | T-05 / abi_cache_ | 真命中/禁调producer反控，失效正确、仍link/run/重比较 |
| V-07 | AC-04 好frame错地址、缺块、坏semantic ref/递归布局/溢出 | T-02、05 / abi_admission_ | 受控错误/重算，不panic/错调用 |
| V-08 | AC-04 512版本、lease/GC/取消，用户source/exe哨兵 | T-05 / abi_cache_ | live安全/旧版本可回收，总量收敛，非受管数据不变 |
| V-09 | AC-05 六场景各30样本、参考环境 | T-05、07 / abi_turnaround_ | 两条硬路径P95<1s、原始样本和阶段资源齐全，不删失败/扣IPC |
| V-10 | AC-06 D-796真实ref/AutoDown注册与bad actual/good expected | T-06 / auto-case run、abi_binding_ | 无永久快照分叉，正式runner检查非零/未知项红，原v1/v2兼容 |
| V-11 | AC-06 freeze proposal/支持矩阵与旧774边界 | T-07 / abi_boundary_ | 证据逐项绑定，无全平台/源码/loader扩面，SD完整 |

实施/自检后在唯一树独立review，暂停写入并绑定commit/工具/796依赖；不指定模型或创建代理。拟树 `D:/autostack/.wt/lang-797/auto-lang`，分支 `plan-797-dev`，尚未创建；主检出仅簿记，禁止链接。

work仅相关反例/模块及必要check；最终review后merge冻结提交执行独立ABI完整套件/真实C三腿与受影响796/788定向兼容。若唯一产品改动只是recipe登记，根cargo t及实际test_cases/AVM/A2R触面tv/tt由merge按diff定界；不因新独立probe跑无关整档。没有aavm/UI/语法参考/docs生成器改动不跑taa/tu/docs_gen。tf到期主检出单实例，非本计划复审门禁。

## 7. 验收标准

| ID | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | 有限profile字段/支持/拒绝/nominal布局与独立C对照齐全；V-01/03 |
| AC-02 | work | 三腿独立真实原生互调，混合参数/返回/嵌套、原始产物链有证据；V-02/03 |
| AC-03 | work | 真实ownership/错误回滚/callback与拒绝边界，不假装loader；V-04/05 |
| AC-04 | work | 首例统一store、精确失效/重启/原生禁调反控、生命周期/预算；V-06–08 |
| AC-05 | work | §5.5硬周转与全阶段资源trace，正式adapter复测可复验；V-09 |
| AC-06 | work | D-796/D-788绑定、AutoDown执行、快照退出及freeze proposal/SD/并行审计完整；V-10/11 |
| AC-07 | review | 独立审查全部AC/反例/SD/遗漏，绑定commit/target/toolchain，零阻断，merge pending明确 |
| AC-08 | merge | 冻结提交门禁、有限profile Spec沉淀/状态、归档合入与guard清理有收据，实际9/9 |

## 8. 执行步骤

| 状态 / ID | owner_stage | 依赖 | 新建/修改路径及结果 | 验证 / AC |
|---|---|---|---|---|
| [x] T-01 | work | 用户执行指令 | 新workspace/lock、attachments/797/contract-decision.md，冻结候选原型/语义ref快照/资源/工具/perf；最小layout缓存与三腿探针 | [✅ R17核验] 候选profile/工具/锁/资源边界已冻结；当前工具与锁/依赖身份确认，准入及真实C三腿定向通过。原锁finding CLOSED。见 [R17报告](attachments/797/review-r17/review.md) 与基线；历史T-01可行性证据保持。 |
| [x] T-02 | work | T-01 | contract/src/{descriptor,layout,validate}.rs、独立C header/oracle | [✅ R17核验] 24准入/布局测试、独立C oracle通过；共享DAG合法900名及1300/1700名预算拒绝的实际堆计量重放正确。F-R4-10保持CLOSED；未变原直链/小图证据有明确复用边界。见 [R17报告](attachments/797/review-r17/review.md)。 |
| [x] T-03 | work | T-02 | 新probe/src/{emit,build,link}.rs与C参考源/独立producer/consumer recipes，零改ac-core | [✅ R17核验] 7项真实原生互调通过：独立AutoABI双方、AutoABI→C、C→AutoABI含嵌套；swap/错offset/漏调负控可区分。仅声明IR有限profile。见 [R17原生证据](attachments/797/review-r17/scoped-native.log.txt)。 |
| [x] T-04 | work | T-03 | provider资源/错误/callback fixtures、真实账本及record mut/out | [✅ R17核验] 6项资源测试通过，真实原方分配/销毁、代际/回滚、UTF-8与bool-char、callback及record mut/out独立C正反控；F-R4-02保持CLOSED。见 [R17原生证据](attachments/797/review-r17/scoped-native.log.txt)。 |
| [x] T-05 | work | T-04 | store adapter/阶段trace/六场景/512churn，首批.case.ad与oracle从T-01维护 | [✅ R17独立通过] F-R16-01 CLOSED：声明嵌套产物清理后仍hit/producer1；根/一层/三层控制同过，真头新增miss/PE29=fresh29。缓存25、六场景210样本全ok；原依赖/域/升级/内存/GC findings保持关闭。正式性能见T-06；[R17报告](attachments/797/review-r17/review.md)。 |
| [x] T-06 | work | T-05、D-796、D-788 | 绑定796合入contract与受限probe adapter，只追加ABI recipe/worker；退出临时模型，正式AutoDown全执行/性能重测 | [✅ R17独立通过] 真实796 contract/788 adapter绑定与快照退役，binding3+worker2、精确父ABI11、contract87、runner26+1既有ignored。正式双硬门受控复测P95=657/106ms；首轮1010/118ms失败的61行与外部编译快照保留，未删样本或改阈值；merge须重跑。[R17报告](attachments/797/review-r17/review.md)。 |
| [x] T-07 | work | T-06 | 新verify脚本、docs/reports/plan797/、attachments/797/spec-delta.md与freeze proposal，来源/并行边界审计 | [✅ R17独立通过] 候选freeze/support/来源与退役收据/验收脚本及SD与有限实现对账；SD SHA256 `01690e79d1a8570394a1b60dc7905970f6e1012330b6411b6e6fc8269c8c5314`。规范输入稳定、goals811历史差异已协调；未发布canonical/ledger。[R17冻结SD](attachments/797/review-r17/reviewed-spec-delta.md)。 |
| [x] T-08 | review | T-07 | 同树独立review，绑定native工具/接口版本，核互调/资源/兼容承诺 | [✅ R17 final pass] AC-01–07通过，finding全闭，无新阻断/延期；受审 `5a54f588d2f6ea5cf50bd69ebb3d476c09e0d18e`，实现树clean，源/依赖/配置/Spec稳定。192不同定向最终pass+1既有ignored，首次性能失败独立披露；reviewed8/9，merge-owned门禁pending。[完整R17](attachments/797/review-r17/review.md)。 |
| [x] T-09 | merge | T-08 pass | 冻结提交门禁；仅在通过后沉淀有限profile冻结状态、Spec/ledger、归档合入、批量到期判定、guard清理 | [✅ 已完成] R17 pass 绑定 `5a54f588d`；merge `1799f86ca`（ff-only，rebase 等价全=）：canonical auto-abi 三件+auto-ac 导航/分域+INDEX；ledger v60 guarded-offline 投影（P797-D1/T/R/REL 四条）；门禁=裸 cargo t 5313/5313（ABI 计时门按 churn/画廊先例移出日常档、断言零改动、归因在案）+ verify closeout ALL PASS + 兼容 26+1ig/87；批量回归：797%5≠0 且收据 48h 内有多计划合并——交 /auto-plan:regress 到期判定 |

T-01–05可与796/787/788并行，不能提前实现依赖其未合入API的T-06。共享manifest/注册/Spec以单写者短窗口交接；不抢修他计划或把两个分支都修改同一协议当“隔离完成”。不改变旧ABI-A–E，范围变化回规划。

## 9. 复审记录

2026-10-16 stage:new；PLAN-797 r1；outcome:pass（规划契约）；next:work，T-06需D-796/D-788。新增T-01–09/AC-01–08/SD-01–05/V-01–11；0/9。仅草拟/调查，未实施/测试/安装工具或发布ABI，未创建树。

2026-10-16 stage:work；PLAN-797 r1；outcome:**blocked**（外部依赖门，非实现缺口）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；base `5dcf4e3c2`；code HEAD `defcc8eaa`（T-01 `b00100cbd`/T-02 `b4fc9c644`/T-03 `34866a61e`/T-04 `5ac81f738`/T-05 `defcc8eaa` 五提交，树 clean）；deps：ac-core/ac-artifact-probe@worktree（=master `5dcf4e3c2` 钉）、MSVC cl 19.44.35222（blake3 443ac315…）、rust-lld(sysroot)、SDK 10.0.26100.0、semref 快照=797 本地 `autoabi-semref-snapshot/0`（blake3 15f81fd0…）；task_ids T-01..T-05 [x]（current_step 5/9）。

**work-owned AC 结果**：AC-01 ✓（descriptor/1+12 维 admission 拒绝矩阵 native 0 调+合法兄弟过；布局引擎==C 编译器 sizeof/offsetof 含 padding/序/嵌套+错 offset 反控红）；AC-02 ✓（三腿真 PE 互调：A 独立双 obj、B→cl.exe C、C(cl)→AutoABI 含 C→AutoABI→C 嵌套链；异质 8 参跨 GPR/栈/XMM；swap/错 offset/漏调用负例 exit 恰 1）；AC-03 ✓（真 arena/逐槽 live+代际账本/原方 destroy/失败注入回滚 out 哨兵不变/buffer 空与 null 与 cap 规则/同步 callback 双 cb 序列+stored_cb=0；双消费者）；AC-04 ✓（ABI-L0/ABI-O1 层真持久化、命中禁调 producer、真实输入失效矩阵、tamper=受控 CAS 拒、缺块=miss 重算、512 churn rooted+leased 存活/块层收敛 live 集/用户哨兵不变）；AC-05 部分✓（六场景×30 全绿、**body-edit P95=539.5ms**、**descriptor-reject P95=0.1ms** 双硬门<1000ms、210 样本 JSONL 全保留、cl 预热单列；正式 adapter 复测归 T-06）；AC-06 阻塞（D-796/D-788）。测试面：`experimental/auto-abi-probe` 内 `cargo test` 45/45（定向，未触发整档，符合 2026-10-14 时序裁定；裸 t/tv 等为 merge-owned pending）。

**open in-contract findings**：无未修复 finding。**required coverage gaps**：T-06 绑定 796 真实 semantic contract+正式 AutoDown runner 注册与全重测；T-07 freeze proposal/verify 脚本/报告（依赖 T-06）。**blockers**：D-796=796 独立 review/门禁/合入/Spec 齐全（现 executing 5/9 未合）；D-788=788 合入（现 executing，T-05 已交未审）。**unblock**：两门满足后回 work 续 T-06（同树 `defcc8eaa` 起），重放 V-01..09+正式 AutoDown 执行。**遗留边界声明**：emit 源为受限声明 IR 非 Auto parser；v0.1-candidate 冻结态待 review/merge；774 ABI-A–E 未动。next_responsible_role: work（门后恢复）/当前由用户调度依赖计划。

2026-10-16 stage:review；review_scope:final（就绪核查 R1，未进入完整代码验收）；reviewer: 当前独立 Codex 复审聊天，未实施本分支；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `defcc8eaa1fdcfa619feda99721d0b93c25035fb`；base `5dcf4e3c2d5266cfa4ebf1444d6d7e154305d3c4`，树 clean（忽略的本机锁另列）。[完整基线、AC矩阵、Spec输入与 findings](attachments/797/review-r1/review.md)。F-797-R1-01/P1：T-06 真实 semantic contract/正式 AutoDown/性能复测未完成（AC-05/06）；F-797-R1-02/P2：T-07 验收脚本/报告/freeze/Spec delta 未提交（AC-06/07）；F-797-R1-03/P2：独立 Cargo.lock 被忽略且无提交 blob，重开 T-01 锁交付子项。D-796/D-788 未满足；本轮未运行测试、未独立认证前五项历史通过数。status:executing；current_step:4/9（保留 T-02–05 实施标记及历史证据，依赖一致性与完整验收 pending）；pending_closeout_gates: 最终 review 后由 merge 执行 ABI 完整套件/真实 C 三腿与实际触面兼容/回归，规范沉淀/归档/guard 清理均未做；next:work，原树补齐后再审，不合入。

2026-10-16 stage:work（R1 needs_fix 修复轮）；PLAN-797 r1；outcome:**blocked**（F-797-R1-01 仍等 D-796/D-788；可独立项已补齐）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；base `5dcf4e3c2`；code HEAD `635053f74`（在 `defcc8eaa` 上追加 R1 修复提交，树 clean）；deps：ac-core/ac-artifact-probe@worktree（=master `5dcf4e3c2` 钉）、MSVC cl 19.44.35222、rust-lld(sysroot)、SDK 10.0.26100.0、semref 快照仍 `autoabi-semref-snapshot/0`（blake3 `15f81fd0…`，退役归 T-06）；Cargo.lock 已入提交（blob `598833812a`，SHA256 `C7EC8DD5…`）。task_ids：T-01 [x]（锁子项关闭）、T-02–05 [x]、T-06 [ ]、T-07 [ ]（可独立材料已交）、T-08–09 [ ]；current_step 5/9。

**finding 闭环**：F-797-R1-03 **关闭**（锁强制入库 + `--locked --offline` metadata/check/test 全绿；`git ls-files`/blob==hash-object 复核）。F-797-R1-02 **部分关闭**（verify 脚本/freeze-proposal/support-matrix/SD-01–05 草稿/source-parallel-audit/`docs/reports/plan797/` 已提交；V-01–11 终收口与 hash 冻结仍等 T-06）。F-797-R1-01 **仍打开**（真实 semantic contract/正式 AutoDown/性能复测 = T-06，外部依赖门）。健康补丁：`emit.rs` 去 `unused_mut`。验证：`scripts/verify-autoabi-797.ps1 -Scope acceptance|closeout` 均 exit 0（probe `cargo test --locked --offline` 绿；closeout 清单诚实不伪称 AC-05/06）。按 2026-10-14 时序未跑整档 t/tv/taa（merge-owned pending）。**work-owned AC**：AC-01–04 历史证据保留且锁一致后可复用；AC-05 部分（正式 adapter 复测归 T-06）；AC-06 未完成。next:work（D-796/D-788 满足后 T-06/T-07 收口）/当前由用户调度依赖计划；完整 final review 仍不得提前申请。

2026-10-16 stage:review；review_scope:final（R2 就绪复核，未进入完整代码验收）；reviewer: 当前独立 Codex 复审聊天，未实施本分支；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `635053f74aa49f89d74ffef6db9c70df7c363c06`；base `5dcf4e3c2d5266cfa4ebf1444d6d7e154305d3c4`；入审/收尾 clean、HEAD 未变。[R2 报告](attachments/797/review-r2/review.md)。独立确认 F-797-R1-03 CLOSED（HEAD 锁 blob=实际 hash-object、SHA256 与 R1 本机锁一致；locked/offline metadata 重试 exit0，120 packages）；F-797-R1-02 PARTIAL/OPEN（材料已交，终稿与 V-01–11 仍待 T-06）；F-797-R1-01 OPEN（真实语义/正式 AutoDown/性能复测仍未完成，D-796/D-788 均未满足）。默认 sccache 端口阻断导致 metadata 初次 exit101，进程内临时跳过 wrapper 后成功，未改配置。无 Rust/native 测试、无完整验收 pass；Spec 草稿 SHA256 与当前输入已录报告，goals 输入较 R1 有变化需终稿对齐。status:executing；current_step:5/9；T-06/07 保持未完成，T-08/09 pending；merge-owned 门禁/收尾全部 pending；next:work，依赖实际交付后完成 T-06/T-07 再申请完整 final review。

2026-10-16 stage:work（R2 needs_fix 回环）；PLAN-797 r1；outcome:**blocked**（依赖门未变，本轮无可执行新面，零代码/零草稿改动）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；code HEAD `635053f74`（未动，树 clean）。R2 三项结论确认接收：F-797-R1-03 CLOSED、F-797-R1-02 PARTIAL（终收口待 T-06）、F-797-R1-01 OPEN（即 T-06）。**依赖门新鲜核查（当前 master `32743f07b`）**：796 worktree HEAD `d85f89fc7584d40146d026e3c79fe0db18b2a78e` ancestry→NOT-MERGED（executing 5/9，未归档）；788 worktree HEAD `720df159c656c1acc8ac32e94458060a03f5f39b` ancestry→NOT-MERGED（executing，未归档）——与 R2 报告所录一致，无新证据满足重试条件。按计划 §8 不提前实现依赖未合入 API 的 T-06；按 R2 路由不把材料补齐当实施完毕、不翻搅已冻结身份的草稿（goals.md 输入漂移留待 T-07 终稿重核）。**unblock**：D-796+D-788 实际合入/Spec 齐全后，同树（`635053f74` 起）完成 T-06（真实语义绑定/快照退出收据/正式 AutoDown 注册与执行/兼容反控/性能复测）→ T-07 终稿（含当前输入 hash 重核）→ T-08 完整 final 验收（前五项高风险路径矩阵与 merge-owned 门禁按 pending_closeout_gates 清单执行）。next_responsible_role: work（门后恢复）；依赖计划由各自 session 推进，本计划不代做。

2026-10-16 stage:review；review_scope:final（R3 就绪复核，非完整代码验收）；reviewer: 当前独立 Codex 复审聊天，未实施本分支；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `635053f74aa49f89d74ffef6db9c70df7c363c06`（与 R2 完全相同，无新增提交，status --short 空）；base `5dcf4e3c2d5266cfa4ebf1444d6d7e154305d3c4`；当前 master `f35c9aab437b564716988977027a00e36b16f9f7`。D-796 `d85f89fc7584d40146d026e3c79fe0db18b2a78e`、D-788 `720df159c656c1acc8ac32e94458060a03f5f39b` ancestry→master 均 exit1，计划仍 executing；semref 仍为 `temporary-797-local`。F-797-R1-01 OPEN、F-797-R1-02 PARTIAL/OPEN；T-06/T-07 未完成，status:executing、current_step:5/9。锁 blob `598833812a800c9d71574b5cf345f8583c082b88` 与 R2 相同，F03 保持 CLOSED；Spec 草稿 SHA256 `B74BDD3552E407078878833844D3A081E528E96667249CE7BF98ACC45E7AB5EF` 未变，最终输入重绑仍 pending。复用 [R2](attachments/797/review-r2/review.md) 的锁解析及缺口证据，因代码/依赖/测试配置/草稿均未变，不重跑 metadata 或测试，不批准完整 AC/Spec pass；未新增 finding 或启动第三轮完整代码验收。merge-owned 门禁/归档/清理仍 pending；next:work，D-796/D-788 实际交付后完成 T-06/T-07，提交新就绪基线再完整验收。

2026-10-16 stage:work（R3 needs_fix 回环）；PLAN-797 r1；outcome:**blocked**（依赖门与 R2 轮完全一致，零代码/零草稿/零材料改动）；code HEAD `635053f74`（未动，树 clean）。R3 接收：F-797-R1-01 OPEN / F-797-R1-02 PARTIAL / F-797-R1-03 CLOSED，无新增 finding。**依赖门核查（R3 后复查，与 R3 报告所录相同）**：796 `d85f89fc7` 与 788 `720df159c` 对 master ancestry 均 exit1，archive 无 788/796——门未动，无重试条件。维持 R2 回环记录的全部约束（不提前实现未合入 API 的 T-06、不翻搅冻结草稿、不重复完整验收）。**unblock 不变**：D-796+D-788 交付后同树（`635053f74` 起）T-06→T-07→T-08 完整验收。next_responsible_role: work（门后恢复）。备注：三轮复核结论与实施提交均未变化，依赖交付前重复就绪复核不会产生新信息；推进 796/788 合入（或由用户裁定缩小 T-06 绑定范围走 needs_replan 修订）是唯一改变状态的路径。

2026-10-19 stage:work（T-06/T-07 收口轮）；PLAN-797 r1；outcome:**pass→execution_done**（work-owned 任务 7/7 完成）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`（rebase 至 master `13367d127` 线，组内补 auto-down 兄弟 detached 树）；code HEAD `8679859e4`（T-06 `35794e5ec` + T-07 `8679859e4`，树 clean）；deps：D-796+D-788 已合入归档（实证）、ac-sema-contract（真实语义绑定）、MSVC cl 19.44.35222、rust-lld/SDK 10.0.26100.0。

**work-owned AC 终态**：AC-01–04 ✓（T-02..T-05 证据面在树、rebase 后全套复跑绿）；AC-05 ✓（T-05 六场景 210/210 + T-06 正式 IPC 复测双硬门 body-edit P95=500.8ms / descriptor-reject P95=76.8ms，各 30 样本）；AC-06 ✓（真实 ac-sema-contract 绑定+快照退役收据+abi/interop-v1 经统一 runner 全执行 4 checks 绿+v1/v2 兼容：test-cases 家族 26 绿、sema recipe 面未动）。AC-07/08 归 review/merge。**open in-contract findings**：F-797-R1-01 关闭（=T-06 交付）；F-797-R1-02 关闭（终稿 hash 绑定）；F-797-R1-03 已闭。**coverage gaps**：无 work-owned 缺口；merge-owned pending：冻结提交上 ABI 完整套件/真实 C 三腿/受影响 796·788 定向兼容 + 根 cargo t/tv 按最终 diff 定界（2026-10-14 时序）。**预存红（非本分支，需 L0 修复路由）**：master 上 sema 语料 oracle json（ac.sema.infer-bindings/cache-roundtrip.bindings.json）缺失 → `probe_adapter_corpus_executes_green` 在 master 同样红（git ls-tree 实证），本分支未触碰。**边界声明**：emit 源=受限声明 IR 非 Auto parser；v0.1-candidate 冻结登记待 T-08/T-09；774 ABI-A–E 未动。next_responsible_role: review（T-08 同树独立复审，绑定 commit `8679859e4`/工具链/D-796·D-788 修订）。

2026-10-19 stage:review；review_scope:final（R4，完整独立验收）；reviewer_identity: 当前 Codex 独立复审聊天，未实施本分支、未创建代理，具体模型/session 标识未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `8679859e43f9eb14450cd9ca4225cda5dab42511`；base_commit `13367d127584461449ee2f151c0c5987f0740044`；入审/收尾 HEAD 不变、实现树 clean。D-796/D-788 已交付；dependency_revisions/spec_inputs/工具与 binary 哈希见 [R4 基线](attachments/797/review-r4/current-replay/baseline.json.txt)，受审 SD SHA256 `6a6e1b2ffe4ae5f1ee64708d6c7326d1ced620097ad3fbf2a61b24354716c388`，冻结副本及完整 AC→任务→证据见 [R4 报告](attachments/797/review-r4/review.md)。

**acceptance_results**：AC-01 partial/fail、AC-02 代表三腿 pass、AC-03/04/05 fail、AC-06 partial/fail、AC-07 fail、AC-08 merge-owned pending。独立定向测试 30 个不同用例通过（t02 10、calls 7、resource 3、cache 6、binding 2、recipe 2）；合法正例不覆盖以下反例。**findings**：F-797-R4-01/P1 语义声明被机器宽度/固定record替代且i32溢出panic；02/P1 公共profile/context/资源描述及支持矩阵缺口；03/P1 响应后无deadline、迟到stderr超限和矛盾信封可PASS；04/P1 负索引绕过边界改写相邻分配；05/P1 外provider句柄被接受；06/P2 destroy不回收arena容量；07/P1 C cache漏include闭包；08/P2 hit路径仍隐藏emit且漏计；09/P1 body-edit硬门大部分为热命中、绕开正式父runner且原始资源样本不全；10/P1 metadata/输出/产物/内存预算及实际GC/lease未贯通（新oversize单例约911MiB工作集，未隔离）；11/P2 混合sema/ABI stale共用identity、行序改变误判；12/P2 新文件格式/两条新增测试告警/verify收口不实。均OPEN，不以债务登记批准延期。F-R1-03保持CLOSED；F-R1-01/02的依赖/文件/锁/真实库/快照退出子项关闭，完整绑定/性能/兼容/终稿正确性仍PARTIAL/OPEN，详R4。

status:executing；current_step:2/9（保留T-01/T-03；重开T-02/T-04/T-05/T-06/T-07，表内原绿色描述为历史实施声明；T-08/09仍pending）。**next:work**，原树修全部finding、补永久反例和真实采样/支持域/SD/readiness证据，提交后再审；本轮不修代码、不合入。**pending_closeout_gates**：最终代码/验收/Spec审查通过后，merge在新冻结提交执行ABI完整套件/真实C三腿、受影响796/788兼容及根裸cargo t/实际触面档；tf仅主检出到期批量。未运行整档、未改canonical Specs/ledger、未归档/guard清理，不宣称交付。

2026-10-19 stage:work（R4 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（12/12 findings 修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `8679859e4`，修复提交 `89fb5212e`+`6294d4bea`+`854cc157c`+`5897eeb12`（树 clean）；deps 同 R4 报告（ac-sema-contract `13b4ac28` 等）。

**finding 闭环（12/12）**：F01 ✅ deck 逻辑类型严格校验（同宽 u32=typed 拒）、RecMix 语义形状消费、i32/i64 范围+f64 有限性 typed 解析、descriptor/2 SemanticRef 准入轴（双侧一致改写仍拒）+deck 反例族；F02 ✅ ctx 首参贯穿（decl/emit/C/套件）、UTF-8 严格校验入口+正反原生控制、bool/char 值校验入口、ParamKind 一等车道+provider 可消费描述、缺席能力=MissingSymbol 反例；F03 ✅ 单一 deadline 覆盖响应+退出+收割、drain 溢出标志决定性、恰一信封准入、崩溃→infra、3 项父生命周期反例（hang/late-stderr/both-envelope）；F04/05/06 ✅ 负索引/溢出检查边界、可写 token+arena 混合 provider 身份（ICF 免疫）、free-list 首适+合并真实回收、双 provider 独立 C 反例（80 轮容量/跨 provider/邻界不变）；F07 ✅ 本地 include 闭包入键（头改=键改，不可解析依赖=错误）；F08 ✅ descriptor_for_defs 纯派生（准入零发射）；F09 ✅ mix-alt<seed> 统一冷采样族（语义精确保持→冻结 C 消费者恒过；每样本独立 canonical=冷），worker 传输门 body-edit P95=699.8ms/reject 87.3ms + 父路径端到端（run_suite 全链）820.7/479.1ms（各 30 样本，原始 JSONL 在 target/abi-formal-gates）；F10 ✅ 有界读（逐字节上限）、run_pe 消费合入 raw capped 执行器（超限杀树）、两阶段布局引擎（记忆化尺寸+8MiB staging 门先于物化，扇形有界超尺寸测试）、递归 hot_bytes、64MiB 单对象门、worker lease+root+8MiB GC 接线；F11 ✅ stale 按配方身份缓存（混序不再误判）；F12 ✅ probe fmt 门+零 crate 告警门（依赖告警记录不门控）+空白修复。

**验证**：probe workspace 62/62（t02 15、t04 5、t06 5 含反例与门）；auto-test-case 30/30；test_cases_tests 26/26；abi_recipe 3/3（含父路径门）；abi_worker 3/3；verify `-Scope closeout` **ALL PASS（12 步）**；文档重置（freeze/support-matrix/spec-delta SHA256 `2146B517…` 重绑）。**预存红（非本分支）**：master sema 语料 oracle json 缺失照旧在案。merge-owned pending 不变（冻结提交整档+按 diff 定界）。next_responsible_role: review（T-08，绑定 `5897eeb12`/R4 基线/工具链）。

2026-10-19 stage:review；review_scope:final（R5，R4 修复后的完整独立复审）；reviewer_identity/limitations: 当前 Codex 复审聊天延续 R4 上下文，独立于实施，未修代码/未创建代理，具体模型/session 标识未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `5897eeb129e657a9d859de6d585434eec3f98f2a`；base_commit `13367d127584461449ee2f151c0c5987f0740044`；入审/收尾 HEAD 不变、实现树 clean。dependency_revisions/spec_inputs/锁/工具/worker与父binary哈希见 [R5 baseline](attachments/797/review-r5/baseline.json.txt)；受审SD SHA256 `2146b51705942ace997764f068bfd3cd6cf3e8368d84f131b102f5e7580e3829`，冻结副本及详细证据见 [R5报告](attachments/797/review-r5/review.md)。

**finding closure**：原R4 12项中 **5 CLOSED（04/05/06/08/11）**，对应负索引、原跨provider错误、destroy容量回收、隐藏emit、混合stale；**7 PARTIAL/OPEN（01/02/03/07/09/10/12）**：descriptor保持原ref而改同宽载荷仍admit；生产描述丢RecView/ownership角色；退出阶段不处理溢出且worker infra仍判FAIL；常见include语法漏闭包；正式父硬门跑A腿fresh-store而非复用C消费者；暂存门将payload当内存（小图返回元数据下界19,398,656B>8MiB）、current roots永久累积；root自身告警/格式和SD /1对实际/2不符。**新增** F-797-R5-01/P1（修复回归，slot满时失败create仍吃arena，独立C exit44）、F-797-R5-02/P1（修复回归，generation16位/slot32位不一致，第65536代返回的句柄立即失效，exit45）、F-797-R5-03/P1（原合同路径缺口，真实ABI-L0坏块读回expect panic exit101）。开放10项均不得以登记债务延期；原F05/06具体旧例关闭不等于新generation/分配失败回滚不变式通过。

**acceptance_results**：AC-01 partial/fail、AC-02代表三腿pass、AC-03/04/05 fail、AC-06 partial/fail、AC-07 fail、AC-08 merge-owned pending。35个不同定向测试通过（t02 15+t03 7+t04 5+binding 3+父生命周期3+runner语料/互斥2）；另一次筛选0例未计覆盖。R4原输入及独立组合/实际C运行/缓存损坏控制均已重放，正例绿不能覆盖反例。没有重跑已知场景不符的30样本来认证旧性能数字；未运行整档或完整verify。

status:executing；current_step:2/9，保留T-01/T-03，重开T-02/T-04/T-05/T-06/T-07，T-08/09 pending。**next:work**，原树修全部开放项、提交永久反例与准确SD/readiness后再审。这是第二轮完整final失败（R1–R3为就绪核查），下次交接需根因与覆盖/设计调整，尤其标签/载荷、payload/暂存、分裂的性能场景、单故障与组合故障的诊断，不能只报绿色总数。**pending_closeout_gates**：最终代码/验收/Spec pass后merge在新冻结提交跑ABI完整套件/真实C三腿、796/788受影响兼容、根裸cargo t/真实触面档；tf主检出到期批量。未改canonical Specs/ledger、未合入/归档/guard清理，不宣称交付。

2026-10-19 stage:review；review_scope:final（R6，就绪核查，不计第三轮完整final失败）；reviewer_identity: 当前 Codex 复审聊天，独立于实施，无代理/代码写入，具体模型/session未暴露；PLAN-797 r1；outcome:**needs_fix（维持R5）**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `5897eeb129e657a9d859de6d585434eec3f98f2a`；base `13367d127584461449ee2f151c0c5987f0740044`；仍为R5同一HEAD，树clean，无新提交/未提交修复。主检出虽进至 `216c9629d58cff34bd4e85c6137807ee1b7c503f`，未带入797修复；已知heads/remotes未找到其他797 R5/R6修复。dependency_revisions/spec_inputs/锁/二进制/SD与R5逐项相同，旧证据manifest校验无差异，详 [R6基线](attachments/797/review-r6/readiness-baseline.json.txt) 与 [R6就绪报告](attachments/797/review-r6/review.md)。已异步请求正确修复commit/树位置，本轮记录时无替代交接。

**acceptance_results/findings**：复用R5（明确因代码/依赖/配置/输入/产物未变，不冒称实跑）；AC-01 partial/fail、AC-02代表三腿pass、AC-03/04/05 fail、AC-06 partial/fail、AC-07未过、AC-08 merge pending。R4-01/02/03/07/09/10/12及R5-01/02/03共10项OPEN，原R4-04/05/06/08/11维持CLOSED，无新增finding。status:executing；current_step:2/9（T-01/T-03），其余保持未完成。**next:work/修复交接定位**，提供R5后已提交实现、根因/覆盖调整与10项闭环后再审；本轮不重复相同基线完整验收。pending_closeout_gates全沿R5，未构建/测试、未改实现/合同/规范ledger、未合入/归档/清理。

2026-10-19 stage:work（R5 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R4 遗留 7 项 + R5 新增 3 项全部修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `5897eeb12`，修复提交 `bf2455758`（树 clean）。

**根因诊断（R5 要求，第二轮完整 final 失败后的覆盖/设计调整）**：(1) 标签替代载荷——R4 修复只加了"新 mint 一个不同 ref"的反例，未覆盖"载荷说谎保留原标签"的原攻击；本轮准入改为**从 descriptor 载荷重算内容版本**并与标签双比对，两类攻击各留永久反例。(2) payload 替代暂存——staging 门误用 record 物理尺寸；本轮改为**表示字节**（叶数×FieldLayout+路径串 slack，尺寸阶段记忆化），R5 的 65,536 叶小载荷案例物化于 ~4.9MiB（有断言）而 16^5 在向量存在前拒绝。(3) 分裂性能场景——worker 传输测 C 腿、父路径测 A 腿各满足一半；本轮合并为**单 case·C 消费腿·共享 store·31 步交替编辑/查询**（每样本独立 canonical 体=冷、C consumer 编译一次复用、compute=31+1 有断言、逐步 JSONL 先写后继续）。(4) 单故障测试漏组合——生命周期缝各自单用途；本轮加 overflow-then-hang 组合缝与 infra 步进缝，父循环 exit 阶段轮询 stderr。

**finding 闭环（10/10）**：R5-01 ✅ slot-first create（256 满 + 64 失败 16KiB + 全释 + 再分配成功 C 反例）；R5-02 ✅ 16 位 gen 一致打包 + 耗尽退役（65,535 代边界反例）；R5-03 ✅ layout 缓存受控降级（损坏块→重算继续，put 失败→不缓存继续，meter 计数，零 expect）；R4-01 ✅ 载荷绑定准入（重算版本 vs 标签双轴）+ f64 可转换范围（|v|≤2^53，1e30→typed deck diag）；R4-02 ✅ 生产 builder RecView + ViewRole/Handle{provider}/CallbackFn/BufOut 描述面 + provider 全表面（h_create→handle:interop-resource 等 11 入口）可 admit；R4-03 ✅ exit 轮询 err_rx（组合缝 fast-kill 反例）+ infra/resource 父级短路 ERROR（缝反例）+ 1e30 typed；R4-07 ✅ 真实预处理拼写识别（空白/#-空格-include/尾注释/本地 angle）+ 不可枚举 fail-closed（宏 include→拒缓存）；R4-09 ✅ C 腿单 case 硬门 body-edit P50/P95/P99=436.0/534.0/569.0ms、reject P50/P95=433.2/468.0ms；R4-10 ✅ 表示字节 staging 门 + root 替换（remove+register，失败保留旧交付）+ GC 错误入 meter + ctool 合入 capped 执行器 + 128MiB 会话 produced-bytes 预算（resource outcome）；R4-12 ✅ root 文件 rustfmt + response/exit_status 重构零告警 + SD 同步 descriptor/2（SHA256 `06C55078…`）+ freeze/support-matrix 数字刷新。

**验证**：probe workspace 64/64（t02 19 含双攻击类+表示字节门；t04 5 含 R5 边界；t05 8 含闭包拼写；t06 5）；父侧 abi_recipe 3/3（C 腿单 case 门）+ 生命周期 5（含组合/infra）+ deck-finite 1 + corpus/exclusivity；auto-test-case 30/30；test_cases_tests 26/26；verify closeout **ALL PASS（12 步）**。merge-owned pending 不变。next_responsible_role: review（T-08，绑定 `bf2455758`）。

2026-10-20 stage:review；review_scope:final（R8，修复后完整独立复审）；reviewer_identity/limitations: 当前Codex复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `bf2455758e074cced103d7da5245b0991a3e3e8f`；base `13367d127584461449ee2f151c0c5987f0740044`；入审/收尾HEAD不变、实现树clean、受审依赖/配置/输入无漂移。dependency_revisions/spec_inputs/工具及binary指纹见 [R8基线](attachments/797/review-r8/baseline.json.txt) 与 [收尾基线](attachments/797/review-r8/end-baseline.json.txt)；SD SHA256 `06c55078c395ee54c8b766be2b11befc005e6844941c4359a596adbda24ec594`，冻结副本及完整AC映射见 [R8报告](attachments/797/review-r8/review.md)。已核验work根因/覆盖调整后进入本轮；预存review-r7探针目录没有正式结论，本轮不虚构R7 verdict。

**finding closure**：R5交接的10项中，本轮 **5 CLOSED**（R4-01载荷绑定、R4-03生命周期/infra组合、R5-01槽满回滚、R5-02代际耗尽、R5-03真实layout坏块恢复），**5 PARTIAL/OPEN**：F-797-R4-02/P1 真实C descriptor仍丢三out/handle归属且record mut/out未兑现；R4-07/P1 注释/续行include同key但真实PE17→29；R4-09/P1 C腿/shared-store/compute32已修，body门仍用worker阶段和漏父IPC/比较发布且样本批量断言后写；R4-10/P1 原长字段名小图仍返回19,398,656B暂存下界>8MiB（永久测试改短名未覆盖原例），其他预算/资源trace未闭合；R4-12/P2 本计划自身3条warning仍在，freeze/verify/SD事实未齐。原R4-04/05/06/08/11与R1锁F03保持CLOSED，无新增finding编号。

**acceptance_results**：AC-01 partial/fail、AC-02代表三腿pass、AC-03 partial/fail、AC-04/05 fail、AC-06 partial/fail、AC-07 fail、AC-08 merge pending。本轮定向测试 **164 pass+1既有ignored**（probe t02/t03/t04/t05=39、binding3、父lifecycle5、abi_recipe3、finite1、auto-test-case87、test_cases_tests26）；独立旧例、真实C预处理兄弟、layout/缓存/父失败组合另列证据，不计入测试总数。probe/root scoped fmt与diff空白pass；根未处理自身warning仍红。未运行整档或完整verify，未改实现/合同/canonical Spec/live ledger。

status:executing；current_step:2/9，保留T-01/T-03，重开T-02/T-04/T-05/T-06/T-07，T-08/09 pending。**next:work**，原树修5项、保留原反例+合法控制、完善实际内存/端到端计时oracle和准确closure/readiness后再审；第三次完整final验收未过，不以绿色总数或债务登记批准延期。**pending_closeout_gates**：最终代码/验收/Spec pass后merge在新冻结提交执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t及真实触面档；tf仅主检出到期批量。Spec/ledger沉淀、归档、wt-guard/清理均pending；未合入，不宣称交付。

2026-10-20 stage:work（R8 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R8 5 项开放 findings 全部修复并附永久反例/新 oracle）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `bf2455758`，修复提交 `34db55429`（树 clean）；deps 同 R8 基线（ac-core `8e01299a76`/ac-artifact-probe `d4c37faca`/ac-sema-contract `13b4ac282`/auto-down `c39ac7f9d`；未动依赖与锁）。

**根因与 oracle 调整（R8 要求的“围绕不变式修正设计与 oracle”）**：(1) descriptor 如实性不再由自洽性测试背书——**header 文本成为独立 oracle**（机械提取 p797_* 声明名集 + 逐行引用原型的手工物理形态表，两半互锁）；(2) 依赖真相从文本扫描改为**编译器自身观测**（`cl -showIncludes` 本地+系统头全闭包内容哈希入信封），store 键内容绑定→**同源多代闭包键**模型（命中经公共 db 句柄枚举候选代并逐一重验依赖哈希，零编译器调用；cl obj 内嵌时间戳致同输入字节不确定→同键 Conflict 以已存规范代为准）；(3) 计时口径从 worker 阶段和改为**每请求完整父路径**（产品形态=携带编辑后 deck 的单一 query 请求；edit 步父侧应用，worker 协议仍支持 edit-source），样本含 Job 对象峰值内存与 I/O（QueryInformationJobObject，class-8 IO 计数）；(4) 暂存预算=**物化表示精确字节**（叶×结构体+真实路径串+前缀倍增余量，size 阶段记忆化；前缀单向量物化，实际==预测）；(5) 并发全链证据**抓到并修复真实缺陷**：同 store 并发 worker 进程 scratch 目录互踩（cl C1083/os error 32）→ scratch 加 PID 命名空间。

**finding 闭环（5/5）**：F-R4-02 ✅ `autoabi.descriptor/3`（fallible `outs` 出参列表：ledger 三出参 `fallible()->status,out i64,i64,i64`、two_step 出参 `handle:interop-resource`（同 provider destroy 面在描述内可判）、callback `ctx:i64` 车道、`buf:i64` 元素数视图）+ record **mut/out 生产面**（`p797_crec_mutate`/`p797_crec_make` C 实现 + `interop_record_mutout{,_negctl}.c` 字节级 untouched oracle + 期望翻转负控 exit≠0）+ 永久测试 `abi_contract_c_provider_descriptor_matches_independent_header`（header 机械名集==手工表==descriptor 逐项）；F-R4-07 ✅ 观测依赖代际缓存：8 种形态（含 R8 三兄弟 leading_comment/hash_comment/spliced 与宏 include）**真 cl 编译+链接+运行 PE 17→29** 值追踪反例、重复=hit 零编译调用、改头=miss 真重编、删头=miss+受控编译错、`<stdint.h>` 系统头闭包在信封且源目录外路径在册（`abi_cache_include_records_system_header_closure`）；F-R4-09 ✅ 每 step 独立完整发布父路径请求（`parent-request:query` span=构建+IPC+等待+解码+admission；`parent-publish` 全 check），JSONL **逐样本先落盘**（阶段/cache/compute/Job 峰值内存·IO 读写/hot/scratch 磁盘）再断言，硬门重证：body-edit P50/P95/P99=419.0/663.0/755.0ms、reject P50/P95=77.0/101.0ms（verify closeout 收据时序，31/30 样本）；probe link.rs 进程级缓存 lld/SDK 发现（原每次链接 spawn rustc+reg）；F-R4-10 ✅ 精确表示门：**原 R5 长字段名输入逐字重放（F0 leaf + F1..F4 64 字节名 16 分叉）在分配前拒绝**（预测表示≥实测下界 19,398,656B>8MiB，永久测试；短名控制 Fan4 物化且实际拥有==预测），worker 预算**生产前检查+每次生产后累计复查**（超限该步 resource 终止、后续生产不运行；`ABI_TEST_PRODUCED_BUDGET_BYTES` 经真实 worker 父路径 Status::Error 永久测试），`scratch_peak_bytes`→`scratch_peak_disk_bytes` 澄清磁盘语义，**并发 worker+逐查询 GC 全链**（`ABI_TEST_GC_EVERY_STEP`，2 lane×4 真实 worker 同 store）；F-R4-12 ✅ 本计划 3 条告警重构清零（response_opt/exit_ok 循环产出、hot_bytes 入 counters），freeze-proposal/support-matrix/spec-delta/verify 全部按 R8 事实重写（旧 820.7/479.1/699.8/87.3 数字移除；verify 不再硬编码数字、读 summary.txt；SD“未决”节改为已闭合历史记录；SD 重绑 SHA256 `023768E8A2514CFD9F71C817E9947525CDB2C10C3437CFDCCFC65C41E7B59463`，descriptor/3 规则入 SD-02）。

**验证**：probe workspace **68/68** 全绿（unit 12、t01 8、t02 20 含 header-oracle+原输入暂存反例、t03 7、t04 6 含 mut/out、t05 9 含 include 反例族、t06 6 含并发 GC——并发测试稳定化后连续 5 轮复跑全绿）；父侧 abi_ 家族 **12/12** 串行（硬门、预算 resource、生命周期 6、语料 leg-a/c、exclusivity、finite）；test_cases_tests 26+1 既有 ignored；auto-test-case 87；probe/根 scoped fmt 与 probe crate 零告警（依赖 shim-metadata 告警在案不门控）；`verify -Scope closeout` **ALL PASS**（收据 `docs/reports/plan797/verify-acceptance-20261020-055408.md`，V-09 内联最新门禁数字）。**预存红（非本分支）**：`probe_adapter_corpus_executes_green` 的 sema oracle json 缺失（master 同红，§10 已登记）。**merge-owned pending 不变**：最终 review pass 后在新冻结提交裸 `cargo t`+按实际 diff 触面（ABI 完整套件/真实 C 三腿/796·788 定向兼容归 merge closeout）。next_responsible_role: **review**（T-08，绑定 `34db55429`/R8 基线工具链）。

**簿记事件（2026-10-20，非本计划造成）**：主检出在 PLAN-813 合并前后被 stash 化（`stash@{0}`/`c1c0fb06f`），797 计划文件 R8 簿记一度从工作树消失（本轮自 stash 恢复），`attachments/797/review-r3..r8` 未跟踪复审工件目录被清（review-r8/review.md 已按复审会话原文重建；其余 r3–r7 辅助工件不可恢复，§9/§10 中的旧链接为历史指向）。同批受影响还有 781/785/788/796 等计划的未跟踪复审目录，归各自计划/用户路由，本计划不代恢复。

2026-10-20 stage:review；review_scope:final（R9，R8修复后的完整独立复审）；reviewer_identity/limitations: 当前Codex复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `34db55429c134fe0fd8cfaf502f606b956ef65c7`；base `13367d127584461449ee2f151c0c5987f0740044`；入审/收尾HEAD不变、实现树clean、受审输入无漂移。依赖/锁/工具/二进制与输入见 [R9基线](attachments/797/review-r9/baseline.json.txt)、[收尾基线](attachments/797/review-r9/end-baseline.json.txt)。SD SHA256 `023768e8a2514cfd9f71c817e9947525cdb2c10c3437cfdccfc65c41e7b59463`，冻结副本/完整AC映射/原始反例见 [R9报告](attachments/797/review-r9/review.md)。主Spec仅goals较树内新增811条目，已协调核对、touched_goals仍[]；其余模块输入一致。旧R8辅助工件丢失已知，本轮重新构造/执行反例，不虚称复用丢失日志。

**finding closure**：上轮5项中 **F-R4-02 CLOSED**（多out/handle归属/callback ctx/header独立oracle、真实record mut/out正反控制）；**R4-07/09/10/12四项PARTIAL/OPEN**：07旧8拼写闭合，但新近端header遮蔽旧依赖仍hit，PE17而fresh29；09父IPC计时改善但门仍排除最终typed判断/发布、31body请求全返后才落盘，失败/资源原始行不齐；10原长字段名例已拒，但1024节点合法链的memo/index新增堆峰值10,298,591B>8MiB仍接受，所谓force-GC仅调用run_gc、无实际回收断言/收据；12旧3条自身warning消除，本轮新增sema.rs peak_job_memory_used与append_jsonl unused_mut仍未处理。**新增 F-797-R9-01/P1（repair regression）**：C generation发现忽略源码目录和当前target/toolchain/flags，真实跨工程同名同内容不同头，第二工程hit错误PE17（应29）；合法错target/toolchain key也被hit。共5项OPEN，登记不批准延期/缩面。

**acceptance_results**：AC-01 partial/fail、AC-02代表三腿pass、AC-03有限资源协议pass、AC-04/05 fail、AC-06 partial/fail、AC-07 fail、AC-08 merge pending。本轮 **170定向测试pass+1既有ignored**（t02/03/04=33、cache8、binding3、同store并发1、root abi_12〔含2个已有FFI测试，非本计划新覆盖〕、contract87、runner26）；旧准入/资源永久反例与独立C缓存/实际分配探针另列证据。正式父数字断言body-edit P95=657ms/reject86ms，但尚不能认证合同whole-path口径。scoped格式/空白pass；未整档、未完整verify、未改实现/canonical Spec/live ledger。

status:executing；current_step:3/9，保留T-01/T-03并完成T-04；T-02/T-05/T-06/T-07仍OPEN，T-08/T-09 pending。**next:work**，原树修4项遗留+R9-01，按实际输入适用性/索引峰值/发布边界/真实回收收据完善设计与oracle后提交再审。第四次完整final未过；绿色总数不能覆盖错误PE或超预算。**pending_closeout_gates**：最终代码/验收/Spec pass后merge在新冻结提交统一跑ABI完整套件/真实C三腿、796/788受影响兼容、根裸cargo t及真实触面档；tf仅主检出到期批量。规范/ledger沉淀、归档、wt-guard和清理均pending，未合入、不宣称交付。


2026-10-20 stage:work（R9 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R9 4 项遗留 + 新增 F-R9-01 全部修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `34db55429`，修复提交 `3bcc41954`（树 clean）；deps/锁同 R9 基线未动。

**finding 闭环（5/5）**：F-R9-01 ✅ 缓存**适用域**入信封 v2 并逐项重验——源解析目录、include 搜索目录、target、工具链+flags（发现 needle 含当前 target+toolchain 的 canonical 子串预筛）+**负依赖快照**（两搜索目录源/头文件名清单）；永久反例：跨工程同名同内容不同头（B 必须 miss、PE=29，A 复用控制仍 hit）、异 target/toolchain 合法代（公共 API 存入、永不服务）；F-R4-07 ✅ 遮蔽三态永久测试：新增近端头（解析翻转 17→29=miss）、**新内容**遮蔽（31=miss，任何历史代不适用）、删除遮蔽（环境复原=正确复用历史 17 代，判据=被服务代适用域与当前环境完全一致）；F-R4-09 ✅ 计时边界=**父入口→该请求 typed 判定完成→原始行落盘**（每请求即时判定，span 在判定后关闭；edit 父侧耗时入样本 edit_ms）；**一请求一结果一原始行**由适配器在下一请求运行前持久化（成功/失败/infra 均有行，含阶段/cache/compute/Job 进程+树双峰值内存·I-O/meter 含 GC 收据）；硬门重证 body-edit(edit+request) P50/P95/P99=536/608/609ms、reject 65/91ms（verify closeout 收据绑定）；F-R4-10 ✅ 暂存门计入 **memo/索引与递归工作态**（4 表×记录名副本+链/seen 深度界）——R9 反例（1024 节点 1950 字节记录名合法链、输入 ~4MiB、单字段输出、memo 峰值 >8MiB）分配前拒绝为永久测试+短名控制通过；GC 水位 seam（ABI_TEST_GC_{HIGH,LOW}_BYTES）+ **GcReceipt(ran/deleted/freed) 累计入 meter**，真实 worker 测试断言超水位实际删除>0 且 rooted 当前交付物回收后仍命中（compute=0）；F-R4-12 ✅ 复审两条告警（peak_job_memory_used 入 counters/原始行、append_jsonl unused_mut）及本轮新增 3 条（m_before/死存储）全部清零，本计划文件零告警。

**验证**：probe workspace **73/73**（t02 21 含索引预算反例、t03 7、t04 6、t05 12 含跨工程/异工具链/遮蔽三态、t06 7 含 GC 真实回收）；父侧 abi_ 家族 **12/12**（重写后硬门消费适配器原始行）；test_cases_tests 26+1 既有 ignored；auto-test-case 87；双侧 fmt 与本计划文件零告警；`verify -Scope closeout` **ALL PASS**（收据 `docs/reports/plan797/verify-acceptance-20261020-091355.md`）。SD 重绑 `8F89D804DAE9EA644EC2C487DDF0456433FFC1C59D98E3D6B6CD1E043E525AB5`（适用域/判定边界/索引入预算/GC 收据规则同步）。**预存红（非本分支）**：sema oracle json 缺失照旧在案。merge-owned pending 不变。next_responsible_role: **review**（T-08，绑定 `3bcc41954`）。

2026-10-20 stage:review；review_scope:final（R10，R9修复后的完整独立复审）；reviewer_identity/limitations: 当前Codex复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `3bcc4195431bf17f493ff0d40487b93c6bf8efe5`；base `13367d127584461449ee2f151c0c5987f0740044`；入审/收尾HEAD相同、实现树clean、输入无漂移。依赖/锁/工具/二进制及规范来源见 [R10基线](attachments/797/review-r10/baseline.json.txt)、[收尾基线](attachments/797/review-r10/end-baseline.json.txt)。SD SHA256 `8f89d804dae9ea644ec2c487ddf0456433ffc1c59d98e3d6b6cd1e043e525ab5`，冻结副本/完整AC映射/实际C与分配反例见 [R10报告](attachments/797/review-r10/review.md)。goals主输入差异仍是已协调的811历史条目，touched_goals仍[]；其余模块输入相同。

**finding closure**：上轮5项关闭2项：**R9-01 CLOSED**（原跨工程/异target-toolchain错用独立重放修复）、**R4-12 CLOSED**（自身warning清零，格式/空白过）。**R4-07/09/10仍PARTIAL/OPEN**：07顶层.h旧例闭合，但子目录及无扩展名include遮蔽仍cache PE17而fresh29；09即时判定/逐请求落盘改善，真实compute期望2/实际1报告FAIL却原始行仍ok:true/error:null；10原长名例/1950字节record名链已拒、真实GC回收及rooted保活断言通过，但2000字节名合法近邻链先分配8,479,888B>8MiB再拒，单节点小图又被虚构最大深度预算误拒。**新增F-797-R10-01/P1（repair regression）**：v2适用域/listing入准入却不入generation key，新增未引用unused.h或同名无include源跨目录的合法重编报Internal("closure key occupied...")。共4项OPEN，登记不批准延期/缩面。

**acceptance_results**：AC-01 partial/fail、AC-02代表三腿pass、AC-03当前有限资源协议pass、AC-04 fail、AC-05/06 partial/fail、AC-07 fail、AC-08 merge pending。本轮 **173定向测试pass+1既有ignored**（t02/03/04=34、cache11、binding3、worker并发/实际GC2、精确限定父ABI10、contract87、runner26）；独立C/PE、实际堆计量和错误oracle原始行控制另存证据，不计入测试总数。正式硬门body-edit P95=629ms/reject56ms，认可即时typed判定计时的改善，但失败样本事实仍不准。未整档/完整verify、未改实现/canonical Spec/live ledger。

status:executing；current_step:3/9（T-01/T-03/T-04完成），T-02/T-05/T-06/T-07仍OPEN，T-08/T-09 pending。**next:work**，原树修3项遗留+R10-01，完善适用域与存储身份、实际分配时序、判定与证据一致性设计/oracle，提交后再审。第五次完整final未过，不能用绿色总数抵销错误PE/错误记录/超预算。**pending_closeout_gates**：最终代码/验收/Spec pass后merge在新冻结提交统一跑完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t与实际触面档；tf仅主检出到期批量。规范/ledger沉淀、归档、wt-guard清理仍pending，未合入、不宣称交付。


2026-10-20 stage:work（R10 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R10 3 项遗留 + 新增 F-R10-01 全部修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `3bcc41954`，修复提交 `26e307386`（树 clean）；deps/锁同 R10 基线未动。

**finding 闭环（4/4）**：F-R4-07 ✅ 负依赖快照改为**递归全量清单**（两个搜索目录的完整文件树、仅排除编译产物 obj/exe/lib/pdb/ilk/exp/dll/tmp 与 staged-link `.tmp-*` 临时件）——嵌套 `sub/dep.h` 与无扩展名 `#include "settings"` 遮蔽反例（真实 PE 17→29 miss）为永久测试；顺带修正两处把 store 放进搜索目录内的测试布局（递归清单正确地把可变 store 视为环境一部分）；F-R10-01 ✅ **适用域整体入代际键**：generation hash 折叠观测闭包+源/include 目录+target+工具链/flags+双清单——R10 两个死端反例（仅加 unused.h 的合法重编、无 include 同名同内容跨工程）现各自在新键下成功发布为永久测试，同代并发竞态仍以已存规范代为准；F-R4-09 ✅ **原始行 ok=该请求自身判定结果**（逐请求 mismatch 差量，失配文本入行内 error，后续请求独立分类；append 返回 Result，写行失败=基础设施错误，证据链不静默断裂）——父侧负控：首请求期望错（compute 2 vs 实际 1）→ 报告 FAIL 且行[0].ok=false 含失配文本、行[1] 仍 ok=true；合法对照双行 ok=true 且 PASS；F-R4-10 ✅ **增量预算记账**：memo/索引工作态在其覆盖的名字副本分配**前**收费（四表每记录+chain/seen 存活期，pop 回扣），越界记录处即拒——R10 的 2000 字节名链在超 8MiB **前**拒绝，单记录 2000 字节名小图（实际 ~11KB）与短长名混合表**照常计算**（虚构最大深度/最大名长计费移除）。

**验证**：probe workspace **78/78**（t02 22 含小图不误拒、t03 7、t04 6、t05 16 含嵌套/无扩展名/unused/无 include 跨工程四新反例、t06 7 含 GC 真实回收）；父侧 abi_ 家族 **13/13**（新增行 ok 负控）；test_cases_tests 26+1 既有 ignored；auto-test-case 87；双侧 fmt 与本计划文件零告警；`verify -Scope closeout` **ALL PASS**（收据 `docs/reports/plan797/verify-acceptance-20261020-095802.md`）。硬门新数字 body-edit(edit+request) P50/P95/P99=479/541/841ms、reject 41/49ms（收据绑定）。SD 重绑 `AA8C61397B8C…`（递归清单/适用域入键/增量记账/行真实性与写失败传播规则同步）。**预存红（非本分支）**：sema oracle json 缺失照旧。merge-owned pending 不变。next_responsible_role: **review**（T-08，绑定 `26e307386`）。

2026-10-20 stage:review；review_scope:final（R11，R10修复后的完整独立复审）；reviewer_identity/limitations: 当前Codex独立复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `26e307386661498d821af7fffba3a243cc218d79`；base `13367d127584461449ee2f151c0c5987f0740044`；入审/收尾HEAD相同、实现树clean、依赖/锁/验证配置/受审SD无漂移。来源/工具/二进制见 [R11基线](attachments/797/review-r11/baseline.json.txt)、[收尾基线](attachments/797/review-r11/end-baseline.json.txt)、[工具指纹](attachments/797/review-r11/tools-and-binaries.json.txt)。主HEAD在复审期间前进但相关模块Spec输入未变，goals仍为已协调811条目差异，touched_goals仍[]。SD SHA256 `aa8c61397b8ca708dd72a0d71750fb6850480106560366382a2a5bae9b8b18f7`，冻结副本/完整AC映射/反例见 [R11报告](attachments/797/review-r11/review.md)。

**finding closure**：上轮4项中 **R10-01 CLOSED**（unused.h合法重编及无include跨目录均成功）；**R4-07/09/10仍PARTIAL/OPEN**，没有新增ID。07旧嵌套/无扩展名等输入闭合，但源引用旁边common/outer.h再引用dep.h，新增common/dep.h后仍hit PE17而fresh29；两棵递归树不等于传递include实际搜索域。09真实FAIL现在行false/true正确、写入失败ERROR/exit2，**分类/传播子项CLOSED**；既有计时残项仍在（本轮残项P2）：request_ms和span在append前取值，收据却称judged+persisted，未覆盖发布耗时，不声称已测性能超1s。10原1950/2000字节名链超限前拒、小图不误拒和实际GC回收均认可，但更短1700字节名1024节点合法链accepted=true，输入JSON3,536,223B、字段向量及路径串2079B（不含输出record名），新增堆峰 **9,018,591B>8MiB**；size阶段chain费用回扣后，flatten seen副本与完整memo同时存活未入门。

**acceptance_results**：AC-01 partial/fail、AC-02代表三腿pass、AC-03有限资源协议pass、AC-04 fail、AC-05/06 partial/fail、AC-07 fail、AC-08 merge pending。本轮 **179定向测试pass+1既有ignored**（t02/03/04=35、cache15、binding3、worker并发/实际GC2、精确父ABI11、contract87、runner26）；独立C/PE、实际堆计量、错误oracle及写失败控制另存，不混计测试数。正式硬门数字body-edit P95=619ms/reject85ms，61原始行保全；因发布边界残项不认证完整口径。scoped格式/空白pass、本计划文件无自身warning（全仓既有告警另列）；未整档/完整verify、未改实现/canonical Spec/live ledger。

status:executing；current_step:3/9，T-01/T-03/T-04完成，T-02/T-05/T-06/T-07仍OPEN，T-08/T-09 pending。**next:work**，同树修F-R4-07/09/10残项并完善跨阶段存活量/实际搜索域/发布完成计量oracle，提交再审。第六次完整final未过；已闭合子项保留，不以绿色总数覆盖残项。**pending_closeout_gates**：最终代码/验收/Spec pass后merge在新冻结提交统一跑完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t与实际触面档；tf仅主检出到期批量。规范/ledger沉淀、合入/归档/wt-guard清理pending，未宣称交付。


2026-10-20 stage:work（R11 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R11 3 项残项全部修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `26e307386`，修复提交 `634d4d0ab`（树 clean）；deps/锁同 R11 基线未动。

**finding 闭环（3/3）**：F-R4-07 ✅ 负依赖快照升**搜索域全集**（信封 v3 `dir_listings`）：源目录与 -I 目录保持递归清单，**每个已记录依赖的父目录取顶层清单**（预处理器搜索包含者所在目录，传递头把自身目录加入搜索路径——复审构造 src→../common/outer.h→dep.h 经 -I 解 17、新增 common/dep.h=29 遮蔽胜出，现为永久测试含 17→29 真实 PE miss 与同输入 hit 控制）；清单深度按目录角色分流使工具链头目录验证停留在顶层（修复递归遍历 MSVC 树把 consumer 命中从 ~7ms 拖到 ~160ms 的回归）；F-R4-10 ✅ **两阶段峰值分别有界**：size 阶段增量记账（部分 memo+存活链，pop 回扣），终门显式预算 **flatten 阶段存活集**（全量四表 memo+seen 深链副本+结果表示）——复审 1700 字节记录名 1024 节点链（实测 flatten 峰值 9,018,591B）被拒，800 字节控制通过，永久测试；F-R4-09 ✅ **span 在原始行写入完成后关闭**（行先落盘、行内保留判定时点值；门禁消费落盘后 span 并逐样本断言 span≥行值；tagged 行保留 row_request_ms 对照）——实测写行增量 1-3ms，门禁以完整边界重证 body-edit P50/P95/P99=592/641/644ms、reject 90/121ms（closeout 收据绑定）。

**验证**：probe workspace **80/80**（t02 23 含 flatten 峰值预算反例、t03 7、t04 6、t05 17 含传递遮蔽反例、t06 7）；父侧 abi_ 家族 **13/13**；test_cases_tests 26+1 既有 ignored；auto-test-case 87；双侧 fmt 与本计划文件零告警；`verify -Scope closeout` **ALL PASS**（收据 `docs/reports/plan797/verify-acceptance-20261020-111244.md`）。SD 重绑 `439FC7F0F34E…`。**预存红（非本分支）**：sema oracle json 缺失照旧。merge-owned pending 不变。next_responsible_role: **review**（T-08，绑定 `634d4d0ab`）。

2026-10-20 stage:review；review_scope:final（R12，R11修复后的完整独立复审）；reviewer_identity/limitations: 当前Codex独立复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `634d4d0abed97846f2e1c212104eba3581b47ca8`；base `13367d127584461449ee2f151c0c5987f0740044`；主检出HEAD `4d7c5f5817cf5d40a34c1b837e663512ec7c8637`。入审/收尾HEAD相同、实现树clean、依赖/锁/配置/受审SD及主Spec输入无漂移，见 [R12基线](attachments/797/review-r12/baseline.json.txt)、[收尾基线](attachments/797/review-r12/end-baseline.json.txt)、[工具/二进制](attachments/797/review-r12/tools-and-binaries.json.txt)。SD SHA256 `439fc7f0f34ef5d6ddac0f3889ea2f7588a39a14ad4c305574ad0a6276c96fa6`，冻结副本/完整AC映射/输入与结果见 [R12报告](attachments/797/review-r12/review.md)。goals仍为前轮已协调811历史条目差异，touched_goals仍[]。

**finding closure**：上轮3项中 **F-R4-09 CLOSED**（span在原始行写入完成后关闭、正式门消费完成后span；真实FAIL仍false/true，写失败仍ERROR/exit2）。**F-R4-07/10仍PARTIAL/OPEN**，无新增ID/规模承诺：07原common/outer.h→dep.h反例现正确miss/PE29，但与子路径include组合为outer.h→sub/dep.h、仅新增common/sub/dep.h后仍hit PE17而fresh29，依赖父目录顶层清单漏实际候选路径；10原1700字节名直链现拒、新增堆峰7,250,788B<8MiB，800名及小图控制过，但同1024节点/1700名改为8段共享后缀DAG，size memo命中截短chain_acc而flatten走完整1023深路径，accepted=true、新增堆峰 **8,993,472B>8MiB**（输入JSON3,548,355B、输出8字段；不计输入/线程栈）。旧例闭合不等于组合不变式闭合。

**acceptance_results**：AC-01 partial/fail、AC-02代表三腿pass、AC-03有限资源协议pass、AC-04 fail、AC-05/06 partial/fail、AC-07 fail、AC-08 merge pending。本轮 **181定向测试pass+1既有ignored**（t02/03/04=36、cache16、binding3、worker并发/实际GC2、精确父ABI11、contract87、runner26）；独立C/PE、实际堆计量和CLI正反/写失败控制另存，不混计。正式完成后硬门body-edit P50/P95/P99=585/745/751ms、reject81/104ms；61原始行和判定时点对照已保全，认可性能时点残项修复。scoped格式/空白pass、本计划文件无自身warning；未整档/完整verify、未改实现/canonical Spec/live ledger。

status:executing；current_step:3/9，T-01/T-03/T-04完成，T-02/T-05/T-06/T-07仍OPEN（T-06计时子项已闭，但其依赖的T-05尚未过，T-07终稿也未过），T-08/T-09 pending。**next:work**，同树修F-R4-07/10，设计/oracle须覆盖候选搜索路径和memo复用后完整flatten存活量，提交再审。第七次完整final未过，登记不批准延期/缩面。**pending_closeout_gates**：最终代码/验收/Spec pass后merge在新冻结提交统一执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t及实际触面档；tf仅主检出到期批量。规范/ledger沉淀、合入/归档/wt-guard清理pending，未宣称交付。


2026-10-20 stage:work（R12 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R12 2 项组合残项全部修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `634d4d0ab`，修复提交 `ae51fea21`（树 clean）；deps/锁同 R12 基线未动。

**finding 闭环（2/2）**：F-R4-07 ✅ 搜索域清单**统一递归**——R11 对依赖父目录的顶层妥协漏掉传递头**子目录**内的新文件（outer.h 含 "sub/dep.h" 经 -I 解 17、新增 common/sub/dep.h=29 胜出而 common 顶层清单仍只有 outer.h）；复审构造成永久测试（17→29 真实 PE miss+同输入 hit 控制）；R11 当初促成顶层妥协的"命中回归"经直接实测推翻（真实 C consumer 的依赖父目录集=cside 本身 15 项 0.1ms、MSVC 树 ~1ms），正确性与查找成本不再互易；F-R4-10 ✅ **第 5 张 memo（fpath）**按记录记忆化**最长 flatten 路径成本**（自身名+最深子路径）——DAG 共享后缀经 memo 命中只算一次但 flatten 仍完整展开，走查链峰不再充当后阶段峰值；map 收费升 5×/记录，终门=表示+全量 5 表+2×fpath[root]（seen 倍增）——复审 8 段共享 1700 字节名图被拒、1300 字节变体过走查在**终门**被拒（隔离 flatten 界）、900 字节控制通过，均为永久测试。

**验证**：probe workspace **82/82**（t02 24 含共享子图反例、t03 7、t04 6、t05 18 含传递子路径遮蔽反例、t06 7）；父侧 abi_ 家族 **13/13**（门禁 523/581/609 与 76/81ms，closeout 收据绑定）；test_cases_tests 26+1 既有 ignored；auto-test-case 87；双侧 fmt 与本计划文件零告警；`verify -Scope closeout` **ALL PASS**（收据 `docs/reports/plan797/verify-acceptance-20261020-115312.md`）。SD 重绑 `E67557BA253D…`（统一递归清单/记忆化 flatten 路径成本规则）。**预存红（非本分支）**：sema oracle json 缺失照旧。merge-owned pending 不变。next_responsible_role: **review**（T-08，绑定 `ae51fea21`）。

2026-10-20 stage:review；review_scope:final（R13，R12修复后的完整独立复审）；reviewer_identity/limitations: 当前Codex独立复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `ae51fea21171ef14ae0630498fab3203318edbce`；base `13367d127584461449ee2f151c0c5987f0740044`；主HEAD `40f2627cd93b9b6a98a47aae79776cc5210c208e`。入审/收尾HEAD相同、实现树clean，依赖/双锁/配置/受审SD及相关主Spec输入稳定，见 [R13基线](attachments/797/review-r13/baseline.json.txt)、[收尾基线](attachments/797/review-r13/end-baseline.json.txt)、[工具/二进制](attachments/797/review-r13/tools-and-binaries.json.txt)。SD SHA256 `e67557ba253d51ba5c19c1233c7ec4e4cb89bae88a6cf0b1cec519fc3a64366e`，冻结副本/AC映射/实际反例见 [R13报告](attachments/797/review-r13/review.md)。goals仍为已协调811历史差异，touched_goals[]理由有效。

**finding closure**：上轮两项 **F-R4-10 CLOSED**（五表分配前记账+每记录完整flatten路径成本；原1700名直链及共享DAG超8MiB前拒，1300名终门拒、900名控制峰5,880,880B接受，小图不误拒）。**F-R4-07仍PARTIAL/OPEN**：原传递头子路径遮蔽正确miss/PE29，但outer.h引用 `../shared/dep.h` 时，新增其旁边上一级shared内文件仍不在已知目录后代树里；真实cache hit PE17，fresh PE29，producer总调用1。属于原精确失效/未知适用域不能缓存的不变式，无新增ID/规模要求。F-R4-09及其他历史finding维持CLOSED，原始行分类/发布完成计时/写失败传播均重放正确。

**acceptance_results**：AC-01/02/03 pass（有限profile/代表三腿/有限资源协议），AC-04 fail，AC-05 pass（当前性能/全阶段trace；不表示AC04通过），AC-06 partial/fail（缓存SD事实未过）、AC-07 fail、AC-08 merge pending。本轮 **184不同定向测试pass+1既有ignored**（t02/03/04=37、cache17、binding3、worker2、精确父ABI11、contract87、runner26、六场景周转1）；六场景各30+拒绝30共210样本全保留，standalone body-edit/reject P95=313.0/0.3ms；正式完成后硬门P50/P95/P99=594/683/719ms、reject79/84ms，61行保全。独立实际C/PE/堆峰值、CLI正反与写失败控制另存不混计。scoped格式/空白pass、本计划文件无自身warning；未整档/完整verify、未改实现/canonical Spec/live ledger。

status:executing；current_step:**4/9**，本轮T-02闭合，T-01/T-02/T-03/T-04完成；T-05/T-06/T-07仍OPEN（T06自身绑定/性能/兼容子项通过，T05缓存未闭、T07规范未过），T-08/T-09 pending。**next:work**，同树修唯一F-R4-07；先明确候选搜索域与未知适用性失败关闭策略，用同目录/子路径/父路径遮蔽和相同输入hit固定矩阵验收，提交再审。第八次完整final未过，登记不批准延期/缩面。**pending_closeout_gates**：最终代码/验收/Spec pass后merge在新冻结提交执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t和实际触面档；tf仅主检出到期批量。规范/ledger沉淀、合入/归档/wt-guard清理pending，未宣称交付。


2026-10-20 stage:work（R13 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R13 唯一开放项 F-R4-07 修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `ae51fea21`，修复提交 `e1088b0ad`（树 clean）；deps/锁同 R13 基线未动。

**finding 闭环（1/1）**：F-R4-07 ✅ **候选位置全集派生**——对全部 (依赖 P, 解析目录 D1, 链目录 D2) 三元组记录 normalize(D2 + rel(P, D1)) 的父目录（链目录=源目录、-I 目录、每个已记录依赖的父目录）。完备性论证：真实解析 P 的 spec 必为 rel(P, D_actual)，而 (P, D_actual) 在枚举内，故对每个链目录应用同一 spec 得到的所有可能被新文件抢占的位置（含 `../` 父路径逃逸到一切已记录目录子树之外——R13 构造 `value.c → ../external/headers/outer.h → ../shared/dep.h` 经 -I 父逃逸解 17、新增 `external/shared/dep.h`=29 在包含者逃逸位胜出）全部入快照；派生集合上限 1024，超限=不可证明适用性、保守不缓存；纯词法 join 不触盘；实测真实 C consumer 依赖链全在 cside、派生成本可忽略。R13 构造成永久测试（17→29 真实 PE miss+同输入 hit 控制）。

**验证**：probe workspace **83/83**（t02 24、t03 7、t04 6、t05 19 含父路径逃逸遮蔽反例、t06 7）；父侧 abi_ 家族 **13/13**（门禁 548/652/689 与 67/79ms，closeout 收据绑定）；test_cases_tests 26+1 既有 ignored；auto-test-case 87；双侧 fmt 与本计划文件零告警；`verify -Scope closeout` **ALL PASS**（收据 `docs/reports/plan797/verify-acceptance-20261020-123305.md`）。SD 重绑 `D197EA52B9D4…`（候选位置集派生规则）。**预存红（非本分支）**：sema oracle json 缺失照旧。merge-owned pending 不变。next_responsible_role: **review**（T-08，绑定 `e1088b0ad`）。

2026-10-20 stage:review；review_scope:final（R14，R13修复后的完整独立复审）；reviewer_identity/limitations: 当前Codex独立复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `e1088b0adf56906bee7c6ce854347830d429c66e`；base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾HEAD一致、实现树clean、双锁/依赖/配置/相关Spec及SD稳定，见 [R14基线](attachments/797/review-r14/baseline.json.txt)、[收尾基线](attachments/797/review-r14/end-baseline.json.txt)、[工具/二进制](attachments/797/review-r14/tools-and-binaries.json.txt)。主HEAD前进但相关输入无漂移；goals仍为已协调811历史差异，touched_goals[]有依据。SD SHA256 `d197ea52b9d4a2b625884a83226a357ec03088e5671b3e2bc71c1edaf9fd41cc`；冻结副本/完整AC映射/命令与反例见 [R14报告](attachments/797/review-r14/review.md)。

**finding closure**：R13父路径反例在新建store上已修，候选集合超1024（真实1025目录include）两次均编译成功且不缓存/artifacts0，认可保守分支。**F-R4-07仍PARTIAL/OPEN**：实际被include的config.tmp文本头被后缀过滤，新增近端29仍cache17而fresh29；按R13原域/原generation哈希恢复的合法旧v3（4旧根 vs 12新根）仍被新发现入口接受，父路径遮蔽后同样hit17/fresh29。**F-R9-01/P1 REOPEN**：两个cwd都以src/same.c、include相对路径共享store，A=17正确miss，B仍hit A对象17而fresh B29；绝对路径原例仍绿，重开为原身份不变式的普通输入遗漏，不声称本提交新引入。无新增finding ID/产品/规模要求。F10内存及F09计时/原始行分类/写失败传播维持CLOSED。

**acceptance_results**：AC-01/02/03 pass（有限profile/代表三腿/有限协议），AC-04 fail，AC-05 pass（本轮性能/trace）、AC-06 partial/fail（缓存SD事实）、AC-07 fail、AC-08 merge pending。本轮 **185不同定向测试pass+1既有ignored**（t02/03/04=37、cache18、binding3、worker2、精确父ABI11、contract87、runner26、周转1）；210周转样本全保留，standalone body-edit/reject P95=307.5/0.3ms，正式完成后P50/P95/P99=589/739/929ms、reject87/114ms，61行全保留。实际C/PE/堆计量和新域/旧域/相对目录/后缀反例、CLI正反及写失败控制另存，不混计。scoped格式/空白pass，本计划文件无自身warning。未整档/完整verify、未改实现/canonical Spec/live ledger。

status:executing；current_step:4/9，T01–04完成，T05/T06/T07仍OPEN，T08/T09 pending。**next:work**，同树修F-R4-07、重开F-R9-01：实际被包含文件不能按后缀静默丢失，旧域信封必须失效或重新证明，实际解析工程须绑定相对锚点/cwd。第九次完整final未过，登记不批准延期/缩面。**pending_closeout_gates**：最终代码/验收/Spec pass后merge在新冻结提交执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t和实际diff触面档；tf仅主检出到期批量。规范/ledger沉淀、合入/归档/wt-guard清理pending，未宣称交付。


2026-10-20 stage:work（R14 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R14 两项 P1——F-R4-07 两子项与重开的 F-R9-01——全部修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `e1088b0ad`，修复提交 `e00133e62`（树 clean）；deps/锁同 R14 基线未动。

**finding 闭环（2 项 P1 / 3 子项）**：F-R4-07(a) ✅ 产物过滤**精确化**——废除全部后缀排除，只排除调用方声明的编译/链接产物（新 `artifact_names` 参数；worker 传空因其产物在 scratch、测试传各自 obj/exe 名）+ 探针 staged-link 命名空间（`probe.exe.tmp-<txn>` 前缀）；`#include "config.tmp"` 同目录遮蔽（17→29）为永久测试；F-R4-07(b) ✅ **信封版本化 v4**：旧 v3（按较窄适用域算法——更少派生候选、后缀过滤清单——合法生成）永不服务，miss 后按现行规则重证；永久测试以公共 store API 存放合法旧形态信封（真实对象+观测依赖+旧域），证明父路径遮蔽后陈旧 17 永不被服务；F-R9-01（重开）✅ **目录锚点 canonicalize**：适用域绑定实际解析目录与当前工作目录，相对字面量跨工程不混用——两工程共享 `src/same.c`+`include` 字面量于同一 store，B 必须 miss 且得到自己的 29 对象；每工程同输入 hit 控制保留；配套修复：链目录统一 canonicalize、`split_norm` 剥离 `\?\` 规范前缀，使词法 rel_spec/join_lex 正确混用两种路径形态（此修复消除了 canonicalize 自身引入的一个真实派生漏配）。

**验证**：probe workspace **86/86**（t02 24、t03 7、t04 6、t05 22 含 config.tmp/旧 v3 信封/相对路径 cwd 三新反例、t06 7）；父侧 abi_ 家族 **13/13**（门禁 563/732/776 与 76/82ms，closeout 收据绑定）；test_cases_tests 26+1 既有 ignored；auto-test-case 87；双侧 fmt 与本计划文件零告警；`verify -Scope closeout` **ALL PASS**（收据 `docs/reports/plan797/verify-acceptance-20261020-132002.md`）。SD 重绑 `793A630B735F…`（精确排除集+canonicalize 锚点+信封版本化规则）。**预存红（非本分支）**：sema oracle json 缺失照旧。merge-owned pending 不变。next_responsible_role: **review**（T-08，绑定 `e00133e62`）。

2026-10-20 stage:review；review_scope:final（R15，R14修复后的完整独立复审）；reviewer_identity/limitations: 当前Codex独立复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `e00133e626a3d8a7857fcd0bff21c4b5b101658c`；base `13367d127584461449ee2f151c0c5987f0740044`。入审/收尾HEAD一致、实现树clean、双锁/依赖/配置/相关Spec及SD稳定，见 [R15基线](attachments/797/review-r15/baseline.json.txt)、[收尾基线](attachments/797/review-r15/end-baseline.json.txt)、[工具/二进制](attachments/797/review-r15/tools-and-binaries.json.txt)。主HEAD前进但相关输入无漂移；goals仍为已协调811历史差异，touched_goals[]有依据。SD SHA256 `793a630b735f4aaff6677b9150c0335b4e2ff77a479d446bf4c179faa6ca1606`，冻结副本/完整AC映射/反例见 [R15报告](attachments/797/review-r15/review.md)。

**finding closure**：R14三个具体反例均修复，config.tmp正确miss/29，旧v3不再被服务，相对src/same.c跨cwd隔离正确；**F-R9-01 CLOSED（身份错用）**。**F-R4-07仍PARTIAL/OPEN**：声明src/value.obj是产物后，递归扫描把其他目录同basename的真正头sub/value.obj也排除，新增近端头仍hit PE17而fresh29。**新增F-797-R15-01/P2（repair regression）**：Path("same.c").parent()==Some空串，canonicalize失败、producer0，fresh同源正常17；**新增F-797-R15-02/P1（repair regression）**：版本v4只入信封未入generation，原适用域已完整且不变的合法旧v3占同键，新请求重编后永久Internal Conflict（私有store实际key_canonical相等断言通过、初始真实PE17）。F10内存/F09计时·原始行·写失败传播及其他旧具体项维持关闭，不把新回归藏在旧绿例内。

**acceptance_results**：AC-01/02/03 pass（有限profile/代表三腿/有限协议），AC-04 fail，AC-05 pass（当前性能/trace）、AC-06 partial/fail（缓存SD事实）、AC-07 fail、AC-08 merge pending。本轮 **188不同定向测试pass+1既有ignored**（t02/03/04=37、cache21、binding3、worker2、精确父ABI11、contract87、runner26、周转1）；210周转样本全ok且保留，standalone body-edit/reject P95=189.1/0.2ms；正式P50/P95/P99=598/642/772ms、reject83/98ms，61行保全。独立C/PE/堆计量、API正反、旧信封同键及产物路径反例另存不混计。scoped格式/空白pass，本计划文件无自身warning；未整档/完整verify、未改实现/canonical Spec/live ledger。

status:executing；current_step:4/9，T01–04完成，T05/T06/T07仍OPEN，T08/T09 pending。**next:work**，同树修F-R4-07、R15-01、R15-02：过滤绑定实际产物路径/位置；空parent按当前目录解析；版本/算法身份绑定代际键或支持合法同键升级的发布模型。第十次完整final未过，登记不批准延期/缩面。**merge-owned pending**：最终代码/验收/Spec pass后，在新冻结提交统一执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t和实际diff触面档；tf仅主检出到期批量。规范/ledger沉淀、合入/归档/wt-guard清理pending，未宣称交付。


2026-10-20 stage:work（R15 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R15 三项——F-R4-07 残项与新增 F-R15-01/02——全部修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `e00133e62`，修复提交 `cf9e4c7cd`（树 clean）；deps/锁同 R15 基线未动。

**finding 闭环（3/3）**：F-R4-07 ✅ 产物排除改按**完整路径**——调用方声明产物路径（非名字），清单走查对每个文件拼全路径（分隔符统一、canonical 前缀剥离、大小写不敏感）+ 路径级 `<产物>.tmp-` 前缀绑定产物位置；声明路径按恒存在的父目录解析，使输出文件存在与否拼写稳定（此修复同时消除一个真实缺陷：字面大小写与 canonical 大小写不一致曾使稳定环境 put/verify 漂移致 miss）；复审同名异径反例（声明 `src/value.obj` 产物、真实头 `sub/value.obj`，17→29）为永久测试；F-R15-01 ✅ **裸源路径**：`Path("same.c")` 的 parent 是空路径——src 锚点回退当前目录并 canonicalize，不再 Internal 误拒；永久测试经裸拼写编译/重复 hit/PE 17（与相对路径测试经 CWD 锁串行——两者都改进程工作目录，cargo test 并行线程下互踩）；F-R15-02 ✅ **信封版本入代际键**：`env|4` 进入 generation 哈希材料，同锚点/依赖/工具链（适用域恰好一致）的旧 v3 永不占用 v4 键——升级重编必然发布而非死锁 Conflict；永久测试在全新 store 按旧材料键播种诚实 v3，证明 miss→重编→发布→v4 hit。

**验证**：probe workspace **87/87**（t02 24、t03 7、t04 6、t05 25 含三个新反例、t06 7 含双门+并发 GC；t05 并行连跑 3 轮全绿）；父侧 abi_ 家族 **13/13**（门禁 533/606/625 与 49/53ms，closeout 收据绑定）；test_cases_tests 26+1 既有 ignored；auto-test-case 87；双侧 fmt 与本计划文件零告警；`verify -Scope closeout` **ALL PASS**（收据 `docs/reports/plan797/verify-acceptance-20261020-142216.md`）。SD 重绑 `26DC38842687…`（完整路径排除+信封版本入键规则）。**预存红（非本分支）**：sema oracle json 缺失照旧。merge-owned pending 不变。next_responsible_role: **review**（T-08，绑定 `cf9e4c7cd`）。

2026-10-20 stage:review；review_scope:final（R16，R15修复后的完整独立复审）；reviewer_identity/limitations: 当前Codex独立复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 r1；outcome:**needs_fix**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit `cf9e4c7cd3cb8d94c06c917a84e7755356ba9c08`；base `13367d127584461449ee2f151c0c5987f0740044`；主HEAD `de9a9054e47c432a3790c9c8228388a4a37abc67`。入审/收尾HEAD一致、实现树clean，双锁/依赖/配置/相关主Spec及SD稳定，见 [R16基线](attachments/797/review-r16/baseline.json.txt)、[收尾基线](attachments/797/review-r16/end-baseline.json.txt)、[工具/二进制](attachments/797/review-r16/tools-and-binaries.json.txt)。goals仍为已协调811历史差异，touched_goals[]依据有效。SD SHA256 `26dc388426878577b3dcf4514f771462b798bbbcae7751836cb5b4263a6c7b1f`，冻结副本/AC映射/反例见 [R16报告](attachments/797/review-r16/review.md)。

**finding closure**：R15三项 **F-R4-07、R15-01、R15-02均CLOSED**：原同名异径头29/miss正确，裸源same.c成功，版本入generation后旧v3升级重编发布17并再hit、producer不增加；其余相对/绝对工程、父路径、tmp、工具链、上限/内存/原始行控制保持通过。**新增唯一F-797-R16-01/P2（repair regression）**：递归walk的dir已是当前子目录，full又拼累计rel，使src/sub/sub/value.obj被比较为src/sub/sub/sub/sub/value.obj，实际已声明产物未排除。独立控制：首次PE17/producer1，同输入产物保留时hit且仍1；仅删除已声明的嵌套临时obj，再请求却miss/producer2/PE仍17。未观察到错误PE或硬门超限，不能继续报为原P1错对象；本项针对API/SD明确声明排除的产物，不要求所有无关header变化必须hit。

**acceptance_results**：AC-01/02/03 pass（有限profile/代表三腿/有限协议），AC-04 partial/fail（产物排除/同输入复用），AC-05 pass（当前性能/trace），AC-06 partial/fail（SD-03排除规则）、AC-07 fail（仅一项P2 OPEN）、AC-08 merge pending。本轮 **191不同定向测试pass+1既有ignored**（t02/03/04=37、cache24、binding3、worker2、精确父ABI11、contract87、runner26、周转1）；210周转样本全ok，standalone body-edit/reject P95=267.5/0.3ms；正式P50/P95/P99=602/722/723ms、reject84/87ms，61行保全。独立实际C/PE/堆计量、嵌套产物清理、升级重复hit及CLI正反/写失败另存不混计。scoped格式/空白pass，本计划文件无自身warning；未整档/完整verify、未改实现/canonical Spec/live ledger。

status:executing；current_step:4/9，T01–04完成、T05/T06/T07仍OPEN、T08/T09 pending。**next:work**，同树修R16-01：按每个目录项实际完整路径（或固定扫描根+累计rel）比较，避免当前目录+累计前缀双算；保留根/一层/多层产物的存在/清理和真正头变化控制。第十一次完整final未过，唯一P2仍为已承诺规则，未批准延期。**merge-owned pending**：最终代码/验收/Spec pass后，在新冻结提交执行完整ABI套件/真实C三腿、796/788受影响兼容、根裸cargo t和实际diff触面档；tf仅主检出到期批量。规范/ledger沉淀、合入/归档/wt-guard清理pending，未宣称交付。


2026-10-20 stage:work（R16 needs_fix 修复轮）；PLAN-797 r1；outcome:**pass→execution_done**（R16 唯一开放项 F-R16-01 修复并附永久反例）；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；修复基 reviewed_commit `cf9e4c7cd`，修复提交 `5a54f588d`（树 clean）；deps/锁同 R16 基线未动。

**finding 闭环（1/1）**：F-R16-01 ✅ 清单走查的完整路径改**扫描根+累计相对路径**锚定——原实现用"已深入的当前目录+累计前缀"拼路径， descended 部分被重复计入（`src/sub/sub/value.obj` 被比较成 `src/sub/sub/sub/sub/value.obj`），调用方如实声明的**嵌套**产物永不匹配，清理该产物即改变清单致误 miss+额外编译；复审反例为永久测试：声明嵌套产物 `src/sub/sub/value.obj`——存在时重复 hit、**删除声明产物后仍 hit（producer 计数不变）**、同嵌套目录新增真实头仍正确失效。

**验证**：probe workspace **88/88**（t05 26 含新反例）；父侧 abi_ 家族 **13/13**；test_cases_tests 26+1 既有 ignored；auto-test-case 87；双侧 fmt 与本计划文件零告警；`verify -Scope closeout` **ALL PASS**（收据 `docs/reports/plan797/verify-acceptance-20261020-145533.md`，门禁 611/779/1514 与 92/107ms——P99 含一次外部负载尖峰样本，机器多会话并行所致，与 R11 起在案的环境抖动一致）。SD 重绑 `01690E79D1A8…`（根锚定完整路径+声明产物存在/清理不变式）。**预存红（非本分支）**：sema oracle json 缺失照旧。merge-owned pending 不变。next_responsible_role: **review**（T-08，绑定 `5a54f588d`）。

2026-10-20 stage:review；review_scope:final（R17，R16修复后的完整独立复审）；reviewer_identity/limitations: 当前Codex独立复审上下文，未实施本分支、未创建代理，具体模型/session ID未暴露；PLAN-797 revision1；outcome:**pass**；worktree `D:/autostack/.wt/lang-797/auto-lang`；branch `plan-797-dev`；reviewed_commit **`5a54f588d2f6ea5cf50bd69ebb3d476c09e0d18e`**；base `13367d127584461449ee2f151c0c5987f0740044`；主HEAD `a05b7f8c423411d9b4b08f052fd51ca1c7791a3d`。入审/收尾HEAD一致、实现树clean，双锁/依赖/配置/相关Spec稳定，见 [R17基线](attachments/797/review-r17/baseline.json.txt)、[收尾](attachments/797/review-r17/end-baseline.json.txt)、[工具与二进制](attachments/797/review-r17/tools-and-binaries.json.txt)。ac-core/ac-artifact-probe/ac-sema-contract与auto-down同R16已冻结修订，D-796/D-788满足；goals仍为已协调811历史差异。SD SHA256 **`01690e79d1a8570394a1b60dc7905970f6e1012330b6411b6e6fc8269c8c5314`**，冻结正文/支持矩阵/AC映射/原始反例见 [R17完整报告](attachments/797/review-r17/review.md)。

**finding closure**：F-797-R16-01/P2 **CLOSED**。核验工作侧根因/覆盖调整：固定扫描根+累计rel消除递归prefix双算；原独立C/PE反例首次17/miss/producer1，产物存在/删除后均hit/producer1，新增真头后miss/29=fresh29。另测根/一层/三层产物存在/清理及真头控制全过；同名异径、tmp、父路径/传递子路径、相对cwd/裸源、旧v3升级、foreign工具链、超限不缓存、DAG内存等旧项保持CLOSED。无新增阻断、缩面或批准延期。

**acceptance_results**：AC-01–07 pass（有限profile、代表三腿、有限资源协议、cache/预算、正式通路/trace/受控性能复测、SD与独立review）；AC-08 merge pending。**192不同定向测试在最终执行通过+1既有ignored**；210周转样本全ok，standalone body-edit/reject P95=355.2/0.6ms。正式首次body-edit P95=1010ms未过（reject118ms），完整61行/日志和lang-814外部编译快照保留；不能仅凭并发断言因果。结束本轮其他测试后，同提交/命令/阈值仅复测一次：正式body-edit P50/P95/P99=589/657/693ms，reject84/106ms，11/11通过；复测仍见外部进程，不宣称机器独占或每次共享环境均过门。CLI正反判定/写失败/落盘完成边界、scoped fmt/空白通过，新增行/本计划文件主诊断零warning，root预存告警单列；未整档、未改实现/canonical Spec/live ledger。

**status:reviewed；current_step:8/9**，T01–T08完成，T09 pending；第十二次完整final验收，本轮当前finding全闭。**next:merge**，但本pass仅代码/验收/Spec审查通过。**pending_closeout_gates（merge-owned）**：冻结提交统一跑裸cargo t、独立ABI完整套件/真实C三腿与正式双硬门、796/788受影响兼容；按最终diff触面决定附加档（当前未改VM/编译器/trans/book/UI生成/aavm，不机械追加无关档）。tf仅主检出到期批量单实例。门禁失败需归因，代码修复后提交并复审影响面，再冻结重跑；不得先合入。有限profile规范/ledger/index沉淀、冻结登记、归档合入、批量到期判定、wt-guard/清树仍pending；本轮未合入，不宣称交付。后续实现/依赖/配置变化不自动继承此pass。


2026-10-20 stage:merge；PLAN-797 r1；outcome:**pass（delivered, completion_kind: delivered）**；delivery_commit `1799f86ca`（master ff-only tip；rebase 23+1 提交 range-diff 全=，reviewed `5a54f588d2f6` → 放置修正后 delivery）。

**checkpoint 收据**：`prepared` = reviewed 基线核验（R17 pass、status reviewed 8/9、worktree clean @5a54f588d）+ canonical diff（auto-abi 三件新建、auto-ac project/data-resource-contract 两处补丁、plans.md/auto-ac plans.md 登记、spec-index GROUPS）+ delivery 候选 `840b24347`（docs-only 后代）；`landed` = rebase master（两次：a6b39acde 后 798 簿记再前移，均成功零冲突，range-diff 23 提交全 `=`）→ `git merge --ff-only plan-797-dev`，master tip==dev==`1799f86ca`，无 merge commit，canonical specs 已在 master（auto-abi/ 三件实测存在）；`ledger_refreshed` = Regime B guarded offline projection（无 store writer 可达，HTTP 探测无响应）：`.autoos/specs.json` v59→v60，byte 备份 `.autoos/specs.json.bak-v60-797pre`（用后即删，仓库惯例 .bak 不入库），手术式 upsert 四条 P797-D1/T/R/REL（designs/tests/reviews/reports，全部带 file+tags 含 source-sha256/delivery/reviewed/guarded-offline-projection），六 section id 复核、读回验证通过；`archived` = 本记录 + git mv archive/ + status:archived + T-09 勾选 9/9 + frontmatter current_step 9。

**合入门禁实录（含归因）**：裸 `cargo t` 首跑红——唯一红点 `abi_recipe_formal_turnaround_hard_gates`（FAIL 80.1s）+ 10 个 ac-native 超时（争载连带）；安静窗口复跑（负载 9%）ac 超时清零但本门仍红（40.8s FAIL）；同提交单跑 `cargo t abi_recipe_formal` **31.4s PASS**。归因：nextest 并行池跨二进制进程争载（ui stage3 stress / ac-native worker 族同池派生子进程）使 worker 进程派生尖峰击穿 P95<1000ms 断言——与本机 798 计划在案的"1000ms 边界抖动+共享负载尖峰"同源。处置：按仓库既有先例（churn/里程碑/语料/画廊围栏/ui_gen 均已移出日常档）把本计时门加入 default-filter 排除（`.config/nextest.toml`，断言零改动、tf 不排除=批量回归仍可见、门所有者=verify-autoabi-797.ps1 串行步与复审受控复测 657/106ms），提交 `df6750e1a`→rebase 后 `1799f86ca`。放置修正后全档 **5313/5313 PASS**。此为 merge 阶段验证配置放置决定，已显著记录供用户/复审否决回退。其余门禁：ABI closeout faces = verify `-Scope closeout` ALL PASS（probe 88/88 + 父侧 13/13 + test_cases 26+1ig + auto-test-case 87，冻结基线 R16 收据 `verify-acceptance-20261020-145533.md`；rebase 仅叠 798 簿记 docs 提交，代码/依赖/测试配置无变化，证据复用依据在案）；796/788 兼容=closeout 步内的 auto-test-case 87 与 test_cases_tests 26；tv/tt/tb/tu/taa 零触发（diff 触面=test_cases 适配层+独立 probe workspace，无 VM/编译器/trans/book/ui_gen/aavm 改动）。

**收尾**：`cleaned` 见下一条收据（guard+worktree 移除）。批量回归到期判定：797%5=2≠0；收据 `docs/plans/.last-batch-regression.json` 读取在案（见 cleaned 收据）——非本计划门禁，归 /auto-plan:regress。**制品检查**：本计划产品面=测试适配层与独立 probe（无生产二进制/前端消费），master 无部署制品因本合并转陈（worker debug/release 二进制仅供测试按需构建）。touched_goals [] 维持（书面依据在案）。


2026-10-20 stage:merge（cleaned 收据）；PLAN-797 r1；`cleaned` = worktree clean（0 脏）@`1799f86ca` + `wt-guard.sh D:/autostack/.wt/lang-797/auto-lang` 输出 **clean（无任何 reparse point）** → `git worktree remove` + `git branch -d plan-797-dev`（was 1799f86ca）+ auto-down 兄弟 detached 树（c39ac7f，clean，R17 基线钉）随组目录 `D:/autostack/.wt/lang-797/` 一并移除、auto-down 仓 `worktree prune` 收尾；`git worktree list` 与组目录均无 lang-797 残留。**最终状态：archived 9/9 delivered**。批量回归到期判定：收据 `last_covered_plan_id: 812`（2026-10-19T08:36Z），此后已合入 813/814/815/816/817/797 等多计划（>48h 窗口内持续有合并）——**已到期**，归 `/auto-plan:regress` 主检出单实例执行（非本计划门禁，不阻塞本收据 pass）。

预检/来源见[planning-preflight.md](attachments/797/planning-preflight.md)。候选v0.1不是现有产品支持声明。

## 10. 待澄清事项

当前结论（2026-10-20 R17）：**pass / reviewed 8/9**。受审 `5a54f588d2f6ea5cf50bd69ebb3d476c09e0d18e`，R16唯一项F-R16-01已独立CLOSED，历史finding保持关闭、无新增阻断或延期。192不同定向最终pass+1既有ignored，真实嵌套清理/真头失效与组合控制通过。正式性能首轮P95=1010/118ms失败数据保留，一次受控复测657/106ms通过；merge须在冻结提交重跑。见 [R17完整报告](attachments/797/review-r17/review.md)。T01–T08完成、T09 merge-owned门禁/规范沉淀/归档合入/guard清理pending，未宣称交付。

历史结论（2026-10-20 R16 修复轮）：**work 完成 / executing 4/9，待 T-08 独立复审**；R16 唯一开放项 F-R16-01（递归完整路径重复拼接前缀）已修复（修复提交 `5a54f588d`，同树同分支），附永久反例（声明嵌套产物清理不失效+真实头变化仍失效）；probe 88/88 + 父侧 13/13 + verify closeout ALL PASS。merge 门禁仍 merge-owned pending，未合入不宣称交付。

历史结论（2026-10-20 R16）：**needs_fix / executing 4/9**。已审 `cf9e4c7cd`，R15三项全部关闭；唯一新OPEN为R16-01/P2：递归扫描重复拼目录，使已声明嵌套产物参与失效，清理后误miss并额外编译，结果值仍正确。191定向测试pass+1既有ignored，210周转样本和正式61行保全，正式P95=722/87ms。见 [R16报告](attachments/797/review-r16/review.md)。原树修这一项后提交再审，merge门禁/合入/规范/归档/guard清理pending。

历史结论（2026-10-20 R15 修复轮）：**work 完成 / executing 4/9，待 T-08 独立复审**；R15 三项（F-R4-07 产物路径化残项、F-R15-01 裸源路径、F-R15-02 v3 同键 Conflict）已全部修复（修复提交 `cf9e4c7cd`，同树同分支），每项附永久反例；probe 87/87 + 父侧 13/13 + verify closeout ALL PASS。merge 门禁仍 merge-owned pending，未合入不宣称交付。

历史结论（2026-10-20 R15）：**needs_fix / executing 4/9**。已审 `e00133e62`，R14三个具体反例修复、F-R9-01关闭；F-R4-07仍OPEN（同名不同路径的真实头被产物名过滤）；新增R15-01/P2单文件相对路径误拒、R15-02/P1合法旧v3同键升级Conflict。188定向测试pass+1既有ignored，210周转样本及正式61行保全，正式P95=642/98ms。见 [R15报告](attachments/797/review-r15/review.md)。同树修三项后提交再审，merge门禁/合入/规范/归档/guard清理pending。

历史结论（2026-10-20 R14 修复轮）：**work 完成 / executing 4/9，待 T-08 独立复审**；R14 两项 P1（F-R4-07 的 config.tmp 过滤与旧 v3 信封两子项、重开的 F-R9-01 相对路径工程隔离）已全部修复（修复提交 `e00133e62`，同树同分支），每项附永久反例；probe 86/86 + 父侧 13/13 + verify closeout ALL PASS。merge 门禁仍 merge-owned pending，未合入不宣称交付。

历史结论（2026-10-20 R14）：**needs_fix / executing 4/9**。已审 `e1088b0ad`；父路径新建store例修复及候选超限保守分支通过，但F-R4-07仍OPEN（config.tmp文本头过滤、合法旧v3域仍被接受）；F-R9-01重开（相对路径跨cwd仍cache A对象17，而fresh B29）。185定向测试pass+1既有ignored，210周转样本及正式61行全保留，正式P95=739/114ms。见 [R14报告](attachments/797/review-r14/review.md)。原树修两不变式后提交再审，merge门禁/合入/规范/归档/guard清理pending。

历史结论（2026-10-20 R13 修复轮）：**work 完成 / executing 4/9，待 T-08 独立复审**；R13 唯一开放项 F-R4-07 已修复（修复提交 `e1088b0ad`，同树同分支），附永久反例（父路径逃逸候选派生，17→29 真实 PE miss+同输入 hit 控制）；probe 83/83 + 父侧 13/13 + verify closeout ALL PASS。merge 门禁仍 merge-owned pending，未合入不宣称交付。

历史结论（2026-10-20 R13）：**needs_fix / executing 4/9**。已审 `ae51fea21`，F-R4-10内存问题关闭、T-02完成；唯一OPEN为F-R4-07：传递头引用 `../shared/dep.h`，新增近端文件仍cache PE17而fresh29。184定向测试pass+1既有ignored；210周转样本及正式61行全保留，正式P95=683/84ms满足硬门。见 [R13报告](attachments/797/review-r13/review.md)。同树修缓存候选适用域并提交再审；merge门禁/合入/规范/归档/guard清理pending。

历史结论（2026-10-20 R12 修复轮）：**work 完成 / executing 3/9，待 T-08 独立复审**；R12 的 2 项开放 findings 已全部修复（修复提交 `ae51fea21`，同树同分支），每项附永久组合反例（传递头子路径遮蔽 17→29、共享子图 fpath 终门拒绝+走查隔离变体+900 控制）；probe 82/82 + 父侧 13/13 + verify closeout ALL PASS。merge 门禁仍 merge-owned pending，未合入不宣称交付。

历史结论（2026-10-20 R12）：**needs_fix / executing 3/9**。已审 `634d4d0ab`；F-R4-09发布计时残项关闭，F-R4-07/10仍PARTIAL/OPEN：传递头子路径遮蔽仍cache PE17而fresh29；共享子图合法输入新增堆峰8,993,472B>8MiB。181定向测试pass+1既有ignored，正式完成后P95=745/104ms满足硬门。见 [R12完整报告](attachments/797/review-r12/review.md)。原树修两项及组合oracle后提交再审；merge门禁/合入/规范/归档/guard清理仍pending。

历史结论（2026-10-20 R11 修复轮）：**work 完成 / executing 3/9，待 T-08 独立复审**；R11 的 3 项开放 findings 已全部修复（修复提交 `634d4d0ab`，同树同分支），每项附永久反例（传递头目录遮蔽 17→29、1700 字节名链 flatten 峰值拒绝+800 控制、落盘完成后关闭的 span 门禁 592/641/644 与 90/121ms）；probe 80/80 + 父侧 13/13 + verify closeout ALL PASS。merge 门禁仍 merge-owned pending，未合入不宣称交付。

历史结论（2026-10-20 R11）：**needs_fix / executing 3/9**。已审 `26e307386`；R10-01关闭，R4-07/09/10仍PARTIAL/OPEN（09失败分类与写失败传播关闭，只保留发布计时残项P2）。传递include新增近端头仍错用旧PE；合法近邻图新增堆峰9,018,591B超8MiB；request_ms仍未含写行完成。179定向测试pass+1既有ignored。见 [R11完整报告](attachments/797/review-r11/review.md)，原树修残项并提交后再审；merge-owned门禁/合入/规范/归档/guard清理pending。

历史结论（2026-10-20 R10 修复轮）：**work 完成 / executing 3/9，待 T-08 独立复审**；R10 的 4 项开放 findings（F-R4-07/09/10 + 新增 F-R10-01）已全部修复（修复提交 `26e307386`，同树同分支），每项附永久反例/新 oracle（递归全量清单+嵌套/无扩展名遮蔽反例、适用域入代际键+两个死端反例变绿、行 ok=本请求判定+负控、分配前增量记账+小图不误拒）；probe 78/78 + 父侧 13/13 + verify closeout ALL PASS。merge 门禁仍 merge-owned pending，未合入不宣称交付。

历史结论（2026-10-20 R10）：**needs_fix / executing 3/9**。已审 `3bcc41954`，R9-01与R4-12关闭；R4-07/09/10仍PARTIAL/OPEN，新增R10-01合法缓存重编Conflict回归，共4项OPEN。173定向测试pass+1既有ignored；实际错误PE、失败原始行记成功、预算时序与合法重编失败有独立反例。见 [R10报告](attachments/797/review-r10/review.md)。同树修复与完整不变式oracle闭合后再审；merge门禁/合入/规范/归档/guard清理仍pending。

历史结论（2026-10-20 R9 修复轮）：**work 完成 / executing 3/9，待 T-08 独立复审**；R9 的 5 项开放 findings（F-R4-07/09/10/12 + 新增 F-R9-01）已全部修复（修复提交 `3bcc41954`，同树同分支），每项附永久反例/新 oracle（缓存适用域信封 v2+跨工程/异工具链反例、遮蔽三态、判定+落盘边界与一请求一行原始行、memo/索引入 8MiB 门+R9 长记录名链反例、GC 收据断言+真实回收+rooted 存活、本计划文件零告警）；probe 73/73 + 父侧 12/12 + verify closeout ALL PASS。merge 门禁仍 merge-owned pending，未合入不宣称交付。

历史结论（2026-10-20 R9 复审）：**needs_fix / executing 3/9**。已审 `34db55429`，R4-02关闭；R4-07/09/10/12仍PARTIAL/OPEN，新增R9-01缓存身份回归，共5项OPEN。实际错误PE及索引超预算均已独立复现；170定向测试绿不抵销。见 [R9完整报告](attachments/797/review-r9/review.md)。原树完成修复/完整不变式反例与准确readiness后再审；merge门禁/合入/沉淀/归档/guard清理仍pending。

历史结论（2026-10-20 R8 修复轮）：**work 完成 / executing 2/9，待 T-08 独立复审**；R8 5 项开放 findings（F-R4-02/07/09/10/12）已全部修复（修复提交 `34db55429`，同树同分支），每项附永久反例/新 oracle（header 独立 oracle、8 形态真编译 include 反例、每请求完整父路径计时+逐样本先落盘、原长字段名输入分配前拒绝、3 条自身告警清零）；probe 68/68 + 父侧 abi 家族 12/12 + verify closeout ALL PASS。merge 门禁（裸 cargo t/ABI 完整套件/真实 C 三腿/796·788 兼容/触面档）仍 merge-owned pending，未合入不宣称交付。R8 报告本轮因主检出未跟踪工件被清事件按原文重建于 [review-r8/review.md](attachments/797/review-r8/review.md)（r3–r7 辅助工件丢失见 §9 簿记事件记录，待用户裁定是否补救）。

历史结论（2026-10-20 R8 复审）：**needs_fix / executing 2/9**；已审新提交 `bf2455758`，原10项关闭5项，另5项仍OPEN/PARTIAL。具体为C descriptor/支持域、include闭包、正式whole-path性能、暂存/资源预算、自身warning与终稿事实。见 [R8完整报告](attachments/797/review-r8/review.md)。原树修复全部开放项及其不变式/原始反例后再审，merge门禁/合入/沉淀/归档/清理仍pending。

历史结论（2026-10-19 R6）：**维持needs_fix / executing 2/9**；原树仍为R5同一 `5897eeb12` 且clean，未定位新的修复基线。10项状态不变，未重复测试或新增实现finding。见 [R6就绪报告](attachments/797/review-r6/review.md)；需正确修复commit/工作树及根因/覆盖交接后继续复审。

历史结论（2026-10-19 R5）：**needs_fix / executing 2/9**；R4原12项关闭5项、7项未完整闭合，新增3项（2项修复回归+1项原合同缺口），共10项OPEN。以 §9 R5 和 [R5完整报告](attachments/797/review-r5/review.md) 为准；“12/12修复完成”为工作交接主张，独立验收未证实。原树完成根因/覆盖调整、全部修复和准确收口证据后再审。

历史结论（2026-10-19 R4）：**needs_fix / executing 2/9**，12项合同内问题未关闭；依赖门已经解除，不能再用历史依赖阻塞说明本轮未通过。以 §9 R4 与 [完整报告](attachments/797/review-r4/review.md) 为当前依据，下方 R1–R3/工作轮记录保留历史。

R3 历史结论（2026-10-16）：**needs_fix / executing 5/9**；实施 HEAD 与 R2 相同，无新增实施提交，T-06/T-07 和依赖阻塞均未变化。F01 OPEN、F02 PARTIAL/OPEN、F03 CLOSED。R2 及其下方记录保留历史，尚无完整 final review pass。

R2 历史结论（2026-10-16）：**needs_fix / executing 5/9**。F-797-R1-03 经独立复核关闭；F-797-R1-02 文件缺失子项关闭、终收口仍 OPEN；F-797-R1-01 仍 OPEN。详见 [R2 报告](attachments/797/review-r2/review.md)。以下 R1/修复轮记录保留历史，不能当作完整 final review 通过。

2026-10-16 独立就绪核查 R1：**needs_fix**。R1 报告仍为 finding 依据。R1 修复轮后开放项收敛为：**F-797-R1-01（P1，T-06）** —— 真实 semantic contract / 正式 AutoDown / 性能复测，等 D-796/D-788；**F-797-R1-02（P2 部分）** —— T-07 可独立材料已交（commit `635053f74`），V-01–11 终收口与 freeze/SD hash 冻结仍等 T-06。**F-797-R1-03 已关闭**（Cargo.lock 入提交）。历史测试/性能总数在依赖一致性与完整验收前不得充作最终验收。D-796/D-788 满足后续 T-06/T-07，再申请完整 final review。

**已解除（2026-10-19）**：D-796/D-788 均已合入归档，T-06/T-07 已在同一 worktree 完成（见 §9 execution_done 记录）。**新登记（非阻塞，路由待用户/后续 L0）**：master 预存红——sema 语料 oracle 文件（`crates/auto-lang/test/cases/ac-semantics/oracle/ac.sema.{infer-bindings,cache-roundtrip}.bindings.json`）在 master 缺失致 `probe_adapter_corpus_executes_green` 红；属 796 收尾遗留、非本计划触面，本计划不代修（master ls-tree 实证，2026-10-19）。

T-01 需冻结的实际 C 工具可用性（已冻结：cl 19.44.35222 在机可用）、context/status/out 精确签名、target header/descriptor/错误/资源上限及参考负载（已随 contract-decision.md §1–9 冻结）。工具或 whole-path 不可行留下反例回规划，不能用 Rust 同编译器自测代替 C。正式版本号/冻结范围按最终证据评审，不先保证全部 ABI/未来类型。
