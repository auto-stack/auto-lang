---
plan_id: PLAN-967
status: executing
feature_name: 完整C/Rust FFI、自动binding与通用宿主资源
author: [Codex]
created_at: 2026-11-06
updated_at: 2026-11-14
plan_revision: 12
current_step: 14
total_steps: 19
depends_on: [PLAN-960, PLAN-962, PLAN-963, PLAN-938, PLAN-939, PLAN-1042, PLAN-1043]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/auto-bindgen/project.md, docs/specs/auto-abi/project.md]
new_spec_components: [docs/specs/ace-migration/m2-host.md]
touched_goals: [GOAL-002]
---

# [PLAN-967] 完整C/Rust FFI、自动binding与通用宿主资源

## 0. 变更摘要

2026-11-13 r12当前：纳入新1043来源后缀装配及最小#in平台静态选择，见§5/AC-LOAD/V-LOAD/SD-LOAD；原任务/状态/完成数与全覆盖不减，仅规划。

2026-11-13 r11当前增量：新增公共标准库能力至少AV/AC/A2R/A2PY，AV为同一执行器下native/C/Rust三provider；原A2C继续。见§5覆盖合同及AC-COVERAGE/V-COVERAGE/SD-COVERAGE；只修规划，状态/完成数不变。

2026-11-13 r10：按用户要求补充Rust ABI兼容与独立分发合同：源码模式的Rust库与shim共同构建，对外以版本化目标C ABI分发；严格区分构建缓存身份和运行兼容身份，明确rustc/target/接口版本/运行依赖变化的失效、重新准入与拒绝规则。新增AC-ABI-COMPAT、SD-ABI-COMPAT、V-ABI-*七案，由原T-09/11..15/T-04承接；19任务、drafting/0进度、原依赖/范围/F0不变。此次仅改文档，不构建、运行探针或实施。


2026-11-13 r9：按用户对PLAN-1026的定向文档复查，补INT-1026/IF-CALL/IF-RES通用交接：Python adapter是具体Rust provider，消费本包生成shim/AV stub及单一资源/Meta账本，不自动导入整个PyO3。参数帧/返回owner/异常/FFI寿命纳入既有T-09/11/12/13/14及T-04验证，新增AC-INT-1026/V-INT-1026；仍19任务/0进度，F0最小范围不扩大，原四端/全现用桥义务保持。仅改文档，不实施。


2026-11-12 r8：按用户要求以963同等标准补本包具体输入/独立预期、类型与代码职责、旧AutoVM/A2R及真实消费者到ACE验证映射；原任务/AC/V/依赖/预算/阶段与进度保留。本轮只修订规划，不实施/建树/生成fixture/运行产品；旧历史记录不作当前完成证明。

历史修订：2026-11-12 r7当前：纳入默认自有Auto基础标准库/非libc默认provider及长度字符串ABI边界，见§5新增合同。保原任务/阶段/范围，新增AC-NATIVE/V-NATIVE/SD-NATIVE，仍只规划。

历史修订：2026-11-11 r6当前：增加F0最小可复用AV/Rust互操作先行切片；先闭普通源码→自动binding/shim→受检调用→真实Rust函数，不等完整C importer、serde_json、泛型/async/全标准库；原完整FFI-M2-01..08不减。 原15项任务/全部AC/V保留，新增4项早期切片任务，现19项、drafting、0完成；只规划。

历史修订：2026-11-10 r5执行版：M2-A。本轮把阶段安排落实到本Plan的开工前置、执行顺序和A/B交接；原任务/AC/范围保留，所有任务仍未执行。以§5的最小接口和§8的分阶段清单为准，不按后文历史轮次的旧总顺序施工。

历史修订：2026-11-09 r4当前：用户正式拆M2为A语言/新库完善、B全产品替换；本包归属**M2-A**，以§5阶段合同为当前启动/验收边界。旧日期范围/义务不删，所有任务仍未执行；仅规划，不授权施工。

历史修订：2026-11-09 r3：按用户要求纳入[Design56：完整FFI与新版标准库](../design/56-ace-m2-ffi-and-native-stdlib.md)的本包义务；当前仍为drafting、0/15，只规划不施工。原任务/AC/历史记录保留，新增任务均在最终work验证与独立review之前。

承938/939有限桥，建立中立typed调用、资源handle、borrow/owned/transfer、同步重入与异步完成端口；C/Rust/Python既有应用桥逐项映射。generation+owner+签名验证，授权每次调用重核，旧native数字ID只作兼容映射。

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

2026-11-14 目标授权（/goal 完成计划967，auto-plan-goal 范式）：用户一次授权整链——创建专属
worktree、按本Plan任务执行work、独立review、批准合同内的有界修复、merge（规范沉淀、归档、
清树）。不改目标/验收阈值/兼容承诺/范围/预算/仓库集；正式依赖（938/939/960/962/963/1042/1043）
当日核验全部 archived，M2-A 前置就绪。各阶段凭证按 §9 goal receipt 追加。

2026-11-11本轮授权：用户要求规划首个ACE UI Hello及AV/Rust FFI路线、核对既有M2冲突并作必要重排；仅规划/设计/导航，不实施、创建worktree或探针/fixture/SDK、构建/运行、提交或发布，未指定模型/工期。沿旧语言用例守恒、新Auto标准库与AutoABI/FFI、Demo递进的三线，不削完整M2范围。

现场语言HEAD 9e36d984e、UI HEAD d88e9c51。已核av-core/src/host.rs仅console/1（sha256前16 30d6dcc475850cd0）、a2x-rust/src/provider.rs仍拒source foreign（3ea453dc2452576f）、core/src/source/cst/frontdoor.rs（dd3d93d06799ae52）；现有auto-bindgen/src/extractor.rs是预置C manifest表（8c7f1295cc83c6fb），不能称Rust元数据导入器已实现。Iced由Rust宿主编译执行，AutoVM不解释其Rust源码；现有DynamicComponent/VmBridge/handler_codegen深依赖旧执行链。

本次授权只含设计、路线与相关Plan修订，未含源码/测试实现、生成fixture/SDK、构建、产品运行或merge。阶段依据为用户本轮正式决定；取材当前959冻结分母、TestCase/固定组/ACE源码重放规范及现存owner入口，旧overview的物理路径/别名不覆盖当前Cargo。

用户2026-11-06授权详细调研并重制M2设计/计划，范围包括完整Auto、AutoUI/AGE、标准库、AutoOS及各应用替换；本轮仅规划，不推定实现/部署授权，未指定工期或模型预算。
读取语言HEAD `de2b606aed298df0f0b342c72d209af990d6d4e2`；共享检出存在其他会话文档WIP。现状证据/哈希见Design55，语义来源按Specs优先，代码差额具名，不改canonical Specs伪造现状。引用会话已通过read_thread读取，其936/944旧状态已由当前归档纠正。

实际入口/文件：
- `experimental/ace/crates/av-core/src/host.rs`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/foreign-contract`（本轮确认存在；开工重核符号/版本）。
- `experimental/ace/crates/abi`（本轮确认存在；开工重核符号/版本）。
- `crates/auto-lang-core/src/vm/host_bridge.rs`（本轮确认存在；开工重核符号/版本）。

外仓路径仅只读调研；跨仓实施前T-01须读取当地AGENTS并由owner确认同组实施路径与实际门禁。工具真实实现归属须按当前模块Spec与源码核定；LSP已由971确认在auto-ui，不能从语言仓残留固定组名推断package归属。

### 2026-11-09 增补调研锚点

依据用户本轮M2范围追加与已读取的标准库引用讨论，读取基线HEAD `b4f1af0be67b91b601345a4284ecd4036cb1a91e`；本轮只读核源码，不宣称现有实现已满足新义务。Design56区分现状与目标；原§4历史快照不覆盖本次范围。

- `crates/auto-bindgen/src/extractor.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/auto-bindgen/src/type_map.rs`（现存入口/规范；开工核真实符号与SHA）。
- `crates/auto-bindgen/src/manifest.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/foreign-contract/src/typed.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/foreign-contract/src/resource.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/foreign-contract/src/lane.rs`（现存入口/规范；开工核真实符号与SHA）。
- `experimental/ace/crates/source-query/src/modules/packages.rs`（现存入口/规范；开工核真实符号与SHA）。

### 2026-11-12 本轮规划取材与授权

用户要求细化其余ACE M2计划，复用已有研究/合同，仍只规划；本轮不创建实施树/集成分支、不执行T-01探针或产品/正式性能测试、不提交/合入/tag/发布、不改canonical Specs或legacy索引。945–948原M1-TAG及各A/B正式来源门均不因本次文档静态取材放行；既有实施证据/暂停/等待由原owner维护。

取材语言主线读取基面 `f9fb3854beb3886338156cfa61d16f7f84893cc9`（不是所有外仓同SHA；各源hash如下，开工重核正式来源）；本包焦点：C/Rust自动导入与中立FFI/资源。已读当前规范入口：[docs/specs/ace-migration/replacement-scope-and-evidence.md](../specs/ace-migration/replacement-scope-and-evidence.md)。下列是已核路径/结构的规划取材，不宣称逐符号全审或运行通过；旧函数同名/目录存在/历史组绿都不等于新ACE支持。

- [crates/auto-bindgen/src/extractor.rs](../../crates/auto-bindgen/src/extractor.rs)：现有预置C表，不能冒完整Rust importer；读取SHA256前16 `8c7f1295cc83c6fb`

- [experimental/ace/crates/foreign-contract/src/typed.rs](../../experimental/ace/crates/foreign-contract/src/typed.rs)：共享签名/资源/borrow事实；读取SHA256前16 `fb148c0a779d5c95`

- [experimental/ace/crates/abi](../../experimental/ace/crates/abi)：独立ABI oracle及目标适配；目录职责已核，确切符号/闭包开工再核

2026-11-13本轮授权：用户明确要求只修改PLAN-967及相关设计，补齐Rust库/shim共同构建、C ABI独立分发、编译器版本/目标平台/接口版本的兼容与失效规则。Design41/56为通用合同，Design58仅消费，不接管978的构建/安装实现。读取主检出HEAD `81d8fc03f10f4a7d179b0bc3a917602406ed7b91`；同检出其他会话WIP/新Plan保持，本轮不取号、不修改canonical Specs/legacy JSON、不建worktree。

技术依据（官方资料，非本仓实测）：[Rust ABI及目标C ABI](https://doc.rust-lang.org/reference/items/external-blocks.html#abi)、[repr(C)布局](https://doc.rust-lang.org/reference/type-layout.html#the-c-representation)、[cdylib/staticlib与Rust库产物](https://doc.rust-lang.org/reference/linkage.html)。已有中立facts/descriptor、ABI oracle与store沿§4实际owner扩展；旧auto-bindgen manifest仍有预置/手工来源，不能把下面目标合同当作已实现。

## 5. 详细设计

### M2阶段执行合同（当前优先）

依据[Design55两阶段合同](../design/55-ace-m2-complete-language-and-product-replacement.md#16-m2两阶段正式合同与验证方法2026-11-09当前)；本包=M2-A。本包完整能力与其永久测试均在A；原review/merge/清理亦在A，正式交付后由985/A消费。

| 阶段 | 当前任务归属（保原ID/义务） |
|---|---|
| M2-A | T-01, T-02, T-03, T-04, T-05, T-06, T-07, T-08, T-09, T-10, T-11, T-12, T-13, T-14, T-15, T-F0-I, T-F0-V, T-F0-R, T-F0-D |

本表中的A是完整能力或具名RG-A验证，不是所有产品迁完；B实现须在M2A-08真实通过后开始，T-01仅准备不越过B门。frontmatter仍是整Plan最终依赖并集，局部A按本节/各批最小接口前置启动，不把985全Plan或B消费者整包置为A前置。原每个AC保完整范围，A只关闭其A切片，不能因B未完提前勾整包review/归档。

跨阶段同Plan按已有phase fold规约先独立复核并正式合入A切片，记录repo/SHA/接口与实际测试来源后供985/A消费；同一唯一tree保留/resync至B完成，不能用分支reviewed/WIP当D。公共文件单写；最终归档/清树仍按全包原门执行。本轮未做fold/merge或建树。

语言/库相关case继承旧公开行为义务并补规范/组合/边界/负控，每条按当前规范与独立expected验证，M1有限豁免不自动继承；产品专有全量用例归B，未执行/缺环境不PASS。详见Design55 §16.5，TestCase/固定原生组沿用，不另建tm/全仓调度。AC自举独立后续、非本包或M2硬门。


承938/939有限桥，建立中立typed调用、资源handle、borrow/owned/transfer、同步重入与异步完成端口；C/Rust/Python既有应用桥逐项映射。generation+owner+签名验证，授权每次调用重核，旧native数字ID只作兼容映射。

支持集继承959分母与Design55的完整语言定义；若本包先于分母完成，仅允许静态准备。T-01冻结到符号/语法/consumer，不能把不支持项从范围删掉。新IR节点、provider、领域接面与测试fixture均为拟新增，位置由已核owner下确定，不声称当前存在。

T-01有界前探：用一个最小真实正例、同类型非法近邻和一次失败后恢复串起本包最长producer→consumer路径；先比较最相关的可行实现策略，形成决定与资源/平台适用表后进入实现。不以任意日历或固定语法深度削范围；探针不能闭合则列具体阻塞/修订，不能只交设计宣布完成。
资源所有者：编译session持候选，发布后当前generation持产物；运行/宿主pin保在途，最后owner释放。取消/错误/late结果不得覆盖当前已发布版本。现行预算保持，新增域的计量单位/上限/超限分类在T-01冻结并有边界反例；改变承诺须显式修订。


### 本次复核补充：双向ABI与异步完成端口（r2）

分import/export、调用方/被调用方、标量/聚合/资源与C/Rust/Python现用桥，核动态库装卸、原方allocator、符号/ABI版本。异步本包只交注册/完成/取消端口及受控真实provider；968消费端口实现语言await，967不反向依赖968。

本节由T-08实施并以AC-06/V-08验收，属本包完成条件。既有任务/验收不被替代；需要的新接口由T-01先冻结，T-08依赖T-01及相关T-02/03结果，必须先于最终work验证与独立review完成。

### 2026-11-09 正式M2范围增补（r3，当前合同）

本次把原有限接面正式扩为FFI-M2-01..08；保C/Rust/Python现用桥及原资源/授权/异步完成义务。四个阶段：A公共输入/共享合同（T-09）、B自动导入（T-10 C、T-11 Rust）、C四端/双向ABI/资源（T-12/13）、D精确缓存/真实库（T-14/15）。每批记录输入、冻结接口、实际consumer、V证据和未闭项后才交下一批；最终review仍由原T-05独立进行。

公共use.c/use.rs不要求用户逐函数手写签名/布局/extern。库路径/工具链/所有权注解可显式给，无法证明的raw边界必须显式；不猜Rust私有ABI或任意库安全性。C用真实预处理和目标布局，Rust用公共元数据/具体实例到C-compatible shim或经证明等价adapter，所有事实汇入同一Checked Foreign IR。

自动导入同时覆盖既有消费者的特征表；varargs/union/bitfield、Rust traits/宏/泛型等按真实用法逐行核，不能缩成scalar demo。具体泛型能力新增依赖963。native直接产机器码，AV/AC/新A2R/新A2C均实际调用；旧AST/VM与手写manifest不能代自动链。异步完成端口由此输出给968，967不依赖968；生成物分发/安装由978/986接，不让本包等最终安装才交基础设施。

真实库最低路径为C标准头及SQLite、Rust serde_json公开API；T-09钉版本/工具链/合法source与独立expected，并用自有callback/owned/具体泛型/future fixture补资源边界。C独立编译器caller/callee、Rust独立公开API测试为oracle；全旧分母与新增义务取并集，不将最低样本当封顶。

本次新增验收/任务如下表映射，详见§6–8；原任务不删除、原非目标中的“本包之外”按本节明确新增范围解释。所有V均为待执行设计，当前不写canonical Specs。

| 任务 | 验收 | 用例 | 本包交付/义务 |
|---|---|---|---|
| T-09 | AC-07 | V-09 | 接通工程use.c/use.rs到共享绑定和typed foreign事实，冻结真实库/全特征required表、来源/注解/工具链和合法/非法公共源码；先走通一个真实导入与失败恢复。 |
| T-10 | AC-08 | V-10 | 实现C真实预处理/头文件闭包、target/sysroot/宏/typedef和ABI布局、函数/record/enum/pointer/callback/opaque、常量/inline shim；将旧用法特征逐行闭合。 |
| T-11 | AC-09 | V-11 | 实现Rust公开元数据到具体类型/泛型实例和自动C-compatible shim；闭合String/Vec/Result/borrow/析构、panic、线程与future边界。 |
| T-12 | AC-10 | V-12 | 接AV/AC/新A2R/新A2C/新A2PY到同一Foreign IR；native机器码调用，A2R等价crate适配需独立证明；各端记录真实producer及dispatch。 |
| T-13 | AC-11 | V-13 | 闭合双向import/export、适用静态/动态链接、callback重入与generation/lease；接future完成/取消端口，保原allocator与跨ABI错误语义，交968调度。 |
| T-14 | AC-12 | V-14 | 使header/crate/toolchain/target/provider/ABI/生成器的真实读取依赖进入现有store，输出binding/shim/runtime分发清单和编译/运行工具分账合同给978。 |
| T-15 | AC-13 | V-15 | 用钉版本真实SQLite和serde_json原工程及完整特征fixture做用户体验与资源/失败回归；汇总四端、独立oracle、缺环境和交978/985材料。 |

### F0：首个UI Hello之前的通用AV/Rust FFI切片（当前可执行入口）

F0最小前置为已交938/939契约与现有text/scalar源码、共享verify/AV/MIR、现有artifact store；不使用新用户泛型/嵌套集合，因而不等962/963整包。T-F0-I自行核这些正式来源并有界比较至多两种Rust公共元数据/具体函数实例→自动shim策略。旧auto-bindgen预置C表不当作Rust导入器；不手写每个函数manifest、假metadata或借UI专用native数字ID通过。首次只支持冻结的一组非泛型Rust公开自由函数：借用UTF-8 str输入、i32/bool结果与结构化失败；泛型/String/Vec/Result/owned句柄/future/全部真实库仍由原T-09..15关闭。

**T-F0-I 冻结决定（2026-11-14 实施，worktree lang-967）**：
- **元数据来源**：带版本化注解的冻结 provider 源文件自身（`auto-ffi provider <id>/<ver> target=… profile=…` 头 + 每 fn `// auto-ffi fn <rust_name> symbol=<点分版本符号> form=pure ret=i32|bool | form=fallible errors=name:-code[,…]`）。importer（`crates/auto-bindgen/src/rust_import.rs`，格式 `auto.bindgen.rust-provider/1`）逐项与真实 `pub fn <name>(<一个 &str>) -> i32|bool|Option<i32>` 签名交叉核验，域外（泛型/owned String/多参/其它返回/坏符号）具名拒绝（`rust-import.*`）；产出 (a) 中性 manifest，(b) 生成 C 兼容 shim 源，(c) 共享 `ace.foreign.manual/2` manifest（复用 foreign-contract，无第二 FFI 世界；`to_manual_manifest` 物理面钉共享 plan 常量）。
- **冻结 provider fixture**：`experimental/ace/crates/abi/rustside/f0_text.rs`（text_byte_len/text_is_ascii/text_first_byte(fallible, empty-input:-1)/text_panic_probe("boom" panic 探针)）。
- **物理帧（版本化）**：`extern "C" fn(data: *const u8, byte_len: u64) -> i32|u8`；fallible `+ out: *mut i32 -> i32 status`（Some→status0+out，None→首名负 status）。**panic 通道**：生成 shim 单元内 catch_unwind + 导出 `f0_panic_flag()` 标志；宿主调用后观察并清零→独立 host 失败（`foreign.panic`），任何 unwind 都不跨 C 边界（跨 DLL unwind 实测不可 catch——`Rust cannot catch foreign exceptions`，此为冻结依据）；AV 侧 catch_unwind 仅作 in-process 皮带。
- **构建单元（B-01 种子）**：provider 源+生成 shim 平铺拼成一个 rustc `--crate-type=cdylib` 闭包（`ace-abi/src/rust_provider.rs`，记录 rustc 身份）；loader（LoadLibraryW/GetProcAddress）把导出解析进 av-core `ForeignStubTable`。
- **use.rs 前门**：CST 语法 `use-rs-item`（仅 text-record profile）+ 投影 `source.use-rs.syntax/domain`；resolve 校验唯一性与 provider==admitted manifest provider（`source.use-rs.duplicate/provider`）；旧手写 parser 同形（differential 927 一致，含 profile 门）。资源域模块走 owned wire family（AVAD，PLAN-939 形）；AV 侧新 `abc_verify::verify_abc_foreign`（无表默认仍拒绝，938 不变）+ `av::execute_foreign`（每 run 重授权 + run_bytes 复位）。
- **AV stub 表**（`av-core/src/foreign_exec.rs`）：`MAX_CALL_BYTES=65536`/`MAX_RUN_BYTES=1048576`；拒绝序=身份(unknown-symbol/signature)→generation→UTF-8→call-bytes→run-bytes→reentry，全部在 callee 前（`callee_count` 只计真实调用；panic 计调用后转 host 失败）；`bump_generation` 会话拆解即全表失效；`admits_declaration` 含 generation 新鲜度。**预算语义裁定（T-F0-R审计#3）**：本表第二预算为**每 run 计费的输入视图片段字节**（`execute_foreign` 每 run 复位），取代 §5 上文"单session保留输出最多1048576字节"的字面读法——F0 输出仅 i32/bool，保留输出预算在此域为空，输入计费是唯一有区分力的读法；保留输出记账归原 T-13 资源域。
- **结构化失败**：fallible status≠0 → `Trap{TrapKind::ForeignStatus}`（命名表在 manifest error model，非普通 i32、非 infra 吸收）。
- **边界（F0 不做）**：ProgramSnapshot/session 链接面（969/E0）、A2R 腿（T-12/AXR0）、C importer/真实库（T-09..15）、跨 rustc 分发（T-11/14）。

