---
plan_id: PLAN-797
status: executing
feature_name: 首版 AutoABI 合同与双向互调探针
plan_revision: 1
author: [Codex]
created_at: 2026-10-16
updated_at: 2026-10-19
current_step: 7
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
| [x] T-01 | work | 用户执行指令 | 新workspace/lock、attachments/797/contract-decision.md，冻结候选原型/语义ref快照/资源/工具/perf；最小layout缓存与三腿探针 | [✅ 已完成] 历史 13/13 与原生可行性记录保留（T-01 `b00100cbd`）。R1 锁子项 **F-797-R1-03 已关闭**（commit `635053f74`）：`experimental/auto-abi-probe/Cargo.lock` 强制入提交（对抗 `.gitignore:8`），blob `598833812a`==worktree `git hash-object`，SHA256 `C7EC8DD5…`，size 27959；`cargo metadata/check/test --locked --offline` 全绿。锁一致性核对后允许引用未变依赖上的历史定向证据；后续依赖/锁变化须重跑受影响面。 |
| [x] T-02 | work | T-01 | 新contract/src/{descriptor,layout,validate}.rs、独立审定C header/oracle | abi_contract_、abi_layout_、abi_admission_；AC-01。[历史实施声明；R4 重开] descriptor/1+fingerprint、12 维 admission 拒绝矩阵（native 0 调）、layout 引擎==C 编译器 sizeof/offsetof（padding/序/嵌套+错 offset 反控红）、递归/超深/超预算受控拒绝；workspace 28/28 绿（commit plan-797-dev T-02） |
| [x] T-03 | work | T-02 | 新probe/src/{emit,build,link}.rs与C参考源/独立producer/consumer recipes，零改ac-core | abi_calls_三腿/异质控制；AC-02。[✅ 已完成] 三腿全套（含 C→AutoABI→C 嵌套链、fallible out-sentinel、record 冻结 offset、可观察计数器）exit 0；swap/错 offset/漏调用负例各 exit 恰 1；produce/consume 配方 admission 先行；abi_calls_ 7/7、workspace 35/35 绿（commit plan-797-dev T-03） |
| [x] T-04 | work | T-03 | 新provider资源/错误/callback fixtures与真实账本 | abi_resource_；AC-03。[历史实施声明；R4 重开] C provider 真 arena/代际账本/原方 destroy/失败注入回滚/buffer 规则/同步 callback；AutoABI 与 C 双消费者全协议 exit 0 + 回滚控制红；abi_resource_ 3/3、workspace 38/38 绿（commit plan-797-dev T-04） |
| [x] T-05 | work | T-04 | store adapter/阶段trace/六场景/512churn，首批.case.ad与oracle从T-01维护 | abi_cache_、abi_turnaround_；AC-04、05。[历史实施声明；R4 重开] ABI-O1 对象层（命中禁调 producer、真实输入失效矩阵、tamper=受控 CAS 拒绝、缺块=miss 重算）、512 修订 churn（rooted+leased 存活、块层收敛 live 集、用户哨兵不变、lease 释放翻转）；六场景×30 全绿，硬门 body-edit P95=539.5ms、descriptor-reject P95=0.1ms（<1000ms），210 样本 JSONL 全保留；abi_cache_ 6/6+abi_turnaround_ 1/1、workspace 45/45 绿（commit plan-797-dev T-05） |
| [x] T-06 | work | T-05、D-796、D-788 | 绑定796合入contract与受限probe adapter，只追加ABI recipe/worker；退出临时模型，正式AutoDown全执行/性能重测 | abi_binding_、原v1/v2兼容；AC-06。[历史实施声明；R4 重开] D-796/D-788 满足（双归档实证）；rebase 到 master+组内 auto-down 兄弟树；semref 退役→ac-sema-contract 真实绑定（收据在案）；contract 追加注册 abi/interop-v1（绑定互斥解码）；abi-worker（abiworker/1）+ 父侧 test_cases/abi.rs + stale 轴族；可执行语料 4 checks 过统一 runner（含跨进程 restart 命中与类型化负例）；probe 51/51（abi_binding_ 4/4）；正式复测双硬门 body-edit P95=500.8ms/reject P95=76.8ms（commit `35794e5ec`） |
| [x] T-07 | work | T-06 | 新verify脚本、docs/reports/plan797/、attachments/797/spec-delta.md与freeze proposal，来源/并行边界审计 | [历史实施声明；R4 重开] 终稿化：freeze-proposal（T-06 交付行+正式硬门数值+spec-delta 终稿 hash `6A6E1B2F..`+当前 canonical 输入 hash 含漂移后的 goals.md 重核）、support-matrix §0 正式通路面、spec-delta 终稿（真实通路字段+实测 SHA256 输入表）、verify 脚本扩面（t06_binding/统一 runner 语料/退役收据工件步、诚实 closeout、sccache 端口排除单次受控重试、零警告限定本 crate）——`-Scope closeout` ALL PASS（10 步，收据 docs/reports/plan797/verify-acceptance-20261019-180106.md）；execution_done=7/9（commit `8679859e4`） |
| [ ] T-08 | review | T-07 | 同树独立review，绑定native工具/接口版本，核互调/资源/兼容承诺 | AC-07；pass→reviewed=8/9，merge pending |
| [ ] T-09 | merge | T-08 pass | 冻结提交门禁；仅在通过后沉淀有限profile冻结状态、Spec/ledger、归档合入、批量到期判定、guard清理 | AC-08；实际完成9/9 |

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

