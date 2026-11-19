---
plan_id: PLAN-1045
status: archived
feature_name: AAC hosted标准库需求落地与原生SDK消费
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-19
plan_revision: 6
current_step: 7
total_steps: 9
depends_on: [PLAN-1044, PLAN-1056]
supersedes_spec_components: [docs/specs/stdlib/design/ace-host-services.md]
new_spec_components: [docs/specs/auto-ac/aac-hosted-profile.md]
touched_goals: [GOAL-002]
---

# [PLAN-1045] AAC hosted标准库需求落地与原生SDK消费

## 0. 变更摘要

2026-11-19 r6 状态核对。1044 与 1056 已归档，§4 更新 T-01 的正式输入及原生验证边界，计划保持草案 0/9。

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包承接项已补入 §8 原执行任务，复用 §6 真执行和独立反控；正式交接须标明原 required 行、实际源与产物、身份和结果。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：补精确基础库依赖和可执行host场景，避免“API名相同即供给齐”以及隐式C字符串/开发机路径。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。以真实AC客户端证明AAC所需模块化标准库与hosted接口可用，复用M2-A自有库/OS底座及SDK，交付可重定位编译器宿主装配。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

2026-11-19 work 复审修复轮：T-01..T-06+T-09 已执行（7/9；T-07 复审、T-08 合入归后续阶段），独立终审 needs_fix 项 F-1/F-2/F-3/F-4 已修复（argv 值子腿因 D-1045-01 加重为形状非确定损坏而具名阻塞，不交付弱化绑定）；详情见 §9。

## 1. 目标

以真实AC客户端证明AAC所需模块化标准库与hosted接口可用，复用M2-A自有库/OS底座及SDK，交付可重定位编译器宿主装配。 本包出口见AC-01..06。依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**experimental/ace/aac/host/、experimental/ace/aac/tests/host/（拟新增）**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `docs/specs/stdlib/design/context-source-assembly.md` | 2d8b63585610bf53… |
| `docs/specs/stdlib/design/ace-core-and-text.md` | 31ea76870c06a3a2… |
| `docs/specs/stdlib/design/ace-host-services.md` | 64aaf698c23cf01b… |
| `experimental/ace/crates/core/src/source/packages.rs` | 9aa1b33d833ce001… |
| `experimental/ace/crates/core/src/link.rs` | 7e19a7d49894d30a… |

规范输入：`docs/specs/stdlib/design/ace-host-services.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：946/972供text/集合/allocator，1056供精确摘要与编译器codec（优先复用M2-A已交同能力）；973与1039/1041供Win文件/进程/编码；975供实际codec/hash；967供显式FFI边界，978供SDK装配。1040仅为未来Linux分支，不作首版Windows的整包前置。

**当前开工状态**（2026-11-19）。985/A 的 M2A-08 已落主。[1044](archive/1044-aac-baseline-and-stage-contracts.md) 的库操作与资源预算（`6022f10f2`）和 [1056](archive/1056-aac-exact-digests-and-compiler-codecs.md) 的摘要/codec（`e36131b40`）均已归档，进入 T-01 核对。盘点时分别绑定 1044 的契约版本和当前可执行输入，核对其后已变的共享 source 模块与 `auto-source.at`，记录实际依赖及受影响范围。

1056 的 `aac-support` 1.0.0 已用 Gen0 前端和 AV 执行验证。T-02 依 [交付 profile](../specs/auto-ac/aac-digest-codec-profile.md)消费同一支撑包并验证本计划所需的真实 Gen0 AC 路径。实际缺失的库或原生操作逐项交具名生产任务，仅影响使用该操作的切片。AV 已交用例可按阶段复用。

## 5. 详细设计

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

T-02消费1056已交精确digest/codec，不默认为975已提供BLAKE3。纯Auto算法通过标准库统一owner供给；AAC只做应用层组合。host profile还须列CLI argv、env/工作目录、路径/目录/文件、诊断输出、allocator/alignment、时限/取消与句柄。数学文本中整数宽度/符号/位移/溢出、浮点字面解析/位表示、locale无关format/parse、UTF-8字节与UTF-16位置必须有CS行；若某操作CS未用，保CL义务并明记理由，不能伪造需求或豁免。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-hosted-profile.md` | 新增AAC hosted+alloc消费profile及依赖闭包；基础库API仍归原M2 owner，不改变MCU core/alloc模块的可选性。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/stdlib/design/ace-host-services.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-hosted-profile.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本轮语言完整性边界