### T-01 决定摘要与责任冻结（2026-11-14，AC-01/AC-PHASE/COVERAGE/LOAD）

**依赖与锚点核验**（resync 树 `76431e6b9`）：938/939/960/962/963/1042/1043 archived（round1核）；§4 锚点现值 SHA256前16——`foreign-contract/typed.rs` `fb148c0a779d5c95`、`a2x-rust/provider.rs` `3ea453dc2452576f`（与§4记录一致）；`auto-bindgen/extractor.rs` `92129699344333e1`（§4旧值 `8c7f1295cc83c6fb` 为2026-11-12快照，master后续计划已演进，现状以本行为准）；`av-core/host.rs` `81f49256ef880fc4`（console域，F0未动）；F0新增锚点 `auto-bindgen/rust_import.rs`、`abi/rust_provider.rs`、`av-core/foreign_exec.rs`、`abi/rustside/f0_text.rs`（收据见§8/§9）。

**有界全链前探（最长 producer→consumer，三态闭）**：正例=`use.rs`源→auto importer→生成shim→rustc构建单元→双真实消费者——(a) AV腿：共享前门→MIR→stub表→真C ABI调用（rust_f0 5 E2E）；(b) native腿：native foreign lowering→caller COFF→rust-lld+import lib→真机器码调用（`f0_native_machine_code_consumes_the_same_auto_binding`：进程exit=9）。非法近邻=ghost符号manifest在**manifest校验**处具名拒绝（`foreign.symbol`，无caller字节）；V-F0-REJECT其余近邻（tamper身份/UTF-8/stale generation/panic）已在rust_f0闭。失败恢复=provider body编辑(+1)→新digest新代次→同入口重编译/重链接→exit=10（native腿）+ AV腿identity测试。

**有界策略比较（两处二选一，实测裁定）**：panic通道 (i)`C-unwind`跨边界 vs (ii)shim内catch+导出标志——实测(i)跨DLL unwind宿主不可catch（`Rust cannot catch foreign exceptions` abort）→**冻结(ii)**。native链接 (i)staticlib vs (ii)cdylib+import lib——(i)的std staticlib携带VC CRT符号与本仓kernel32-only PE链接线冲突→**冻结(ii)**（DLL自带静态CRT，exe仅kernel32+import lib，装载器解析）。

**新域资源/预算表（冻结，边界反例已测或具名）**：

| 域 | 计量/上限 | 超限分类 | 边界证据 |
|---|---|---|---|
| AV外来调用输入 | 单调用视图≤65536B | 拒绝（`foreign.call-bytes`，callee前） | rust_f0 65536/65537 |
| AV外来计费 | 每run≤1048576B（execute_foreign复位） | 拒绝（`foreign.run-bytes`） | 单测16×65536满+1B拒 |
| 构建单元 | rustc cdylib闭包：deadline 120s、捕获1MiB/流 | infra失败（具名错误文本） | 失败路径返回Err（缺工具=环境infra，不充语言负例） |
| 装载 | LoadLibraryW句柄，Drop即FreeLibrary；缺导出即拒 | infra失败 | loader源码+ghost拒绝 |
| stub表 | 身份=(provider,symbol,content_version,form)+generation新鲜度 | 拒绝（`foreign.signature/unknown-symbol/generation`） | 单测form负控+tamper E2E |

**COVERAGE 责任映射（六路径+原A2C，未交不升状态）**：

| 路径 | 现状 | 闭合owner（case） |
|---|---|---|
| AV/native | F0已交（AV腿+native probe） | T-03生产consumer接线；T-04汇总 |
| AV/Rust | F0已交（冻结provider） | T-03/T-12扩真实库；T-15 |
| AV/C | 未交（C importer前无自动C来源） | T-10 importer→T-12接AV→V-COVERAGE C行 |
| AC（native消费） | 938 manual已交；auto binding native腿已由probe交付 | T-03真实consumer；V-COVERAGE AC行 |
| A2R | 未交（`a2r.foreign-unsupported`仍拒F0域外） | T-12 AXR0腿（V-AXR0-RUST） |
| A2PY | 未交 | T-12消费1042接缝；T-14分发；T-15实跑 |
| A2C（原四端） | 938 manual已交 | T-12 auto形态并集 |

**LOAD 责任映射**：1043正式来源archived已核；FFI provider消费其来源后缀装配/#in namespace在T-03/T-12腿接入（复用同一Auto适配源码+真实foreign调用，不另造av.c/av.rs loader）；V-LOAD按其owner行在最终work汇总（T-04）闭，独立review核AC-LOAD——本冻结不减原范围。

**具名 unknown（有 owner，不漏计）**：跨rustc/双身份运行腿（T-11/T-14，V-ABI-*）；C真实预处理/sysroot闭包（T-10）；A2PY最小接缝（T-12←1042）；1043来源后缀/#in消费（T-03/T-12）；异步完成/取消端口（T-08/T-13）；GrantSet/owner授权账与generation+owner+签名每调用重核的全维（T-13，F0已交generation+签名+form维）；双向export/回调重入（T-08/T-13）；SQLite/serde_json真实库（T-15）。

### T-09 冻结记录（2026-11-14，AC-07/V-09 + ABI-COMPAT/INT-1026 字段冻结）

**工程入口（use.rs 腿已交）**：`open_project_session(project_root, provider, generation)`（ace-abi rust_provider）——源内 `use.rs <provider>` 解析到工程文件 `<root>/<provider>.rs`（单段 [a-z0-9_]+；多段/后缀装配归 T-12 消费1043）；binding/shim/构建单元/manifest 全部在 open 时从该文件派生（V-09"删手写manifest"由构造性满足并断言：工程内不存在 manifest 文件）；具名拒绝 `rust-import.provider-name/provider-missing/provider-read` + importer 域拒绝。永久正反例：`f0_project_import_runs_rejects_and_recovers`（真导入跑9 → 坏注解 `rust-import.domain` 阶段拒绝 → 缺文件 `provider-missing` → 修复后同入口恢复9）。use.c 工程腿已随 T-10 首切片落（CST `use-c-item`，text-record profile；resolve 按 lang 校验 manifest 家族前缀 c/ vs rust/）。

**真实库/全特征 required 表（冻结，owner 到任务）**：

| 行 | required | 最低真实样本 | owner |
|---|---|---|---|
| C 标准头 | 函数/record/enum/pointer/callback/opaque/常量/inline shim、宏/typedef/ABI布局/target/sysroot | string.h strlen 线（V-DETAIL-01 旧行）+ T-10 全闭包 | T-10 |
| C 真实库 | SQLite 钉版本公开 API | sqlite3 open/exec/step/close 最线 | T-15 |
| Rust 公开 API | String/Vec/Result/borrow/析构/panic/线程/future 边界、泛型实例 | serde_json from_str/to_string/Value | T-11/T-15 |
| 旧用法特征 | varargs/union/bitfield、traits/宏/泛型按真实用法逐行 | ffi_dual/020 dep traits generics 旧行 | T-10/T-11 |
| 默认自有库边界 | ptr+len/CStr/String 入出参、NUL 拒绝/回读/allocator 销毁 | V-NATIVE 五字节行 | T-10/11/12/13/15 |

**ABI-COMPAT 字段冻结（T-14 实现清单）**：BuildIdentity = {provider源+shim源 bytes-digest, crate解析/lock/features/cfg/实例集合, 生成器身份(auto-bindgen rust_import 版本), rustc完整身份(--version+host), target/SDK/linker/flags/native输入}；RuntimeCompatibility = {目标ABI profile(autoabi.win-x64.v0.1-candidate), OS-arch-env/位宽/端序/必要ISA, descriptor/ABI域版本, required符号/签名/布局(=ManualManifest facts), 接口语义/ownership/错误/线程合同, 运行依赖(CRT等)}；ProviderArtifact/Selection = {工件hash, BuildIdentity, 来源与pin/lock策略, 每运行授权+provider代次}。V-ABI-CROSS 双rustc fixture 计划：本机 stable 工具链 + 第二工具链（AC741_RUSTC 覆盖变量，缺第二工具链=pending 不充PASS）。

