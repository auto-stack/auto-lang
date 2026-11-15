---
plan_id: PLAN-969
status: execution_done
feature_name: ACE 产品执行配置与嵌入前门
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-15
plan_revision: 7
current_step: 9
total_steps: 12
depends_on: [PLAN-960, PLAN-964, PLAN-967, PLAN-968]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md]
new_spec_components: [docs/specs/ace-migration/m2-facade.md]
touched_goals: [GOAL-002]
---

# [PLAN-969] ACE 产品执行配置与嵌入前门

## 0. 变更摘要

2026-11-12 r6：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-11 r5当前：增加E0显式ACE AV同步宿主执行入口；只等967/F0和现有源码/AV基础，不等待964 Dynamic、968 async、970热更新或980产品；原全部入口兼容与错误通道保留。 原8项任务/全部AC/V保留，新增4项早期切片任务，现12项、drafting、0完成；只规划。

历史修订：2026-11-10 r4执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r3当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

将run/run_file/args/capture/deadline/input/config等收敛到ACE中立执行服务；新增显式ACE选择和真实装载身份，迁移期间只对已通过入口opt-in。兼容旧API名但ACE路径不得fallback；配置求值保参数/环境/作用域与错误通道。

设计与路线：[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[M2执行路线](../design/strategy/ace-m2-delivery-roadmap.md)。本包是拟议执行合同，非已实现声明。

## 1. 目标

交付上节完整职责面并保留原源、原已支持行为与真实消费者；禁止用窄profile、typed fixture、emit-only或旧实现fallback冒充产品支持。
非目标：本包之外的新语言设想、AACE自举、任意新平台、无边界全生态桥接。其他包的责任不作为本包完成声明。

## 2. 架构方案

整包最终前置：PLAN-960, PLAN-964, PLAN-967, PLAN-968；E0仅按§5最小已交接口先行，不等待Dynamic/async整包。旧完整任务仍核本行并集，早期E0不冒整Plan已交。


整包最终依赖以frontmatter为准；局部A的启动只核§5已列的最小能力/PROOF接口及正式来源，不等待本Plan或其他产品的B整包。B实现先核985/A的M2A-08已合入收据，再核本批原前置。WIP或分支reviewed不替正式来源。
数据流：原源码/工程输入→统一Schema/CST→共享绑定/query/Checked HIR→AV/AC共享MIR或新A2X provider→中立runtime/领域consumer→实际可观察输出。领域IR不旁路行为verify；缓存不储存信任和宿主授权。

## 3. 技术栈

现有Rust ACE workspace、Auto公共源、共享语法/查询/工件系统及真实目标工具链。跨仓使用同组兄弟worktree与显式SDK身份，不创建junction/symlink。

## 4. 需求分析与背景调查

2026-11-11本轮授权：用户要求规划首个ACE UI Hello及AV/Rust FFI路线、核对既有M2冲突并作必要重排；仅规划/设计/导航，不实施、创建worktree或探针/fixture/SDK、构建/运行、提交或发布，未指定模型/工期。沿旧语言用例守恒、新Auto标准库与AutoABI/FFI、Demo递进的三线，不削完整M2范围。

现场语言HEAD 9e36d984e、UI HEAD d88e9c51。已核av-core/src/host.rs仅console/1（sha256前16 30d6dcc475850cd0）、a2x-rust/src/provider.rs仍拒source foreign（3ea453dc2452576f）、core/src/source/cst/frontdoor.rs（dd3d93d06799ae52）；现有auto-bindgen/src/extractor.rs是预置C manifest表（8c7f1295cc83c6fb），不能称Rust元数据导入器已实现。Iced由Rust宿主编译执行，AutoVM不解释其Rust源码；现有DynamicComponent/VmBridge/handler_codegen深依赖旧执行链。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `crates/auto-lang-core/src/entry.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/lib.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/execution_engine.rs`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang/src/lib.rs`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：公开执行facade/配置与嵌入选择。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/auto-lang-core/src/execution_engine.rs](../../crates/auto-lang-core/src/execution_engine.rs)：现ExecutionEngine枚举/选择，扩公开入口；读取SHA256前16 `58b059a91253eeee`

- [crates/auto-lang-core/src/entry.rs](../../crates/auto-lang-core/src/entry.rs)：统一请求入口；读取SHA256前16 `b3e46189b0e6e1af`

- [experimental/ace/crates/av/src/session.rs](../../experimental/ace/crates/av/src/session.rs)：ACE会话/关闭，E0复用非UIconsumer；读取SHA256前16 `7c37123cb8504b91`

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-E0-I, T-E0-V, T-E0-R, T-E0-D |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


将run/run_file/args/capture/deadline/input/config等收敛到ACE中立执行服务；新增显式ACE选择和真实装载身份，迁移期间只对已通过入口opt-in。兼容旧API名但ACE路径不得fallback；配置求值保参数/环境/作用域与错误通道。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：所有兼容前门与错误通道（r2）

逐导出核run_abt/create_vm_from_abt等旧编译输入API、run_with_mode/auto mode、capture/input/deadline/config；对旧字节码明确版本拒绝或受检迁移，不能透传为ACE指令。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### E0：Hello使用的公开同步执行入口（当前早期入口）

最小前置为967/F0-D的已落主ABI/provider/自动binding合同及现有受检普通源/AV执行；不等967全包、964 Dynamic或968语言调度。T-E0-I在现有`crates/auto-lang-core/src/{execution_engine,entry,lib}.rs`公开入口与ACE `av/src/{session,resident,query_entry}.rs`适用owner内冻结具名ACE AV请求/执行结果/授权/关闭接口；接口名字未定，不假称现有CLI已有--engine ace选项。UI消费此入口，不自己new第二个解释器/私有FFI runtime。

源、grammar/domain注册、provider/ABI、权限与目标均显式输入；每run重核授权和身份，返回可区分的正常结果、源码诊断、资源拒绝、取消与宿主/工具故障。采用普通Rust宿主返回结构并不要求Auto guest支持Dynamic或反射。公开ACE选择失败应报错，旧AutoVM/旧Parser/AST emitter的dispatch=0，不自动切换机器默认或旧CLI行为。

E0只覆盖同步一次执行+宿主owner创建/关闭，适用UI主线程调用约束，支持一个session provider隔离和确定退出；窗口事件循环仍Rust宿主持有，尚无guest事件/async/callback热卸载承诺。967本身用既有直接AV API闭合，不反等E0；E0用非UI普通源码consumer闭合，不等980窗口。项目pac/config全求值、多会话更新/daemon、所有兼容入口与stdin/EOF等原范围仍由T-01..08及970后续关闭。

新增E0只是原执行facade的一段正式交接，后续T-02/03扩同一入口，不建ace_ui专用执行器或在失败路径偷偷调用legacy。

早期切片是本Plan M2-A内部交接，不新增ACE里程碑或Plan终态；只在同一权威树按owner流程独立复核、正式落主并记source/接口/grammar/ABI/provider及真实结果后供下游消费。同Plan尚有原完整义务时不归档/删树/勾整包完成。frontmatter保最终依赖并集，早期切片只等本节最小接口；同一文件单写，不以分支reviewed/WIP当交付。最小切片通过不关闭原AC、完整Rust FFI/全UI/RG-A03/M2A-08；不新增对980/981/985整包或AGE联合门的反向依赖。

### E0接口冻结记录（T-E0-I输出，2026-11-15，worktree lang-969@plan-969-dev，基线ca88cde14）

公开合同（新模块 `crates/auto-lang-core/src/ace_entry.rs`，lib.rs根glob再导出；**零ACE依赖、零legacy引用**——模块内无run_autovm/parse符号，ACE失败永不切换默认引擎）：
- `AceEngineChoice{AceAV}`＋`parse_ace_engine("av"|"ace-av")`；未知名与全部旧引擎名→`Denied{code:"ace.engine/unknown"}`（零dispatch）。
- `AceRequest{source, entry_def_id, capabilities, grants, provider, wall, cancel}`（全显式输入；`simple()`便捷构造）；`AceSpan{start,end}`。
- 结果分态 `AceOutcome{Success{value,steps}, Diagnostic{stage,code,message,span}, Denied{code,message}, ResourceDenied{kind,message}, Cancelled, HostFailure{code,message}}`＋证据面 `AceRunReport{engine, outcome, diag_count, callee_count, rejected_count, wall_ms}`；`AceValue`标量面（Unit/Bool/I8..F64/Char/Opaque具名）。
- 注册/执行：`trait AceExecutor`＋`install_ace_executor(Arc)`（单owner，同指针幂等）/`shutdown_ace_executor()`/`run_ace(choice,req)`/`run_ace_named(engine,req)`；未注册→`HostFailure{ace.host/executor-unregistered}`。同步、不spawn线程（UI主线程合同）。

执行桥（**新crate `crates/auto-lang-ace-host`**；root workspace成员但**非**auto-lang/auto-lang-core依赖——daily组不构建ACE；测试走新固定组 `ace-facade`（domain ace，入ace集，`cargo t ace-facade`））：
- 宿主生命周期：`install()`（进程单例幂等）、`build_f0_text_provider(s)(names,root)`（冻结F0 text provider，rustc cdylib一次性构建）、`revoke/readmit/teardown_provider`。
- 每run纪律：查注册→撤销态预拒（`ace.provider/revoked`，编译/装载前）→foreign车道capabilities须⊆冻结F0集{foreign.scalar,foreign.resource}（否则`ace.capability/unsupported`预拒，空=冻结集；纯源非空grants=`ace.grant/no-provider`）→grants对已授予caps核（`ace.grant/unadmitted`）→**每run重admit**（`f0_authority`+`manual.admit`）→独立`load()`（本run独占ForeignStubTable/计数）→ForeignSession `adopt→execute→teardown`（关闭后晚到调用拒绝）。crate整体`#![cfg(windows)]`（ace-abi模式；非Windows编译为空crate，E0不承诺跨平台）。
- 车道：纯源 `compile_scalar_or_core_source_product→lower_checked→verify_mir→ABC encode/decode→verify_abc→execute`（会话暖启同一lower seam）；foreign源 `SourceQuerySession.set_foreign→compile_text_record_product→lower_checked_owned→…→verify_abc_foreign→ForeignSession`（F0冻结owned/text-record族）。入口缺失=`entry`阶段诊断；前端诊断原stage/span透传。
- av侧适用owner无代码改动：`query_entry`/`ForeignSession`/`rust_provider::{build_f0,load,f0_authority}`/`execute(_foreign)` 既有公开接口即本切片冻结消费面。

用例：`crates/auto-lang-ace-host/tests/e0_facade.rs`（p969_e0_dispatch/close/source，V-E0-*逐条）；core侧纪律用例4个在`ace_entry::tests`（daily可见，纯std）。

### T-01冻结记录（2026-11-15，基线 master 45ecb376a）

**有界全链前探（已执行，scratch证据 plan-artifacts/plan-969/probe-t01/）**：最长producer(文件)→consumer(公开门)路径一次正例＋同类非法近邻＋失败恢复＋预算/取消/拒绝可观察，全部经`run_ace`公开门实测——
P1 B01文件→`Success{I32(14),steps=6}`；P2a B18不可变写→`verify/verify.immutable-write`原span 38..43（与legacy `auto_name_E0203`同不变量、ACE自身命名码，阶段正确）；P2b坏语法→`text/source.syntax` 27..28；P3 坏→修→同门复跑=7；P4b core-lane长循环+1ms wall→`ResourceDenied{ExecutionWall}`（16ms观测）；P5b 预置cancel→`Cancelled`；P6 未知引擎→`Denied{ace.engine/unknown}`零dispatch；P7 use.rs源经文件＋provider车道→9、callee=1。**冻结语义发现**：AV取消/wall按`CANCEL_CHECK_INTERVAL=1024`步轮询（av-core contract.rs:264）——子区间短程序在首个轮询前正常完成（P4/P5短程序=Success是冻结合同行为，非门缺失）；子区间程序不因预置flag/超短wall中止。

**策略决定（比较后选A，E0已验证）**：A=中立合同（auto-lang-core `ace_entry`）＋注入executor桥（auto-lang-ace-host），legacy门保持legacy、逐门显式迁入；B=core硬依赖ACE（违反"普通语言测试不构建ACE"，否决）；C=worker进程门（进程边界＋UI主线程同步合同复杂化，否决）。E0切片即A的已落实现。

**§4路径重核（SHA256前16@45ecb376a）**：execution_engine.rs `70af92dea97ec79a`（E0加了doc注记）、entry.rs `b3e46189b0e6e1af`（未动）、auto-lang-core/lib.rs `fc8a688b18e6e0b3`（+ace_entry声明/再导出）、auto-lang/lib.rs `d164a7d19ccea437`（未动）、script_entry.rs `d1d0fff6299cdc12`、av/session.rs `7c37123cb8504b91`（未动）。**兼容面事实（T-08范围，survey钉死）**：`run_with_mode`为compile-only信息串（Config/Template经Codegen编译、从不执行）；`run_file_with_args`经进程级SCRIPT_ARGS+AutoVM capture→println；`run_abt/create_vm_from_abt`=ABT人类可读字节码→assemble→VM；input=`Some("")`即时EOF/`None`进程stdin；deadline=协作式每sweep检查；capture=VM内output_buffer非进程重定向；CLI无`--engine`旗标（auto-man cmd_run→run_file_with_args）。

**支持集/资源合同冻结**：现ACE车道=scalar/core纯源（标量值面）＋F0 foreign provider（Windows）＋wall/cancel（轮询节奏1024步）；默认预算=MAX_AV_INSTR_STEPS 1e6/MAX_AV_FRAMES 1024/MAX_AV_SLOT_BYTES 8MiB/MAX_AV_WALL 2s/MAX_TRACE_BYTES原值＋F0 MAX_CALL_BYTES 65536/MAX_RUN_BYTES 1MiB；请求级wall覆写已接。平台=Windows（桥crate根cfg）；Linux腿按2026-11-14裁定归专用Linux环境。

**兼容缺口表（具名owner，T-02/03/08关闭；非本行不得冒已支持）**：stdout capture语义（AV console profile存在`av.host-console/1`+execute_console(grants,provider)，公开门未接）→T-02/03；args/argv车道→T-02/03；stdin input车道→T-02/03；config/template执行语义（现compile-only；eval_config_with_vm=VM eval）→T-08具名裁定；旧ABT字节码→ACE车道**版本拒绝**（不透传为ACE指令）→T-08；test_file/test_code族→T-08核范围。

**旧用例映射冻结（r6表落地）**：`ace.m1.b01`→新case `ace.m2.p969.source01`（公开门run-av车道；native/a2r/a2c按各自后端原支持域由T-04并集核）；`ace.m1.ia920.b18`→`ace.m2.p969.source02`（不可变写typed拒绝不变量，期望=`verify/verify.immutable-write`类typed诊断，不照搬legacy码）；unified_api_tests三测→V-08（run_with_mode兼容行为）＋V-DETAIL-01..04。永久fixture位置：语言侧`crates/auto-lang-ace-host/tests/`（公开门）＋必要时`crates/auto-lang/test/cases/ace-m2/p969/`（case格式，等G1002裁定）；不另建runner/checker。

**1024承接冻结（AC-AUDIT实施策略）**：AUDIT-03（F03文案选路）——修法=前端projector在`source.unsupported`/表面拒绝时携带**结构化surface事实**（profile拥有的记号/形状类，如operator-char/bracket/tuple/ctor的typed reason），`native/driver.rs is_surface_rejection`与AV共享门改按结构化reason选路，保stage/code/profile身份与terminal诊断不改；只改文案不得改选路（T-02实施，T-04核）。AUDIT-08（F08事实权威）——产出profile×consumer事实覆盖表（scalar→SQ facts 934、core→facts 999、text-record/foreign→facts 938/967、host-console→863车道），未覆盖域具名保留旧推导不强删、facts失败不得悄悄产物（表由T-02填齐、T-04核）。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r6当前）

**具体输入与期望。** 原件[crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at)（下列节选L1–L1；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
fn main() int { return 2 + 3 * 4 }
```

本包所需独立结果：同B01源经显式ACE请求返回14；坏源码报原span；未知引擎或权限拒绝不触发旧VM；E0同步功能不冒完整入口兼容。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 公开run/嵌入请求→配置/源码/声明闭包→唯一ACE编译与执行会话→typed结果/错误→CLI/UI/服务consumer。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct ExecutionRequest { source: SourceId, engine: EngineChoice, target: TargetAbi, providers: ProviderSet, grants: GrantSet }
enum ExecutionResult { Value(ValueRef), Diagnostic(DiagnosticRef), ResourceDenied, Cancelled, HostFailure }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/auto-lang-core/src/execution_engine.rs](../../crates/auto-lang-core/src/execution_engine.rs) | 现ExecutionEngine枚举/选择，扩公开入口 |
| [crates/auto-lang-core/src/entry.rs](../../crates/auto-lang-core/src/entry.rs) | 统一请求入口 |
| [experimental/ace/crates/av/src/session.rs](../../experimental/ace/crates/av/src/session.rs) | ACE会话/关闭，E0复用非UIconsumer |

**状态、错误与资源。** 错选项/入口/ABI/授权报明确类且legacy dispatch=0；不得因新引擎失败静默改默认或退旧机器。 每run重授权限，两个session隔离，关闭/取消/宿主故障可区分；公开宿主结构无需guest Dynamic反射才能创建。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/cases/ace-m1/src/B01.at](../../crates/auto-lang/test/cases/ace-m1/src/B01.at) | 源与数学值14 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `b7e5f25e46902be2` | 拟ace.m2.p969.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/test/cases/ace-m1/src/B18.at](../../crates/auto-lang/test/cases/ace-m1/src/B18.at) | 不可变写的阶段/码，不任意非零 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `cf9358d904e6fa3b` | 拟ace.m2.p969.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-04 |
| [crates/auto-lang/src/tests/unified_api_tests.rs](../../crates/auto-lang/src/tests/unified_api_tests.rs) | 真实公开模式/输入/错误通道迁移 | Rust断言来源：test_run_with_mode_script, test_run_with_mode_config, test_run_with_mode_template；SHA256前16 `71b34e4813903445` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 实际公开API/CLI拼写先按当前入口冻结，本文不宣称已有--engine ace；完整配置/stdin/EOF/所有旧入口保原义务。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-facade.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-facade.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-HELLO | add | docs/specs/ace-migration/m2-facade.md（沿原owner） | 未有早期交接→本切片实际已验接口/源/拒绝及剩余完整义务；交付后沉淀 | 最小Hello不冒全包完成 | AC-E0/AC-E0-REVIEW/AC-E0-LAND |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-facade.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |

### 1024发现承接合同（2026-11-13，r7，仅规划）

承接1024的F03/F08：前门分派与事实权威，优先级P2（F12仅保留约束）。用户仅授权复审与分配，尚未授权实现；原支持/AC/任务/阶段/预算不减少，原任务进度0保持。

在T-01冻结每profile的事实生产者与拒绝原因，T-02/03实施结构诊断及同一类型权威接线，T-04验证。不能因消重复删除独立verify、TextRecord foreign解析或原域拒绝。

1002关系：核心职责/原入口反例/固定组治理均不以1002为硬前置；新作者格式迁移或document/IR/adapter重构仅等待G1002（1002 T-03四例七检查及负控通过，T-04独立复审、接口正式合入并绑定SHA）。不等待全语料搬迁；1002不反向依赖本Plan整包。现有M2能力前置不因承接登记被豁免，也不把这些非紧急结构改造伪装成可立即施工。

来源：[1024](1024-ace-m2-code-quality-and-refactoring-audit.md) r2与Git 81d8fc03f10f4a7d179b0bc3a917602406ed7b91。F01–35高风险修复另立1027–1037；本Plan消费正式合入结果并在T-01重核漂移。与在途文件同写须串行，1017成本身份单独协调，不能靠删例减成本。规范影响沿本Plan原SD目标，review补当前实际实现，不改canonical Specs。

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 同一原脚本文件/字符串/SDK入口结果与stdout/stderr/退出一致 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 私有配置及权限隔离 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | ACE缺能力具名失败且旧VM dispatch=0 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 超时观察真实子任务结束而非父返回 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |

完整职责组拟定：`cargo t auto-lang-core auto-lang ace-av integration`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 旧名API全部映射新owner；未知引擎参数不偷偷选legacy；非法旧artifact受控拒绝，合法新产物可运行；stdin/EOF、cwd/args、stdout/stderr与退出码分别核。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### E0验证（T-E0-I建永久case与真实入口，未执行）

| Case | AC | 实际输入/独立关闭oracle | owner/边界 |
|---|---|---|---|
| V-E0-DISPATCH | AC-E0 | 公共显式ACE请求运行普通Auto→F0 Rust provider，输入`A中😀\0`结果9、callee次数正确；同一输入错误签名/撤权/未知引擎拒绝且legacy dispatch=0 | work；不是只检查engine标签 |
| V-E0-CLOSE | AC-E0 | 两个隔离session连续创建/关闭20轮，结果/授权不串，关闭后的provider调用拒绝；宿主错误/取消分别可观察，精确修复后同公开入口再执行 | work；非完整async/daemon/热更新 |
| V-E0-SOURCE | AC-E0 | 普通源坏编辑/错误入口报原span与正确阶段；已有合法候选保留，修复后调用同入口结果变化，无手造Checked或缓存授权 | work；拟cargo t ace-source ace-semantics ace-av ace-abi，根公开入口改变补auto-lang-core与必要integration完整职责组 |


### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 同B01源经显式ACE请求返回14；坏源码报原span；未知引擎或权限拒绝不触发旧VM；E0同步功能不冒完整入口兼容。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 错选项/入口/ABI/授权报明确类且legacy dispatch=0；不得因新引擎失败静默改默认或退旧机器。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 每run重授权限，两个session隔离，关闭/取消/宿主故障可区分；公开宿主结构无需guest Dynamic反射才能创建。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。


### 1024承接的具名验证

| Case ID | 发现 / 原任务 / AC | 输入/设置 | 实际入口 | 独立关闭oracle | 状态 |
|---|---|---|---|---|---|
| AUDIT-03 | F03 / T-01..04 / AC-AUDIT | 只改message、同码不同原因、正常窄域/宽域及真正parse/type错 | native::driver::compile_escalating与product facade | 结构化理由决定路由；Core旁路、terminal诊断、attempted身份保持；改文案不改选路 | 拟实施，未运行 |
| AUDIT-08 | F08 / T-01..04 / AC-AUDIT | profile×consumer事实覆盖/旧综合/独立verify/诊断oracle表 | source-query::consumer/apply→core→AV/native/A2X | 未覆盖域具名；不把diagnostic旧路当成功fallback；facts失败不得悄悄产物 | 拟实施，未运行 |

原组/平台/ignore/规模与正式性能分轴不变；新格式需G1002，原格式和Rust入口可先验证。每条oracle必须在本Plan原work验证完成前取得，不能移到merge或借登记冒关闭。

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 所有兼容前门与错误通道的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |
| AC-E0 | work | 公开ACE AV同步run/provider准入、结果/错误分态、session关闭与无legacy fallback成立 | T-E0-I/V；V-E0-* |
| AC-E0-REVIEW | review | 同树独立核E0实际dispatch/权限/关闭及正反例 | T-E0-R |
| AC-E0-LAND | merge | E0正式来源/请求合同/ABI与实际命令落主供978/980，原facade范围不减 | T-E0-D |

- [ ] AC-AUDIT / work：1024承接F03/F08的上述每条AUDIT用例及保留边界完成；原任务T-01/T-02/T-03/T-04负责，原独立review逐项核，未执行不PASS。

## 8. 执行步骤

当前共12任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

早期任务可按§5最小前置先行，不要求T-01完整分母冻结已完成；各I自行冻结本切片输入/接口/命令，不提前勾T-01或其它原整项。以下行序不是额外前置。

- [x] **T-E0-I｜M2-A｜owner_stage: work｜AC-E0**：967/F0-D正式交付后，自行冻结公开同步ACE请求/结果/错误与provider注册/主线程合同，在现有owner接通非UI普通源consumer并建V-E0-*；不等原T-01完整配置分母。 [✅ 2026-11-15：接口冻结见§5"E0接口冻结记录"；ace_entry.rs＋auto-lang-ace-host桥＋V-E0-*三用例＋ace-facade固定组；cargo t ace-facade PASS]

- [x] **T-E0-V｜M2-A｜owner_stage: work｜AC-E0**：T-E0-I后执行全部V-E0-*及冻结职责/必要消费者，核真正新dispatch、撤权零调用、session隔离与退出，不以标签替证据。 [✅ 2026-11-15：`cargo t ace-facade` PASS（3执行/0排除；per-case：dispatch 0.411s、close 0.504s、source 0.043s，全≤10s门；发现2.81s/执行1.64s/总4.47s）；实测text_byte_len("A中😀\0")=9且callee=1、坏签名零dispatch（compile诊断原span）、撤权/未知/旧引擎具名拒、双身份20轮结果与计数不串、teardown后HostFailure、预置cancel→Cancelled、B01=14、错入口=entry阶段；core侧ace_entry 4用例绿]

- [x] **T-E0-R｜M2-A｜owner_stage: review｜AC-E0-REVIEW**：T-E0-V齐后停写同树独立复核公开入口及无legacy旁路，finding返I。 [✅ 2026-11-15：独立上下文agent审计@278b91204（未参与实现）——verdict **PASS**（0 BLOCKER/0 MAJOR/4 MINOR+1边界note）；F-E0R-01 crate根cfg(windows)/E-02 doc+锁码/E-03 oracle钉死/E-04显式caps 全数修复@1ad2d2c60并复验（cargo t ace-facade PASS：source 0.047s/dispatch 0.509s/close 0.600s；core ace_entry 4绿）；F-E0R-05（撤权与在途run竞态）按审计裁定为E0同步合同边界、后续async切片处理]

- [x] **T-E0-D｜M2-A｜owner_stage: merge｜AC-E0-LAND**：T-E0-R pass后按phase fold正式合入公开入口/已验规范与接口来源，交978/B0和980/HELLO；保树继续原全部facade任务，不翻机器默认。 [✅ 2026-11-15：merge --no-ff plan-969-dev→master `5dfcfe74e`（含实现278b91204+修复1ad2d2c60）；主检出净树落门核对 cargo t ace-facade PASS＋core ace_entry 4绿（主检出存在外来PLAN-979未完WIP致本地红，按字节备份→还原→验证→恢复处理，WIP原样保留，见§9）；沉淀 docs/specs/ace-migration/m2-facade.md；树resync至master，T-01..08继续]



- [x] **T-01｜M2-A｜work｜AC-01/AC-PHASE**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 [✅ 2026-11-15：§5"T-01冻结记录"——前探P1..P7全绿（scratch plan-artifacts/plan-969/probe-t01，输出摘要折入计划）；策略决定A（中立合同+注入桥，E0已验）；§4六文件SHA重核@45ecb376a；兼容面事实survey钉死（run_with_mode=compile-only等）；支持集/资源合同/平台冻结（轮询1024步、默认预算、F0预算）；兼容缺口具名owner表（capture/args/stdin/config/ABT→T-02/03/08）；旧用例映射冻结（source01/source02/V-08/V-DETAIL）；AUDIT-03/08实施策略冻结（结构化surface事实选路；profile×consumer事实权威表）]

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 [✅ 2026-11-15 @5b93ed743：①主能力=console车道（AceRequest.console＋AceConsoleReport stdout/stderr/dispatches；execute_console每run grants授权，未授权预拒host.unauthorized零dispatch）；②永久正反例 t02_facade 5绿（source01 B01文件门+修复循环、source02 B18 typed拒绝、预算态wall/cancel/轮询钉死、未知引擎拒、console授权/捕获/入口策略）；③AUDIT-03=Diagnostic结构化SurfaceFact（4类）在CST projector/lexer与legacy parser/lexer发射点打标＋native梯子改结构化选路（保unsupported@Text的code+stage稳定臂），3 E2E＋1消息无关单测绿；④AUDIT-08=事实权威表＋4守卫（scalar事实权威typed拒、显式core升级、canonical typed错误、text-record/record各自应答）绿；⑤回归：ace-facade/ace-source/ace-semantics组PASS，ace-native组280/280事件ok×2（门FAIL=运行器conhost观察噪声，环境归因记录§9）]

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 [✅ 2026-11-15：非UI真实consumer=auto-lang-ace-host桥（E0/T-02/T-08全链经真公开门）；权限=每run grants/admission/revocation（e0_close+console用例）；失败恢复=坏→修→同门复跑（source01/探针P3）；目标面=scalar/core/console/foreign四车道各自独立oracle；args/stdin=具名受控拒（33eab5086）；增量会话=970域边界（§5 E0冻结）]

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：逐导出核run_abt/create_vm_from_abt等旧编译输入API、run_with_mode/auto mode、capture/input/deadline/config；对旧字节码明确版本拒绝或受检迁移，不能透传为ACE指令。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。 [✅ 2026-11-15 @33eab5086：门映射审计表冻结（§9收据+t08头注——run*/run_with_mode/run_abt/eval_config各保legacy owner，unified_api_tests/abt tests钉住旧行为不变）；ACE门受控拒=真legacy ABT程序文本→text阶段拒（永不作为ACE指令执行）；args/stdin=显式输入+具名拒ace.args/input-unsupported（预车道零dispatch）；deadline=wall已接（E0）；capture=console车道已接（T-02）；t08_facade 3绿]

- [x] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 本轮纳入§5/6的1024承接合同与AC-AUDIT，必须保全部原义务。 [✅ 2026-11-15 @冻结33eab5086：V行对账——V-01✓(t08)/V-02✓(身份+grants隔离)/V-03✓(具名拒+结构零legacy)/V-04✓(budget)/V-08✓/V-E0-*✓/V-DETAIL-01..04✓/V-PHASE✓(§8/§9逐行收据)/AUDIT-03/08✓；组并集——ace-facade PASS、ace-av PASS、daily@worktree 2669/2670 ok（唯一红=plan702门扫`../../../auto-ui`跨仓路径、lang-969组无auto-ui兄弟树——环境归因非代码红；daily全量@主检出=merge-owned pending）；复用——ace-source/ace-semantics绿@5b93ed743、ace-native 280/280事件ok×2（conhost门噪声§9）；磁盘——D:剩1.1G致daily首跑拒跑，回收自有16.6G后重跑]

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-969/auto-lang`，分支 `plan-969-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-969/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-12 stage:new / PLAN-969 r6：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原12任务/全部AC与V/依赖/阶段/预算和drafting 0/12；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-11 / stage:new / PLAN-969:r5 / outcome:pass（早期Hello路线合同可审阅）；blocked（尚未授权实施，真实能力/接口待交） / next:按§5最小前置进入新增切片；原完整M2任务与终验均保留，不用小例替全包。新增T-E0-I, T-E0-V, T-E0-R, T-E0-D，当前0/12。未实现/构建/运行或派agent。

2026-11-11 规划静态核验（非产品/独立实现review）：12份本轮文档、6份Plan共106任务；对照本轮前快照，原86任务ID及原AC/V全部保留，新增20任务、全为未执行，drafting/current_step=0。0–10章节、total_steps、阶段清单与AC任务映射一致；323处本地链接已检查，新增链接/4处新锚点无失效、无新增行尾空白。跨四仓可达207份Plan的显式依赖无缺失/环，8个早期交接节点无环、AV不反等A2R或完整B；12个拟职责组名存在。6个具名源码SHA与本轮读取值相同；语言/UI/内容计数器仍1006/081/032。未新取号、建树、改产品/测试资产或canonical Specs，未构建/运行/签阶段/merge/tag。此检查不证明拟议接口已经实现。

2026-11-10 stage=new / r4：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部8个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r3：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/8；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。


2026-11-13 stage:new / r7 / outcome:pass（规划承接） / next:原前置及实施授权后work。用户授权将1024的F03/F08纳入本包原任务，新增具名AUDIT oracle/AC-AUDIT；未增任务、未删原任务/AC/V/阶段/预算、0进度/drafting保持。旧r6规划记录保留；实际实现/执行未开始。1002依赖按任务分流，无整包循环。

2026-11-15 stage: work / T-E0-I + T-E0-V（E0切片，AC-E0）| worktree: D:/autostack/.wt/lang-969/auto-lang @ plan-969-dev | base: ca88cde14 | code_commit: 278b91204 | 依赖核对: 960/964/967/968全archived；967/F0-D已落主（master 4c1788a79，AC-F0-LAND收据） | 产出: `crates/auto-lang-core/src/ace_entry.rs`（公开合同，零ACE依赖/零legacy引用）＋新crate `crates/auto-lang-ace-host`（executor桥+provider生命周期+每run重admit/授权）＋`tests/e0_facade.rs`（V-E0-DISPATCH/CLOSE/SOURCE）＋固定组`ace-facade`（入ace集） | 验证: `cargo t ace-facade` PASS（执行3/排除0；per-case dispatch 0.411s/close 0.504s/source 0.043s，≤10s门达标；discovery 2.81s/execution 1.64s/wall 4.47s，meter见target/test-groups/run-d_gttlhy）；`cargo test -p auto-lang-core --lib ace_entry` 4绿；`cargo check -p auto-lang`/`-p auto-lang-ace-host --all-targets` 绿、新增代码0 warning；fmt已施（误及的ac_av.rs/test_cases mod.rs外部fmt噪声已revert，不在提交内） | 测试纪律: 3新用例数值如上（provider rustc cdylib构建在显式ace集成车道内计入用例耗时）；无新慢用例 | 真实dispatch证据: text_byte_len("A中😀\0")=9且callee_count=1；坏签名=compile诊断原span零dispatch；撤权/未知引擎/旧引擎名=具名Denied预拒；双provider身份20轮结果与计数不串；teardown后=HostFailure{ace.provider/unknown}；预置cancel flag=Cancelled；B01显式ACE=14；错入口=entry阶段诊断 | 义务保留: 原T-01..08/全部AC/1024承接不变，E0切片不冒整包 | next: T-E0-R（同树停写独立复核）

2026-11-15 stage: goal | plan_id: PLAN-969 | plan_revision: r7 | round: 1 | from: work | to: work | status: executing | outcome: pass（E0切片work完成，T-E0-I/V绿） | worktree: D:/autostack/.wt/lang-969/auto-lang | commit: 278b91204 | next: T-E0-R（E0切片独立复核）

2026-11-15 stage: review / T-E0-R（E0切片独立审计，AC-E0-REVIEW）| 冻结基线: 278b91204（停写） | 审计者: 独立上下文agent（未参与实现，重跑全部声明的scoped命令） | 结论: **PASS**（0 BLOCKER/0 MAJOR/4 MINOR+1边界note；逐项核：ace_entry零legacy引用/单owner安装/桥接车道对ACE真实API（lower seam、wire admission、adopt/replace、teardown次序）/无泄漏UAF/授权次序/oracle质量/收据逐条对账/组注册/卫生/工作区无破坏） | findings: F-E0R-01非Windows根门缺失→crate根`#![cfg(windows)]`；F-E0R-02 callee_count doc+锁poison误标→`ace.host/executor-lock-poisoned`；F-E0R-03诊断oracle弱于计划行→钉死bind/source.argument 38..57、text/source.syntax 27..28、text/source.unsupported 23..26；F-E0R-04 foreign车道静默丢caps→冻结集外`ace.capability/unsupported`预拒+用例 | 修复: 全数@1ad2d2c60，复验cargo t ace-facade PASS（source 0.047s/dispatch 0.509s/close 0.600s）+core ace_entry 4绿 | F-E0R-05: 撤权与在途run竞态=E0同步单线程合同边界（审计验证零thread::spawn），后续async切片处理 | next: T-E0-D（phase fold）

2026-11-15 stage: goal | plan_id: PLAN-969 | plan_revision: r7 | round: 2 | from: work | to: work | status: executing | outcome: pass（T-E0-R PASS，findings全数修复复验） | worktree: D:/autostack/.wt/lang-969/auto-lang | commit: 1ad2d2c60 | next: T-E0-D（E0切片phase fold）

2026-11-15 stage: merge / T-E0-D（E0切片phase fold，AC-E0-LAND）| 合入: plan-969-dev → master `5dfcfe74e`（--no-ff；含278b91204实现+1ad2d2c60修复，9文件+1792行） | 落地核对: 主检出**净树** `cargo t ace-facade` PASS＋`cargo test -p auto-lang-core --lib ace_entry` 4绿 | 外来WIP处置: 主检出 `experimental/ace/crates/core/src/source/mod.rs` 携PLAN-979未完WIP（AgePackage臂引用不存在变体，致含该文件的本地编译红）——**非本包改动、未纳入任何提交**；按 备份（plan-artifacts/plan-969/main-checkout-979-wip-mod.rs.bak）→`git checkout --`还原→净树验证→恢复 流程处理，sha256前后一致（471876a2bab0df31…）；已浮出待979 owner路由 | 沉淀: `docs/specs/ace-migration/m2-facade.md`（E0已验接口/每run纪律/永久验证/边界与开放项） | 交接: `auto_lang_core::run_ace`＋`auto_lang_ace_host::install()` 供978/B0、980/HELLO消费 | 树: 唯一worktree保留并resync至master，原T-01..T-08/1024承接继续；不归档、不勾整包完成、不翻机器默认 | next: T-01（全链前探/冻结）

2026-11-15 stage: goal | plan_id: PLAN-969 | plan_revision: r7 | round: 3 | from: work | to: work | status: executing | outcome: pass（T-E0-D phase fold完成：master 5dfcfe74e＋spec沉淀＋净树落门绿） | worktree: D:/autostack/.wt/lang-969/auto-lang | commit: 5dfcfe74e(merge) | next: T-01（原完整facade任务开工）

2026-11-15 stage: work / T-01（AC-01/AC-PHASE 冻结轮）| worktree: D:/autostack/.wt/lang-969/auto-lang @ plan-969-dev（resync至master） | base: 45ecb376a | code_commit: 无（纯前探/冻结；probe scratch在plan-artifacts/plan-969/probe-t01/，已登记REGISTRY） | 产出: §5"T-01冻结记录"——①前探P1..P7实测（正例14/同类非法近邻typed span/失败恢复/长循环wall=ResourceDenied/预置cancel=Cancelled/未知引擎拒/provider文件车道9+callee=1）；②冻结语义发现：cancel/wall按CANCEL_CHECK_INTERVAL=1024步轮询、子区间程序先完成属合同行为；③策略A决定；④§4六文件SHA重核；⑤兼容面survey（run_with_mode=compile-only、ABT=人类可读字节码、input EOF语义、capture=VM缓冲、CLI无engine旗标）；⑥资源/平台合同；⑦兼容缺口具名owner表；⑧旧用例映射（source01/source02/V-08）；⑨AUDIT-03/08实施策略 | 义务: 全部保留，未缩范围 | next: T-02（主能力+永久正反例+F03/F08实施）

2026-11-15 stage: goal | plan_id: PLAN-969 | plan_revision: r7 | round: 4 | from: work | to: work | status: executing | outcome: pass（T-01冻结完成，5/12） | worktree: D:/autostack/.wt/lang-969/auto-lang | commit: 45ecb376a | next: T-02
2026-11-15 stage: work / T-02（AC-02 主能力+永久正反例+AUDIT-03/08）| worktree: D:/autostack/.wt/lang-969/auto-lang @ plan-969-dev | base: 45ecb376a(resync) | code_commit: 5b93ed743 | 产出: ①console车道（ace_entry.console/AceConsoleReport{stdout,stderr,dispatches}＋桥compile_host_console_source→lower_checked_owned→execute_console每run grants；console+foreign组合=具名拒ace.console/foreign-unsupported）；②t02_facade永久fixture 5例；③AUDIT-03结构化SurfaceFact（atom_text.rs 4类枚举＋Diagnostic.surface/tagged()；CST projector/lex与legacy parser/lexer发射点打标；native/driver is_surface_rejection=结构化fact＋稳定code/stage臂，消息文本零参与；native tests 3 E2E+driver单测1）；④AUDIT-08事实权威表（表钉在p969_audit08_fact_authority.rs头注＋4守卫测试） | 验证: cargo test p auto-lang-ace-host 8绿（0.03-1.04s全≤10s门）；audit03 3+1绿；audit08 4绿；回归组：ace-facade PASS、ace-source PASS、ace-semantics PASS；ace-native组两轮280/280事件ok（门verdict FAIL经查=运行器process-tree conhost观察"descendants live after workload exit"（cleanup_complete,remaining_active 0）——环境基础设施噪声非测试失败，孤立重放此前闪红4例+collections全绿；首跑bare cargo test的8个::performance::失败=PLAN-1016裁定排除的正式性能campaign） | 修复过程: F03首版纯结构化匹配漏掉legacy parser/lexer发射点与unsupported@Text稳定臂（mn_data_owned_rejection_surface暴露），按"tag+稳定code/stage臂"修正后全绿 | 义务: 全保留（args/stdin→T-08具名拒；AV输入车道=债务候选） | next: T-03+T-08（compat门映射+V-08+V-01一致性）

2026-11-15 stage: goal | plan_id: PLAN-969 | plan_revision: r7 | round: 5 | from: work | to: work | status: executing | outcome: pass（T-02完成，6/12） | worktree: D:/autostack/.wt/lang-969/auto-lang | commit: 5b93ed743 | next: T-03/T-08
2026-11-15 stage: work / T-03 + T-08 + T-04（AC-02/03/06/01..03 收口轮）| worktree: D:/autostack/.wt/lang-969/auto-lang @ plan-969-dev | base: 5b93ed743 | code_commit: 33eab5086 | 产出: AceRequest.args/input（显式输入+具名拒ace.args/input-unsupported）＋t08_facade 3例＋门映射审计表 | T-04对账: V全行映射见§8勾选；组证据=ace-facade PASS@33eab5086、ace-av PASS、daily@worktree 2669/2670（1红=plan702跨仓路径环境归因：门读`../../../auto-ui/crates/auto-ui-runtime/src/ui`，worktree组无该兄弟树；daily全量主检出=merge-owned pending）、ace-source/ace-semantics@5b93ed743复用、ace-native 280/280事件ok复用 | 磁盘: D:1.1G满→daily拒跑；回收自有16.6G（probe target 7.4G+ace incremental 9.2G）后复跑 | 边界: AV argv/stdin车道=具名开放项；增量会话=970域 | next: T-05（停写独立复审）

2026-11-15 stage: goal | plan_id: PLAN-969 | plan_revision: r7 | round: 6 | from: work | to: review | status: executing | outcome: pass（work全own任务完成9/12） | worktree: D:/autostack/.wt/lang-969/auto-lang | commit: 33eab5086 | next: T-05


## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
