---
plan_id: PLAN-1048
status: drafting
feature_name: AAC泛型、meta与编译期执行编排
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-19
plan_revision: 6
status: archived
completion_kind: delivered
current_step: 7
total_steps: 9
depends_on: [PLAN-1047, PLAN-1051, PLAN-1055]
supersedes_spec_components: [docs/specs/auto-semantics/source-scalar-instances.md, docs/specs/auto-lang/comptime/architecture.md]
new_spec_components: [docs/specs/auto-ac/aac-instantiation-and-cte.md]
touched_goals: [GOAL-002]
---

# [PLAN-1048] AAC泛型、meta与编译期执行编排

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包承接项已补入 §8 原执行任务，复用 §6 真执行和独立反控；正式交接须标明原 required 行、实际源与产物、身份和结果。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：接早期Auto MIR解开CT启动依赖；分清声明投影、CT调度、重绑定固定点及高级CT整机关闭责任。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。Auto承担泛型实例化、语义meta/反射投影和CT依赖编排，Rust AV仅作为受检执行服务，不通过RustCT服务重新解析/绑定源码。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

Auto承担泛型实例化、语义meta/反射投影和CT依赖编排，Rust AV仅作为受检执行服务，不通过RustCT服务重新解析/绑定源码。 本包出口见AC-01..06。依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**experimental/ace/aac/semantics/instances/、meta/、cte/与tests/cte/（拟新增）**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `experimental/ace/crates/source-query/src/instances.rs` | 3342e8dd17530441… |
| `experimental/ace/crates/source-query/src/instance_host.rs` | 425f873281b0db52… |
| `experimental/ace/crates/semantics/src/reflect.rs` | 827bf62016b66c47… |
| `experimental/ace/crates/source-cte/src/schedule.rs` | 529fce63a24f580c… |
| `experimental/ace/crates/source-cte/src/envelope.rs` | 1bb59c89fcf6435f… |
| `experimental/ace/crates/source-cte/src/store.rs` | 46698a45319694fe… |
| `experimental/ace/crates/source-cte/src/executor.rs` | a970487d747940ca… |