**INT-1026 最小字段冻结（T-11/12 实现清单）**：受检 tag 参数帧字段 = {receiver tag, positionals[](每元素 tag+data+byte_len), kwargs[](name+tag+data+byte_len), 帧物理投影版本}；返回 = {小标量 | opaque引用 | 结构化错误(status+named)}；借用/寿命 = 输入 noescape borrow、宿主保留副本在调用返回前完成、输出 owner=host；资源 = 复用 foreign-contract ResourceSig/HostAuthority 单一账本（generation+owner+alive/pin/drop），不建第二 Python 账本。

源调用经原计划公共use.rs前门、Rust元数据/版本化注解生成binding和C-compatible shim，再由唯一verify得到签名/borrow/权限/效果事实，AV消费通用foreign指令与stub。物理边界用目标C ABI的data+byte_len，不跨界暴露Rust str胖指针/String/Vec/trait对象布局；shim内部调用Rust &str函数。先覆盖同步、同线程、noescape借用；宿主若要持久显示文本须在调用返回前复制为自身拥有数据，不能保留AV指针。AV执行Rust函数字节码并非本目标，Iced仍由rustc编译。

拟永久Rust provider与Auto调用源位于本包既有abi/av测试owner内（T-F0-I创建，具体模块/target与公共use.rs实际拼写先冻结）；Rust importer新增模块拟`crates/auto-bindgen/src/rust_import.rs`，复用foreign-contract/ABI/store，不另造FFI世界。generic positive API应能观察输入字节长度/返回值，至少含独立Rust结果对照与非UIconsumer；UI provider函数由980持有，仅消费同一通用入口。

F0资源域：单调用UTF-8最多65536字节、单session保留输出最多1048576字节（本切片候选上限，原域不扩）；分配/复制前核准，合法边界与超一字节分别验证。错签名/ABI/provider代次/缺权限在dispatch前拒绝且callee_count=0；合法空串/中文/嵌NUL按长度传递。Rust panic在边界转独立宿主失败，不允许unwind穿AV或当作正常i32；abort保受控进程失败而非语言负例。session销毁使旧provider授权失效，加载后的函数与库owner保活到调用结束；不承诺动态热卸载或跨线程回调。

A2R并行腿归原T-12/AC-10的首个简单签名切片：复用F0的自动binding/共享事实与现有a2x-rust/store，把目前a2r.foreign-unsupported的该有限形态接到生成Rust adapter或C兼容shim，真实编译并调用同一Rust函数；独立证明参数/错误/borrow与AV同合同，禁AV/旧VM解释provider仍执行。该腿按已有phase fold独立复核/正式落主后交980/RUST，不等T-12所有四端或967全包。V-AXR0-RUST是原T-12的必需证据，F0-R/D只要求AV的V-F0-*，AV首窗口不反等A2R；完整T-12和M2-A仍须全原四端/资源义务。验证使用已有构建consumer，不反等978/B0。

F0不依979/980最终结果或969/978新入口：用现有源码编译与AV执行API自证，供969/E0和980/HELLO消费；GUI初始化/文字投影不进967。新增T-F0-I/V/R/D是原T-09/11/12/13/14所需接口的先交实现与证据，原任务后续复用并扩全范围，不重复实现、不提前勾整项。T-F0-D正式交接后下游可消费，不等967整Plan归档。

早期切片是本Plan M2-A内部交接，不新增ACE里程碑或Plan终态；只在同一权威树按owner流程独立复核、正式落主并记source/接口/grammar/ABI/provider及真实结果后供下游消费。同Plan尚有原完整义务时不归档/删树/勾整包完成。frontmatter保最终依赖并集，早期切片只等本节最小接口；同一文件单写，不以分支reviewed/WIP当交付。最小切片通过不关闭原AC、完整Rust FFI/全UI/RG-A03/M2A-08；不新增对980/981/985整包或AGE联合门的反向依赖。

### 默认自有标准库与FFI边界（2026-11-12当前）

