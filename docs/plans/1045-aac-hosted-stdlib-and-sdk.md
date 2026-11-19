---
plan_id: PLAN-1045
status: executing
feature_name: AAC hosted标准库需求落地与原生SDK消费
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-19
plan_revision: 6
current_step: 1
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

### 当前阶段交接验证（未执行）

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

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1045/auto-lang`，分支 `plan-1045-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1045/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | ✅ 2026-11-19（/goal 完成计划1045 授权整链）。G-AAC 985/T-A05 M2A-08 `95c7fbad8` 在 master；1044（`6022f10f2`）与 1056（`e36131b40`）均落主归档。contract-profile 11 项身份 9 项 OK，2 项漂移：`core/src/source/mod.rs`→`7DA6DCE5AF07DF1C`（PLAN-1078 AVP-00 SourceProfile::Auto+语料）、`grammars/auto-source.at`→`6EE7996EE3808C93`（PLAN-104/1072/1078 AGE/AV 视图语法增量，均加法不改既有面）；受影响验证由 T-06 在冻结树上跑 ace-source/ace-abi/ace-native/ace-aac 覆盖。§4 指纹重绑：ace-core-and-text `097cc4269d587f68`、ace-host-services `97cee88b84550b7b`、packages.rs `95cc76cfdb247a96`、link.rs `a74f557c299f8f66`（context-source-assembly 不变）。支持 profile：host/target=windows-x64、abi=autoabi-windows-x64(candidate)、CT=hosted-av、执行=单请求串行；资源预算取 1044 冻结值（front 64MiB/parse 1<<20/语义 1M 步+5s/CT 根 10s）。专树 `D:/autostack/.wt/lang-1045/auto-lang`（branch `plan-1045-dev`，base `670c720a893d226338ef9ad4a89475bc957b2da4`=master HEAD）；材料 scope `D:/autostack/plan-artifacts/plan-1045/` 已登记。已核实在册供给：aac-support 1.0.0（组 ace-aac）、sys 库集（973/1041/1065 Win32 实装）、mn 原生路径（MnDriver::build_collections_checked / ac-native build-source / console lane）、ac-build 可重定位 SDK（978）。后续任务入口：T-02 新宿主客户端落 `experimental/ace/aac/host/`+`tests/aac-host/`（Gen0 前端→mn 原生真 exe）；AC-01/04 |
| [ ] T-02 纯计算与alloc客户端 | work | T-01 | 创建能由Gen0真实AC编译运行的Auto客户端：UTF-8/bytes/span/intern、异构Map/Set/Vec/arena、精确整数/位运算、确定性排序和摘要；把CS每个操作映射到公共库入口及真实provider。 修 AAC 启动实际所需的现有 Auto 原生库文本/bytes/分配操作，沿默认 provider 合入共享实现。若变更公共库 API，按 Design56 验 AV/AC/A2R/A2PY 和 AV native/C/Rust provider 原合同。精确哈希/codec 取 1056。 | host需求最小客户端，覆盖D-S01/02/03；不复制Ruststdlib；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-03 文件、路径与进程装配 | work | T-02 | 消费新版库实现文件读取/目录与模块定位、环境/target、管道/退出码、链接器发现与受控回收；Auto长度字符串到OS编码显式适配。验证Unicode路径、空文件、坏编码和外部工具缺失。 修启动用真实 Windows 文件/路径/进程接面，使用现存 OS API 方案。启动必需能力在本任务或已确定的上游交付，不反等 1051/1052。 | host文件/进程adapter与OS依赖账，默认Auto原生provider；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-04 失败与发布原语 | work | T-03 | 将allocator/句柄/子进程使用纳入1044预算；测试部分读写、OOM/取消/超时、输出超限、临时文件与rename失败，确认精确错误和最后owner释放，发布失败保旧。 验证现 AC 的部分写入/失败分类/临时文件清理与提交后可重试，错误回公共库来源。 | 命名故障客户端及资源计数/文件hash；工具实际回收不能以父exit0代替；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-05 仓外SDK闭包 | work | T-04 | 用978已交可重定位SDK在计划材料区创建仓外host消费，显式输入/输出/target，列Auto/OS与backend/link/AV工具依赖；拔掉开发树隐式路径后真运行，缺依赖具名拒绝。 正式交现 AC 编译并运行的仓外真实 AAC host 模块、SDK/ABI 身份和独立 IO 证据，供 1068 T-05。 | CS库需求逐操作关闭收据及可搬迁host recipe；所有材料归plan-1045；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-09 host场景闭包和诊断 | work | T-05；§5实际输入合同 | 补完整CS库操作客户端：argv/环境/相对路径独立cwd、stderr/退出码、精确数值和Unicode位置、空/长/坏输入、资源回收；核core/alloc/hosted/可选adapter依赖分层和所有声明provider，普通标准库默认链不隐入libc。 用实源覆盖 Unicode/嵌零、失败后同源重试、路径/CLI 与源 span；Linux 运行留 1063。 | §6 V-05实际结果及输入/产物身份；AC-07/AC-04 |
| [ ] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 实际命令/退出码/输入和产物hash、覆盖与缺口；AC-01..04。完成即execution_done交review，无需预勾T-07/08 |
| [ ] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（补精确基础库依赖和可执行host场景，避免“API名相同即供给齐”以及隐式C字符串/开发机路径。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

## 10. 待澄清事项

无须当前用户新增决定。1044已交CS库操作与预算；若实际M2-A供给缺失，回真实owner并挂具名关闭条件，不在AAC临时实现通用stdlib。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
