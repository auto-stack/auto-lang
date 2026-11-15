---
plan_id: PLAN-978
status: execution_done
feature_name: 工程增量多目标构建与预编译SDK
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-16
plan_revision: 14
current_step: 9
total_steps: 14
depends_on: [PLAN-960, PLAN-966, PLAN-972, PLAN-967, PLAN-942, PLAN-1040, PLAN-1042]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/stdlib/design/assembly-manifest.md, docs/specs/stdlib/design/backend-assembly.md]
new_spec_components: [docs/specs/ace-migration/m2-build.md]
touched_goals: [GOAL-002]
---

# [PLAN-978] 工程增量多目标构建与预编译SDK

## 0. 变更摘要

2026-11-15 r14当前：执行用户Linux先代码后测试及普通math/io/net自有实现裁定，§5/6/7新增当前验收与1063移交规则，优先于旧Linux当期实跑表述；原任务ID/勾选/进度保留，Windows必需验证和全部Linux原分母保留。此次仅修订合同，不实施或运行测试。

2026-11-13 r13当前：纳入新1043来源后缀装配及最小#in平台静态选择，见§5/AC-LOAD/V-LOAD/SD-LOAD；原任务/状态/完成数与全覆盖不减，仅规划。

2026-11-13 r12当前增量：新增公共标准库能力至少AV/AC/A2R/A2PY，AV为同一执行器下native/C/Rust三provider；原A2C继续。见§5覆盖合同及AC-COVERAGE/V-COVERAGE/SD-COVERAGE；只修规划，状态/完成数不变。

2026-11-13 r11：参考Zig/Go的原生平台I/O设计承接；新增AC-PLATFORM-IO/V-PLATFORM-IO/SD-PLATFORM-IO，由原任务负责。保全部原任务/数量/进度/既有AC/V及其他会话修订，本轮仅规划。

2026-11-13 r10：按用户要求与967 r10 ABI-COMPAT对账，将BuildIdentity、RuntimeCompatibility和ProviderArtifact/Selection明确分账；细化源码构建/预编译包消费、精确缓存/重新装配、版本/目标/运行依赖、恢复/失败保旧和967→978→986交接。修正“provider变就一律重编”的歧义，新增AC-ID-BUILD/RUN/PACK、SD-ID-*及七个V-ID-*并绑定原T-01/02/03/08/09/10/T-04。14任务、drafting/0进度、原depends_on/B0/AXR0/1024-F12审计及全部原义务保持。此次只改文档，不实施、构建、运行探针或生成分发物。


2026-11-12 r8：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-12 r7当前：纳入默认自有Auto基础标准库/非libc默认provider及长度字符串ABI边界，见§5新增合同。保原任务/阶段/范围，新增AC-NATIVE/V-NATIVE/SD-NATIVE，仍只规划。

历史修订：2026-11-11 r6当前：增加B0最小foreign/provider/执行宿主构建装配切片；复用942/944/960及967/F0、969/E0，不等待完整966/972/新库、最终SDK/安装退役。AV首个预编译宿主先交，原T-03/09并行扩同源A2R recipe并共用身份。 原10项任务/全部AC/V保留，新增4项早期切片任务，现14项、drafting、0完成；只规划。

历史修订：2026-11-10 r5执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r3：按用户要求纳入[Design56：完整FFI与新版标准库](../design/56-ace-m2-ffi-and-native-stdlib.md)的本包义务；当前仍为drafting、0/10，只规划不施工。原任务/AC/历史记录保留，新增任务均在最终work验证与独立review之前。

承942/930/918等实际生成治理，统一工程源图、包/stdlib/host/ABI/target/toolchain身份；开发AV/AC预构建宿主，发布A2R/C显式编译。跨模块真实依赖/负依赖与无预热恢复首编辑、工作集/GC纳入实工程。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-16本轮授权（goal链）：用户以/goal指示"完成计划978"，按auto-plan-goal范式一次性授权全链：work（含建树 `D:/autostack/.wt/lang-978/auto-lang`、实施与验证）、独立review、有界修复与merge（规范沉淀、归档、清树）。实施范围/验收/预算按本Plan r14合同（含Linux腿移交1063规则），不因本条扩大或缩减。材料scope：`D:/autostack/plan-artifacts/plan-978/`。

2026-11-11本轮授权：用户要求规划首个ACE UI Hello及AV/Rust FFI路线、核对既有M2冲突并作必要重排；仅规划/设计/导航，不实施、创建worktree或探针/fixture/SDK、构建/运行、提交或发布，未指定模型/工期。沿旧语言用例守恒、新Auto标准库与AutoABI/FFI、Demo递进的三线，不削完整M2范围。

现场语言HEAD 9e36d984e、UI HEAD d88e9c51。已核av-core/src/host.rs仅console/1（sha256前16 30d6dcc475850cd0）、a2x-rust/src/provider.rs仍拒source foreign（3ea453dc2452576f）、core/src/source/cst/frontdoor.rs（dd3d93d06799ae52）；现有auto-bindgen/src/extractor.rs是预置C manifest表（8c7f1295cc83c6fb），不能称Rust元数据导入器已实现。Iced由Rust宿主编译执行，AutoVM不解释其Rust源码；现有DynamicComponent/VmBridge/handler_codegen深依赖旧执行链。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `experimental/ace/crates/artifacts`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/source-query`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/a2x-store`（本轮确认存在；开工重核符号/版本）。
- `docs/design/53-independent-build-and-generated-cargo-governance.md`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-09 增补调研锚点

依据用户本轮M2范围追加与已读取的标准库引用讨论，读取基线HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；本轮只读核源码，不宣称现有实现已满足新义务。Design56区分现状与目标；原§4历史快照不覆盖本次范围。

- `crates/auto-lang-core/src/stdlib_assembly/manifest.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/auto-lang-core/src/stdlib_assembly/loader.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/source-query/src/modules/packages_store.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/source-query/src/modules/persistence.rs`（现存入口/规范；开工核真实符号与SHA）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：工程增量/foreign闭包与候选SDK。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [experimental/ace/crates/a2x-store/src/lib.rs](../../experimental/ace/crates/a2x-store/src/lib.rs)：精确语义/目标key和产物装配；读取SHA256前16 `eccd5ff0f35f982e`

- [crates/auto-lang-core/src/stdlib_assembly/model.rs](../../crates/auto-lang-core/src/stdlib_assembly/model.rs)：声明/目标/环境身份；读取SHA256前16 `95fbc7911a149d36`

- [crates/auto-lang-core/src/stdlib_assembly/loader.rs](../../crates/auto-lang-core/src/stdlib_assembly/loader.rs)：候选来源/权限重验，不缓存运行授权；读取SHA256前16 `0be54327bf6130a2`

### 2026-11-13 ABI兼容边界对账（r10，文档授权）

用户明确只核对/必要修订本计划的构建缓存、装配和分发，与967的ABI边界一致，不实施。读取主检出HEAD `9b68da44ab6ea09be508129bed355dd54cec56e1`，978入场为r9/0/14、含1024-F12承接WIP；本轮保留该内容及其他会话修改，不取号、不建worktree、不运行任何Cargo/工具链/包消费探针、不改canonical Specs/legacy JSON。执行仍须后续明确work授权及实际接口就绪。

已读现行Spec：stdlib/assembly-manifest与ace-binding-and-assembly、auto-abi/conformance-cache、generated-cargo-build-lifecycle；源码锚点`a2x-store/src/lib.rs`保语义/目标键分层，`stdlib_assembly/{manifest,model,loader}.rs`保消费者/来源/权限边界。944共同/消费者指纹与legacy schema4已有精确含义，不能将它们直接重命名为新运行兼容身份，亦不能据已有有限profile推定下面规则已实现。Design53/942只作生成构建/cohort治理来源，ABI兼容不意味着Rust构建上下文可合并。

本轮消费来源：967 r10 SHA256=`ec7b2ff42f465b9524c02aa0f84cefb8c8010ab7bf6bb7dd461f4148d72a28ea`；Design41 SHA256=`758277d92fff66fcd61a4b88b416667d6ad651230ef840e90f2301a284d9b606`；Design56 r6 SHA256=`6dbba6bca99827950fe1716b3acd3d00248ec50be3c2ffb24b9dfba3603e8a36`；assembly-manifest Spec SHA256=`e5659cb7d9871af0243335116da60488537a5e5cab330f6e343c8fe4546f8ba5`；ace-binding-and-assembly Spec SHA256=`6c8c8b068a3c235c23599e319c37d0a76bd7887901826ed8c4ef679218d77c68`。源码/依赖后续漂移只重核受影响证据，以上hash是文档取材身份，不是运行授权。

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-09, T-10, T-B0-I, T-B0-V, T-B0-R, T-B0-D |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


承942/930/918等实际生成治理，统一工程源图、包/stdlib/host/ABI/target/toolchain身份；开发AV/AC预构建宿主，发布A2R/C显式编译。跨模块真实依赖/负依赖与无预热恢复首编辑、工作集/GC纳入实工程。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：候选SDK和最小安装闭环（r2）

本包产可重定位候选SDK/领域包/runtime清单及安装recipe供985使用，正式升级/默认/发布归986/987；核包版本选择、互斥profile/feature/cfg/ABI混装与多进程store并发。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 2026-11-09 正式M2范围增补（r3，当前合同）

新增FFI-M2-07与LIB-M2-02/06：候选SDK携带公共源/受检IR/ABC/native/A2X适用片段、stdlib/runtime、binding/shim/provider及版本/目标/链接闭包；构建时首次import所需工具和已生成程序运行所需依赖分账。不得要求native程序启动时隐式找开发仓、旧VM、Python或rustc/clang；实际声明的动态库正常保留。