预检/来源见[planning-preflight.md](attachments/797/planning-preflight.md)。候选v0.1不是现有产品支持声明。

## 10. 待澄清事项

当前结论（2026-10-19 R4）：**needs_fix / executing 2/9**，12项合同内问题未关闭；依赖门已经解除，不能再用历史依赖阻塞说明本轮未通过。以 §9 R4 与 [完整报告](attachments/797/review-r4/review.md) 为当前依据，下方 R1–R3/工作轮记录保留历史。

R3 历史结论（2026-10-16）：**needs_fix / executing 5/9**；实施 HEAD 与 R2 相同，无新增实施提交，T-06/T-07 和依赖阻塞均未变化。F01 OPEN、F02 PARTIAL/OPEN、F03 CLOSED。R2 及其下方记录保留历史，尚无完整 final review pass。

R2 历史结论（2026-10-16）：**needs_fix / executing 5/9**。F-797-R1-03 经独立复核关闭；F-797-R1-02 文件缺失子项关闭、终收口仍 OPEN；F-797-R1-01 仍 OPEN。详见 [R2 报告](attachments/797/review-r2/review.md)。以下 R1/修复轮记录保留历史，不能当作完整 final review 通过。

2026-10-16 独立就绪核查 R1：**needs_fix**。R1 报告仍为 finding 依据。R1 修复轮后开放项收敛为：**F-797-R1-01（P1，T-06）** —— 真实 semantic contract / 正式 AutoDown / 性能复测，等 D-796/D-788；**F-797-R1-02（P2 部分）** —— T-07 可独立材料已交（commit `635053f74`），V-01–11 终收口与 freeze/SD hash 冻结仍等 T-06。**F-797-R1-03 已关闭**（Cargo.lock 入提交）。历史测试/性能总数在依赖一致性与完整验收前不得充作最终验收。D-796/D-788 满足后续 T-06/T-07，再申请完整 final review。

**已解除（2026-10-19）**：D-796/D-788 均已合入归档，T-06/T-07 已在同一 worktree 完成（见 §9 execution_done 记录）。**新登记（非阻塞，路由待用户/后续 L0）**：master 预存红——sema 语料 oracle 文件（`crates/auto-lang/test/cases/ac-semantics/oracle/ac.sema.{infer-bindings,cache-roundtrip}.bindings.json`）在 master 缺失致 `probe_adapter_corpus_executes_green` 红；属 796 收尾遗留、非本计划触面，本计划不代修（master ls-tree 实证，2026-10-19）。

T-01 需冻结的实际 C 工具可用性（已冻结：cl 19.44.35222 在机可用）、context/status/out 精确签名、target header/descriptor/错误/资源上限及参考负载（已随 contract-decision.md §1–9 冻结）。工具或 whole-path 不可行留下反例回规划，不能用 Rust 同编译器自测代替 C。正式版本号/冻结范围按最终证据评审，不先保证全部 ABI/未来类型。