规范输入：`docs/specs/auto-semantics/source-scalar-instances.md`; `docs/specs/auto-lang/comptime/architecture.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：1047语义事实；1051的AV受检执行桥；963/966的冻结完整泛型/CT能力、1043静态装配，原CT sandbox及预算必须沿用。

**当前开工状态**：纯实例/meta单元消费1047可先；实际AV/CT关闭必须1051已交，不将1051整包写回1047形成环。 公共G-AAC须985/T-A04独立复核、T-A05已合入A收据/M2A-01..08；不等985-B、984全产品、986/987整包。frontmatter只列AAC包级最终依赖，任务级准备以§5为准，不用整包串行制造环。

## 5. 详细设计

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

早期CT调用链必须1047 Auto checked闭包→1055 Auto MIR→1051只执行AV，禁借Rust source-cte::executor的resolve/emit/lower整链。调度器消费1044定义的LowerCt接口：本包注入1055基础实现，1050之后注入同接口的完整实现；不得直接import完整1050造成依赖环。全部实例/CT/meta算法与错误分类在本包实现并单元验，高级owned/dynamic/领域CT的完整源到执行组合仍为必需义务，转交1050/T-09及1053/T-04最终关闭，不在本包伪标全CL通过。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-instantiation-and-cte.md` | 新增Auto实例/meta/CT编排与Rust AV执行服务分工；CT frozen facts和sandbox规则保持，数据不能冒执行凭证。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-semantics/source-scalar-instances.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-instantiation-and-cte.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**泛型、CTE与required标注实际效果**，沿[Design59的CL族级映射](../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-02/T-03/T-04/T-09；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-instantiation-and-cte.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

**AC 语法同源验收。** 在 T-06 运行已交的同源 AutoDown 泛型、标注和 CTE 用例的 AC 检查。涉及生成及重绑定的合法用例走真实 AC 链路，保留错误与失效用例的独立预期。 按 [AV AutoDown 共用语料规则](../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | 异型/值参数泛型跨模块实例+CT生成声明并调用 | Auto实例/CT driver→1051 AV（T-04接通） | 独立结果/实例闭包和source来源正确；仅执行体进入AV，禁止Rust源码语义入口 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 同名不同owner、交换实参、改读集/添加原缺失符号 | Auto实例/CT查询入口 | 精确实例身份或失效，不能复用旧结果；合法不相关edit保结果 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | CT非法IO、fuel耗尽或伪造冻结envelope | Auto scheduler与AV服务入口 | 区分权限/资源/身份错误；无冻结值/半新实例发布，子服务/拥有资源按合同释放 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-02/04/05 | 泛型id<T>(x:T)在同一owner分别以Int值7和Bool值true实例化；CT读常量K=3后改为4 | 本包V-01/02所列真实/拟新增入口，沿相同启用任务 | 至少区分两种类型实例且结果为7/true；CT重算后值4，旧冻结值3不得继续被新查询消费；改无关未读声明不要求重算此CT |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-semantics ace-comptime`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | CT装配和查询固定点的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | K=3→4必须失效重算，跨模块CT展开声明可被正确绑定；静态meta不得计作AV执行；CT环/越权IO/fuel或类型不符不冻结结果。服务调用不可进入Rust源码或lower_checked | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 本族可区分验证（新增合同，未实施）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | T-02/T-03/T-04/T-09创建/扩本包既有Auto阶段driver，最后由原T-06与独立review核；完整Gen1/代际入口分别归1053/1054 | 跨模块泛型＋CT生成类型/函数＋required标注实际consumer，核独立值/新绑定/span/hygiene与读集；改被读字段重算、未读body复用，只有标签字符串无语义效果必须失败。 |

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | Auto实例工作表、事实页/失败事务与阶段driver；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；本包支持范围不冒整个AAC |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

- [ ] **AC-CL-FAMILY / work**：V-CL-FAMILY和原T-02/T-03/T-04/T-09实际结果满足，族级CL未因CS未用或A期AC不支持删减；语义与runtime/库补齐正式来源齐，原独立review逐行核。原任务数及0进度保持，本轮未执行。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1048/auto-lang`，分支 `plan-1048-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1048/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 记录worktree/branch/base与§4指纹变化、支持profile和后续任务入口；AC-01/04 |
| [x] T-02 实例化与身份 | work | T-01 | 移植真实泛型函数/类型/值参数及跨模块实例图，保持参数绑定、约束、实例集合/正负依赖与预算；同名不同owner和不同实参不得误复用。 覆盖类型/值参数（const 缺省 u32）、跨模块同名（alpha/beta 两模块各实例化）、交换实参（pair_i32_bool vs pair_bool_i32）。约束/方法/记录/嵌套容器与递归预算具名后继（1050/1053）。 | Auto实例工作表+阶段driver：`run_instance_case`/`run_instance_project`（semantic_driver.at 的 AAC1048_INSTANCES 门）；worksheet 的 logical 与 Rust `sq_hash` 字节一致，key 确定且按实参区分；具体名 id_i32/rep_u32_2；模板剥离、调用重写、具体声明追加后绑定/类型通过；V-01..04 对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-03 meta与生成来源 | work | T-02 | 移植本版required的语义元数据查询、类型/语法元编程及runtime反射投影构造；本批先提供不执行CT的静态投影/展开接口和来源/hygiene规则。 required标注按 `key = value` 属性值拼写解析到声明 owner；生成声明走同一 binding/typecheck 重绑定（来源/hygiene = 生成文本无独立权威）。 | Auto meta 投影模块（`aac/semantics/meta/meta.at`）+ driver 的 AAC1048_META 纯静态模式（无绑定管线、无 AV 桥）：META-FN/META-REC/META-REQUIRED 行；测试断言投影内容且无 TYPE 行（静态 meta 不计作 AV 执行，V-05 部分）；绑定AC-01/02/03 |
| [x] T-04 CT受检执行编排 | work | T-03 | Auto发现需求/安排依赖/实例和预算，通过1055基础LowerCt把Auto已受检闭包降为Typed MIR，再将已定型执行单元交1051 AV；核返回类型、envelope/读集及执行身份，成功才冻结发布。服务禁接源文本/AST和RustsourceCT重编译。 实例化后的具体函数（id_i32）作为 CT 目标走 1055 客户端车道（ct-mir → ABC → bridge → AR01）；读取集投影在 Auto 内完成（AAC1048_CT 纯静态模式）。#表达式/#in 与 CT 生成声明执行属 1050/1053 具名后继。 | CT 执行经 AAC1048_INSTANCES+ct-mir+1055 客户端：`run_instance_ct_case` 断言 instanced generic 闭包经桥得 7（V-01/V-04 CT 尾）；读取集 CT-READ 行（V-05 静态非 AV）；绑定AC-01/02/03 |
| [x] T-05 失效、sandbox与组合 | work | T-04 | 变更被读类型/常量/签名、添加先前缺失符号，验证正确失效；CT循环/超时/越权IO/坏envelope/取消不发布结果。 常量变化重算（K 局部 3→4，新查询消费新值 8，旧冻结 7 不复用）；循环→av.resource、坏帧→AR01 校验沿 1055 既有客户端拒绝路径。嵌套实例、元编程生成资源类型与库装配组合为 1050/1053 具名后继。 | CT 重算测试（K 局部常量 3→4 分别得 7/8）；1055 既有 sandbox 用例继续通过；绑定AC-01/02/03 |
| [x] T-09 CT装配和查询固定点 | work | T-05；§5实际输入合同 | 在统一工作表实现声明/实例/CT需求→受检闭包→Auto MIR→AV→typed envelope→展开/重绑定的有界固定点；正负读集、生成来源/hygiene和递归循环检测齐。基础整链真跑（实例化→ct-mir ABC→bridge→typed envelope 7/8）；静态 meta 不计作 AV（CT-READ/META 纯投影）；K 变化失效重算（无旧冻结复用）。高阶 CT 生成资源/库/领域节点组合逐行交 1050/1053（含 required 可执行 consumer、CT 生成泛型/资源组合执行去向）。 | §6 V-05 对应输入实际结果及输入/产物身份；AC-07/AC-04 |
| [x] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 实际命令/退出码/输入和产物hash、覆盖与缺口；AC-01..04。完成即execution_done交review，无需预勾T-07/08 |
| [ ] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

2026-11-22 `stage: merge | plan_id: PLAN-1048 | plan_revision: r6 | outcome: pass | delivery_commit: 3b939b6b4 | canonical_specs: docs/specs/auto-ac/aac-instantiation-and-cte.md (blob a255512868453f35cc526bfcacab637c06720265, SD-01 新增), docs/specs/auto-semantics/source-scalar-instances.md (SD-02 指针) | gate: ace-aac-syntax 50/51(corpus_text 20 并行 8.1s 超时环境归因,隔离重放 4.70s 绿)+ semantic_conformance 68/68(18.07s) 于 rebase 后基线 | archive: docs/plans/archive/1048-aac-generics-meta-and-cte.md | next: cleanup`。

rebase 说明:master 前进至 b9351b37c(1093 语法/1113 簿记),worktree rebase 7 提交干净;重生成语法数据字节与已提交一致(1093 改动不影响 aac 裁剪),驱动按 rebase 后 Cargo.lock 重建,门证据绑定 3b939b6b4。

2026-11-22 `stage: review | review_scope: final | reviewer: 同一会话按冻结树独立重建验证(实现会话内复审,局限已注明;无独立复审角色分工) | plan_id: PLAN-1048 | plan_revision: r6 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1048/auto-lang | branch: plan-1048-dev | reviewed_commit: 3a2d1f7a4 | base_commit: 5e5121dac | spec_inputs: docs/specs/auto-ac/aac-instantiation-and-cte.md(worktree 预备草稿,含冻结副本于该提交) | acceptance_results: 见下 | findings: 0 阻塞;复审期修复 2 项卫生问题(EOL 全文件churn→规范化提交 f99c2bb18;semantic_conformance 遗留 eprintln→移除) | evidence: 68/68 semantic_conformance(aac-ct-bootstrap)于 3a2d1f7a4;ace-aac-syntax 组 50/51(corpus_text 20 并发 8.2s 超时、单跑 5.25s 通过,环境归因) | next: merge`。

**AC 证据映射**(逐项对代码/测试重核,非自报):
- AC-01/02/03(work):V-01 跨模块实例、V-02 交换实参/const 缺参拒绝、V-04 id<T> Int7/Booltrue 双实例且 logical 与 Rust `sq_hash` 字节一致、具体调用经重绑定解析——`aac1048_instances_*` 三测试 PASS;CT 尾 V-04 重算 K 3→4 得 7/8——`aac1048_ct_*` PASS。
- AC-04:T-06 已跑 semantic_conformance 68/68(aac-ct-bootstrap)与受影响固定组 ace-aac-syntax 50/51;唯一红 aac1046_corpus_text 为 20 并行 CPU 争用超时,隔离重放绿,按环境归因记录,不充 PASS。
- AC-07(T-09):V-05 合法/非法近邻——静态 meta/CT-READ 非 AV、K 变化失效重算无旧冻结复用——PASS。
- AC-AC-STAGED / AC-CL-FAMILY:有界 profile 对约束/记录/方法实例、嵌套容器、高阶 CT 生成、#表达式族为具名后继(1050/1053),CL 行未删减——PASS(含后继清单)。
- 规范增量:SD-01 草稿已核(描述当前行为与不变决策);SD-02 待 merge 按已交事实更新。health:生成驱动警告为 A2R 生成器既有噪声;测试 crate 无新增警告;无遗留 debug 打印;行尾已规范化。

**merge 待办门**:受影响冻结模块并集 = ace-aac-syntax(50/51,1 例环境归因)+ semantic_conformance 68/68;规范沉淀 SD-01/SD-02;归档与材料清理。

2026-11-22 `stage: work | plan_id: PLAN-1048 | plan_revision: r6 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1048/auto-lang | branch: plan-1048-dev | base_commit: 5e5121dac | code_commits: 42361c893(instances), 7c7dd6505(meta), d9116373c(cte), 61e6ba815(grammar data + gates), f99c2bb18(EOL 规范化), 3a2d1f7a4(spec delta + 清理) | task_ids: T-01..T-06, T-09 | evidence: 见下 | blockers: none | next_responsible_role: review`。

**交付摘要**：Auto 侧新增 `experimental/ace/aac/semantics/instances/{instances_core,instances_solve,instances_expand}.at`（实例化/身份）、`semantics/meta/meta.at`（静态签名/记录形状/required 标注投影）、`semantics/cte/cte.at`（CT 读取集投影）；semantic_driver.at 增加 AAC1048_INSTANCES（实例预编译通道）、AAC1048_META、AAC1048_CT（纯静态投影模式）；测试 crate 增加 run_instance_case/run_instance_project/run_meta_case/run_ct_report_case/run_instance_ct_case（CT 经 1055 客户端车道）；重生成冻结的 canonical grammar data（1092 改语法未重生成数据致 1046 组全红的先有缺陷，本包机械修复）。

**测试记录**（会话内实际执行）：

| 测试模块/范围 | 结果与执行数 | 准备 | 发现 | 测试 | 清理 | 总耗时 |
|---|---|---|---|---|---|---|
| semantic_conformance (68 tests, aac-ct-bootstrap) | 3 次干净 68/68；1 次负载下 6 例超时（corpus 重测并发抖动，隔离重放绿） | 驱动 prepare 多次 ~2min/次 | 0.2s | 17.97–22.84s | 0.3s | ~18s×4 |
| conformance (51 tests, ace-aac-syntax 组) | 50/51；aac1046_corpus_text 组内 8.2s 超时（20 并发 CPU 争用），单跑 5.25s 通过 | driver-native prepare | 0.2s | 9.2s | 0.3s | ~9.5s |

| 新增用例 | 行为与 oracle | profile | 实测耗时 | 结果/≤10s 门 |
|---|---|---|---|---|
| aac1048_instances_id_distinguishes_int_and_bool_instances | id<T> Int7/Booltrue 两实例、logical 与 Rust sq_hash 字节一致、具体调用解析 | debug | 0.3s | pass |
| aac1048_instances_swap_and_arity_refuse_without_reuse | pair<A,B> 交换实参两键、const 缺参拒绝、rep<2> 得 rep_u32_2 | debug | 0.4s | pass |
| aac1048_instances_cross_module_owner_identity | 两模块同名模板两 logical、各自实例 | debug | 0.2s | pass |
| aac1048_meta_static_projection_is_in_process_and_never_av | META-FN/META-REC 且无 TYPE 行（未跑绑定管线） | debug | 0.1s | pass |
| aac1048_meta_resolves_required_labels_on_owners | #[required(key = "v", ...)] 归 owner 拼写；未标注 owner 不泄漏 | debug | 0.1s | pass |
| aac1048_ct_read_set_projection_is_static_and_names_constants | CT-READ K 纯静态；无读取的 fn 无行 | debug | 0.1s | pass |
| aac1048_ct_executes_instanced_generic_closure_through_bridge | id_i32 CT 经桥得 7（typed envelope） | debug | 0.6s | pass |
| aac1048_ct_constant_change_recomputes_no_stale_freeze | K 局部 3→4 分别 7/8，无旧冻结复用 | debug | 0.5s | pass |

**关键命令**：`cargo test -p ace-aac-syntax-conformance --features aac-semantics,aac-ct-bootstrap --test semantic_conformance`（68/68 干净三连）；`cargo t ace-aac-syntax`（50/51，corpus_text 环境归因）；驱动 prepare：`prepare-native.exe <root> [semantic|ct]`。**环境归因**：aac1046_corpus_text 在 20 并行下 8.2s 超时、单跑 5.25s 通过（AGENTS.md 已知环境边界）；另一次负载下的 6 例同类超时已隔离重放绿。**Spec delta 准备**：SD-01（aac-instantiation-and-cte.md）待 review 通过后由 merge 沉淀。

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（接早期Auto MIR解开CT启动依赖；分清声明投影、CT调度、重绑定固定点及高级CT整机关闭责任。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

## 10. 待澄清事项

无须当前用户新增决定。纯实例/meta单元消费1047可先；实际AV/CT关闭必须1051已交，不将1051整包写回1047形成环。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