复用942/944/960/967既有store和装配事实，明确来源/锁/目标/toolchain/ABI/runtime/provider/生成器身份；实际读取才依赖，避免每次Auto body编辑都跑Cargo或全量bindgen。失败原子保旧、恢复重新授权；installer不能把已编译库视为新运行授权。978交候选与recipe，986/987最终安装/升级/退役闭合，不新增对985/986反向依赖。

本次新增验收/任务如下表映射，详见§6–8；原任务不删除、原非目标中的“本包之外”按本节明确新增范围解释。所有V均为待执行设计，当前不写canonical Specs。

| 任务 | 验收 | 用例 | 本包交付/义务 |
|---|---|---|---|
| T-09 | AC-07 | V-09 | 将新库/foreign生成物/预编译runtime/provider/ABI依赖打入可重定位候选SDK，分别列生成工具与程序运行依赖，接既有模块/装配loader及源映射。 |
| T-10 | AC-08 | V-10 | 落实headers/crate/lock/features/cfg/target/toolchain/provider/ABI/runtime/生成器身份的精确缓存和失败恢复，保持原工程P90及更严门，不用全量失效掩正确性。 |

### B0：首个Hello的最小构建与宿主闭包（当前早期入口）

最小前置为已交942/944/960可用装配/store接口，967/F0-D的binding/shim/provider及ABI清单、969/E0-D公开执行入口。不要求完整CT、全新stdlib、File/网络、968/970或全产品SDK；如果真实修改用到这些尚未交能力则具名停相应腿，不从代码标题推断具备。Rust构建工具属于宿主/FFI构建端依赖，不能把Rust标准库当成Auto公共标准库的语义基底。

T-B0-I在现有artifacts/source-query/a2x-store、stdlib_assembly manifest/loader及生成构建治理owner内冻结最小源/grammar/domain/ABI/provider/运行时闭包和调用命令；只包装已交可执行代码，不先发空SDK。以967非UIconsumer和969入口自证，B0不等待980/HELLO或UI宿主成品；UI提供自己的Iced provider实现与Cargo依赖，980/HELLO再按B0配方组装窗口host，语言仓不反依UI crate。

AV宿主和Rust shim先明确构建一次；之后相同宿主/ABI下改Auto文字或普通函数，运行前门/AV更新不得重跑Cargo/rustc/bindgen。源码构建实际读取的Rust provider、ABI、features等真实输入变化须重建受影响闭包；选择已发布的兼容C ABI provider则重新选择/装配/准入，不默认重编无变化consumer或要求运行端rustc。首个Hello不承诺同PID在线热更新/未改函数P/IO0全工程门，970及原T任务保留；这里只核重复运行/改源不重建不变宿主。

A2R并行装配腿属于原T-03/09的AXR0切片，消费967/T-12首个简单签名正式来源；同源新A2R生成/编译的recipe复用F0的等价Rust adapter或C ABI shim、同一签名/资源事实，不重新设计安装系统。A2R正常调用Rust工具链，运行包不隐式依赖开发仓/旧VM/Python/rustc；AV开发闭包明示其运行宿主和源/字节码需求。清单含源/grammar/ABI/runtime/provider/toolchain/target/产物hash，接收方可拒错/缺shim；不把版本/授权缓存成永久信任。

B0-V/R/D先闭AV的V-B0-*即可正式交最小闭包/recipe供980，不等待AXR0-PACK；AXR0腿另独立复核/phase fold交980/RUST，不能提前勾原完整T-03/09。原T-08/09/10扩完整新库、外库、重定位候选SDK与精确缓存/原正式性能门，完整SDK/发行退役仍未完成。

早期切片是本Plan M2-A内部交接，不新增ACE里程碑或Plan终态；只在同一权威树按owner流程独立复核、正式落主并记source/接口/grammar/ABI/provider及真实结果后供下游消费。同Plan尚有原完整义务时不归档/删树/勾整包完成。frontmatter保最终依赖并集，早期切片只等本节最小接口；同一文件单写，不以分支reviewed/WIP当交付。最小切片通过不关闭原AC、完整Rust FFI/全UI/RG-A03/M2A-08；不新增对980/981/985整包或AGE联合门的反向依赖。

### 默认自有标准库与FFI边界（2026-11-12当前）

依据[Design56 §0.2](../design/56-ace-m2-ffi-and-native-stdlib.md#02-默认auto系统库可选ffi与字符串边界2026-11-12当前)。候选SDK/manifest列默认自有stdlib实现、OS适配、目标运行时、可选Rust/C库/工具/宿主依赖，分别冻结默认与外库选择。默认AV/native与独立AC的库调用链不得隐藏libc/Rust std；A2R/A2PY/原A2C目标运行依赖须显式记录并保持Auto语义，不把目标运行时依赖冒原生无libc。AV宿主含C/Rust运行支持亦分账，不能自称整个二进制无libc。

本包验收：仓外默认库不带可选provider仍运行；显式外库腿单独装配；移除开发仓/缺默认原语/错误自动回落/隐去链接或字符串适配依赖均拒绝，旧有效候选保留。 原任务T-09,T-10承接；不新增实现任务，不以新义务扩大M1/F0或解除945–948的M1-TAG。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r8当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at)（下列节选L1–L1；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn main() int { return 2 + 3 * 4 }
```

本包所需独立结果：B01在仓外异根/任意cwd运行14；body改4→5得17且不变宿主cargo/rustc/bindgen计0；源码构建实际ABI/provider输入改变须重建受影响依赖；兼容预编译provider换包需更新选择/装配与准入，不自动重编不变的consumer。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 普通工程/锁→944声明与A2X store→源/语义/目标key→预编译runtime/provider/SDK清单→冷消费/恢复。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct ArtifactClosure { source: SourceClosure, grammar: GrammarIdentity, stdlib: LibraryIdentity, abi: AbiIdentity, providers: ProviderSet, tools: BuildToolSet, build_identity: BuildIdentity, runtime_compatibility: RuntimeCompatibility, selection: ProviderArtifactSelection }
struct SdkRecipe { target: TargetAbi, build_inputs: InputSet, run_dependencies: DependencySet, artifacts: ArtifactSet, consumer_requirements: RuntimeCompatibility, provider_selection: ProviderArtifactSelection }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [experimental/ace/crates/a2x-store/src/lib.rs](../../experimental/ace/crates/a2x-store/src/lib.rs) | 精确语义/目标key和产物装配 |
| [crates/auto-lang-core/src/stdlib_assembly/model.rs](../../crates/auto-lang-core/src/stdlib_assembly/model.rs) | 声明/目标/环境身份 |
| [crates/auto-lang-core/src/stdlib_assembly/loader.rs](../../crates/auto-lang-core/src/stdlib_assembly/loader.rs) | 候选来源/权限重验，不缓存运行授权 |

**状态、错误与资源。** 缺shim/错ABI/错target/profile/陈旧库、错误锁引用准入拒；失败不覆盖旧可运行包，路径重定位不偷偷找开发树。 构建工具与用户运行依赖分账；lease保在途产物，缓存不存进程地址/权限；默认Auto原语与显式FFI两闭包均核。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at) | 值14/17与编译调用计数 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `b7e5f25e46902be2` | 拟ace.m2.p978.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at](../../crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at) | foreign/shim实际链接闭包，显式外库不混默认库 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `4af83cfa91c6ba9c` | 拟ace.m2.p978.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [docs/design/53-independent-build-and-generated-cargo-governance.md](../design/53-independent-build-and-generated-cargo-governance.md) | 复用942生成构建治理，不建第二套缓存 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `208e3d74988645f0` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** B0只交最小AV宿主recipe，完整SDK保CT/新库/四端和原性能门；实际安装/升级归986，不反等986才能给候选。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-09`、`T-10`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### ID-ABI：承接967 r10的构建、装配与分发身份（当前规则，未实施）