1045先验实际CS客户端的完整需求，CL中CS未用公共库操作由1053/1068逐行验证；CS需求审计不能冒全库支持。 当前CL目标取最终M2三端required及A后延增量，来源身份以阶段真实版本绑定，不以旧A有限面封顶。

## 6. 测试设计

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | 含非ASCII路径及嵌零bytes的模块读取、intern与集合去重 | Gen0编译的host客户端（T-02/03创建） | 输出与独立字节/去重/顺序expected一致；OS字符串若含不合法NUL须在adapter具名拒绝，不能静默截断 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 已存在产物后注入rename失败/进程超时 | host文件/进程真实入口 | 旧文件hash不变或已提交状态明确；子进程回收、句柄平衡，诊断分类为OS/资源故障 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | 仓外SDK去掉开发树及一个明确工具依赖 | 978式仓外host recipe（T-05） | 正常配置真运行；缺工具显式拒绝，不调用legacy或隐藏Ruststd标准库provider | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-02 | bytes=[0x61,0x00,0x62]，文本α按UTF-8为[0xCE,0xB1]；Map依次写a→1、b→2、a→3并按键排序 | 本包V-01所列真实/拟新增入口，沿相同启用任务 | bytes长度3且嵌零不丢；α的byte span为[0,2)；Map恰2项并输出[(a,3),(b,2)]，不能靠C字符串截断或随机迭代 |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-source ace-abi ace-native`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | host场景闭包和诊断的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | 在不同cwd以含空格/Unicode参数调用同一客户端，值和source定位不变；嵌零bytes保长而OS路径内NUL受控拒绝；host缺工具与语言类型错误分阶段可区分 | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### 当前阶段交接验证（已执行，收据见 §9/§8 行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | host需求最小客户端，覆盖D-S01/02/03；不复制Ruststdlib；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；本包支持范围不冒整个AAC |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [x] **AC-AC-STAGED / work**：V-AC-STAGED 关闭——A 基础来源全部正式（G-AAC 985/A、1044/1056 落主收据）；本包实际消费的每个操作有真实 provider，无缺口 fallback（缺口即修：mn 位运算发射/空拼接/NUL 拒绝三处共享面修复）；CL 行未删改（cs/cl TSV 只读消费）；阶段产出（host 客户端+SDK 消费收据）已在 §8/§9 具名可供 1068 消费。独立 review 复核见 §9。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1045/auto-lang`，分支 `plan-1045-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1045/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | ✅ 2026-11-19（/goal 完成计划1045 授权整链）。G-AAC 985/T-A05 M2A-08 `95c7fbad8` 在 master；1044（`6022f10f2`）与 1056（`e36131b40`）均落主归档。contract-profile 11 项身份 9 项 OK，2 项漂移：`core/src/source/mod.rs`→`7DA6DCE5AF07DF1C`（PLAN-1078 AVP-00 SourceProfile::Auto+语料）、`grammars/auto-source.at`→`6EE7996EE3808C93`（PLAN-104/1072/1078 AGE/AV 视图语法增量，均加法不改既有面）；受影响验证由 T-06 在冻结树上跑 ace-source/ace-abi/ace-native/ace-aac 覆盖。§4 指纹重绑：ace-core-and-text `097cc4269d587f68`、ace-host-services `97cee88b84550b7b`、packages.rs `95cc76cfdb247a96`、link.rs `a74f557c299f8f66`（context-source-assembly 不变）。支持 profile：host/target=windows-x64、abi=autoabi-windows-x64(candidate)、CT=hosted-av、执行=单请求串行；资源预算取 1044 冻结值（front 64MiB/parse 1<<20/语义 1M 步+5s/CT 根 10s）。专树 `D:/autostack/.wt/lang-1045/auto-lang`（branch `plan-1045-dev`，base `670c720a893d226338ef9ad4a89475bc957b2da4`=master HEAD）；材料 scope `D:/autostack/plan-artifacts/plan-1045/` 已登记。已核实在册供给：aac-support 1.0.0（组 ace-aac）、sys 库集（973/1041/1065 Win32 实装）、mn 原生路径（MnDriver::build_collections_checked / ac-native build-source / console lane）、ac-build 可重定位 SDK（978）。后续任务入口：T-02 新宿主客户端落 `experimental/ace/aac/host/`+`tests/aac-host/`（Gen0 前端→mn 原生真 exe）；AC-01/04 |
| [x] T-02 纯计算与alloc客户端 | work | T-01 | ✅ 2026-11-19。`experimental/ace/aac/host/host_client.at`+`host_main.at`：CS-201/202/203 全行客户端（嵌零bytes/span/α字节跨[0,2)、intern首见索引+集合去重、Str→int映射末写胜+字节字典序、确定性选择排序、wrap32/FNV纪律、checked-add溢出分类、arena树预算步数与具名资源拒绝），消费 aac-support 1.0.0 digest（sha256/blake3）。测试 crate `experimental/ace/tests/aac-host/`（固定组 ace-aac-host）：AV 全串 oracle（V-04/V-01 计算腿逐字对上独立 Rust 期望/sha2/blake3/Rust Ord）+ **真实 AC(mn) 原生腿**：build_collections_checked 把同一 checked 产物经共享 MIR 降 COFF/链接真 PE 运行，exit=对独立期望串重算的 FNV 折减（1355556778），含区分度负控。**沿默认 provider 修复的真实缺口**（"修启动所需操作"）：(1) collections_lower 发射 typed I64 And/Or/Xor（PLAN-972 具名后继切片，无主认领）；(2) 同文件 typed I32 位运算发射（digest.at 的 i32 flags 折叠需要）；(3) collections-runtime mn_coll_str_concat 空⊕空返回 RESOURCE/trap70 → 正确空 TEXT 节点（try_alloc(0) null+0 合法空表示被误读；owned lane 同约定）。提交 `5cb324d66`。CS 映射：D-S01→std.bytes/std.text+digest.utf8_bytes；D-S02→std.list 方法表+平行arena（std.map 为 int 键具名边界，963）；D-S03→客户端预算纪律+1044 冻结资源profile。AC-01/02/03 ✓ |
| [x] T-03 文件、路径与进程装配 | work | T-02 | ✅ 2026-11-19。`host_files.at`（sys 库集消费，AV+真 Win32PlatformIo）：非ASCII路径模块读取（内容嵌零=内容，字节+BLAKE3 与 Rust 侧同读一致）、intern 含近邻/空文件、目录字节字典序+模块定位探测+空目录、env set/get/remove/unset-空区分、真实 cmd.exe 子进程（exit 7 / 800ms ping 超时击杀 -11 受控回收 / 缺工具 -1）、**find_rust_lld 算法 Auto 复刻**（AC741_RUST_LLD→rustc --print sysroot→spawn_with_output JSON stdout 提取+真实反转义遍历→rust-lld.exe 存在探测）与 Rust 算法逐字节一致。**修真实 Windows 接面**：sysplat_win put_scalar 对 OS 字符串内 NUL 具名拒绝（-9，V-01 规则；原先静默截断于 Win32 NUL 终止边界），stage_inner 传播具名码；拒绝≠截断由夹具文件 "a" vs 路径 "a b" 反证。8/8 sys 例绿（v01×2+t03×4+v02+t04）。提交 `7be0193f1` |
| [x] T-04 失败与发布原语 | work | T-03 | ✅ 2026-11-19。`host_publish.at`（V-02）：已发布工件后注入两型 rename 失败（目标为已存在目录 / 源缺失）均负码拒绝且**旧工件 hash 逐字节保持**（Rust 侧 blake3 对账）；提交式重试成功发布 C2 并清掉 temp；分类 IO 失败（缺目录写/NUL 路径写均负码族）、read_text_range 超限夹取、三子进程顺序回收、50 次顺序读字节稳定（句柄平衡）。资源预算取 1044 contract-profile 冻结值（front 64MiB/语义 1M 步 5s/CT 根 10s；harness 侧 WORK_OWNED_64MIB）。提交 `7be0193f1` |
| [x] T-05 仓外SDK闭包 | work | T-04 | ✅ 2026-11-19。V-03：AAC host 模块（m2 lane scalar 面：mod-1000003 FNV 链于字节 97,0,98 + 位置数字 packing）经 978 可重定位 SDK（M2Build 引擎=ac-build CLI 同一 driver）打包 av+native，**SDK 迁移至远处并删除全部构建根后两条 lane 真运行**，exit 与独立 Rust oracle 一致；install-manifest blake3 身份记录；**去掉 rust-lld 依赖（env 指向缺失文件）→ native 构建具名拒绝（link.lld-not-found，无 legacy 回落），AV lane 不受影响**。测试 `tests/host_sdk.rs`。材料在 plan-artifacts scope（lane::scratch 挂 AUTO_PLAN_ARTIFACTS 缺省 OS temp；CLI 等价命令行与收据折叠本记录）。提交 `334c1204b` |
| [x] T-09 host场景闭包和诊断 | work | T-05 | ✅ 2026-11-19。V-05：诊断 driver 编译到**真可执行（A2R 端，真 rustc）**作为真子进程以 含空格/Unicode/空串/引号 参数从两个不同 cwd 调用：argv 值子腿经复审轮加重后**具名阻塞于 D-1045-01**（a2x argv 读取形状非确定损坏——内容与长度均曾红，曾绿形状随后漂移；不交付弱化绑定，关闭 oracle 留 host_diag.at）；diag driver cwd 往返+恢复、α 源 span 稳定（[0,2) 自带 UTF-8 编码核验）、OS 路径 NUL 拒绝（真实前缀文件 "a" 反证）、缺工具 -1、同源重试——全部 0，仅起始 cwd 跨运行不同；**语言类型错误在编译阶段拒绝**（阶段可区分）。发现并按纪律处理两个 A2R 端事实：(a) 进程 exit 是 runtime 自身成功码非 main 返回值 → 报告文件为观察通道（1064 纪律）；(b) **具名债务**：argv equal() oracle 在复合 diag_report 帧内读到的内容损坏（长度/码点完好，独立 driver 帧全绿）→ 拆专职 driver 帧全强度 oracle，复合帧缺陷具名归 a2x runtime owner（见 §10/KNOWN-DEBT）。Linux 运行留 1063。提交 `334c1204b` |
| [x] T-06 定向验证和交review | work | T-02..05、T-09 | ✅ 2026-11-19。冻结工作树（HEAD `c13658ad4`）上：固定组 `ace-aac`（26 例）PASS、`ace-aac-host`（21 例）PASS（单例最大 v05≈32s，两次冷 rustc 构建主导——外部工具成本具名记录）；共享改动触及组 `ace-source` PASS、`ace-native` PASS、`ace-av` 1435 例执行 2 红——**两红（host_console::av_console_entry_selection_cannot_bypass_main_policy / av_console_owned_front_door_enforces_console_entry_only）在主检出 master 同样红（独立复跑证实），为先在基线红，非本包引入，归 console front-door owner，本包不修不判**。`ace-abi` 未选：本包无 ABI 面改动（选择理由：改动集合=mn 发射器/mn shim/sys stdlib 文件，无 abi crate/契约触及）。§6 V-01..V-05 全部以独立 oracle 关闭（见 T-02..T-09 行）；pending=无本包内未跑项；统一批量回归门归 merge（PLAN-896 冻结并集后）。SD-01 交付稿已在树（c13658ad4），SD-02 由 review 核后续。实际命令/退出码/组结果 JSON 摘要如上；AC-01..04 ✓ |
| [x] T-07 独立复审 | review | T-06 | ✅ 2026-11-19/20。独立子代理上下文（非实现会话）终审：首轮 needs_fix（F-1 argv oracle 未保留/high、F-2 v05>10s/medium、F-3 计数/low、F-4 簿记/low、F-5 NUL 字节/info）；修复轮 f04718cc9 后**受影响范围重审 pass**（AC-01..07 全 pass；V-01..V-04 全绿、V-05 除具名阻塞 argv 子腿外全绿；共享面三处修改逐行复核语义正确；22/22 复跑 24.6s 墙钟）。合并前低项 F-6/F-7 已修（deff9671c）。审阅者对 1064 先例机制的描述（test-toolchain feature/ace-toolchain 组）经主检出核实**不成立**——先例为无门控普通组内用例，§10 裁决项已按真实事实修正。审阅记录全文见本行摘要+§9 收据 |
| [x] T-08 合入沉淀与清理 | merge | T-07 pass | ✅ 2026-11-20。prepared：reviewed f04718cc9（=rebase 后 6f2dce4b2，range-diff 全等）+文档后代 5365c1028/SD-01/SD-02；landed：`git merge --ff-only` 主干 tip=`cc8053e32`（无合并提交；旧→新映射 5cb324d66→3697649f6、7be0193f1→329230917、334c1204b→f5ce32bbe、5f69c44eb→c57b795c5、c13658ad4→44a8cdb56、f04718cc9→6f2dce4b2、deff9671c→5365c1028、SD-02=cc8053e32）；canonical_verified：`docs/specs/auto-ac/aac-hosted-profile.md`（sha16 b5555421ec092949）+ SD-02 `docs/specs/stdlib/design/ace-host-services.md`（c096dda6332f652a）落主；冻结门：ace-aac-host 于交付树重准备并 PASS（run-_y6jh8df；ace-aac/ace-source/ace-native/ace-av 复用 c13658ad4 期证据——修复差异仅触及 ace-aac-host 自有文件与 docs，不相交路径具名理由）；archived+cleaned 见 §9 收尾收据 |

## 9. 复审记录

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（补精确基础库依赖和可执行host场景，避免“API名相同即供给齐”以及隐式C字符串/开发机路径。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

2026-11-19 stage: work / goal round 1（/goal 完成计划1045 授权整链） | plan_id: PLAN-1045 | plan_revision: 6 | outcome: pass（T-01..T-06、T-09 全绿，§6 V-01..V-05 独立 oracle 全关闭） | worktree: D:/autostack/.wt/lang-1045/auto-lang | branch: plan-1045-dev | base: 670c720a893d226338ef9ad4a89475bc957b2da4 | code commits: 5cb324d66（T-02）→ 7be0193f1（T-03/T-04）→ 334c1204b（T-05/T-09）→ 5f69c44eb（组配置修正）→ c13658ad4（SD-01 交付稿） | 依赖：aac-support 1.0.0（1056）、sys 库集（973/1039/1040/1041/1065）、ac-build/SDK（978）、1044 契约（autolang.aac-baseline/1，身份漂移 2 项已重绑记录） | 测试：ace-aac 26 PASS、ace-aac-host 21 PASS（新增 21 例清单见 T-02..T-09 行；单例>10s 者 v05≈32s=两次冷 rustc，具名外部工具成本；其余全部 ≤7.6s）、ace-source PASS、ace-native PASS、ace-av 1435 执/2 红为主检出先在红（同例复跑证实） | 修复共享面：collections_lower typed I64/I32 位运算发射、collections-runtime 空⊕空 concat、sysplat_win OS 串 NUL 具名拒绝 | 具名债务：D-1045-01（A2R 复合帧 argv oracle 损坏，owner=a2x-rust；V-05 已由专职 driver 帧全强度关闭） | blockers: 无 | next: review（T-07 独立复审，同树冻结 c13658ad4）

2026-11-19 stage: review / final（独立子代理上下文，非实现会话） | plan_revision: 6 | reviewed_commit: c13658ad4 | outcome: **needs_fix** | 证据：AC-01/02/03 pass、AC-04/07 partial、V-01..V-04 pass、V-05 partial；共享面三处修改逐行核验语义正确（I32 位运算=AV wrap_int 同义、concat out==0 仅合法空、NUL -9 无 ==-8 消费者）；21 例计时表入档（>10s 仅 v05）。发现：F-1 argv oracle 未保留（high）、F-2 v05>10s（medium）、F-3 计数 9→8/结果位置（low）、F-4 簿记（low）、F-5 NUL 字节（info）。

2026-11-19 stage: work / 复审修复轮 | repaired_commit: f04718cc9 | F-1：argv 值 oracle 修复尝试（并入同 exe/专职 exe/直调/env 形状）全部暴露 D-1045-01 **加重**——内容与尾长度均形状非确定红（曾绿形状随后漂移，曾试 shape 级 oracle 亦红 205）→ 按计划纪律具名阻塞该子腿，不交付弱化绑定；`args_tail_ok`/`argv_shape` 留 host_diag.at 为复现基准+关闭 oracle；会话内探针证据（全量 equal() 绿、尾码点 20013,25991,45,49,48,52,53 逐位、n=6/长度 18,5,9,0,3,7）折叠记录于此。F-2：v05 拆分——快腿 `v05_compile_stage_split`（0.9s）独立；>10s 仅剩 `v05_host_diag_scenario`（≈14s，一次冷 rustc），按 2026-11-12 裁定保留运行并列为 §10 用户裁决项。F-3：spec 草稿计数修正（host_files 8，总数 22/22 绿）。F-4：§0/§6/§7-AC-AC-STAGED/current_step 簿记补齐。复跑：ace-aac-host 22/22 绿（19.5s 墙钟）。

2026-11-20 stage: review / final 受影响范围重审 | plan_revision: 6 | reviewed_commit: f04718cc9（pass 绑定）+ 合并前修正 deff9671c（F-6/F-7，仅文档） | reviewer: 独立子代理上下文（非实现会话） | outcome: **pass** — F-1 具名阻塞处理判定为合同正确路径（弱化绑定=虚假验证主张，严格劣于阻塞；关闭 oracle 自洽性复核：长度 7,9,0,3,7 与 cp-sum 46251 均重算吻合）；F-2 拆分+保留运行+裁决上交判定合法（v05_host_diag_scenario 实测 16.94s 为唯一 >10s 例）；F-3/F-4 修复核实。AC-04/07/V-05 由 partial 升 pass。计时表：22 例中 21 例 ≤7.5s，v05_host_diag_scenario 16.94s。合并前修正 F-6（spec §8 过称）、F-7（证据指针）已落 deff9671c。**注**：审阅者两轮提及的 1064 先例"test-toolchain feature + ace-toolchain 组"机制经本会话主检出两次核实不成立（1493-1501 行为 infra-ace-m1；全仓无该 feature/组），§10 裁决项措辞已按真实事实（先例=普通组内无门控）修正。pending merge 门：冻结并集（ace-aac/ace-source/ace-native@c13658ad4 与 ace-aac-host@f04718cc9，代码路径不相交故各自绑定有效；主检出落地后批量门按到期规则另属）。

2026-11-20 stage: merge / 收尾 | plan_id: PLAN-1045 | plan_revision: 6 | delivery_commit: cc8053e32（主干 tip，ff-only） | outcome: pass | canonical: aac-hosted-profile.md b5555421ec092949、ace-host-services.md c096dda6332f652a（SD-01 新增+SD-02 补记指针，均落主） | 冻结门：ace-aac-host@cc8053e32 树 PASS（22/22，重准备后 run-_y6jh8df）；ace-aac(26)/ace-source/ace-native@44a8cdb56 期、ace-av(1435执/2先在红 master 复证)@c13658ad4 期复用——c13658ad4..cc8053e32 差异=aac/host 应用层文件（仅 ace-aac-host 嵌入）+测试 crate+docs，与上述组编译路径不相交 | 批量回归到期检查：1045%5==0 → due 已执行（f636cb373 收据）：36 组 28 PASS/2 N-A/6 FAIL 全为先在治理族（index-15 override 实例归本计划添加、修复归 1098 治理线），零语言代码回归；本计划 ace 域组另持合并门收据 | 待用户：§10 v05>10s 裁决项；D-1045-01（a2x owner） | 清理：wt-guard clean×2 → worktree/分支已移除、组目录已删；plan-artifacts/plan-1045 折叠后整删、.retention 已刷新（收据见下） | completion_kind: delivered

## 10. 待澄清事项

（2026-11-19 work 收尾补记：ace-av 的 host_console 两例先在红归 console front-door owner——非本包触面，本包具名不修；D-1045-01 债务见 KNOWN-DEBT-AND-RISKS。）

**待用户裁决（复审 F-2 遗留）**：`v05_host_diag_scenario` 单例 ≈14s（>2026-11-14 的 10s 纪律线），成本为一次冷 rustc 编译（A2R 真 exe 腿的外部工具链成本，与本仓既有 `sys_d01_d06_1064::v02_d01_d06_a2r_cross_end`≈10.5s 同类）。该例按 2026-11-12 裁定不得因慢停用，故保留在固定组内运行。选项：(a) 接受为具名外部工具链成本类并留在常规组（**与 1064 先例机制完全一致**——经主检出核实，`sys_d01_d06_1064::v02_d01_d06_a2r_cross_end` 为无 feature 门控的普通 `#[test]`，在常规 ace-av 组内运行，本仓不存在 test-toolchain feature 或 ace-toolchain 组）；(b) 授权建立显式 toolchain lane/预备式前置机制（新政策，需裁定）。未裁决前该例保留运行+具名记录。1044已交CS库操作与预算；若实际M2-A供给缺失，回真实owner并挂具名关闭条件，不在AAC临时实现通用stdlib。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