依据[Design56 §0.2](../design/56-ace-m2-ffi-and-native-stdlib.md#02-默认auto系统库可选ffi与字符串边界2026-11-12当前)。完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。

本包验收：空串/中文/ab\0cd五字节在长度接口原样往返；CStr不适用内嵌NUL时具名拒绝，越界/无长度/错allocator/unwind控制拒绝且资源正确回收；四端各对独立expected。 原任务T-10,T-11,T-12,T-13,T-15承接；不新增实现任务，不以新义务扩大M1/F0或解除945–948的M1-TAG。

### 具体输入、类型/代码设计与旧用例映射（2026-11-12，r8当前）

**具体输入与期望。** 原件[crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at](../../crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at)（下列节选L3–L8；完整源/模块/资源闭包仍是验收输入，不把截断程序当可运行fixture）：

```auto
use.c <string.h>

fn main() {
    let len = strlen(c"hello")
    print(len)
}
```

本包所需独立结果：旧显式use.c strlen(c"hello")需求为5；新F0非UI Rust输入A中😀加NUL的长度9/空0必须真实导入/生成shim/调用，不让strlen截断定义Auto字符串。 本轮未运行；这些断言是实施目标，不是已观察PASS。

**类型/代码设计。** 公共C/Rust导入源/真实metadata→自动binding/C兼容shim→共享foreign事实→四端adapter→真实外部callee。沿已有owner扩展，不因下面的名字另建平行模型/执行器。字段草图为拟设计，不是现存API或可编译声明；等价复用当前类型允许在原有界决定任务冻结，不能改变字段所表达的不变量：

```rust
// Proposed semantic fields; not existing/compiled API.
struct ForeignCallContract { signature: SigRef, abi: TargetAbi, ownership: BorrowContract, effects: EffectSet, provider: ProviderId }
struct ProviderLease { generation: Generation, owner: OwnerId, grants: GrantSet }
```

| 实际代码owner | 本包具体职责与改面 |
|---|---|
| [crates/auto-bindgen/src/extractor.rs](../../crates/auto-bindgen/src/extractor.rs) | 现有预置C表，不能冒完整Rust importer |
| [experimental/ace/crates/foreign-contract/src/typed.rs](../../experimental/ace/crates/foreign-contract/src/typed.rs) | 共享签名/资源/borrow事实 |
| [experimental/ace/crates/abi](../../experimental/ace/crates/abi) | 独立ABI oracle及目标适配 |

**状态、错误与资源。** 错签名/代次/授权/ABI先拒callee=0；泛型/String/Vec/Result/panic/callback/future按原完整FFI集逐项关闭，不能F0小例冒全库。 借用返回前宿主复制，allocator/deleter匹配，在途lease保活，cancel/late/最后owner不越代；默认Auto路径不隐式libc。 沿原数值宽度/长度字符串/平台适用性/受管预算，不从Rust/C默认行为猜Auto语义，不以timeout/缺工具冒语言反例。

**旧输入到新检查的映射。**

| 原源码/测试/研究来源 | 保留的公开义务与新ACE验证 | 原期望/证据形态与读取身份 | 新ACE检查映射与原owner任务 |
|---|---|---|---|
| [crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at](../../crates/auto-lang/test/vm/17_cffi/001_strlen/strlen.at) | 显式外库CStr互操作，不是默认libc依赖 | 无邻接expected；沿原loader/合同核，不补假绿；SHA256前16 `4af83cfa91c6ba9c` | 拟ace.m2.p967.source01：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [crates/auto-lang/test/ffi_dual/020_dep_traits_generics/input.at](../../crates/auto-lang/test/ffi_dual/020_dep_traits_generics/input.at) | 真实Rust依赖/trait泛型需求 | `expected_output.txt`；SHA256前16 `4534b2dfd189cc03` | 拟ace.m2.p967.source02：run-av/run-native/run-a2r/run-a2c按原支持域；负控/资源见V-DETAIL-02/03；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |
| [docs/design/41-interface-metadata-and-foreign-abi.md](../design/41-interface-metadata-and-foreign-abi.md) | 复用已有ABI/shim研究，不另立FFI世界 | 旧合同/研究/协议来源；不当实际运行PASS；SHA256前16 `ef91fcbaaa7a60e1` | 原具名test/check或合同规则→V-DETAIL-01..04对应断言；T-01冻结完整ID，不当独立可运行源；原任务 T-02/T-03/T-08/T-09/T-10/T-04 |

上表拟ID为可追踪规划键，尚未注册/执行；已有同义case/check优先复用并绑定，不重复累计；原支持域外的新语义先具名裁定。直接复用旧源码/公开行为断言与有效独立expected；旧VM opcode/对象布局、旧A2R生成全文金样保留给旧实现，不照搬为新后端唯一语义oracle。错误期望、空expected、仅parse/emit、句柄/内部显示编码分别登记；已知旧bug与规范冲突具名裁定，不能静默改expected或将全旧行为当新规范。表中hash为本轮读取快照（同轮修订的Plan按最终revision再核），不是永久固定接口；表为已核代表/已有研究入口，不是覆盖上限；沿原完整分母逐条映射，不以几份例子关闭本包全部原义务。

未来永久用例沿现TestCase/ACE worker或当地既有测试owner补充；语言侧可在`crates/auto-lang/test/cases/ace-m2/`以本Plan具名子目录承载，UI/Man/Web/OS真实消费者仍在对应仓既有tests/fixture，不能语言fixture代产品。准确新文件/命令由下列有界决定任务登记，当前不生成源码/测试资产、不另建runner/checker。

**原任务内的决定和交接。** 真实metadata策略/注解边界、C/Rust工具依赖、callback/future和四端签名选择先冻结，保原FFI-M2-01..08与F0有限界限。 必须补齐旧case/test/check ID＋source/hash＋原断言/期望种类→新case/后端/独立oracle→原任务/AC/组合owner；已定部分按本节执行，只有具名未决交原有界决定任务，不再把全部设计留空。不存在等价旧用例的目标/安装/可信来源/性能不变量明确标“无旧同形”，复用现机制并补区分性正反控，不伪造旧覆盖。

本节由原`T-01`冻结来源/未决，`T-02`、`T-03`、`T-08`、`T-09`、`T-10`、`T-04`按各自原owner_stage接实现/验证（纯分析包只审计），对应AC-01/AC-02/AC-03/AC-06/AC-PHASE；原review/merge任务分别核结论与正式落地，不能用规划检查提前勾选。范围外能力回原owner，真实新增依赖须改DAG，不补假的整包反向等待。

### INT-1026：AV调用Rust Python provider的通用边界（2026-11-13，r9）

消费方：[PLAN-1026 r3 §5.4..6](1026-python-interop-standard-auto-syntax.md)，交接IF-CALL/IF-RES；964只消费中立引用/cleanup事实，本包不反向依赖964执行器、1026或PyO3。本节细化既有自动Rust FFI与通用资源义务，不要求C/Rust所有能力、async/四端整包全部完成后才交此最小扩展。

1. **同一FFI入口**：ACE受检Python操作合成对具体Rust adapter入口的调用，经本包自动binding/C兼容shim及AV foreign stub执行；Rust adapter内部才依赖PyO3并调用CPython。由本包T-09/11/12承接生成和签名/ownership事实，不以手写每库manifest/privileged数字ID或另造Rust importer替代；应用不手写use.rs/extern。
2. **IF-CALL**：receiver、源序positionals和kwargs采用本包受检带tag参数帧（逻辑签名/限额与物理投影版本一并冻结）。支持小标量/长度文本/受支持有限结构化值/opaque引用；C边界禁止Rust私有DynValue、String/Vec/trait/Py/Bound布局。帧中指针+长度的有效区/对齐/嵌NUL、borrow/noescape、结果owner及错误相位明确；来自guest的tag/kind/长度不直接授信，坏帧在callee前拒绝。不是把每个Python成员声明成静态C函数。
3. **IF-RES**：provider/session/entry代次、resource-kind、alive/owner/pin/retain/drop为本包唯一权威。adapter真实Py对象作为entry私有payload，不另建授权/生命周期账本；slot复用后的旧token拒绝。lookup校验并pin后，释放全局账本锁再执行可能回调/析构的外部代码；返回/异常/close释放自有pin与owner。通用资源kind/descriptor进中立foreign-contract，不进Python依赖。
4. **返回/错误**：小值或opaque结果、结构化错误、临时输入借用/宿主保留副本、输出allocator/destroy责任由T-12/13冻结；panic/错误不跨C unwind。T-14保provider/ABI/工具链/工件/权限身份，代码缓存不持地址/Py对象/活引用或旧授权。Python异常类名/动态getattr与Tensor shape/dtype等provider私有行为归1026，本包不建通用Python reflection引擎。
5. **接口不反等consumer**：用独立Rust generic provider（标量/text/opaque/kwargs/失败，绝非Torch）证明自动shim→AV真实调用、帧寿命/返回销毁/代次与错误。T-09核最小字段，T-11/12实现，T-13核资源，T-14核身份，T-04汇总。对应正式切片SHA/接口/独立trace交1026；不新增967→1026 depends_on，也不要求其训练先跑通。

**F0仍非此接口的完成证据**：F0仅text借用+i32/bool及有限失败；Python消费所需owned引用、float、通用tag帧与返回资源属于原T-09..14扩展交付。原C/Rust/Python现用桥、四端、自动导入/异步/安装交接完整义务保持，不能用此通用切片或1026 AV训练替代剩余迁移行。

### ABI-COMPAT：Rust库与shim的构建、分发与失效（2026-11-13，r10）

设计单源：[Design41 §5.1](../design/41-interface-metadata-and-foreign-abi.md#51-rust-provider的构建与兼容合同2026-11-13目标态)，M2交付约束见[Design56 §3.3](../design/56-ace-m2-ffi-and-native-stdlib.md#33-rust-provider的二进制兼容与独立分发)。本节为待实施规则；T-09冻结准确字段/版本和适用目标，不新造loader、manifest或缓存体系。

**B-01 构建单元。** 源码导入模式中，生成的shim和目标Rust crate及其Rust依赖进入同一个兼容Cargo/rustc构建闭包；泛型/trait具体化、布局私有访问和PyO3等Rust调用留在这一闭包内。源内容/lock/features/cfg、生成器/实例集合、rustc完整身份、target/SDK/linker/flags/依赖均可追溯。不能用新rustc直接给任意旧`.rlib`/Rust ABI `.dylib`套shim，也不能凭同rustc版本猜依赖/布局一定兼容；只有原构建匹配或受支持源码重建才进入这一模式。来源/MSRV/构建依赖不满足具名拒绝，不静默下载/更换工具链。

**B-02 对外产物。** 默认可独立分发的Rust provider为`cdylib`一类系统动态库：Rust库＋shim及Rust内部依赖封装成一个对外C ABI边界，配同版本的受检接口/工件/运行依赖清单。`staticlib`为原生链接分发，可需重新链接调用方，不能冒称无需重新链接的动态替换；其native依赖和LTO/bitcode/链接工具条件明确。普通`.rlib`/Rust ABI动态库不是跨rustc稳定分发格式。已有正确C API的预编译库可直接按其C契约消费，不要求多套一层shim。A2R可选择原T-12已规定的等价直接crate adapter，但那是同Rust构建闭包路径，不具有“跨rustc独立二进制”承诺，需单独证明等价。

二进制消费模式只使用包中已声明的C ABI导出/具体实例与受检接口元数据，不能为了新Rust API或泛型需求从旧二进制猜签名/寻找未导出符号；需扩面时明确转受支持源码重建或拒绝。源码模式构建期可以需要rustc/rustdoc，二进制包正常装配/调用不因此隐式需要它们。

**B-03 ABI合同。** `extern "C"`约束调用约定，跨边界record采用`repr(C)`或已证明等价的明确目标布局，字段递归满足FFI条件；这不会稳定String/Vec/trait/Result/Py等私有布局。标量/长度文本/opaque引用/错误/allocator/deleter/线程/回调/取消及不跨界unwind均属版本化合同。C ABI是**具体目标的**C/system约定，不是跨OS/arch/env/指针宽度/端序的通用二进制格式；Windows MSVC/GNU或不同OS runtime不能只看CPU同名就接受。

**B-04 两类身份分离。**

| 身份 | 必须描述的事实（语义字段，准确Schema由T-09复用现有owner冻结） | 使用规则 |
|---|---|---|
| BuildIdentity | Rust库及shim源码、crate解析/lock/features/cfg/实例，生成器、rustc/toolchain/target/SDK/linker/flags/native输入 | 产生物/构建缓存/证据的精确身份；真实输入改变失效受影响构建，不以C ABI兼容洗白旧构建缓存 |
| RuntimeCompatibility | 目标ABI profile/OS-arch-env/位宽/端序/必要ISA及OS/runtime版本；descriptor Schema与ABI域版本；required符号/签名/布局；接口语义/ownership/错误/线程合同；运行依赖约束 | 决定可否装配/重新准入/调用；不要求consumer的rustc等于provider的rustc，但必须独立验证实际提供者与需求兼容 |
| ProviderArtifact/Selection | 具体provider发布/工件hash、BuildIdentity、来源及选择策略/lock pin、每次运行授权、provider代次 | 兼容不自动替换精确pin，也不恢复旧授权；新工件须重新核验/装配，旧证据只覆盖原身份。运行manifest无裸地址/活句柄 |

`interface version`（业务/资源语义）、`ABI/descriptor version`（物理/编码合同）、`provider release/build`与`rustc version`是不同维度。不能用一个“版本相同”混合处理。仅按已声明的兼容规则接受增量；minor/新增符号/相同repr(C)标签并不自动证明兼容；required子集须实核，by-value字段/签名/ownership变化通常是破坏性变更，除非已有明确扩展/适配协议和独立证据。

| 变化/请求 | 构建或缓存动作 | 已分发provider的运行规则 |
|---|---|---|
| 源码构建请求的rustc/flags/生成器/真实依赖改变 | 新BuildIdentity；受影响库+shim重建，重验对应ABI/行为；旧构建缓存不得命中 | 若只是consumer工具链升级，未变化的已发布C ABI provider不因rustc标签不同就失效；核当前consumer需求/target/runtime/接口及选择策略 |
| Rust内部私有布局/实现改变，对外契约保持 | 库与shim共同重建，产生新工件/收据；不能重用旧provider行为证明 | 可在重新准入后与旧consumer兼容，仍需保持可观察语义/资源合同；不是自动热替换或跨代句柄兼容 |
| provider与consumer使用不同rustc版本 | 不能把双方Rust私有metadata/rlib混成一个构建闭包；同源构建按各自兼容工具链独立产出 | 同目标C ABI及接口/runtime/所有权等合同匹配时允许；runtime准入不得把rustc精确相等设成跨C边界前提 |
| OS/arch/env/ABI profile/指针宽度/必要ISA或runtime条件不兼容 | 按新target重建库+shim及相应consumer stub；更新布局/ABI证据 | 不兼容工件明确拒绝，不能重解释旧布局；不同target字符串只有具名兼容profile/证明才可能接受 |
| required符号/签名/布局、接口语义/错误/owner、ABI域/Schema不兼容 | 相应binding/stub/ABI证据失效；兼容适配需明确受检规则 | 业务调用前拒绝或选已验证adapter；不能先盲调再判断，缺版本/缺描述符不默认兼容 |
| 仅增加明确可选符号、保持旧required子集 | 新provider构建身份与证据更新；旧consumer绑定可按实际读取依赖复用 | 在明确兼容规则下接受，不能因完整manifest字节hash变化一律判接口破坏，也不能只看语义版本号放行 |
| CPython/系统库/CRT等外部运行依赖缺失或不满足 | 构建/分发清单记录其真实身份/要求；必要时另产兼容provider | 属运行依赖不满足而非“有C ABI就可运行”；已安装rustc不代所需运行库，反之运行不应隐式需要rustc |

**B-05 资源与升级。** opaque值只交其owner provider/代次处理，释放回原allocator/deleter。在途callback/future/borrow/handle pin旧provider；同C ABI兼容的新版本也不能直接接管旧对象。未经迁移/排空合同拒绝卸载或错误跨代调用；此处继承T-13，不新增无条件热升级承诺。运行前静态工件/目标预检和已知ABI bootstrap/握手分层，不把native库当安全沙箱或保证OS loader初始化无副作用；拒绝反例的零dispatch指业务入口调用次数。

**B-06 分发/职责。** 967 T-11产库+shim、T-12证明消费、T-13资源/装载、T-14交两类身份和manifest闭包，T-15给独立消费夹具及证据；978继续拥有构建装配/缓存/实际分发，986拥有隔离安装/升级。独立分发不是单DLL无依赖：系统库/CRT、CPU/OS约束和显式CPython依赖均列清；在受支持运行环境中，已打包provider的调用不依赖开发树、Rust源码或rustc/Cargo。不得为本节加967→978/986整包反等，也不接管那些owner的在途实现。共享store/现有manifest扩字段，禁止平行ABI缓存/第二loader。

### 后端与AV来源覆盖（2026-11-13当前）

以[覆盖设计](../design/stdlib/backend-provider-coverage.md)为本包当前增量：旧“四端”指AV/AC/A2R/A2C，不再代表全部required。新增公共能力最少AV/native、AV/C、AV/Rust、AC、A2R、A2PY六路径，原A2C及原平台/数值/资源/oracle不减。AV只一套解释器/API/语义，三来源共享Auto算法，仅适配必要原语；不能以配置名切换冒真实dispatch。

T-03/T-12扩947既有装配：同一AV的native/C/Rust来源绑定与资源原provider销毁；T-12消费1042的Python目标接同一Foreign IR及自动C兼容shim。T-14分发Python所需受检adapter/依赖，T-15实际Python与AV C/Rust调用。不能将C ABI等同C实现，也不能直接暴露Rust私有ABI。F0早期借用切片不新增Python/完整provider前置。

1042是最小Python接缝，不自动证明本包全部语言域支持；需要更大Python语义时消费977/PY已交能力切片或先具名补齐，不反等977整包。C/Rust provider复用967，native底座沿1039/1040原依赖。纯函数可说明身份/算法/输入/观察适用性后复用证据，资源边界需各有实际调用与原provider销毁；不机械跑全后端×全provider笛卡尔积。缺目标工具/runner/依赖保持pending，不能以emit、宿主fallback或同错结果关行。显式第三方引擎的依赖另列，native来源不冒所有第三方算法都已Auto重写。

### 新加载机制完成依赖（2026-11-13当前）

依据[设计§8–10](../design/stdlib/backend-provider-coverage.md)与[1043](1043-ace-stdlib-context-assembly.md)。公共.at＋at/c/rs/py来源后缀组成同一逻辑模块，运行后端另选；#in打开临时namespace并让is/if/for继承编译期属性，不扩平台后缀。最小层由1043先交，966继续完整CTE，不能反向等966整包。

完整C/Rust FFI provider消费1043装配/namespace合同，复用同一Auto适配源码并接真实foreign调用，不另造av.c/av.rs loader；F0早期切片仍沿原最小正式来源，不等1043整包。

本包整包/当前阶段required完成须核1043实际source/统一绑定/最小#in/增量恢复及所需真实后端的正式来源，不以文件存在、provider名字、旧归档或候选语法冒已交。948只审分母/去向，978验装配闭包，985验完整A；普通能力owner仍另交真实外库dispatch与OS/资源结果。原早期切片/调查范围不据此扩大，公共能力缺路径不勾整包完成。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/ace-migration/m2-host.md`（拟新增；可在review归并真实模块Spec，必须保持可追溯） | 未交付/有限切片→本包实际已验证支持、接口、拒绝及资源边界 | 防计划代现状 | AC-01..03/AC-06 |
| SD-02 | modify | `docs/specs/ace-migration/replacement-scope-and-evidence.md` | 仅口径→关联本包已验证新producer及未闭缺口；不改旧义务 | 替换追踪 | AC-03/05 |
| SD-03 | modify | `docs/specs/auto-bindgen/project.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08/AC-09/AC-10/AC-11/AC-12/AC-13 |
| SD-04 | modify | `docs/specs/auto-abi/project.md` | 当前有限能力→仅在本包实际验收后沉淀本次新增合同/来源/拒绝与资源边界；不改未交付为已实现 | Design56/本轮用户M2追加 | AC-07/AC-08/AC-09/AC-10/AC-11/AC-12/AC-13 |
| SD-PHASE | add | `docs/specs/ace-migration/m2-host.md`（沿原owner；仅实际交付后沉淀） | 单一M2口径→已验A/B能力/consumer及来源与旧新义务映射；未交B保持pending | 阶段不减原范围 | AC-PHASE |
| SD-HELLO | add | docs/specs/ace-migration/m2-host.md（沿原owner） | 未有早期交接→本切片实际已验接口/源/拒绝及剩余完整义务；交付后沉淀 | 最小Hello不冒全包完成 | AC-F0/AC-F0-REVIEW/AC-F0-LAND |
| SD-NATIVE | add | `docs/specs/ace-migration/m2-host.md`（沿原owner，仅真实交付后沉淀） | 默认实现选择未明→自有Auto/OS默认与显式FFI分账、长度字符串/目标ABI及来源验证 | LIB-M2-07/08 | AC-NATIVE |
| SD-INT-1026 | modify | `docs/specs/ace-migration/m2-host.md`（拟交付后沉淀） | 通用FFI/资源→已验tag帧/返回/统一账本、自动Rust shim与AV消费、ABI/元数据边界及F0区别 | 防重复建设/错误授信 | AC-INT-1026 |
| SD-ABI-COMPAT | modify | `docs/specs/ace-migration/m2-host.md`（实际交付后沉淀） | Rust shim一般描述→已验证库+shim构建单元、独立C ABI分发、双身份/target/接口/资源兼容与失效及拒绝边界 | 不把C ABI兼容与缓存命中混同 | AC-ABI-COMPAT |


| SD-DETAIL | add | `docs/specs/ace-migration/m2-host.md`（沿原owner，交付后沉淀） | 仅范围→实际已验输入/类型/状态/旧新check映射与限制 | 不以旧emit/规划代能力 | AC-01/AC-02/AC-03/AC-06/AC-PHASE |

### 覆盖规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-COVERAGE | add/modify | docs/specs/ace-migration/m2-host.md | 旧四端→新增A2PY、同AV三来源及真实逐操作证据；未交不升支持状态 | 用户最低覆盖 | AC-COVERAGE |

### 加载机制规范增量（待实际交付）

| delta_id | add/modify | docs/specs target | before/after | rationale | AC |
|---|---|---|---|---|---|
| SD-LOAD | add/modify | docs/specs/ace-migration/m2-host.md | 旧有限后端侧文件→本包实际消费1043来源装配/#in/目标依赖及正式来源，未交保持缺口 | 用户确定新加载机制 | AC-LOAD |

## 6. 测试设计

### 阶段适用验证（新增合同、未执行）

| Case | AC | 入口 | 独立判据 |
|---|---|---|---|
| V-PHASE | AC-PHASE | T-04及本包原固定组/真实owner入口，按§5阶段表选取 | 该阶段原required义务与新check逐行可追溯；普通源/独立expected、适用后端/平台/错误/资源真实执行；A仅RG指定产品，B全范围。删行/错producer/零选择/缺工具不能过；B-pending不改NA。 |


以下全部未执行；T-01落实具体源码与调用符号，T-02/03创建永久fixture，T-04闭合。原源与独立expected/现行语义裁定为oracle；旧引擎仅作为有版本的调查对照，不能以四端相等掩盖共同bug。

| Case ID | AC/invariant | 输入/区分性断言 | 真实入口与证据 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-02/03 | 跨会话/过期handle及错误签名调用前拒绝 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-02 | AC-02/03 | 合法非零handle成功 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-03 | AC-02/03 | cancel后late callback不能访问已关会话 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-04 | AC-02/03 | 原方destroy一次 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-05 | AC-02/03 | 重入不得持锁死锁 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-06 | AC-02/03 | 未授权恢复dispatch=0 | T-01将本行落实为真实源码/操作序列；T-04运行真实producer/consumer，记录值、trace、资源和诊断，错误路径配合法近邻 | work；本包支持矩阵 |
| V-INT-1026 | AC-INT-1026 / IF-CALL/IF-RES | 独立Rust provider经真实metadata→自动C兼容shim→AV stub：小标量/带NUL文本/opaque/kwargs合法形；坏tag/长度/ABI/过期代次；返回对象后错误及panic、临时输入回收后宿主保留值 | 原T-09/11/12/13/14接实现和独立Rust caller/oracle，T-04汇总；callee/析构trace证明合法路径及错形dispatch=0、owner/destroy一次且错误不跨C；不把直接Rust单测当AV证据 | work；无Python/1026前置，不扩F0 |
| V-ABI-BUILD | AC-ABI-COMPAT / B-01..03 | 真实Rust源码crate+生成shim共同构建；仅有不匹配旧rlib/Rust dylib、缺源码或不满足MSRV的近邻 | T-09冻结来源/工具链与准确入口，T-11构建闭包；独立Rust API oracle核结果，受支持源路径成功，不受支持预编译私有ABI/工具缺依赖具名拒绝；不能只手写extern或换compiler隐式拼接 | work；不承诺任意历史Rust产物 |
| V-ABI-CROSS | AC-ABI-COMPAT / B-02..04 | 同一冻结接口的provider分别以两个不同、均能构建该源的实际rustc版本产cdylib；AV宿主以其中一个/已知版本构建，并加载另一版本provider | T-11/12真实自动shim→AV/原已支持四端适用路径，独立C或Rust C-ABI caller核值/布局/错误/destroy；compiler标签不同但接口匹配必须成功；调用期禁rustc/Cargo/开发源码仍可运行。版本/target与实际binary均绑定收据，缺第二工具链不得充PASS | work；有限已证版本/目标，不宣称全部组合 |
| V-ABI-TARGET | AC-ABI-COMPAT / B-03..04 | 合法目标近邻及错OS/arch/env/ABI profile/位宽/必要ISA/runtime字段；实际工件头与清单交叉核验 | T-12/14按既有ABI/装配入口在业务dispatch前拒绝不兼容项，合法配置成功；改target必须使对应构建/布局证据失效，不能仅看arch或abi=C字符串 | work；已声明支持目标；加载器初始化非沙箱承诺 |
| V-ABI-IFACE | AC-ABI-COMPAT / B-03..04 | 冻结required符号/签名/布局/owner/错误协议；逐轴改变、缺ABI/Schema；明确兼容规则下仅加可选符号 | T-09/12/14；不兼容者业务callee_count=0、不能盲调；合法可选增量且required子集不变可重新准入。不得用相同版本号/外层repr(C)/完整manifest hash单独裁决兼容 | work；无现成adapter时必须拒绝 |
| V-ABI-CACHE | AC-ABI-COMPAT / B-04 | 源构建rustc/flags/crate私有实现/生成器/实际target变化与无关输入控制；已发布provider字节不变而consumer rustc变化 | T-14使用现有store/构建收据核真实key/编译事件与独立结果；真实构建输入变则失效受影响闭包，无关变化不重建；运行加载不能因consumer rustc标签变化强制重建不变provider。精确lock pin不匹配仍按选择策略拒绝/显式更新 | work；缓存与ABI兼容分账 |
| V-ABI-LIFE | AC-ABI-COMPAT / B-05 | ABI兼容v1/v2并存，v1有效opaque对象/borrow/callback pin，尝试交v2或提前卸载，再按原owner释放 | T-13独立provider分别计析构/业务调用；旧引用只交v1，错误跨代不dispatch，最后pin释放后才能卸载；同签名不证明对象内部可互换，不新增状态迁移承诺 | work；沿既有资源/异步适用条件 |
| V-ABI-PACK | AC-ABI-COMPAT / B-02/06 | 库+shim的cdylib/接口清单/依赖置于统一scope隔离消费根；移除开发树和rustc/Cargo访问；缺运行库/错manifest/坏工件控制；适用staticlib另测原生重链接 | T-14/15交978前的独立消费夹具真实运行、记录所有输入/工件/动态依赖；动态包合法环境可运行，缺依赖/错身份具名失败；staticlib不声称无需链接或任意LTO兼容。978/986全产品/安装仍由原owner，不以此fixture代最终安装验收 | work；准确路径和工具在T-09冻结，全部未执行 |

完整职责组拟定：`cargo t ace-host ace-abi ace-foreign-contract ace-av ace-native ace-a2x ffi`（组名已静态核test-groups.toml，未运行）。新包/用例必须由T-04注册进完整职责组；共享IR/语义修改加真实下游四端组，不能只跑前端。外仓产品在T-01记录当地已存在命令与必需环境，未冻结命令前T-02不得开始；不能以语言仓cargo替代UI/Man/OS真实产品验证。AGE按受影响文件/命名case与原单进程条件，全量AG留里程碑。
发现/执行/条件排除、准备/运行/清理/总时分列；零选择/缺工具/timeout/编译失败不是PASS。正式性能与窗口操作错峰，周期全量不变成每计划门。最终提交职责并集沿有效证据核验，不重复全仓。
独立review在同一worktree、实施停写后进行；本轮未派agent、未执行独立产品复审。

### 补充验证

| Case | AC | 区分性输入与预期 | 入口/证据 | owner/边界 |
|---|---|---|---|---|
| V-08 | AC-06 | 真实独立C caller调用ACE导出，ACE调用C provider各验证资源释放；卸库前callback pin仍保活；过期generation或错误ABI在执行前拒绝。 | 实际公开入口/独立oracle；T-08创建永久fixture并记真实命令，适用四端/产品分别观测 | work/T-08；不扩未批准平台 |

### 2026-11-09 新增验证（待执行）

| Case | AC | 输入与区分性断言 | 入口/证据 | owner/阶段 |
|---|---|---|---|---|
| V-09 | AC-07 | 删除手写manifest/extern源码仍可从原工程生成binding并运行；不存在库、缺元数据/工具、歧义符号及错所有权注解在正确阶段具名拒绝。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-09；最终验证汇总 |
| V-10 | AC-08 | 改变传递header/宏/packing或target产生正确binding；独立C caller/callee验证聚合传参/返回/写回/释放；缺include/错误调用约定拒绝，static inline/常量真实消费。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-10；最终验证汇总 |
| V-11 | AC-09 | serde_json公开API自动导入并处理成功/坏JSON；独立Rust测试对值与错误；自有泛型/owned/drop计数fixture验证一次释放、借用拒绝、panic转换/隔离及不可跨线程对象拒绝。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-11；最终验证汇总 |
| V-12 | AC-10 | 相同C/Rust工程四端实际调用并正确写回/返回/释放；拒绝旧VM/AST fallback和错误签名；分别观测各端dispatch而非仅比较结果文件。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-12；最终验证汇总 |
| V-13 | AC-11 | 独立C caller调用ACE导出并反向调用；错ABI/过期generation在调用前拒绝；callback重入无锁死，cancel后late completion不发布、drop一次，在途卸库被阻/延迟，合法解除后可卸载。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-13；最终验证汇总 |
| V-14 | AC-12 | 分别修改传递header、Cargo.lock/features/cfg、target、ABI或工具链应失效；改无关Auto body不重做importer；坏缓存/生成失败保旧，修复恢复；异根可复现身份且不依赖开发绝对路径。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-14；最终验证汇总 |
| V-15 | AC-13 | 从干净工程仅配置依赖和公共use导入，自动生成并实际读写SQLite/解析JSON；去掉生成缓存重建仍成功；错版本、缺库、非法参数、取消及资源退出独立断言，无手工签名替代。 | 本包§4真实owner下新增永久fixture，公共source/worker/实际产品或仓外安装入口；独立expected、资源trace、命令/子终态、身份分别记录 | work/T-15；最终验证汇总 |

新增触面职责组：`cargo t auto-bindgen ace-source ace-semantics ace-artifacts ace-foreign-contract ace-abi ace-av ace-native ace-a2x ace-a2c-abi`，与原§6组按实际修改取必要并集；T-01核真实参数，新增用例接固定组，不减原平台/ignore/压力条件。948若仅文档审计则只做目标静态检查，真实探针沿原指定组；安装/外库工具和外仓产品必须另核真实命令，语言组不替代它们。零选择/未执行/缺环境不充PASS；本轮没有运行这些测试。

### F0永久验证（由T-F0-I创建，当前未执行）

| Case | AC | 输入/入口及独立关闭oracle | owner/边界 |
|---|---|---|---|
| V-F0-SOURCE | AC-F0 | 普通Auto use.rs源→自动Rust元数据/binding/shim→Checked→AV真调；删手填manifest仍可从冻结Rust源生成。`A中😀\0`按UTF-8长度9、空串0，与独立Rust测试及字节oracle一致；至少另一个非UIconsumer复用 | work；非泛型借用UTF-8，非完整Rust importer |
| V-AXR0-RUST | AC-10 | 同一普通Auto import源经新A2R真实生成/编译/调用Rust函数，输入字节9/0与独立Rust结果一致；禁AV/旧VM执行provider仍运行，错签名/ABI同样拒绝且无dispatch | work/T-12；并行AXR0腿，拟职责组补ace-a2x及真实受影响native，emit-only不PASS |
| V-F0-REJECT | AC-F0 | 错签名/ABI/授权/provider代次/非法UTF-8在准入拒绝，callee_count=0；合法非零身份仍接受；panic/缺工具/abort分别保持host/infra失败 | work；不把timeout当语言诊断 |
| V-F0-LIFE | AC-F0 | 65536/65537字节、保留账边界；调用后guest文本释放/替换而宿主已复制值保持正确，20轮创建/调用/关闭无owner增长；失败保旧、修复后同入口下一次正常 | work；不承诺异步/热卸载 |
| V-F0-IDENTITY | AC-F0 | 改Rust函数/feature/shim/ABI身份使应失效产物重建；普通Auto body修改不重编未变provider，真实dispatch/计数与失败修复可核 | work；T-F0-I冻结新case/现有target参数，拟职责组cargo t ace-source ace-semantics ace-foreign-contract ace-abi ace-av ace-artifacts，实际修改a2x/native才补原组 |


### 默认库/外库与字符串验证（新增、未执行）

| Case | AC | 实际入口/输入 | 独立结果 |
|---|---|---|---|
| V-NATIVE | AC-NATIVE | T-10,T-11,T-12,T-13,T-15，沿本包既定四端/固定职责组与真实owner入口补永久case；默认provider与显式外库各一腿；源码/符号/链接/运行依赖与NUL边界对照 | 空串/中文/ab\0cd五字节在长度接口原样往返；CStr不适用内嵌NUL时具名拒绝，越界/无长度/错allocator/unwind控制拒绝且资源正确回收；四端各对独立expected。 |

### 本轮细化的可区分验证（拟实施；纯分析包只核推理/来源）

| Case ID | AC/原任务 | 输入→入口→独立关闭oracle | 边界/错误近邻 |
|---|---|---|---|
| V-DETAIL-01 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 旧显式use.c strlen(c"hello")需求为5；新F0非UI Rust输入A中😀加NUL的长度9/空0必须真实导入/生成shim/调用，不让strlen截断定义Auto字符串。 | 按上述原完整输入/真实consumer验证；原旧检查继续保留，未运行不PASS |
| V-DETAIL-02 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 错签名/代次/授权/ABI先拒callee=0；泛型/String/Vec/Result/panic/callback/future按原完整FFI集逐项关闭，不能F0小例冒全库。 | 配合法相邻输入；infra/timeout/缺工具不充语言负例 |
| V-DETAIL-03 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 借用返回前宿主复制，allocator/deleter匹配，在途lease保活，cancel/late/最后owner不越代；默认Auto路径不隐式libc。 | 源/ABI/target/配置变化按实际读取失效；失败与修复/关闭各观察，不自报布尔 |
| V-DETAIL-04 | AC-01/AC-02/AC-03/AC-06/AC-PHASE；T-02/T-03/T-08/T-09/T-10/T-04 | 按本节旧新表逐原case/check与完整分母对账；新后端/新不变量有独立oracle及原件/来源 | 删必需检查、错candidate/source或仅emit时必须被发现；该包非运行职责不得冒产品PASS |

真实入口沿本Plan原§6/当地owner命令，原固定组/平台/ignore/正式性能/AG定向规则不变；新增fixture只由原实施任务建立。发现/执行/原条件排除、准备/运行/清理、首次红/重放与缺证分列。新表不代原case/check、不新增任务号、不让原review/merge阶段提前通过。


### 后端/provider验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/终态 |
|---|---|---|
| V-COVERAGE / AC-COVERAGE | 同一公共Auto源码及独立expected，在六路径/原A2C按原OS表实跑；配置C实跑native、缺Python/helper、错provider销毁、静默fallback、emit冒run与NUL/部分IO/错误/资源适用反例。 | 各路径真实producer/运行/dispatch身份、值或字节/副作用和资源结果正确；负控拒绝，合法邻例通过；不能只比较后端相等或改required为NA。 |

实现/验证沿本包原owner任务、现TestCase/ACE worker与固定职责组；新增Python接缝按真实修改选`ace-a2x`/`a2py`/`auto-test-case`，AV来源边界选现`ace-foreign-contract`/相关AV、host、stdlib组，T-01冻结精确并集，985按T-A01/02。缺平台环境不伪造支持，不把正式ace-perf附加日常收尾；既有完整职责/必要消费者保持。

### 加载机制验证（新增、未执行）

| V / AC | 输入与入口 | 独立观察/owner |
|---|---|---|
| V-LOAD / AC-LOAD | 本包普通公共库源/所选provider与#in平台模块；错误后缀/缺或重复实现、错target/namespace、未选SDK缺失、错误producer/旧配方冒新、修改选中与未选输入后恢复 | 本包真实消费者及独立expected/依赖/源身份；负控在调用前拒、失败保旧，未选库不绑定链接。948验去向不代运行，978验候选闭包，985核正式A分母；能力包验证真实执行而非仅文件选择。复用1043身份适用证据，不重复跑其整档代本包；原最终work汇总，独立review核AC-LOAD。 |

## 7. 验收标准

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 本包完整支持集、符号/入口/平台/依赖及资源合同冻结，所有unknown有owner且不能漏计 | T-01逐行映射Design55/分母与实际源码 |
| AC-02 | work | §5能力经真实源码/工程入口执行，§6所有正反例满足独立值/效果/资源oracle | T-02..04；每个V行记录真实命令/输入/结果 |
| AC-03 | work | 生产consumer实际接线，无legacy fallback；变化/失败/恢复/权限与旧支持格保持 | T-03/04；源与producer身份及真实dispatch证据 |
| AC-04 | review | 独立核全部AC/V/规范增量、遗漏/延后/workaround、fmt/warning/debug，结论绑定revision/SHA | T-05复审；未闭项不得pass |
| AC-05 | merge | 已验证知识落模块Spec，来源/依赖一致，归档与所有输出路径清理完成 | T-06/07真实收据，不把正确性留到merge |
| AC-06 | work | 双向ABI与异步完成端口的全部义务完成，V-08有真实结果，无缺项或循环交付 | T-08及最终work验证；原review任务同时审核 |
| AC-07 | work | FFI-M2-01自动公共入口可用，用户不手抄签名/布局，依赖和不确定所有权具名诊断；全部批准特征有关闭owner。 | T-09/V-09；原最终work验证和独立review同时核本项 |
| AC-08 | work | FFI-M2-02依独立C ABI oracle闭合，不能regex抽签名/宿主布局冒目标；required特征缺口阻本包。 | T-10/V-10；原最终work验证和独立review同时核本项 |
| AC-09 | work | FFI-M2-03不调用任意Rust私有ABI；具体实例与资源约束可追溯；不能要求用户为每个库手写ABI adapter。 | T-11/V-11；原最终work验证和独立review同时核本项 |
| AC-10 | work | FFI-M2-04四端公共源码实际执行并满足独立值/效果/资源oracle，不以emit或新语言源码等同库FFI。 | T-12/V-12；原最终work验证和独立review同时核本项 |
| AC-11 | work | FFI-M2-05/06及原AC-06共同满足；在途callback/析构/future保活，卸载无悬空/双释放，967不等待968完整await。 | T-13/V-13；原最终work验证和独立review同时核本项 |
| AC-12 | work | FFI-M2-07精确失效、失败保旧/恢复和生成物身份闭合，不存地址/授权，无新全局缓存。 | T-14/V-14；原最终work验证和独立review同时核本项 |
| AC-13 | work | FFI-M2-08真实库自动体验及全部required子行关闭；最低样本不能抵掉旧消费者/特征缺口。 | T-15/V-15；原最终work验证和独立review同时核本项 |
| AC-INT-1026 | work | IF-CALL/IF-RES按INT-1026实际生成/执行，单一账本与Meta边界、参数/返回/错误/借用/destroy有V-INT-1026；不导入PyO3泛型API，不反等1026，F0不冒充接口已交 | T-09/11/12/13/14与T-04；原T-05独立复审 |
| AC-PHASE | work | §5任务阶段/实际前置及原义务→新检查/独立oracle映射齐；对应阶段required闭合，B开工已核A正式收据，无scope缩减 | V-PHASE、T-04；原独立review复核 |
| AC-F0 | work | F0自动源码导入/共享受检FFI/AV真实Rust调用及拒绝、文本寿命/费用/恢复闭环 | T-F0-I/V；V-F0-* |
| AC-F0-REVIEW | review | 同树独立复核F0普通源码、真实shim/ABI/dispatch及全部正反控，绑定revision/SHA | T-F0-R |
| AC-F0-LAND | merge | F0已审接口正式落主并交source/ABI/provider/测试来源，原完整FFI义务保留 | T-F0-D |
| AC-NATIVE | work | 空串/中文/ab\0cd五字节在长度接口原样往返；CStr不适用内嵌NUL时具名拒绝，越界/无长度/错allocator/unwind控制拒绝且资源正确回收；四端各对独立expected。 | T-10,T-11,T-12,T-13,T-15 / V-NATIVE；原独立review核新增义务 |
| AC-ABI-COMPAT | work | B-01..06和全部V-ABI-*成立：真实库+shim构建、跨rustc C ABI消费、target/接口/运行依赖准入与双身份失效、独立包/原方释放；保旧全部AC和F0范围 | T-09/11/12/13/14/15及T-04；原T-05独立复审，不留到merge补正确性 |

- [ ] **AC-COVERAGE / work**：§5本次覆盖责任及V-COVERAGE关闭；新增公共能力六路径与原A2C、独立oracle/真实来源/资源证明齐全。948仅验调查分母不签产品，978验装配，985验A完整性；原独立review逐行核，原生四格不代全覆盖。

- [ ] **AC-LOAD / work**：§5本包新加载机制责任和V-LOAD关闭，正式1043来源/独立结果与缺口分账齐全；原native底座/六路径/旧A2C和阶段不减，最终work与独立review逐行核。

## 8. 执行步骤

当前共19任务，0完成。下面是唯一当前执行清单；原ID/AC保留。任务按A/B分组，组内按§5实际前置选择就绪任务，不把行号当依赖。完整Plan只有原终态归档，A部分完成不清唯一树。

### M2-A：能力与必要真实验证

早期任务可按§5最小前置先行，不要求T-01完整分母冻结已完成；各I自行冻结本切片输入/接口/命令，不提前勾T-01或其它原整项。以下行序不是额外前置。

- [x] **T-F0-I｜M2-A｜owner_stage: work｜AC-F0**：不等T-01全分母；按§5核已交基础、决定公共元数据策略/支持域/调用错误合同，打通通用AV foreign与自动Rust binding/shim，创建V-F0-*及实际命令。 [✅ 2026-11-14]
  证据：`crates/auto-bindgen/src/rust_import.rs`（importer+shim生成+manual投影，9单测绿）、`experimental/ace/crates/abi/rustside/f0_text.rs`（冻结provider）、`experimental/ace/crates/abi/src/rust_provider.rs`（cdylib构建+LoadLibrary装载）、`av-core/src/foreign_exec.rs`（stub表/准入/预算/panic标志协议，3单测绿）、`av-core/src/av.rs` ForeignCall臂+`execute_foreign`、`av-core/src/abc_verify.rs` `verify_abc_foreign`、CST `use-rs-item` 规则+投影、resolve use.rs 校验；冻结决定见§5 F0节。

- [x] **T-F0-V｜M2-A｜owner_stage: work｜AC-F0**：T-F0-I后运行全部V-F0-*与冻结完整受影响组/必要消费者，记录独立Rust oracle、dispatch、权限/资源/工具身份及恢复，未执行不PASS。 [✅ 2026-11-14]
  证据：`ace-abi/tests/cases/rust_f0.rs` 5 E2E 全绿（V-F0-SOURCE：use.rs源→自动binding/shim→rustc cdylib→共享verify→AV真调 "A中😀\0"=9/空=0+独立直dispatch oracle同值；V-F0 bool/fallible 65+空输入Trap ForeignStatus；V-F0-REJECT：tamper身份/非法UTF-8/stale generation admission拒绝 callee_count=0、真panic→`foreign.panic` host失败后会话可用；V-F0-LIFE：65536/65537字节界+20轮无增长；V-F0-IDENTITY：Auto body编辑共享声明身份、provider body编辑新digest新代次+1值、接口facts不变）。受影响组：auto-bindgen 9绿；av-core lib 62+conformance 44+foreign_resources 11 绿；ace-core lib 57+conformance 461 绿（differential 927 一致）；source-query 21、semantics 98、artifacts 19+1ignored、av 11、native lib 13 绿；ace-abi conformance 全量复跑收据见§9。

- [x] **T-F0-R｜M2-A｜owner_stage: review｜AC-F0-REVIEW**：T-F0-V的AV全部V-F0-*齐后停写，同一树独立审计F0源码→真实Rust调用/资源/负控；finding返I；只pass本切片。 [✅ 2026-11-14]
  独立审计（新上下文agent，冻结提交 f5bf3cd50）：**PASS，0 BLOCKER/0 MAJOR，5 MINOR**。逐项核验A–I（合同行忠实性/每次调用准入完整/负控区分力/预算语义/panic通道/noescape借用/前门边界/卫生/遗留物）全过；G前门与H卫生为NOTE。5 MINOR 全数修复于 d77e369b2：#1 fallible None→最小命名status（shim与manifest永不背离，+单测）、#2 装载器缺导出路径双FreeLibrary、#3 预算语义裁定入§5（每run计费输入字节取代保留输出字面读法）、#4 panic通道文档改为冻结标志协议、#5 form/ABI负控补齐（bool形声明vs i32 callable→foreign.signature，callee_count=0）。修复后 scoped 复验全绿（auto-bindgen 10、av-core foreign_exec 3、rust_f0 5）。绑定：审计基线 f5bf3cd50，修复后 d77e369b2。

- [x] **T-F0-D｜M2-A｜owner_stage: merge｜AC-F0-LAND**：T-F0-R pass后按既有phase fold正式合入并沉淀已验F0接口，交969/E0、978/B0和980/HELLO；保同树/resync继续全部原任务，不归档或冒完整FFI已交。 [✅ 2026-11-14]
  phase fold：plan-967-dev（f5bf3cd50+d77e369b2）→ master `4c1788a79`（--no-ff，无冲突）；master 侧构建核对（auto-bindgen + ace-abi 绿）+ rust_f0 5 E2E 主检出复跑绿；spec 沉淀 `docs/specs/ace-migration/m2-host.md`（F0 已验接口表+未交付边界，SD-HELLO/SD-01 起步）+ 计划收据 `8d66dd7f7`；worktree 保留并 resync 至 `8d66dd7f7` 继续原任务。下游（969/E0、978/B0、980/HELLO）消费来源：上表 + 冻结测试。



- [x] **T-01｜M2-A｜work｜AC-01/AC-PHASE**：核依赖与§4路径，完成§5有界全链前探、语义/支持/owner/命令冻结；输出决定摘要入本Plan，不扩大或缩减范围。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 [✅ 2026-11-14 三腿交付：AV-C（真cl /LD DLL驱动AV道）、A2R AXR0（生成Rust经同一shim导出真编译真运行、域外具名拒绝保持）、A2PY ctypes（生成Python经ctypes CDLL真运行、typed A2xForeignStatus、域外具名拒绝保持）；V-COVERAGE六路径全实跑=AV/native✓ AV/Rust✓ AV/C✓ AC✓ A2R✓ A2PY✓（同一binding/shim/provider跨全部lane）；具名边界=1043来源装配腿（#in/后缀）仍按T-01冻结映射归后续LOAD收口（T-04汇总核）] [✅ 2026-11-14]
  决定摘要+冻结见§5"T-01 决定摘要与责任冻结"：锚点核验（resync树76431e6b9）、最长链前探三态闭（native机器码probe `f0_native_machine_code_consumes_the_same_auto_binding` exit 9/ghost拒绝/重绑定10 + F0 AV腿）、两处策略实测裁定（panic标志通道、cdylib+import lib native链接）、新域资源表、COVERAGE六路径+A2C owner映射、LOAD映射、具名unknown清单。

- [x] **T-02｜M2-A｜work｜AC-02**：在§4对应producer实现§5主能力与永久正反例；先共享事实/类型/资源合同，再接实际后端，不创建平行语义分析。 [✅ 2026-11-14 会话层+映射收尾；V-01..06执行证据按owner行在T-04汇总]
  交付：`ForeignSession`（av-core foreign_exec：open/bind/execute/teardown/rebind/adopt，per-run授权+账本复位，teardown对后继admission与已admitted模块双拒且dispatch计数冻结）+ `F0Session`/`open_f0_session`（ace-abi：build→admit→load→bind一体保活，Drop恰释放库一次）；永久正反例 `f0_session_lifecycle_success_teardown_unauthorized_rebind`（V-02合法身份真成功/V-01 stale双拒/V-06空绑定unauthorized callee=0/provider重绑定恢复）；既有桥逐项映射+旧数字ID兼容裁定入 `docs/specs/ace-migration/m2-host.md` T-02节（C桥→T-09/10、Rust桥→F0已交+T-11、Python桥→1042/T-12、数字ID只作兼容映射）。共享事实/类型/资源合同=938/939 foreign-contract+F0 stub表（无平行语义分析）；V-03/V-04回调全量留939 native道至T-08，V-05重入=foreign_exec单测。

- [x] **T-03｜M2-A｜work｜AC-02/03**：完成真实consumer、目标/装配/权限/增量/失败恢复接线，保原API行为；每个适用端对独立oracle，不以模型替运行。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。 [✅ 2026-11-14 生产consumer道交付；1043来源装配腿按T-01冻结归T-12接入]
  交付：**生产TestCase道**——deck（Auto use.rs源+注解provider源，manifest在worker内派生）→ `abi/rust-f0-certify-v1`（abi-worker新配方，legs av=进程内ForeignSession / native=caller COFF+import lib+真机器码）→ 固定组 `case-ace-m2-f0-rust-967`（6案：len 9/0、first-byte 65、rebound +1 body，av+native腿）PASS；配方注册贯通 worker dispatch/TestCase contract（exit oracle、final query、av|native legs、abi binding）/adapter routing/probe judge；生成构建单元导出 `ace_foreign_fail`（native fallible 传输，exit 70，同C侧语义）。目标=冻结plan常量；权限=F0粒度每调用admission（owner/grant维T-13）；增量=会话粒度接口身份复用+重绑定恢复（工件缓存增量为T-14）；失败恢复=rebuild/rebind/panic道全闭。LOAD腿：1043来源后缀/#in装配消费按T-01冻结映射在T-12接FFI provider目标时落地（V-LOAD按owner行T-04汇总闭）。回归：938套件PASS、auto-test-case 45绿、ace-adapters PASS、rust_f0 7绿、auto-bindgen 10绿。

- [x] **T-08｜M2-A｜owner_stage: work｜AC-06**：分import/export、调用方/被调用方、标量/聚合/资源与C/Rust/Python现用桥，核动态库装卸、原方allocator、符号/ABI版本。异步本包只交注册/完成/取消端口及受控真实provider；968消费端口实现语言await，967不反向依赖968。 [✅ 2026-11-14]
  交付：**异步完成端口**——中立合同 `foreign-contract/port.rs`（`autoabi.contract.completion-port/1`：PortValue/PortEvent+唯一 `completion_disposition` 规则表 publish/drop-late/unknown；cancel墓碑把迟到完成归为DropLate而非UnknownPort）；宿主注册表 `av-core CompletionPorts`（register/complete/cancel/teardown 绑会话generation——迟到完成可观察丢弃、绝不发布、绝不触已闭会话；每次状态变更是因果 PortEvent=消费者唯一oracle，968的await绑此、967不依968）；**受控真实provider** `rustside/f0_async.rs`（真cdylib、真OS线程、真C-ABI完成回调；raw cdylib构建/装载 helper——域外fixture是手写PROVIDER，手写禁止的是binding/manifest）。E2E（abi f0_async 2案）：真线程回调发布 i32:10；cancel+强制迟到=LateDropped；teardown后真线程in-flight完成=LateDropped。动态库装卸=RawLibrary Drop恰释放一次；V-08 C侧行（export方向/资源释放/callback pin/过期generation）=939永久fixture（ce-*/callback-*/handle-* decks）已交证据。验证：foreign-contract 53绿、av-core foreign 5绿、rust_f0 7绿、f0_async 2绿。 验证V-08；已有T-04（945–948为T-05）汇总此证据后才交独立review。

- [x] **T-09｜M2-A｜owner_stage: work｜AC-07**：接通工程use.c/use.rs到共享绑定和typed foreign事实，冻结真实库/全特征required表、来源/注解/工具链和合法/非法公共源码；先走通一个真实导入与失败恢复。 [✅ 2026-11-14]
  交付：`open_project_session` 工程级 use.rs 入口（provider=工程文件，全派生，具名拒绝，V-09 正反例 `f0_project_import_runs_rejects_and_recovers` 全绿）；§5 T-09 冻结记录（真实库/全特征 required 表 owner 到任务、BuildIdentity/RuntimeCompatibility/ProviderArtifact 字段清单、V-ABI-CROSS 双工具链计划、INT-1026 最小字段）。use.c 工程腿冻结为 T-10 随 C importer 落（拼写已定）。 验证V-09，证据汇入原最终work验证。 ABI-COMPAT：冻结BuildIdentity/RuntimeCompatibility/ProviderArtifact字段与兼容规则、两个实际可构建fixture的rustc/target组合、准确既有入口和V-ABI-*预期；缺环境记pending。

- [x] **T-10｜M2-A｜owner_stage: work｜AC-08/AC-NATIVE**：实现C真实预处理/头文件闭包、target/sysroot/宏/typedef和ABI布局、函数/record/enum/pointer/callback/opaque、常量/inline shim；将旧用法特征逐行闭合。 验证V-10，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。 [✅ 2026-11-14 三片交付：真预处理importer+use.c前门+native腿（首片）、inline shim/常量/形状审计/target轴（续片）、opaque handle生命周期+callback映射（收尾片）；c_import 4案全绿；具名边界=AV-C执行腿归T-12四端、COFF callback trampoline具名发现F-967-COFF-CB归T-13、源级常量binding随manifest schema扩展、V-NATIVE五字节行为=长度域已证+完整CStr边界随T-12/15]

- [x] **T-11｜M2-A｜owner_stage: work｜AC-09/AC-NATIVE**：实现Rust公开元数据到具体类型/泛型实例和自动C-compatible shim；闭合String/Vec/Result/borrow/析构、panic、线程与future边界。 验证V-11，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。 ABI-COMPAT：B-01/02库+shim共同构建与cdylib/staticlib产物合同，记录rustc/依赖/flags；执行V-ABI-BUILD/CROSS适用部分。 [✅ 2026-11-14：混合view+标量帧、具名形状边界（泛型/owned String/Vec/Result/async/trait对象/&mut各有名）、真双rustc C-ABI交叉（stable+nightly：身份异、接口facts同、AV宿主跑另一工具链provider；缺第二工具链=具名pending）、staticlib同闭包产物；f0_rich 3案全绿；具名余量=owned lane/async importer面(T-13)、MSRV门(T-14)、真实库腿(T-15 serde_json)、A2R crate adapter腿(T-12)]

- [x] **T-12｜M2-A｜owner_stage: work｜AC-10/AC-NATIVE**：接AV/AC/新A2R/新A2C到同一Foreign IR；native机器码调用，A2R等价crate适配需独立证明；各端记录真实producer及dispatch。 验证V-12，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。 ABI-COMPAT：核跨C ABI独立消费与target/接口准入，V-ABI-CROSS/TARGET/IFACE；A2R直接crate adapter单列等价与同构建闭包，不冒跨rustc二进制保证。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [x] **T-13｜M2-A｜owner_stage: work｜AC-11/AC-NATIVE**：闭合双向import/export、适用静态/动态链接、callback重入与generation/lease；接future完成/取消端口，保原allocator与跨ABI错误语义，交968调度。 验证V-13，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。 ABI-COMPAT：V-ABI-LIFE核兼容provider新旧代的owner/allocator/pin/卸载边界，不跨代接管旧handle。 [✅ 2026-11-14：future端口绑会话generation（teardown/rebind杀in-flight端口、迟到丢弃实测）；F-967-COFF-CB处置=整模split道对callback资源manifest具名fail-closed（foreign.callback-lane→939逐函数session道）；V-13行映射=export方向=939 CE bridge decks（ce-record/ce-text-view，生产case组重跑PASS）+重入/destroy一次/stale/cancel-late=同套callback/handle/txn decks（native+c腿）+本f0_t13 3案；GrantSet owner/grant维=F0身份+generation+form轴上的具名余量（§0全维账本随T-14身份键扩展）]

- [x] **T-14｜M2-A｜owner_stage: work｜AC-12**：使header/crate/toolchain/target/provider/ABI/生成器的真实读取依赖进入现有store，输出binding/shim/runtime分发清单和编译/运行工具分账合同给978。 验证V-14，证据汇入原最终work验证。 ABI-COMPAT：V-ABI-CACHE分离精确构建失效和运行兼容/选择，V-ABI-PACK交版本化工件/依赖清单给978，不造第二缓存/loader。 [✅ 2026-11-14：S967B层进现有783 store（BuildIdentity全轴真读：provider字节/shim字节/rustc --version/生成器身份/target/crate-types）；精确失效实测（provider编辑翻身份、shim轴独立翻、工具链轴独立翻、AUTO body编辑不触键、删工件=miss不stale-green、双rustc两身份一合同）；分发清单BuildRecord JSON（cdylib+staticlib+import lib+全轴+dumpbin核实runtime note）交978；runtime兼容轴按T-09冻结与构建缓存分账；f0_t14 2案全绿] COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [ ] **T-15｜M2-A｜owner_stage: work｜AC-13/AC-NATIVE**：用钉版本真实SQLite和serde_json原工程及完整特征fixture做用户体验与资源/失败回归；汇总四端、独立oracle、缺环境和交978/985材料。 验证V-15，证据汇入原最终work验证。 新增V-NATIVE：完整FFI扩面时分别处理ptr+len、CStr和Rust String/borrow/owned入出参；只在需CStr的边界转换/终止，明确内嵌NUL拒绝/显式策略、有界回读和正确allocator销毁。不得暴露Rust私有布局或把C ABI等同libc。F0保原长度借用域，其早期非UI验证/Hello不等待完整C importer。 ABI-COMPAT：汇总七个V-ABI-*真实结果及隔离消费夹具，原真实库/四端/资源义务不减，交978/985/986按既有owner接续。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [ ] **T-04｜M2-A｜work｜AC-01..03**：运行§6全部V行与完整职责组，记录真实命令、子终态、覆盖及身份；阻塞项具名，不把环境红或未执行改NA。 COVERAGE：承担§5本包覆盖责任，补V-COVERAGE/AC-COVERAGE，不减少本任务原范围/早期切片合同。 LOAD：按§5消费1043正式加载层，闭V-LOAD/AC-LOAD，原早期范围与全部任务义务不变。

- [ ] **T-05｜M2-A｜review｜AC-04**：独立复审，逐项核验和遗漏扫描；债务候选写KNOWN-DEBT，in-scope缺陷回同树修复，停止实现并发写入。

- [ ] **T-06｜M2-A｜merge｜AC-05**：review pass后核冻结并集与来源，落规范增量、合回、归档，保M1/M2证据分账。

- [ ] **T-07｜M2-A｜merge｜AC-05**：命令实际输出逐目录对账；摘要折入Plan，删除统一scope（具名未解决持有除外），刷新retention，wt-guard clean后清树/分支并核最终状态。

A交接：本组work/原正反例齐→独立复核相应A切片→按owner流程正式合入并绑定source/SDK/接口/实际结果→985/A消费。若本包还有B，不提前勾全包review/归档；同树保留/resync。

### 工作位置与材料

预定唯一实施树 `D:/autostack/.wt/lang-967/auto-lang`，分支 `plan-967-dev`（沿仓规），尚未创建。跨仓同组每仓唯一树。计划簿记留主检出；实现和构建测试只在专属树。

T-02依赖T-01；T-03依赖T-02；T-04在所有work能力完成后；T-05→06→07串行。外仓owner交付可按稳定接口提前准备，但最终不得拿未交付接口背书。
材料唯一目录 `D:/autostack/plan-artifacts/plan-967/`，开工登记owner/用途/在途；本轮未创建。不可另建attachments/证据仓目录，不留第二套target/SDK副本。


本轮细化交接：原任务ID/阶段/进度保持；T-01须在依赖实现前把本轮具体输入/类型/旧新check与未决来源冻结；T-02/T-03/T-08/T-09/T-10/T-04分别落实对应V-DETAIL-01..04与原所有验证。分析/格式/性能/发行包沿各自职责，不把例子复制成另一执行器或新框架。未决改变既有公共语义/预算/来源门则具名修订，不静默缩分母。

## 9. 复审记录

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 17 | from: work | to: work | status: executing | outcome: T-14完成（BuildIdentity精确缓存+分发清单；勾选） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-15 真实库SQLite/serde_json）

2026-11-14 stage: work / T-14（AC-12/V-14+V-ABI-CACHE/PACK适用部分）| 交付: provider_build_cache（S967B层、现有store、全轴真读身份、hit复用产物集rustc不重跑、六向失效/保留语义实测、双工具链两身份、BuildRecord分发清单JSON+runtime note）| 回归: f0系23案全绿 | 具名余量: 978侧安装/升级消费工件清单、ac-av分账、V-ABI-TARGET/IFACE运行轴（按T-01冻结归对应owner）| next: T-15。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 16 | from: work | to: work | status: executing | outcome: T-13完成（端口接线+COFF-CB具名fail-closed+V-13行映射；勾选） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-14 精确缓存/BuildIdentity）

2026-11-14 stage: work / T-13（AC-11/V-13）| 交付: CompletionPorts会话generation绑定（bind_session_generation实测teardown杀in-flight+迟到丢弃）、foreign.callback-lane具名fail-closed（整模split道不再产出垃圾caller——差分探针的垃圾路径现已不可发射）、f0_t13 3案全绿（端口绑定/fence用939 callback deck真manifest触发/CE export decks身份绑定）| V-13行=939生产组重跑PASS背书（case-ace-ca-d-resources-939）| 清理: AV-C期误入的3个.obj已untrack+.gitignore | 回归: f0系21案全绿 | GrantSet全维=具名余量随T-14身份键 | next: T-14。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 15 | from: work | to: work | status: executing | outcome: T-12完成（A2PY ctypes腿+V-COVERAGE六路径全实跑；勾选） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-13 双向资源/GrantSet/callback重入）

2026-11-14 stage: work / T-12收口（AC-10/V-COVERAGE）| 交付: a2x-python foreign_bridge（绝对路径CDLL装载同一provider cdylib、per-decl argtypes/restype、typed A2xForeignStatus异常通道、validate_view形状检查域外保持具名拒绝、lowering重复门提升同检查+Call臂路由包装器）、f0_a2py 2案全绿（真python真运行：value:9/0/65、具名miss→foreign.status=-1非零exit；record-view域外具名拒绝）| V-COVERAGE六路径全实跑（AV/native·AV/Rust·AV/C·AC·A2R·A2PY，同一binding/shim/provider）| 清理: AV-C提交误入的failure_runtime.obj已untrack | 回归: f0系18案全绿、a2x-python tests 2绿、case套件PASS | LOAD边界: 1043来源装配腿按T-01冻结归T-04汇总核 | next: T-13。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 14 | from: work | to: work | status: executing | outcome: T-12 AXR0腿交付（A2R经同一shim导出真编译真运行；域外具名拒绝保持）；A2PY ctypes腿待续 | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-12收口：A2PY ctypes腿+COVERAGE六路径收口）

2026-11-14 stage: work / T-12 AXR0腿（V-AXR0-RUST/AC-10首片）| 交付: a2x-rust foreign_bridge（extern "C"+A2xResult包装器/冻结域形状检查/域外保持a2r.foreign-unsupported）、AutoTrap::Foreign(status)陷阱变体、lowering Call臂路由foreign被调者（AsStr借用）、f0_axr 2案全绿（真rustc编译fileset链接provider import lib真运行：value:9/0/65、trap:Foreign具名；record-view域外拒绝保持具名；938标量视图拒绝由原938测试保持绿）| ace-a2x组7红=cl.exe deadline并行争用（串行重放绿）| COVERAGE现值: AV/native✓ AV/Rust✓ AV/C✓ AC✓ A2C✓ **A2R✓**；A2PY待ctypes腿 | next: A2PY+收口。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 13 | from: work | to: work | status: executing | outcome: T-12 AV-C腿交付（C provider真DLL驱动AV道；15/65/具名trap全绿）；A2R AXR0与A2PY腿待续 | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-12续：AXR0 Rust腿+A2PY腿+COVERAGE收口）

2026-11-14 stage: work / T-12 AV-C腿（V-COVERAGE行）| 交付: build_c_dll（cl /LD真DLL：provider.c+生成inline shim+CRT-free失败runtime同一闭包、导出.def经-link透传、显式MSVC include+CRT lib路径）、AV stub表装载C DLL、同一use.c普通源经AV解释器执行真C机器码（scaled_len=15、status_of=65、空输入具名ForeignStatus trap）| AV-C片=标量视图帧；record/callback/inline/常量条目留各自交付道 | 验证: f0系14案全绿 | COVERAGE现值: AV/native✓ AV/Rust✓ AV/C✓ AC✓ A2C✓（938 manual+T-10 native）；A2R/A2PY待AXR0/ctypes腿 | next: T-12续。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 12 | from: work | to: work | status: executing | outcome: T-11完成（混合帧+具名边界+双rustc交叉+staticlib；勾选） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-12 四端接入）

2026-11-14 stage: work / T-11（AC-09/V-11+V-ABI-BUILD/CROSS适用部分）| 交付: importer参数域扩至 view+i32/bool混合（≤4）、RustParam形状表、manifest lanes派生、shim混合帧、T-11形状审计（泛型/owned文本/owned列表/Result/async/trait对象/&mut逐项具名拒绝=边界行闭合）、foreign_exec T-11帧+dispatch_view_scalars+AV臂标量编组+lookup对应、loader形状映射、冻结rich fixture、f0_rich 3案（混合帧真跑/clamp 4·3、码点bool、find 5+具名miss trap、具真实例65；边界具名；V-ABI-CROSS真双工具链stable+nightly身份异/接口同/AV宿主跑对方provider；B-02 staticlib产物断言）| 回归: rust_f0 8、c_import 4、av-core foreign 5、auto-bindgen 10、case套件PASS | 具名余量: owned lane/async importer面T-13、MSRV门T-14、真实库腿T-15、A2R crate adapter T-12 | next: T-12。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 11 | from: work | to: work | status: executing | outcome: T-10完成（收尾片：opaque handle生命周期真机码+callback合同映射；勾选T-10） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-11 Rust公开元数据importer）

2026-11-14 stage: work / T-10收尾片（AC-08）| 交付: opaque解析（typedef struct无体）+handle参数/返回按解析集消解（record vs opaque不猜）+i32域slot id编码（真指针会触发wide-range传输）+生命周期真机码（create(7)→get 7→destroy 0→slot释放→recreate）+callback合同映射（i64回调typedef+ctx种子I32 lane+fallible out，manifest准入+decls断言）| 具名发现 F-967-COFF-CB: COFF道callback trampoline非交付执行面（939 fold差分探针COFF道得垃圾而交付a2x-c产品道绿）→归T-13/V-13；calling腿走a2x-c（939 decks绿）| c_import 4案全绿；rust_f0 8、auto-bindgen 10绿 | next: T-11。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 10 | from: work | to: work | status: executing | outcome: T-10续片交付（inline shim/常量/形状审计/target轴）；opaque与callback消费函数及AV-C(T-12)仍具名 | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-10收尾片：opaque+callback消费；随后T-11）

2026-11-14 stage: work / T-10续片 | 交付: provider头注解（name/target两轴具名）、raw头形状审计（varargs/union/bitfield/packing逐行具名拒绝=旧用法特征行闭合）、static inline体捕获+生成shim入provider构建（真机码42）、enum常量提取+真消费（41）、c_import 3案全绿 | 验证: rust_f0 8、auto-bindgen 10 | 具名余量: auto-C面opaque handle与callback消费函数、AV-C执行腿(T-12) | next: T-10收尾。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 9 | from: work | to: work | status: executing | outcome: T-10首切片交付（真实预处理importer+use.c+native腿V-10）；varargs/union/bitfield/常量/inline/packing轴与AV-C腿待续 | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-10续片）

2026-11-14 stage: work / T-10首切片（AC-08/V-10部分）| 交付: ctool::preprocess_c（真cl /EP）、auto-bindgen c_import（注解=元数据；有界解析器：record/enum/callback/view/out，具名c-import.*拒绝；空标量spine遵939；content_version折叠header digest）、open_c_provider（工程.h+.c→派生manifest+cl编译provider obj+观察到的include闭包）、use.c前门（CST use-c-item+lang家族校验+legacy oracle一致）、冻结fixtures cside/f0_c.{h,c}（声明位宏证真实预处理）| V-10腿（native）: record聚合3+4=7真机器码、scaled_len 9、fallible 65/命名空status exit70（CRT-free失败runtime）、header编辑翻身份+stale源拒绝、ghost注解具名拒绝、同入口恢复 | 回归: rust_f0 8、core lib 63、differential 927 13案、case套件PASS、auto-bindgen 10 全绿 | 边界: varargs/union/bitfield旧行、独立常量binding、static-inline shim、packing/target轴、AV-C腿(T-12)待续 | next: T-10续片。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 8 | from: work | to: work | status: executing | outcome: T-09完成（工程入口+冻结记录） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-10 C真实预处理importer）

2026-11-14 stage: work / T-09（AC-07/V-09）| 交付: open_project_session 工程级入口 + f0_project_import 正反例（真导入9/坏注解具名拒/缺文件具名拒/同入口恢复）+ §5 T-09冻结记录（required表/身份字段/双rustc计划/INT-1026字段） | use.c腿=T-10 | next: T-10。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 7 | from: work | to: work | status: executing | outcome: T-08完成（异步完成端口+受控真实provider） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-09 公共use.c/use.rs工程入口）

2026-11-14 stage: work / T-08（AC-06/V-08端口腿）| 交付: port.rs中立合同+规则表、CompletionPorts宿主注册表（generation纪律+cancel墓碑）、f0_async受控真线程provider+f0_async 2 E2E（发布/cancel迟到丢/teardown迟到丢）| C侧行=939 fixtures已交 | 验证: foreign-contract 53、av-core foreign 5、rust_f0 7、f0_async 2 全绿 | next: T-09。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 6 | from: work | to: work | status: executing | outcome: T-03完成（生产TestCase道） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-08 双向ABI与异步完成端口）

2026-11-14 stage: work / T-03（AC-02/03 生产consumer）| 交付: abi/rust-f0-certify-v1 worker配方+contract/adapter/judge注册+固定组case-ace-m2-f0-rust-967（6案av+native PASS）+ace_foreign_fail构建单元导出 | 边界: 1043装配腿T-12（按T-01冻结）、owner/grant维T-13、工件增量T-14 | 回归: 938套件/auto-test-case/ace-adapters/rust_f0/auto-bindgen全绿 | next: T-08。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 5 | from: work | to: work | status: executing | outcome: T-02完成（会话层+永久正反例+桥映射） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-03 真实consumer/目标/装配/权限/增量/失败恢复接线）

2026-11-14 stage: work / T-02（AC-02）| 交付: ForeignSession+F0Session组合层、f0_session_lifecycle永久正反例、桥映射与数字ID裁定入spec | 验证: rust_f0 7 E2E全绿（5 F0+native probe+session lifecycle）、av-core foreign_exec单测绿、fmt清理 | 边界: V-03/V-04回调全量939 native道至T-08；V-01..06全量执行证据按owner行T-04汇总 | next: T-03。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 4 | from: work | to: work | status: executing | outcome: T-01完成（前探+冻结） | worktree: D:/autostack/.wt/lang-967/auto-lang | next: work（T-02 主能力实现）

2026-11-14 stage: work / T-01（AC-01/AC-PHASE/COVERAGE/LOAD 冻结）| worktree: lang-967（resync 76431e6b9） | 前探: 最长链 native 机器码腿新测试 `f0_native_machine_code_consumes_the_same_auto_binding`（use.rs源→auto importer→native lowering→caller COFF→import lib链接→真callee exit 9；ghost manifest 校验拒绝；provider重绑定 exit 10）——正例/非法近邻/失败恢复三态 | 策略裁定: panic标志通道（C-unwind跨DLL实测不可catch）；cdylib+import lib（staticlib与kernel32-only PE冲突） | 冻结: §5 T-01决定摘要（锚点SHA、资源表、六路径+A2C owner映射、LOAD映射、具名unknown） | 验证: rust_f0 5+native probe 1 全绿；锚点SHAs核验 | next: T-02。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 3 | from: work | to: work | status: executing | outcome: T-F0-D完成（phase fold落主+spec沉淀+resync） | worktree: D:/autostack/.wt/lang-967/auto-lang（resync至8d66dd7f7） | commit: master 4c1788a79(merge)/8d66dd7f7(docs) | next: work（T-01 全链前探/COVERAGE/LOAD 冻结）

2026-11-14 stage: merge / T-F0-D（F0切片phase fold，AC-F0-LAND）| 合入: plan-967-dev → master `4c1788a79`（--no-ff；基线含并发1040收据，无冲突） | 落地核对: master构建绿（auto-bindgen/ace-abi）+ rust_f0 5 E2E 主检出绿 | 沉淀: docs/specs/ace-migration/m2-host.md（F0接口表+未交付边界）+ 计划收据 8d66dd7f7 | 树: 唯一worktree保留并resync至master，T-01..15继续 | 不归档、不勾整包完成。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 2 | from: work | to: work | status: executing | outcome: T-F0-R完成（独立审计PASS+5修复落地） | worktree: D:/autostack/.wt/lang-967/auto-lang | commit: d77e369b2 | next: work（T-F0-D phase fold 正式落主）

2026-11-14 stage: review / T-F0-R（F0切片独立审计，AC-F0-REVIEW）| 冻结基线: f5bf3cd50（停写） | 审计者: 独立上下文agent（未参与实现） | 结论: PASS（0 BLOCKER/0 MAJOR/5 MINOR），逐行证据见§8 T-F0-R | findings返I并全数修复: d77e369b2（#1最小命名status、#2单FreeLibrary、#3预算语义裁定入§5、#4文档改标志协议、#5 form负控） | 修复复验: auto-bindgen 10绿、av-core foreign_exec 3绿、rust_f0 5绿 | 只pass本切片；原T-09..15/完整FFI义务不变 | next: T-F0-D。

2026-11-14 stage: goal | plan_id: PLAN-967 | plan_revision: r12 | round: 1 | from: work | to: work | status: executing | outcome: F0-AV切片交付（T-F0-I/V完成，T-F0-R/D与T-01..15继续） | worktree: D:/autostack/.wt/lang-967/auto-lang | commit: 10a3dd46e基 + F0切片提交（31文件，+2599/−39） | next: work（T-F0-R独立审计F0切片）

2026-11-14 stage: work / PLAN-967 T-F0-I+T-F0-V（F0-AV切片）| worktree: D:/autostack/.wt/lang-967/auto-lang | branch: plan-967-dev | base: 10a3dd46e4a4c15948f120e0654fce2182078ccd | 依赖核验: 938/939/960/962/963/1042/1043 全 archived（2026-11-14当日）。实施: importer/shim/manual（auto-bindgen rust_import，9测）、冻结provider（abi rustside/f0_text.rs）、cdylib构建+装载（abi rust_provider.rs）、AV stub表+ForeignCall臂+execute_foreign+verify_abc_foreign（av-core foreign_exec/av/abc_verify/contract，3+62+44+11测）、CST use-rs-item语法+投影+resolve校验+legacy oracle一致（core，57+461测）、全下游UseRs臂（source-query/semantics/artifacts/av/native/binding，21+98+19+11+13测）、rust_f0 5 E2E（真实rustc cdylib→LoadLibrary→use.rs源→共享前门→AV真调；V-F0-SOURCE/REJECT/LIFE/IDENTITY及bool/fallible/panic腿）。验证: 上列全绿；ace-abi conformance 串行148/148绿（裸并行cargo test下939恢复类deadline敏感测试见1-2个flake——串行与官方组复验）；官方冻结组 `cargo t ace-abi`（nextest全隔离）PASS、执行261用例（results: target/test-groups/run-sp0a7t5x/results.json）。边界: ProgramSnapshot/session链接面归969/E0，A2R腿归T-12/AXR0；panic通道实测冻结为shim内catch+导出标志（跨DLL unwind宿主不可catch）。outcome: F0切片pass；整Plan继续executing。next: T-F0-R（同树停写独立审计F0）。

CONTEXT-LOAD（历史，2026-11-13）: 规划静态预检通过：20份设计/计划553处本地链接路径、Markdown围栏与Plan 0–10节/任务数一致；原144任务ID/勾选/状态/进度/旧AC/V及原依赖保留，新1043十任务均未完成。可达149个语言计划依赖无缺失/环、ID无冲突，中央.next-id为1044。1043不依完整966/FFI/库，1042不反依1043；966改为消费1043，原F0/B0/Hello与M1门不减。新计划候选固定组名和新增设计锚静态核对，git diff --check通过。未实施/建树/编译/运行产品或测试，不是独立实现review PASS；canonical Specs及归档历史未改。

2026-11-13 stage:new / COVERAGE | outcome: pass（规划静态核验，非产品或独立实现review） | 19份设计/计划519处本地链接存在；14计划共150任务，含新1042八项，其余142任务ID/勾选/状态/进度及原AC/V保留；0–10节/围栏/任务数一致。可达148个语言计划依赖无缺失/环，1042不反依完整库/977，967不反依968；原早期F0/B0范围不扩。历史5份非UTF8归档只按可解读frontmatter只读扫依赖，未修改。新增能力六路径尚未实施/实跑，不计产品PASS。

2026-11-12 stage:new / PLAN-967 r8：按963标准补本包具体输入/独立预期、类型与模块职责、旧源/check映射及V-DETAIL-01..04。保原19任务/全部AC与V/依赖/阶段/预算和drafting 0/19；只做文档与源码静态核对，不实施/建树/生成fixture/运行/派agent/合入或发布。已有研究/已归档成果复用，新增草图不冒已有API或产品支持。

2026-11-12 本轮规划静态核验：32份合同原505任务ID/顺序/进度、全部旧AC/V/Spec-delta及原依赖/阶段/预算保留；新增128条V-DETAIL待实施验证，未生成任何测试源码。505处本地引用/围栏/0–10章节/原任务映射核对通过，跨四仓可达222份Plan的显式依赖无缺失/环。只证明规划结构与取材来源，不是旧VM/新ACE运行或独立产品review PASS；959–962归档和963r5未改。

2026-11-12 stage=new / r7：按用户标准库/FFI方向补LIB-M2-07/08本包义务，绑定原任务与新增AC/V/SD；任务ID/总数、原AC与阶段门保留，drafting零进度。只修改规划，未实现/运行/签章或改变M1。

2026-11-11 / stage:new / PLAN-967:r6 / outcome:pass（早期Hello路线合同可审阅）；blocked（尚未授权实施，真实能力/接口待交） / next:按§5最小前置进入新增切片；原完整M2任务与终验均保留，不用小例替全包。新增T-F0-I, T-F0-V, T-F0-R, T-F0-D，当前0/19。未实现/构建/运行或派agent。

2026-11-11 规划静态核验（非产品/独立实现review）：12份本轮文档、6份Plan共106任务；对照本轮前快照，原86任务ID及原AC/V全部保留，新增20任务、全为未执行，drafting/current_step=0。0–10章节、total_steps、阶段清单与AC任务映射一致；323处本地链接已检查，新增链接/4处新锚点无失效、无新增行尾空白。跨四仓可达207份Plan的显式依赖无缺失/环，8个早期交接节点无环、AV不反等A2R或完整B；12个拟职责组名存在。6个具名源码SHA与本轮读取值相同；语言/UI/内容计数器仍1006/081/032。未新取号、建树、改产品/测试资产或canonical Specs，未构建/运行/签阶段/merge/tag。此检查不证明拟议接口已经实现。

2026-11-10 stage=new / r5：直接整理§2与§8当前执行合同，A/B清单分组、最小前置与阶段交接明示，消除旧单阶段总顺序/旧批次数量的冲突。保全部15个原任务ID/原AC、drafting与0进度；未新增实现任务/用例/编号。仅文档核对，未施工/运行产品/独立实现review/阶段fold或归档。

2026-11-09 stage=new / r4：正式按M2-A拆任务归属、补AC-PHASE/V-PHASE/SD-PHASE，当前0/15；原AC/T及历史记录保留。只做文档静态预检，未施工/跑测试/独立实现review/合入或签阶段。

2026-11-09 r3范围修订：新增T-09..15、AC-07/AC-08/AC-09/AC-10/AC-11/AC-12/AC-13及V-09..15，按Design56正式接M2 FFI/新版标准库责任，0/15。stage=new；仅文档合同与静态预检，非实施授权、产品验证或独立实现复审PASS。旧r1/r2/r3记录为历史。

2026-11-06第二轮计划复核：r2，新增T-08/AC-06/V-08并修依赖/阶段边界；0/8，未实施。此前0/7记录为历史。stage=new；方案修订已写入，implementation仍待正式前置/有界决定与实施指令；非独立产品复审PASS。


2026-11-06：stage=new，revision=1，outcome=blocked（实施前须依赖正式交付、T-01接口与跨仓执行门冻结，并取得实施指令），next=按路线依赖进入work。文档静态检查另记路线；非产品或独立final review PASS。0/7。

### 2026-11-13 r9接口规划复核（未实施）

仅补INT-1026、IF-CALL/IF-RES、SD/V/AC-INT-1026，绑定原有自动Rust FFI/AV资源任务；19任务/0进度、frontmatter依赖、F0范围与完整义务保持。将Python consumer所需帧/owner/Meta变成可核接口要求，不建立另一套FFI/资源注册；实际交付仍pending，无构建/运行/探针。文档复核不作为实现final pass。

### 2026-11-13 r10 ABI兼容边界修订（仅文档）

stage=new/revision；revision=10；保持drafting、0/19、原depends_on与全部既有任务/AC不变。用户本次明确只授权文档；新增B-01..06、AC/SD-ABI-COMPAT及七个V-ABI-*，绑定原T-09/11..15/T-04，原T-05独立复审核全部新义务。Design41/56为通用构建/兼容单源，Design58只消费；1026仅刷新967修订/源身份，其他owner不接管。

同一C ABI边界可跨rustc消费与构建缓存精确失效明确分账；所有跨版本/目标/分发结果均是未来验收，未运行任何Rust构建、工具链探针、ABI/产品测试或生成分发物。本文档检查不是实现final pass；next=后续work授权且对应依赖正式就绪后按既有任务执行。

r10文档预检：19个原任务ID及19个原AC完整保留，新增AC-ABI-COMPAT后共20个AC；七个V-ABI-*与SD-ABI-COMPAT唯一、已接入原表并绑定T-09/11..15/T-04。PLAN-967仍drafting/0/19、depends_on不变；Design41/56/58的新增交叉链接/锚点、围栏/空白及git diff --check通过。Design41 SHA256=`758277d92fff66fcd61a4b88b416667d6ad651230ef840e90f2301a284d9b606`；Design56 r6 SHA256=`6dbba6bca99827950fe1716b3acd3d00248ec50be3c2ffb24b9dfba3603e8a36`。仅文档检查，未执行任何上述运行验收；次级消费方引用刷新不改变其目标或进度。

## 10. 待澄清事项

T-01负责未决实现策略/平台provider/真实入口和资源门，缺决定不得进入依赖实现；无需为等价内部实现重复审批。若领域现行Spec与原行为冲突，记录最小输入、双方结果与建议，不私改expected。
本包涉及批量产品/目标时，T-01按独立可验证consumer拆执行批次；若需要多个中等Plan承载，必须在本Plan修订中逐AC迁移到正式后继，保持本包终验责任和分母，不能以一份盘点替全部交付。