单源规则：[967 r10 B-01..06](archive/967-ace-m2-host.md)、[Design41 §5.1](../design/41-interface-metadata-and-foreign-abi.md#51-rust-provider的构建与兼容合同2026-11-13目标态)、[Design56 §3.3](../design/56-ace-m2-ffi-and-native-stdlib.md#33-rust-provider的二进制兼容与独立分发)。本包负责消费/精确缓存/装配与候选包，不重新制定Rust ABI、建第二loader/store或接管967的shim生成/资源协议。

**ID-01 身份投影和职责。** 上面的ArtifactClosure/SdkRecipe仍是拟议闭包清单，不是所有阶段共享一个全量hash。新增字段为语义草图，实际Schema/符号在T-01复用既有owner冻结。

| 身份/事实 | 来源与本包owner | 缓存/准入使用边界 |
|---|---|---|
| 既有语义/声明/真实读集 | 960/944、source-query、A2X已有分层；978 T-02/03消费 | 逻辑签名/语义真实改变才影响对应body/语义键，物理provider构建信息不灌进每个语义键；原正负依赖/授信规则保持 |
| BuildIdentity | 967 T-14提供provider库+shim的源码/lock/features/cfg/实例、生成器/rustc/toolchain/flags/target/SDK/linker/native输入；978 T-10纳入对应构建节点 | 精确产物缓存/生成收据按实际读取核身份与内容，真实输入变化失效受影响节点；ABI兼容不能让错误BuildIdentity命中 |
| RuntimeCompatibility | consumer当前required符号/签名/布局/ABI域/Schema/语义/owner/线程/运行依赖要求，与967 provider的offer独立核对；978 T-03/08/09装配 | target OS/arch/env/位宽/端序/必要ISA/OS-runtime及接口合同须兼容；**不比较双方rustc必须相等**，不把当前构建工具安装/缺失当运行能力 |
| ProviderArtifact/Selection | 实际工件hash/发布与构建收据、包/lock pin、实际运行依赖选择、provider代次与当前授权；978 T-08/09/10消费 | 具体选择、完整性和授权与“ABI可兼容”分开；兼容不自动解除精确pin、不继承旧PASS/授权/活对象，新选择要重新核验 |

现有schema4/ace-stdlib-assembly/1、共同/consumer指纹以及AXS语义/AXR目标等历史身份保持原义；扩展新投影或适用profile必须版本化、旧新拒绝策略明确，不能放松既有未知/错版/不符来源拒绝。新runtime投影不替旧root恢复/ABI/工件内容验证，完整manifest hash不等同接口兼容结论。942的Cargo cohort/可写target复用仍按真实构建条件，不能因C ABI相同就混用不同rustc的Rust中间产物。

**ID-02 两种工作模式。**

- 源码构建模式：从原use.rs工程/库源需求，经967自动生成库+shim并在兼容Rust闭包中构建；978只调度实际缺失/失效节点并记录工具与身份。新rustc、flags、features、真实header/crate/native输入或生成器改变对应BuildIdentity，应重建并取得新证据；任意旧rlib/Rust ABI dylib不因C ABI wrapper名而可复用。泛型/API超出已发布导出需明确源码重建或拒绝。
- 预编译provider消费模式：按版本化manifest/工件与当前consumer需求选择已经封装C ABI的provider，核RuntimeCompatibility、Selection/lock和当前授权后装配/调用。正常运行不用Rust源码、rustc/Cargo、bindgen或编译期SDK；它们可作为构建来源信息记录，不要求机器上装有相同工具链。AV开发模式仍可按清单需要Auto源/字节码/预编译宿主，不能误删这类合法运行输入。

**ID-03 改变什么、失效什么。**

| 变化/请求 | 978必须采取的动作 | 必须避免的误判 |
|---|---|---|
| 源码构建请求换rustc/flags/生成器、实际Rust库/header/features/cfg等 | 变更相应BuildIdentity，失效/重建真实受影响库+shim/消费者节点，按新工件复核证据 | 不能因外部ABI没变就用旧精确构建缓存；不全量清库掩盖读集缺陷 |
| consumer工具链升级，已发布provider字节/接口/运行条件不变 | consumer自身构建节点按真实依赖失效；独立C ABI provider保原产物，运行兼容/选择重验 | 不把provider rustc标签不同当ABI不兼容，不强制重建它或要求运行时rustc存在 |
| 选中新的C ABI兼容provider工件，旧required子集仍满足 | 更新具体选择/hash/消费者装配收据并重新准入；实际链接输入改变时可重链接/重打包，静态链接尤需核；调用签名/代码不变可复用相应consumer代码 | 不能把换包一概当源码编译，也不能保证所有模式零链接/零工作；新工件不能沿用旧执行证明，精确pin需显式更新选择 |
| provider破坏required签名/布局/ABI域/Schema/语义owner或target/runtime不兼容 | 准入前具名拒绝，或仅选已验证adapter；受影响binding/stub/布局/证据失效；保旧可用包 | 不先盲调再验，不凭major相同/abi=C/repr(C)或同CPU就放行 |
| 仅按已声明规则增加可选符号、旧required子集不变 | 工件完整性/选择及兼容重新核，实际未读取的函数body/逻辑语义不因此重产 | 不把全manifest字节hash不同直接判ABI破坏，也不能只看minor版本号放行 |
| 显式运行依赖缺失/不满足，如CRT/CPython/系统库 | 拒绝运行或按明确供应策略交安装owner处理，保构建工具/运行依赖分账 | 有C ABI不等于单文件/无依赖；不能为普通native默认拉Python，也不能拒绝明确声明的合法Python provider |

**ID-04 装配与恢复。** 按当前source/consumer需求及选择策略形成独立期望，不能从候选payload反抄期望。构建路径先核精确BuildIdentity/内容；预编译路径核选中工件/来源/版本化required-offer关系和当前target/runtime/授权，再按既有机制绑定/链接/发布。装配运行兼容只给使用资格，不提升受检/执行证明等级，也不持久化权限、地址或活句柄。恢复重新核选择、声明/依赖、内容、授权和runtime，不能因历史cache/PASS/相同ABI跳过任一步。

兼容规则只作用于对应C ABI provider边界，不把旧互斥profile/Schema或AutoABI跨域强行混装。新旧版本opaque对象仍pin原provider/代次，GC/卸载按967；本包不得只因ABI相容就将旧handle重路由到新库。失败/取消/陈旧候选按既有CURRENT事务保旧、释放仅候选自有lease；修复后同一选择/加载入口再验证可用，不能只证明一次fallback。

兼容换包仍须建立与当前provider/选择相符的装配收据、consumer context/根引用（适用时），不能让旧精确root/consumer_fingerprint跳过当前上下文重验。仅实际未改变的语义/代码/目标片段按其自身有效身份复用；选择变化、重新绑定、链接、发布与代码生产分别计数，不以“代码可复用”声称整个装配零工作。

**ID-05 候选SDK与分发清单。** 复用现有manifest/store/recipe，不新增第三套cache或loader。至少明确以下投影：

| 清单内容 | 构建时/运行时要求 |
|---|---|
| provenance/BuildIdentity、库+shim来源、生成器与实际构建工具 | 构建可再现/缓存/证据身份；不是运行主机必须安装的工具清单 |
| consumer requirements/provider offers、ABI/接口/Schema/target/runtime | 装配/恢复/运行实际核兼容；版本策略具名，不默认兼容 |
| provider artifact/hash/lock pin/发布与代次选择 | 工件完整性/绑定/lease归属与显式选择；不携旧授权/裸指针/活对象 |
| 实际运行闭包 | native/provider、相关C ABI描述/导入库（适用时）、CRT/系统/可选CPython等动态依赖、合法Auto源/字节码/预编译AV宿主、相对资源与许可/分发条件 |
| 构建recipe闭包 | 生成工具/SDK/编译器/链接器/header/crate/source/flags等实际构建依赖，与运行闭包分列；staticlib须明确再链接与LTO/bitcode条件 |

可重定位不等于source或provider内容身份可忽略；可以剥离不参与语义的安装绝对路径，却必须在新根实际解析并核到所声明依赖，不能偷偷回开发树。独立分发/离线启动不等于无运行依赖或未知平台可执行。A2R直接crate adapter仍属同Rust构建闭包，不能冒称独立跨rustc C ABI路径。

**ID-06 单向交接与启动条件。**

| 交接 | 原owner / 本包任务 | 正式关闭证据 |
|---|---|---|
| 967 T-11/12/13/14/15 → 978 | 967产生成库+shim、C ABI事实/资源规则、双身份/工件/依赖与独立ABI证据；978 T-01/02核接口，T-03/09消费 | 正式落地SHA、descriptor/接口版本、实际artifact与BuildIdentity/RuntimeCompatibility/Selection映射；已有且身份适用的967 V-ABI-*证据复用，不重跑同一整档 |
| 978 T-08/09/10 → 985/986 | 本包交可重定位候选、精确缓存/选择/恢复、依赖和recipe及独立消费结果；986仍管真实安装/升级 | 实际包/工件hash、目标/接口/运行依赖、授权/资源/失败恢复证据、具名未交环境；本包夹具不代最终安装验收 |

原最小B0仍只等967/F0与969/E0正式有限切片，不被本节升级成“全SDK或967全包先完”；F0不足支持的新opaque/浮点/参数帧/跨版本分发验证须核对应扩展已交后再开始该腿。不得为本节加967→978或978→986整包反等；F12/TargetProvider/ManagedStore/PageSink/QueryHost/CtExecutor与1002按任务分流的既有约束保留。

### Windows/Linux无libc平台I/O（2026-11-13当前）

依据[原生平台I/O Draft 0.1](../design/stdlib/native-platform-io.md)。完整SDK消费1040 Linux ELF及1039/1041/973实际来源；target/object/ABI/入口/stub/runtime/工具身份入依赖/恢复。独立AC PE/ELF、AV宿主和构建工具分账，无libc调用链不冒整个AV无libc；最终依赖补1040，早期B0/F0/Hello前置不变。

### 后端与AV来源覆盖（2026-11-13当前）

以[覆盖设计](../design/stdlib/backend-provider-coverage.md)为本包当前增量：旧“四端”指AV/AC/A2R/A2C，不再代表全部required。新增公共能力最少AV/native、AV/C、AV/Rust、AC、A2R、A2PY六路径，原A2C及原平台/数值/资源/oracle不减。AV只一套解释器/API/语义，三来源共享Auto算法，仅适配必要原语；不能以配置名切换冒真实dispatch。

候选SDK列唯一AV、可选native/C/Rust provider工件、原provider销毁/版本/lease与Python生成helper/目标运行依赖；仓外重定位后实跑六路径及原A2C。Python自身的CPython/系统运行依赖如实单列，不把无libc要求错误扩给Python宿主。B0早期范围和前置不变。

1042是最小Python接缝，不自动证明本包全部语言域支持；需要更大Python语义时消费977/PY已交能力切片或先具名补齐，不反等977整包。C/Rust provider复用967，native底座沿1039/1040原依赖。纯函数可说明身份/算法/输入/观察适用性后复用证据，资源边界需各有实际调用与原provider销毁；不机械跑全后端×全provider笛卡尔积。缺目标工具/runner/依赖保持pending，不能以emit、宿主fallback或同错结果关行。显式第三方引擎的依赖另列，native来源不冒所有第三方算法都已Auto重写。

### 新加载机制完成依赖（2026-11-13当前）

依据[设计§8–10](../design/stdlib/backend-provider-coverage.md)与[1043](archive/1043-ace-stdlib-context-assembly.md)。公共.at＋at/c/rs/py来源后缀组成同一逻辑模块，运行后端另选；#in打开临时namespace并让is/if/for继承编译期属性，不扩平台后缀。最小层由1043先交，966继续完整CTE，不能反向等966整包。

经966/972/967消费1043正式source/context/namespace/ABI身份；候选SDK明确所选源/工件与实际构建/运行依赖，跨根恢复不能借开发树旧侧文件；B0早期闭包和前置不扩。

本包整包/当前阶段required完成须核1043实际source/统一绑定/最小#in/增量恢复及所需真实后端的正式来源，不以文件存在、provider名字、旧归档或候选语法冒已交。948只审分母/去向，978验装配闭包，985验完整A；普通能力owner仍另交真实外库dispatch与OS/资源结果。原早期切片/调查范围不据此扩大，公共能力缺路径不勾整包完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-build.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/stdlib/design/assembly-manifest.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08 |
| SD-04 | modify | `docs/specs/stdlib/design/backend-assembly.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-build.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-HELLO | add | docs/specs/ace-migration/m2-build.md（沿原owner） | 未有早期交接→本切片实际已验接口/源/拒绝及剩余完整义务；交付后沉淀 | 最小Hello不冒全包完成 | AC-B0/AC-B0-REVIEW/AC-B0-LAND |
| SD-NATIVE | add | `docs/specs/ace-migration/m2-build.md`（沿原owner，仅真实交付后沉淀） | 默认实现选择未明→自有Auto/OS默认与显式FFI分账、长度字符串/目标ABI及来源验证 | LIB-M2-07/08 | AC-NATIVE |
| SD-ID-BOUNDARY | modify | `docs/specs/ace-migration/m2-build.md`（实际交付后沉淀） | 混合SDK闭包→已验BuildIdentity/RuntimeCompatibility/Selection分账及967/986交接 | 构建命中不等于运行兼容 | AC-ID-BUILD/AC-ID-RUN/AC-ID-PACK |
| SD-ID-ASSEMBLY | modify | `docs/specs/stdlib/design/assembly-manifest.md` | 既有有限版本保持→版本化新增consumer需求/provider供给/工件选择投影，旧指纹不重释 | 不靠全manifest hash或rustc标签判断ABI | AC-ID-BUILD/AC-ID-RUN |
| SD-ID-ADMISSION | modify | `docs/specs/stdlib/design/backend-assembly.md` | 原有限装配→已验模式/目标/接口/当前授权/恢复与失败保旧，runtime和构建工具分列 | 不新增loader或永久trust | AC-ID-RUN/AC-ID-PACK |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-build.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |

### 1024发现承接合同（2026-11-13，r9，仅规划）

承接1024的F12：保留现有provider/store/executor接缝，优先级P2（F12仅保留约束）。用户仅授权复审与分配，尚未授权实现；原支持/AC/任务/阶段/预算不减少，原任务进度0保持。

F12是正向保留约束，不是待修缺陷。T-01列真实consumer和身份版本，T-02/03沿现provider/sink/executor扩展，T-04做错身份/恢复当前权限负控；不把保留实践计为修复数。

1002关系：核心职责/原入口反例/固定组治理均不以1002为硬前置；新作者格式迁移或document/IR/adapter重构仅等待G1002（1002 T-03四例七检查及负控通过，T-04独立复审、接口正式合入并绑定SHA）。不等待全语料搬迁；1002不反向依赖本Plan整包。现有M2能力前置不因承接登记被豁免，也不把这些非紧急结构改造伪装成可立即施工。

来源：[1024](1024-ace-m2-code-quality-and-refactoring-audit.md) r2与Git 81d8fc03f10f4a7d179b0bc3a917602406ed7b91。F01–35高风险修复另立1027–1037；本Plan消费正式合入结果并在T-01重核漂移。与在途文件同写须串行，1017成本身份单独协调，不能靠删例减成本。规范影响沿本Plan原SD目标，review补当前实际实现，不改canonical Specs。

### 平台I/O规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-PLATFORM-IO | add/modify | docs/specs/ace-migration/m2-build.md | 笼统provider→本节平台IO/算法/运行/依赖分账 | 无libc平台设计 | AC-PLATFORM-IO |

### 覆盖规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-COVERAGE | add/modify | docs/specs/stdlib/design/assembly-manifest.md | 旧四端→新增A2PY、同AV三来源及真实逐操作证据；未交不升支持状态 | 用户最低覆盖 | AC-COVERAGE |

### 加载机制规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-LOAD | add/modify | docs/specs/stdlib/design/assembly-manifest.md | 旧有限后端侧文件→本包实际消费1043来源装配/#in/目标依赖及正式来源，未交保持缺口 | 用户确定新加载机制 | AC-LOAD |

### Linux延期与默认自有实现（2026-11-15当前优先）

本包适用范围：**候选SDK中标准库的Linux编译/运行/恢复验证腿**。[覆盖设计](../design/stdlib/backend-provider-coverage.md)和[平台设计](../design/stdlib/native-platform-io.md)为共同合同。当前Linux分支继续写代码、保完整fixture/expected，但不执行Linux测试、不从Windows启动WSL/VM或隐式准备Linux runner；按1061隔离入口，环境完善后由[1063](1063-ace-stdlib-linux-deferred-validation.md)专项验证。

本包当前完成门调整为Windows原required门＋Linux实现/完整移交审查；Linux每个原操作/case/check及增量保留backend/provider/target/ABI、源/依赖/输入/oracle/未跑原因，标`deferred-required / NOT_RUN`，明确owner=1063。不能改PASS/NA、删分母或用Windows/emit-only/交叉编译代实际Linux运行；实现与验证状态分开。1063不是本包反向依赖；本包可完成当前排程，不冒Linux已验证。旧1040/1041等归档实际证据保留，变化后的实现不自动继承旧绿。

SDK保Linux代码/目标/provider/ABI/依赖与未验证状态；不得把跨编译成功或Windows仓外运行当Linux支持证据。Windows固定系统DLL可用PE imports/IAT；按需解析复用既有受检loader。构建工具、AV宿主、目标程序/外库依赖分账。

AV仍一个执行器，native/C/Rust为显式来源；Windows AV三来源、AC、A2R、A2PY及原A2C等本包原适用门不减。公共Auto算法尽量共享，C/Rust可选实现和目标runtime adapter保留；高级外库显式依赖不代默认Auto库。已完成任务证据保留，剩余work及最终验证/review按本修订核Linux移交；不得把合同修订计作实现进度。非标准库Linux专属项目不由本条自动豁免。

### 当前延期规范增量（待实际交付）

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-LINUX-PHASE | modify | docs/specs/stdlib/design/assembly-manifest.md | Linux当期必跑→代码/完整义务移交1063，verified保持pending；Windows原门不减 | 用户明确排程 | AC-LINUX-PHASE |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 稳定宿主Auto body编辑实际cargo/rustc=0、未改函数各生产点与工件IO=0 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | CT读body正确失效 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | 坏缓存拒绝保旧且下一恢复读回修复 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 原工程P90≤2000ms及既有更严门 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-ID-BUILD | AC-ID-BUILD / ID-01..03 | 源构建分别改provider rustc/flags/features/header/生成器，及无关Auto body/只改consumer工具链控制；引用967有效provider构建证据 | T-02/03/10按现有真实读集/producer/store/编译进程计数及独立结果核受影响BuildIdentity miss，无关节点复用；ABI相同不能洗白旧构建产物。consumer变化不强制重建未读且未变的独立provider | work；准确组/入口T-01冻结，非全局失效或仅自报bool |
| V-ID-RUN | AC-ID-RUN / ID-02..04 | 使用967已验不同rustc构建的C ABI provider与固定consumer，运行环境去掉rustc/Cargo及开发Rust源码；AV所声明源/字节码/宿主保留 | T-03/08/09实际装配并执行对独立值/ABI oracle；compiler标签不同不误拒，工具启动计0，source/provider身份适用；不得让ABI兼容触发错误构建cache hit。缺第二toolchain或provider正式证据时pending | work；复用967工件/证据，不重建第二FFI实现 |
| V-ID-SELECT | AC-ID-RUN / ID-03 | 同ABI/旧required子集保持但工件hash改变、明确可选符号增量；精确pin匹配/不匹配对照 | T-08/09/10记录新selection/装配收据与当前准入；策略允许者正常执行，精确pin不符不得自动升级；不因全manifest改变就破坏ABI。只重做实际受影响装配/链接/打包，不凭同ABI重用旧工件证明 | work；dynamic与static链接动作分列 |
| V-ID-REJECT | AC-ID-RUN / ID-01/03/04 | 逐轴错target/ABI域/Schema/required签名/布局/owner/runtime；同major/repr(C)/CPU等伪兼容近邻与合法相邻输入 | T-03/08/09按独立consumer要求验证；业务dispatch=0，分类为身份/ABI或运行依赖不符，不能先调用再验；合法控制真实成功。原版本manifest/profile拒绝面不减 | work；不将OS loader初始化宣称为沙箱 |
| V-ID-PACK | AC-ID-PACK / ID-02/05 | 候选SDK/manifest/artifacts复制到统一scope中的隔离异根，隐藏开发cache/Rust工具/源根；缺动态依赖或伪用构建工具补运行闭包；staticlib另走原生重链接 | T-08/09实际公共入口运行，不读开发树；provenance中记录rustc不使启动依赖rustc；native默认无隐式Python，显式声明的Python/provider依赖仍实核；所有路径/hash/依赖/工具与实际链接/执行分账，缺环境不PASS | work；不代986正式安装/升级，不承诺所有平台 |
| V-ID-RESTORE | AC-ID-RUN/AC-ID-PACK / ID-04 | 冷恢复后撤权/改pin/坏工件/不兼容或缺依赖；兼容新provider但仍有旧代handle/pin；失败/取消/迟到与修复后再启动 | T-08/10沿原CURRENT/lease/loader实核，不信历史PASS或候选自带期望；旧可运行包/旧owner保活，坏候选不覆盖/不误GC，旧handle不交新代；修复后同实际入口成功。装配修复不充ABI接口实现 | work；沿967资源规则与既有预算 |
| V-ID-HANDOFF | AC-ID-BUILD/AC-ID-PACK / ID-01/06 | 967正式工件/descriptor/两类身份与selection映射、978候选清单/recipe与986安装需求逐项对账；去掉required字段/错身份/混淆工具运行域控制 | T-01/09/10/T-04查真实代码/包/执行收据：无来源或缺字段不能交付，无第二loader/store/FFI生成器，三类身份字段不互当cache key/准入条件；F12及旧B0/AXR0所有验证保留 | work；独立trace/运行结果才闭行为，清单检查不替验收 |

完整职责组拟定：`cargo t ace-artifacts ace-semantics ace-a2x ace-av ace-native generated-delivery-controls`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 同内容异根构建身份稳定，错ABI包拒绝；并发写CURRENT无丢更新；候选包仓外运行；Auto编辑无cargo而真实Rust输入变化仍重建。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 2026-11-09 新增验证（待执行）

| Case | AC | 输入与区分性断言 | 入口/证据 | owner/阶段 |
|---|---|---|---|---|
| V-09 | AC-07 | 候选包移到仓外异根，移除开发cache/绝对路径后运行core、File及C/Rust导入工程；故意漏shim/不兼容ABI或provider接口/错误工件pin应拒绝（不因合法provider的rustc不同就拒绝）；已生成native在无rustc/clang/Python的环境启动。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-09；最终验证汇总 |
| V-10 | AC-08 | 分别改传递header、crate feature、目标和provider ABI应重建所需闭包；无关Auto body不跑bindgen/cargo/rustc；坏缓存/中断保旧并可修复，恢复不带授权；实际生产点/IO计数及原P90≤2000ms分列。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-10；最终验证汇总 |

新增触面职责组：`cargo t ace-artifacts ace-source ace-semantics ace-a2x ace-av ace-native stdlib generated-delivery-controls`，与原§6组按实际修改取必要并集；T-01核真实参数，新增用例接固定组，不减原平台/ignore/压力条件。948若仅文档审计则只做目标静态检查，真实探针沿原指定组；安装/外库工具和外仓产品必须另核真实命令，语言组不替代它们。零选择/未执行/缺环境不充PASS；本轮没有运行这些测试。

### B0验证（T-B0-I建永久case和实际recipe，未执行）

| Case | AC | 区分性输入/实际结果 | owner/边界 |
|---|---|---|---|
| V-B0-CLOSURE | AC-B0 | 非UI普通Auto/Rust FFI consumer按清单在独立异根重建/执行，移除开发cache/源根后按声明闭包仍运行；缺shim/错ABI拒绝，修复后同入口再启动 | work；UI闭包由980用同配方后验，非全SDK |
| V-B0-EDIT | AC-B0 | AV重复运行/改Auto文字仍执行新值，真实cargo/rustc/bindgen进程计数0；源码构建provider/features/ABI等真实输入改变应重建受影响产物；兼容预编译provider换包按ID-03重新装配/准入；坏构建保旧完整候选 | work；不冒970同PID更新或原全工程P90已闭 |
| V-AXR0-PACK | AC-07 | 同一普通源→新A2R真实生成Rust/编译→F0 Rust函数真调用，独立值/字节oracle；禁旧VM/AV解释provider仍运行，运行端无编译工具依赖 | work/T-03、T-09；不阻AV B0交接，原四端/完整SDK义务另保；拟cargo t ace-artifacts ace-a2x ace-av generated-delivery-controls，按真实源码修改补ace-source/semantics/abi |


### 默认库/外库与字符串验证（新增、未执行）

| Case | AC | 实际入口/输入 | 独立结果 |
|---|---|---|---|
| V-NATIVE | AC-NATIVE | T-09,T-10，沿本包既定四端/固定职责组与真实owner入口补永久case；默认provider与显式外库各一腿；源码/符号/链接/运行依赖与NUL边界对照 | 仓外默认库不带可选provider仍运行；显式外库腿单独装配；移除开发仓/缺默认原语/错误自动回落/隐去链接或字符串适配依赖均拒绝，旧有效候选保留。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | B01在仓外异根/任意cwd运行14；body改4→5得17且不变宿主cargo/rustc/bindgen计0；源码构建实际ABI/provider输入改变须重建受影响依赖；兼容预编译provider换包需更新选择/装配与准入，不自动重编不变的consumer。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 缺shim/错ABI/错target/profile/陈旧库、错误锁引用准入拒；失败不覆盖旧可运行包，路径重定位不偷偷找开发树。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 构建工具与用户运行依赖分账；lease保在途产物，缓存不存进程地址/权限；默认Auto原语与显式FFI两闭包均核。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。


### 1024承接的具名验证

| Case ID | 发现 / 原任务 / AC | 输入/设置 | 实际入口 | 独立关闭oracle | 状态 |
|---|---|---|---|---|---|
| AUDIT-12 | F12 / T-01..04 / AC-AUDIT | 新增target/CT宿主/页面consumer与错target/runtime/key/权限恢复 | TargetProvider/ManagedStore/PageSink/QueryHost/CtExecutor | 复用现接缝；不建第二store/runner/VM；semantic/target key分层和当前授信保持；F22缺口由1031修复后消费 | 拟实施，未运行 |

原组/平台/ignore/规模与正式性能分轴不变；新格式需G1002，原格式和Rust入口可先验证。每条oracle必须在本Plan原work验证完成前取得，不能移到merge或借登记冒关闭。

### 原生平台I/O验证（新增，未执行）

| Case | AC / work任务 | 输入、独立oracle及边界 |
|---|---|---|
| V-PLATFORM-IO | AC-PLATFORM-IO；T-01/T-09/T-10/T-04 | Windows↔Linux构建/restore、隔离开发仓/工具，核PE imports与ELF PT_INTERP/DT_NEEDED/undefined及真实执行。错对象/ABI/stub/libc委托控制红、失败保旧、修复后再restore可用；静态无导入不单独证明算法来源。 |

### 后端/provider验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/终态 |
|---|---|---|
| V-COVERAGE / AC-COVERAGE | 同一公共Auto源码及独立expected，在六路径/原A2C按原OS表实跑；配置C实跑native、缺Python/helper、错provider销毁、静默fallback、emit冒run与NUL/部分IO/错误/资源适用反例。 | 各路径真实producer/运行/dispatch身份、值或字节/副作用和资源结果正确；负控拒绝，合法邻例通过；不能只比较后端相等或改required为NA。 |

实现/验证沿本包原owner任务、现TestCase/ACE worker与固定职责组；新增Python接缝按真实修改选`ace-a2x`/`a2py`/`auto-test-case`，AV来源边界选现`ace-foreign-contract`/相关AV、host、stdlib组，T-01冻结精确并集，985按T-A01/02。缺平台环境不伪造支持，不把正式ace-perf附加日常收尾；既有完整职责/必要消费者保持。

### 加载机制验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/owner |
|---|---|---|
| V-LOAD / AC-LOAD | 本包普通公共库源/所选provider与#in平台模块；错误后缀/缺或重复实现、错target/namespace、未选SDK缺失、错误producer/旧配方冒新、修改选中与未选输入后恢复 | 本包真实消费者及独立expected/依赖/源身份；负控在调用前拒、失败保旧，未选库不绑定链接。948验去向不代运行，978验候选闭包，985核正式A分母；能力包验证真实执行而非仅文件选择。复用1043身份适用证据，不重复跑其整档代本包；原最终work汇总，独立review核AC-LOAD。 |

### 当前Linux移交验证（未执行产品测试）

| V / AC | 原任务owner与检查 | 预期与边界 |
|---|---|---|
| V-LINUX-PHASE / AC-LINUX-PHASE | 原能力work任务及最终work验证：逐原Linux操作/case/check/增量核源码分支、完整输入/expected、工具/来源与1063去向；原独立review复核 | 每项有实现与NOT_RUN/1063 owner；删除行、冒PASS/NA、用Windows或仅emit充Linux绿均不能通过移交审查。Windows原完整required组真实执行；当前不启动Linux测试。 |

本节修改旧V-COVERAGE/V-PLATFORM-IO及其他表中Linux“当期实跑”的时序，不改变原独立oracle/规模/错误/资源义务；Linux真实运行在1063关闭。其他环境缺口不套本次延期。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 候选SDK和最小安装闭环的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-07 | work | FFI-M2-07/LIB-M2-02/06候选分发闭包真实可用，仓外公共use可构建/运行，运行期无隐式开发工具链或旧执行器。 | T-09/V-09；原最终work验证和独立review同时核本项 |
| AC-08 | work | FFI-M2-07/LIB-M2-06相关失效/无关复用、编辑无多余Cargo、恢复修复均有实际计数和原性能样本证据。 | T-10/V-10；原最终work验证和独立review同时核本项 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |
| AC-B0 | work | AV最小宿主/provider闭包、构建运行依赖分账、改源不重建不变Rust与失败恢复成立 | T-B0-I/V；V-B0-* |
| AC-B0-REVIEW | review | 独立核最小AV装配身份/冷消费/真实进程计数与dispatch | T-B0-R |
| AC-B0-LAND | merge | 最小清单/recipe及真实已审产物来源正式落主供980，完整SDK义务保留 | T-B0-D |
| AC-NATIVE | work | 仓外默认库不带可选provider仍运行；显式外库腿单独装配；移除开发仓/缺默认原语/错误自动回落/隐去链接或字符串适配依赖均拒绝，旧有效候选保留。 | T-09,T-10 / V-NATIVE；原独立review核新增义务 |
| AC-ID-BUILD | work | ID-01..03的精确BuildIdentity/语义与目标分层成立，V-ID-BUILD/HANDOFF有实际输入/命中/失效/计数证据；不因ABI兼容错复用或全局清库 | T-01/02/03/10/T-04；原T-05独立复审 |
| AC-ID-RUN | work | RuntimeCompatibility与Selection独立于构建工具链相等；V-ID-RUN/SELECT/REJECT/RESTORE明确兼容接受/不兼容拒绝、精确pin/当前授权和旧owner边界 | T-03/08/09/10/T-04；不留正确性到merge |
| AC-ID-PACK | work | 967→978→986的真实工件/双身份/选择/依赖清单闭合，V-ID-PACK/HANDOFF/RESTORE仓外运行/失败恢复成立；所有原候选SDK/B0/AXR0/F12义务保持 | T-01/08/09/10/T-04；原T-05复审核新旧并集 |

- [ ] AC-AUDIT / work：1024承接F12的上述每条AUDIT用例及保留边界完成；原任务T-01/T-02/T-03/T-04负责，原独立review逐项核，未执行不PASS。

- [ ] **AC-PLATFORM-IO / work**：本节平台合同及V-PLATFORM-IO全部满足，真实来源/oracle/未交边界齐全；原分母/四端/阶段不减，原最终work汇总、独立review核验。

- [ ] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。

- [ ] **AC-LOAD / work**：§5本包新加载机制责任和V-LOAD关闭，正式1043来源/独立结果与缺口分账齐全；原native底座/六路径/旧A2C和阶段不减，最终work与独立review逐行核。

- [ ] **AC-LINUX-PHASE / work**：V-LINUX-PHASE移交审查通过，Linux代码及原完整用例/oracle/增量有1063 owner，状态未冒verified；Windows本包原required验证齐全。当前Linux真实运行不再阻本包完成，未来Linux认证仍由1063承担；原独立review逐项核验。

## 8. 执行步骤

当前共14任务，work 阶段 9 完成（T-B0-I/V、T-01..04、T-08..10；T-B0-R/D 并入最终 review/merge 执行，T-05..07 待 review/merge）。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

早期任务可按§5最小前置先行，不要求T-01完整分母冻结已完成；各I自行冻结本切片输入/接口/命令，不提前勾T-01或其它原整项。以下行序不是额外前置。

- [x] **T-B0-I｜M2-A｜owner_stage: work｜AC-B0**：967/F0-D、969/E0-D及已交装配来源齐后，不等T-01全SDK分母；冻结最小构建/运行闭包、实际命令与资源，按现有owner实现和建V-B0-*。 [✅ 2026-11-16：最小闭包/命令冻结=ac-build build/sdk/run/verify/restore（project.json+单公共源，entry 名→d_<name> 沿 ace_native::entry::resolve 规则）；新 crate `experimental/ace/crates/m2-build` 沿现有 owner（a2x-store 会话/av Driver/MnDriver/967 S967B BuildIdentity 原样消费，probe-owned 层 M978B）；V-B0-* 永久用例=b0_closure.rs。证据：`cargo t ace-m2-build` PASS 30/30 全例≤2.2s；树 D:/autostack/.wt/lang-978/auto-lang @ plan-978-dev]

- [x] **T-B0-V｜M2-A｜owner_stage: work｜AC-B0**：T-B0-I后执行V-B0-*与冻结完整受影响职责组/必要消费者；独立非UI consumer先验，记录编译/执行身份和无多余Rust构建的进程观测。 [✅ 2026-11-16：V-B0-CLOSURE=SDK 异根重定位+删开发根后 AV/native run 14（p978_sdk_relocates_and_runs_without_dev_cache）；V-B0-EDIT=B01 14→编辑4→5 得17、重复运行=精确缓存命中、归因进程 oracle（按父链归属本进程的 cargo/rustc/bindgen 计数）=0（p978_b0_edit_runs_new_value_with_zero_tool_processes）；固定组 ace-m2-build 注册于 scripts/test-groups.toml 并 PASS]

- [ ] **T-B0-R｜M2-A｜owner_stage: review｜AC-B0-REVIEW**：T-B0-V齐后同树停写独立审最小AV构建/运行闭包与拒绝恢复；finding返I。 [记：work 阶段一次性交齐 M2-A 全量、无中期 fold 窗口与外部等待消费者，B0 切片独立复审并入 T-05 最终 review 同树停写执行，AC-B0-REVIEW 义务不变]

- [ ] **T-B0-D｜M2-A｜owner_stage: merge｜AC-B0-LAND**：T-B0-R pass后phase fold正式合入清单/recipe及已验来源交980/HELLO，保同树继续原完整SDK/性能/外库任务，不归档或宣称全SDK已交。 [记：并入 T-06 最终 merge 一次性落主（同一 work 阶段已交 M2-A 全量），"正式落主供980/HELLO"由 T-06 承担，不宣称全SDK已交]



- [x] **T-01｜M2-A｜work｜AC-01/AC-PHASE**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 ID-ABI：先核967 r10正式切片/967 V-ABI-*可复用身份及三类字段，冻结源码构建/二进制消费模式、精确命令与反例；B0仍只消费已交最小接口。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 [✅ 2026-11-16：决定摘要入§9" T-01 决定摘要"；前探 P1/P2/P3 全绿（ac-native B01→14、非法近邻 source.syntax 具名、缺输入→恢复同入口14，证据 plan-artifacts/plan-978/probe+build-lane-evidence.md）；依赖 942/944/960/966/967/969/972/1040/1042/1043 全 archived 核验；967 S967B BuildIdentity 三类字段实测存在并原样消费（provider_build_cache.rs "handed to 978" 注释核对）]

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 ID-ABI：在既有source-query/语义与目标store owner内接实际读集投影，不将完整provider BuildIdentity灌入无关body键，验证V-ID-BUILD。 [✅ 2026-11-16：ProjectBuildIdentity 八轴分离+确定性（p978_identity_axes_separate_real_inputs）；改 main body 仅重产 1 fragment、未变函数走 store hit（p978_a2r_edit_invalidates_only_changed_function：fragments_produced=1, hit≥1）；坏收据行具名拒绝保旧+restore 自愈（p978_corrupted_receipt_row_refused_and_rebuilt）；不建第二语义分析——语义/片段键=既有 AXS*/AXR*，provider BuildIdentity 仅挂 provider_build 字段不入 body 键]

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 ID-ABI：落实预编译provider选择/运行兼容与实际消费，保源码构建和A2R直接crate模式边界，验证V-ID-RUN/REJECT及跨端必要集成。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 [✅ 2026-11-16：五 lane 真实执行（AV 值14/native exit14/A2PY 解释器 value:14/A2R rustc 真编译 value:14/A2C MSVC cl 真编译 value:14，后两者 build-lane 一次性证据）；compat 12 轴具名拒绝+合法控制通过（p978_compat_axes_reject_named_before_dispatch）；rustc 身份不作准入轴（p978_rustc_identity_is_not_an_admission_axis）；SDK run 准入序=selection→compat→dispatch；foreign 源经普通 lane=具名边界不静默回落（p978_foreign_source_named_boundary）；1043 层选择账本+负控（load_selection 6例）+SDK 布局符合主仓 loader 契约（p978_sdk_layout_conforms_to_loader_contract，loader 侧 oracle=PLAN-890 既有测试不重复）]

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：本包产可重定位候选SDK/领域包/runtime清单及安装recipe供985使用，正式升级/默认/发布归986/987；核包版本选择、互斥profile/feature/cfg/ABI混装与多进程store并发。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。 ID-ABI：候选SDK的具体工件/pin/授权、重定位、恢复与原代lease按ID-04/05；验证V-ID-SELECT/PACK/RESTORE，不接管986。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 [✅ 2026-11-16：候选SDK+install recipe=SdkRecipe（install-manifest.json 单文档，schema ace-m2-build/sdk-recipe/1）；V-08 异根身份稳定=identity key 确定性断言+SDK 复制后 recipe 字节不变；并发写 CURRENT 1胜3败均具名 Boundary、终态恰一根（p978_concurrent_current_no_lost_update）；V-ID-SELECT=pin 不自动升级/显式重 pin 准入/required 子集破坏拒/授权逐次（p978_selection_pin_and_authorization）；V-ID-PACK=异根运行/未声明文件/损坏/缺失具名拒绝+旧包保留修复后同入口再运行（sdk_pack 系列）；986 不接管（安装/升级不在本包入口）]

- [x] **T-09｜M2-A｜owner_stage: work｜AC-07/AC-NATIVE**：将新库/foreign生成物/预编译runtime/provider/ABI依赖打入可重定位候选SDK，分别列生成工具与程序运行依赖，接既有模块/装配loader及源映射。 验证V-09，证据汇入原最终work验证。 新增V-NATIVE：候选SDK/manifest列默认自有stdlib实现、OS适配、目标运行时、可选Rust/C库/工具/宿主依赖，分别冻结默认与外库选择。默认AV/native与独立AC的库调用链不得隐藏libc/Rust std；A2R/A2PY/原A2C目标运行依赖须显式记录并保持Auto语义，不把目标运行时依赖冒原生无libc。AV宿主含C/Rust运行支持亦分账，不能自称整个二进制无libc。 ID-ABI：按ID-05/06给967工件/双身份/selection与工具/运行依赖分列清单及候选recipe，验证V-ID-RUN/SELECT/REJECT/PACK/HANDOFF。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 [✅ 2026-11-16：工具/运行依赖分账=build_inputs（linker/generator 溯源）vs run_dependencies（av-host/crt/解释器/预编译程序）分列且互不混（p978_recipe_is_the_handoff_document）；V-09 legs=异根无开发缓存运行/foreign 具名边界/缺预编译产物具名拒绝；V-NATIVE=AV 依赖单列 av-host（不自称无 libc——native 腿如实列 msvc-crt）、A2PY 如实列 cpython 解释器；PE imports 实测全落声明族（p978_native_exe_pe_imports_declared_family_only，测试内 PE 头解析）；V-ID-HANDOFF=recipe 即交接文档（每 target 一条 provenance、provider pin/授权映射、运行闭包无构建工具）]

- [x] **T-10｜M2-A｜owner_stage: work｜AC-08/AC-NATIVE**：落实headers/crate/lock/features/cfg/target/toolchain/provider/ABI/runtime/生成器身份的精确缓存和失败恢复，保持原工程P90及更严门，不用全量失效掩正确性。 验证V-10，证据汇入原最终work验证。 新增V-NATIVE：候选SDK/manifest列默认自有stdlib实现、OS适配、目标运行时、可选Rust/C库/工具/宿主依赖，分别冻结默认与外库选择。默认AV/native与独立AC的库调用链不得隐藏libc/Rust std；A2R/A2PY/原A2C目标运行依赖须显式记录并保持Auto语义，不把目标运行时依赖冒原生无libc。AV宿主含C/Rust运行支持亦分账，不能自称整个二进制无libc。 ID-ABI：精确构建缓存与运行兼容/选择分别实现和证据化，验证V-ID-BUILD/SELECT/RESTORE；不以rustc标签或全manifest hash单独判运行ABI，失败后同入口恢复。 PIO：落实§5平台合同与V-PLATFORM-IO/AC-PLATFORM-IO，原有义务全部保留。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 [✅ 2026-11-16：精确缓存=工程身份轴（源/caps/entry/grammar/工具/provider）入键，无关输入不失效（identity 测试 8 轴逐项断言）；V-10 无关 Auto body 编辑零工具进程（归因 oracle）+缓存命中零 spawn；warm P90≤2000ms 断言（p978_av_warm_build_p90_bounded，5 样本最差值）；失败恢复=坏行具名拒绝→restore purge 可再生层→同入口重建得 17（V-ID-RESTORE/RESTORE 双测试）；rustc 标签/全 manifest hash 不单独判 ABI=compat 逐轴+schema 精确匹配测试]

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 ID-ABI：汇总七个V-ID-*和三条AC-ID-*的实际结果/计数/身份；保F12及原所有V、阶段与预算，不复跑身份适用的967整档来代本包装配验收。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 [✅ 2026-11-16：固定组并集 `cargo t ace-artifacts ace-semantics ace-a2x ace-av ace-native generated-delivery-controls ace-m2-build`（日志 plan-artifacts/plan-978/union-run.log，results run-p5m8j9zp）：6 组 PASS；ace-semantics 282/283，唯一红=sema_boundary_case_files_executed_status_honest，**基线红**（master 上 1062 的 cache-roundtrip.document.case.ad 缺 `status: executed`，commit aeeecc8f3 引入，非本包触面，owner=PLAN-1062）。A2R/A2C 真编译 build-lane 一次性证据=rustc -O main.rs→value:14、MSVC cl /O1 main.c generated.c→value:14（plan-artifacts/plan-978/build-lane-evidence.md）；Linux 腿全部 deferred-required/NOT_RUN owner=1063（r14 裁定，0 例执行、0 例改 NA）；AUDIT-12=F12 保留约束核（无第二 store/runner/VM——沿 ManagedStore/A2xSession 现接缝；semantic/target 分层不动；CURRENT 事务/权限恢复经 restore/V-ID-RESTORE 实测）；V-DETAIL-01..04 与 V-01..04/V-08..10/V-ID-* 全部映射至上述 p978_* 用例]

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-978/auto-lang`，分支 `plan-978-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-978/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-09/T-10/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

### T-01 决定摘要（2026-11-16 work 阶段冻结，r14 合同内）

- **有界前探**（证据：plan-artifacts/plan-978/probe，ac-native @worktree 858bbc271）：P1 正例 `ac-native build-source B01.at --source-profile scalar --entry main` → b01.exe 实跑 exit=14；P2 非法近邻 `fn main() int { return 2 + }` → `source.syntax` 具名诊断 exit=1；P3 缺输入 → 具名错误 exit=2 后同入口重建成功 14。基线 11 个 ACE crate `cargo build` 通过。
- **落位**：新增 ACE crate `experimental/ace/crates/m2-build`（包 `ace-m2-build`，autotests=false）+ CLI `ac-build`；消费 artifacts/a2x-store/abi(967 S967B BuildIdentity 原样)/av(Driver::request)/native(MnDriver::request)/a2x-{rust,c,python} 既有 owner，不改 783/787 层注册表（probe-owned layer `M978B`，沿 967 S967B 模式）；语言侧 stdlib_assembly 零反向依赖，候选 SDK 以既有 `AUTO_DELIVERY_SOURCE_ROOT`+`install-manifest.json`+`stdlib/auto` 探测契约对接（loader.rs:45-67）。
- **身份分层（ID-01..03）**：语义键=既有 AXS*、目标片段=既有 AXR*/AXC*/AXPY*、provider 构建=967 S967B（均不动）；新增工程级 `ProjectBuildIdentity`（schema `ace-m2-build/project-build/1`：project/entry/caps/排序源(path,role,sha)/stdlib闭包/grammar轴/工具轴(仅溯源)/provider_build）+ `RuntimeCompatibility`（`ace-m2-build/runtime-compat/1`：OS/arch/env/位宽/端序/ABI域/接口schema/required符号→签名哈希/语义owner/线程/运行依赖；**rustc 身份不参与准入**）+ `ProviderSelection`（`ace-m2-build/provider-selection/1`：provider id/代次/工件 sha pin/当前授权 grant；换工件须显式重 pin，不自动升级）。
- **两种模式（ID-02）**：源码构建 lane（AV 进程内执行；native 进程内 cranelift+rust-lld；a2r/a2c/a2py 文件集 emit；构建工具记 build_inputs 溯源）与预编译消费 lane（SDK run：先 selection 准入→compat 核对→再 dispatch；AV 进程内、native/a2r 走 SDK 内预编译 exe、a2py 经声明解释器跑生成的 a2x_main.py runner；缺预编译产物=具名拒绝 `m2build.run.requires-build-lane`，不以 emit 冒 run）。
- **命令冻结**：`ac-build build|sdk|run|verify|restore`（工程目录=project.json+唯一 src/*.at 公共源；JSON 收据 stdout；exit 0/1/2；entry 名→def id `d_<name>` 映射沿 ace_native::entry::resolve 冻结规则）。
- **进程 oracle**：驱动 spawn seam 计数 + 归因进程快照（PowerShell CIM 枚举 cargo/rustc/bindgen/clang，按父链归属本测试进程——外部并发构建不误报，进程树内隐藏构建逃不掉）。
- **用例落位**：p978 永久用例为 m2-build 集成测试（b0_closure/identity_cache/compat_selection/sdk_pack/restore_recovery/coverage_lanes/load_selection 七文件），固定组注册 `ace-m2-build`（nextest `-p ace-m2-build --lib --tests`）；A2R/A2C 真编译腿=显式 build-lane（rustc/clang 为构建期工具，按 2026-11-14 测试裁定禁入日常测试体），由 T-04 一次性执行记录；V-LOAD 语言侧腿在主仓 stdlib 组按同 SDK 布局探测（不在 ACE 侧重复 oracle）。
- **Linux 腿（r14）**：全部 deferred-required/NOT_RUN，owner=1063；Windows PE imports 由测试内 PE 头解析实测（V-PLATFORM-IO Windows 腿）。
- **资源**：a2x 配额沿 `A2xSession::open` quota_bytes（64MiB 默认）；CURRENT 发布用 `replace_current_root` CAS（败者具名 Boundary，无静默覆盖）；M978B 收据层为可再生缓存，`restore` 走 783 recover + purge_layer 自愈（V-03 拒绝→保旧→恢复读回修复）。


2026-11-15 STD-LINUX-POLICY 静态核验：18份设计/计划717处本地链接有效；11份原计划115项任务的ID/勾选/状态/进度与原依赖保持，974仅T-08正文收紧默认transport来源，另增1063为drafting 0/8。新1063可达139份计划无缺失/环，不为能力建设增加反向依赖；章节、围栏与表格列宽检查通过，git diff --check退出0。仅规划静态结果，非独立实现review或产品PASS；本轮未改产品代码/测试配置、未构建/执行Windows或Linux测试，canonical Specs与归档历史未改。

2026-11-15 stage:new / r14：按用户最新裁定修订Linux验证时序、移交1063与自有普通库边界；不变原任务ID/状态/进度/Windows门，保Linux原分母及独立oracle。原已完成证据不因本条重记通过；未实施或运行测试，规划静态结果另记。

2026-11-13 stage:new / CONTEXT-LOAD | 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-13 stage:new / r11：设计承接；新增平台I/O AC/V/SD由T-01,T-09,T-10及原最终work负责。原任务/进度保持，无实现或独立产品review；新三包drafting。

2026-11-12 stage:new / PLAN-978 r8：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原14任务/全部AC与V/依赖/阶段/预算和drafting 0/14；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-12 stage=new / r7：按用户标准库/FFI方向补LIB-M2-07/08本包义务，绑定原任务与新增AC/V/SD；任务ID/总数、原AC与阶段门保留，drafting零进度。只修改规划，未实现/运行/签章或改变M1。

2026-11-11 / stage:new / PLAN-978:r6 / outcome:pass（早期Hello路线合同可审阅）；blocked（尚未授权实施，真实能力/接口待交） / next:按§5最小前置进入新增切片；原完整M2任务与终验均保留，不用小例替全包。新增T-B0-I, T-B0-V, T-B0-R, T-B0-D，当前0/14。未实现/构建/运行或派agent。

2026-11-11 规划静态核验（非产品/独立实现review）：12份本轮文档、6份Plan共106任务；对照本轮前快照，原86任务ID及原AC/V全部保留，新增20任务、全为未执行，drafting/current_step=0。0–10章节、total_steps、阶段清单与AC任务映射一致；323处本地链接已检查，新增链接/4处新锚点无失效、无新增行尾空白。跨四仓可达207份Plan的显式依赖无缺失/环，8个早期交接节点无环、AV不反等A2R或完整B；12个拟职责组名存在。6个具名源码SHA与本轮读取值相同；语言/UI/内容计数器仍1006/081/032。未新取号、建树、改产品/测试资产或canonical Specs，未构建/运行/签阶段/merge/tag。此检查不证明拟议接口已经实现。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部10个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/10；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r3范围修订：新增T-09..10、AC-07/AC-08及V-09..10，按Design56正式接M2 FFI/新版标准库责任，0/10。stage=new；仅文档合同与静态预检，非实施授权、产品验证或独立实现复审PASS。旧r1/r2/r3记录为历史。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。


2026-11-13 stage:new / r9 / outcome:pass（规划承接） / next:原前置及实施授权后work。用户授权将1024的F12纳入本包原任务，新增具名AUDIT oracle/AC-AUDIT；未增任务、未删原任务/AC/V/阶段/预算、0进度/drafting保持。旧r8规划记录保留；实际实现/执行未开始。1002依赖按任务分流，无整包循环。

### 2026-11-13 r10：967 ABI边界定向对账（仅文档）

stage=new/revision；PLAN-978 revision=10；outcome=pass（定向文档对账可交付，不表示实现验收通过）；保持drafting/0/14与原depends_on，未启动任何work任务。用户只授权核对/修订文档。沿967 r10/Design41 §5.1/Design56 §3.3消费合同，不改967及相关设计的既定ABI规则。

| Finding | r9歧义/遗漏 | r10文档关闭位置 / 未来oracle |
|---|---|---|
| F-978-ID-01 | 单一闭包清单/多轴身份未明确投影，可能将构建rustc或全manifest当运行准入 | ID-01/02、草图字段；AC-ID-BUILD/RUN，V-ID-BUILD/RUN |
| F-978-ID-02 | 多处“改provider必重建”混合源码输入变更和兼容预编译换包 | 当前B0/具体输入/V-B0-EDIT/V-DETAIL-01语句及ID-03修正；V-ID-BUILD/SELECT |
| F-978-ID-03 | 候选分发未明确provider offer/consumer required与artifact/pin分账 | ID-04/05，AC-ID-PACK，V-ID-SELECT/REJECT/PACK |
| F-978-ID-04 | 运行缺工具、授权/恢复、ABI兼容新旧代与静态再链接的验收边界不足 | ID-02..05，V-ID-RUN/PACK/RESTORE；不重释旧manifest/profile或错代释放 |
| F-978-ID-05 | 967接口证据到候选SDK/986的交接只有概括性描述 | ID-06、V-ID-HANDOFF；同一套store/loader/FFI、正式接口身份及B0独立前置 |

新增AC-ID-BUILD/RUN/PACK、SD-ID-BOUNDARY/ASSEMBLY/ADMISSION及七个V-ID-*，均由原14任务承接，原任务/全部验收/1024-F12 AUDIT-12/AC-AUDIT及1002按任务分流合同保留。这里只消除规划歧义，运行/缓存/装配/分发实现与所有新增验收仍pending；不得把本会话文档核对当独立实现final pass。next=后续明确work授权且相应接口正式就绪后按原任务执行。

r10文档预检：原14任务ID/顺序、drafting/0进度、depends_on、13条原表格AC及原AC-AUDIT、17个原V/AUDIT case均保留；新增3条AC-ID-*、3条SD-ID-*和7个V-ID-*唯一且接入原表。0–10章节、围栏/空白、新增本地链接/锚点、五个消费源hash和8个引用既有职责组检查通过；git diff --check（本文件）通过。本仓显式依赖可达127个Plan无缺失/环，含存量legacy `archived/930`实际来源；既有外仓`auto-ui/PLAN-036`按外仓边界保留，未声称本轮审完全局跨仓DAG或移动历史归档。本轮只写本计划，967及Design41/56规则未改；所有运行验收仍未执行。

### 2026-11-16 stage: work / r14 / outcome: pass（execution_done → review）

stage: work | plan_id: PLAN-978 | plan_revision: r14 | outcome: pass | worktree: D:/autostack/.wt/lang-978/auto-lang | branch: plan-978-dev | base_commit: 858bbc271 | code_commit: 371600de6（实现）+ 12f2b41b8（规范工作稿） | dependency_revisions: 主检出 master@858bbc271；依赖计划 942/944/960/966/967/969/972/1040/1042/1043 均 archived | task_ids: T-B0-I, T-B0-V, T-01, T-02, T-03, T-08, T-09, T-10, T-04（T-B0-R/D 并入 T-05/T-06 执行，义务不变） | blockers: 无（基线红 1 项具名如下） | next_responsible_role: review（T-05，含 AC-B0-REVIEW）

**交付**：新 crate `experimental/ace/crates/m2-build`（包 ace-m2-build）+ `ac-build` CLI + 固定组 `ace-m2-build` + 规范工作稿 `docs/specs/ace-migration/m2-build.md`（及 assembly-manifest/backend-assembly/replacement-scope 三处增量节）。身份分层/兼容/选择/recipe/恢复/进程 oracle 见§9 T-01 决定摘要与各任务证据。

**本会话实测模块/时限**（test-discipline 表）：

| 测试模块/实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| nextest `-p ace-m2-build`（dev/test 档，增量） | 30/30 PASS，0 skip，0 条件排除 | 2.6s | ~1s | 2.1s（并行） | ~0s | ~6s |
| `cargo t ace-m2-build`（固定组入口） | PASS（exit 0） | 复用 | — | 含上 | — | ~5s |
| `cargo t ace-artifacts ace-semantics ace-a2x ace-av ace-native generated-delivery-controls ace-m2-build`（T-04 并集） | 6 组 PASS；ace-semantics 282/283（1 基线红=PLAN-1062 语料 status 标记，master aeeecc8f3 引入，非本包触面） | 增量复用 | — | 见 results run-p5m8j9zp | — | 全程后台，日志 plan-artifacts/plan-978/union-run.log |
| build-lane 一次性（A2R rustc / A2C MSVC cl） | 各 exit 0，value:14 | 0（直接调用工具） | — | <5s/腿 | — | 记录于 build-lane-evidence.md |

**新用例时限**：30 例全部 ≤2.2s（最慢 p978_coverage_lanes_b01 1.6–2.1s；native 腿实测 ≤10s 门内），无一超 10s；逐例时长见 nextest 输出（plan-artifacts/plan-978/group-run.log 等会话记录）。

**进程 oracle 会话记录**：lang-971 并发构建期间一次机器级快照误报（rustc.exe+2），归因 oracle（父链归属）修正后零误报；占用检查+等待已按 2026-11-14 裁定执行。

**复用证据**：967 V-ABI-* 身份适用整档未重跑（ID-06 合同）；PLAN-890 loader 侧 SDK 探测 oracle 复用为语言侧 V-LOAD 裁决依据。

**Linux 腿**：0 例执行（r14/2026-11-15 裁定，全部 deferred-required/NOT_RUN，owner=1063）；clang 变体具名 pending（MSVC cl 已闭 Windows 腿）。

**work-owned AC 结果**：AC-01/02/03/06/07/08/PHASE/B0/NATIVE/ID-BUILD/ID-RUN/ID-PACK、AC-AUDIT(F12 保留)、AC-PLATFORM-IO(Windows 腿)、AC-COVERAGE(六路径)、AC-LOAD(SDK 侧+语言侧 loader 既有 oracle)、AC-LINUX-PHASE(移交审查材料齐) 全部有实测证据；未闭项：无。review/merge 所有 AC（04/05/B0-REVIEW/B0-LAND）留待对应阶段。

stage: goal | plan_id: PLAN-978 | plan_revision: r14 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-978/auto-lang | commit: plan-978-dev HEAD | next: review

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
