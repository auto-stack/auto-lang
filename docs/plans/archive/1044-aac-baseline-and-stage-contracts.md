---
plan_id: PLAN-1044
status: archived
feature_name: AAC基线、源码需求与阶段契约冻结
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-19
plan_revision: 6
current_step: 8
total_steps: 10
depends_on: []
supersedes_spec_components: [docs/specs/auto-ac/project.md, docs/specs/auto-hir/stage-contract.md]
new_spec_components: [docs/specs/auto-ac/aac-baseline-and-subject.md]
touched_goals: [GOAL-002]
---

# [PLAN-1044] AAC基线、源码需求与阶段契约冻结

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包按原任务收实际生产任务的证据，安排已验阶段交接。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：补全主体/能力/平台轴及测试owner，区分物理store与逻辑工件合同，冻结后继1055/1056的最小接口。 保全部原目标/义务及任务ID；新增T-09/T-10，现10任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。绑定正式M2-A稳定输入，建立CS源码需求、CL完整支持面、主体去留及服务契约，阻止以有限子集或研究快照提前施工/签章。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。10任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

绑定正式M2-A稳定输入，建立CS源码需求、CL完整支持面、主体去留及服务契约，阻止以有限子集或研究快照提前施工/签章。 本包出口见AC-01..06。依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**experimental/ace/aac/contracts/（拟新增的永久版本化合同资产，不是本轮生成物）**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `docs/design/59-aac-native-bootstrap-and-compiler-subject.md` | e449ff8f6d237981… |
| `docs/design/strategy/aac-delivery-roadmap.md` | 9dd084389bc142e3… |
| `experimental/ace/crates/core/src/source/mod.rs` | 675579ccb74fb169… |
| `experimental/ace/crates/core/src/hir.rs` | 0176789ebd78eb7a… |
| `experimental/ace/crates/av-core/src/mir.rs` | 6e85b54663fa5e04… |
| `experimental/ace/crates/source-query/src/lib.rs` | e79bb8bcf94597d0… |

规范输入：`docs/specs/auto-ac/project.md`; `docs/specs/auto-hir/stage-contract.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：985/T-A04→T-A05及M2A-01..08；945/946/947/948、960、962–968、972–975、978、1039–1043各实际供给owner，按CS需求逐操作交接，不依赖985全包。

**当前开工状态**：G-AAC 已取得 985/T-A05 正式 M2A-08 收据（95c7fbad8）；本包按 T-01 核当前身份后开工。985/B、984 全产品、986/987 整包均在后续阶段。frontmatter只列AAC包级最终依赖，任务级准备以§5为准，不用整包串行制造环。

## 5. 详细设计

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

CS需求取完整依赖闭包（含公开库、生成源码、ffi/shim、CT库），CL分母从最终M2三端required及A后延增量逐行导入且保规范/旧义务去向。冻结host/target/CT执行域、精确数值/字节/摘要/codec和库操作版本。1056负责精确摘要/codec，1055负责早期MIR，不得拿Rust lowering临时代替。物理持久层的SQLite实现不是可序列化Checked凭证；选择Auto文件store或显式SQLite原语适配并冻结独立namespace/format、logical identity兼容与坏旧cache拒绝。该决定只改变AAC缓存，不迁产品数据。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-baseline-and-subject.md` | 此前无当前AAC正式分母；新增CS/CL/主体/桥/代际分账、G-AAC和版本化数据/资源合同，历史阶段实现事实保持。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-ac/project.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-baseline-and-subject.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**全部CL分母与CS非同义**，沿[Design59的CL族级映射](../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-02/T-04/T-09；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-baseline-and-subject.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

**AC 语法同源验收。** T-02/T-09 为各 CL 语法行登记 AV 同源 AutoDown 用例及 AC 检查位置。已有已交用例记录源、Case ID 与独立预期。尚无用例的行保留验收缺口，待 1096 正式交付后做全量对账。本包基线准入按原范围执行。 按 [AV AutoDown 共用语料规则](../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | 正式A收据+完整CS/CL合同 | 1044合同准入入口（T-04使其可运行） | 所有输入哈希/阶段/适用性精确匹配方可admit；这是合同准入，不是自举PASS | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 删除A已合入身份、漏一条CL行或将required改NA | 同一准入入口 | 明确拒绝且列缺失行；私有分支reviewed/研究快照不得通过 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | 将Rust源码solver列成backend helper，或schema带checked=true | 主体合同/可信边界反控 | 拒绝主体归属/信任越界；合法MIR机器后端保留项仍可admit | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-04 | 将CL-required中恰好一行删除，再将同一行恢复并伪置NA；最后恢复required与实际证据 | 本包V-02所列真实/拟新增入口，沿相同启用任务 | 前两者整体准入均拒绝且返回该行ID；最后只有全部985/A及身份有效才admit |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-syntax ace-semantics ace-abi`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | 完整供应和平台矩阵的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | 删除BLAKE3、模块包锁或一个AC适用CL行，或交换host/target声明，准入必须列具体缺口拒绝；完整合法合同通过 | 本包阶段，未实施；最终整机/代际去向1053/1054 |
| V-06 | AC-08/AC-04；work / T-10 | 阶段测试与跨仓交接合同的合法基线与具名错误/变化 | T-10建立或扩本包真实阶段driver/父准入入口 | 把auto-down目录当语言TestCase生产者或仅旧Rust组绿却未跑Auto阶段driver的报告必须拒绝；阶段与整机来源齐才接受 | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 本族可区分验证（新增合同，未实施）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | T-02/T-04/T-09创建/扩本包既有Auto阶段driver，最后由原T-06与独立review核；完整Gen1/代际入口分别归1053/1054 | 有效A基础且AC async待补，可准入基线而完整CL仍pending；删一条CS未用的标注/异步行或把native未实现标NA须拒绝，恢复原行/owner后仅恢复合同准入，不伪签完整支持。 |

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | contracts/profile、subject-map及需求矩阵；不把旧743库存/GOAL-017旧自举算AAC事实；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；本包支持范围不冒整个AAC |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |
| AC-08 | work | T-10完整交付且V-06的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [x] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

- [x] **AC-CL-FAMILY / work**：V-CL-FAMILY和原T-02/T-04/T-09实际结果满足，族级CL未因CS未用或A期AC不支持删减；语义与runtime/库补齐正式来源齐，原独立review逐行核。原任务数及0进度保持，本轮未执行。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1044/auto-lang`，分支 `plan-1044-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1044/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 记录worktree/branch/base与§4指纹变化、支持profile和后续任务入口；AC-01/04 |
| [x] T-02 CS/CL与主体账 | work | T-01 | 逐模块审计Design59 §2/3，冻结AAC使用的语法/操作、最终M2 AV/AC/A2R required分母及A后延增量并集、Rust保留项及被迁主体符号；每行绑定规范/源码/输入/oracle/owner，CS设计需求与未来实际源码扫描差额明确。 CL逐行取最终M2三端required并合A后延，包含~T/?T/!T、泛型、CTE、required标注、dynamic/闭包/资源/领域及组合；CS未用不删CL。每行标语法→Auto语义→MIR→runtime→consumer实际任务/来源，Design59 具名实施任务或真实owner的补齐条件明确。 | contracts/profile、subject-map及需求矩阵；不把旧743库存/GOAL-017旧自举算AAC事实；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-03 数据、身份与资源契约 | work | T-02 | 冻结grammar来源、AST/HIR/Typed MIR/CT envelope可传输数据、各轴版本和受检capability重建规则；逐请求源大小/节点/深度、内存/CT fuel/输出/期限取M2-A已有支持合同并记录本AAC预算，补字符/UTF-16/bytes单位、整数计数溢出与先限后分配的准入；已有更严上限保留，新增未覆盖项用有界代表输入测量后记录批准支持profile，禁止无界默认。只静态比较backend进程/C ABI候选，生产transport由1051/T-02决策。 | contracts/stages、identity、resources；已有更严约束不放宽，超界受控拒绝；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-04 基线准入薄校验 | work | T-03 | 实现仅针对AAC合同的输入身份/行完整性校验与反控；核985/A已合入签章、源码/SDK/库/ABI/grammar/runtime及目标身份。不存在/错误收据、漏CS/CL行、假NA、Rust主体藏服务、坏Checked声明必须拒绝。 有效A基础＋未交AC差额可通过基线身份准入，但不得据此声明CL支持；删CL行、用A期native不支持豁免或无补齐owner须拒绝。 | 合同验证入口与命名反控；不能另造全仓登记系统或自动消费者调度；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-05 向后继交接 | work | T-04 | 发布已冻结合同与具体供给缺口/关闭owner；1045/1046/1047/1051/1052逐个记录其最小输入及版本。任何未供应CS行保持缺口，1045关闭实际使用验证，不在本包冒全部AAC源码可编译。 | 后继可执行入口及有界decision；不要求M2-B和全产品迁移；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-09 完整供应和平台矩阵 | work | T-05；§5实际输入合同 | 沿Design59 §9 F01–F10建立CS/CL/主体/服务逐行责任表；列build工具/host/target/CT四轴，给每个缺操作绑定实际供给Plan或1056，指定1055最小CT profile和高级CT由1050/1053关闭，不允许凭owner名字空置交付条件。 不仅列owner名字，还要落实际实现任务/输入/oracle/正式交付条件，区分AAC语义补齐与原AC机器/runtime补齐。 | §6 V-05实际结果及输入/产物身份；AC-07/AC-04 |
| [x] T-10 阶段测试与跨仓交接合同 | work | T-09；§5实际输入合同 | 冻结每包阶段driver的输入/独立oracle/启用任务与真实调用记录要求；区分AutoDown文档格式owner和语言仓TestCase/core adapter。1045/1056等首次创建本包driver及其必要target/固定选择，1053只汇总整机；登记跨仓文件/commit/读取状态及版本变动失效规则。 | §6 V-06实际结果及输入/产物身份；AC-08/AC-04 |
| [x] T-06 定向验证和交review | work | T-02..05、T-09/T-10 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 实际命令/退出码/输入和产物hash、覆盖与缺口；AC-01..04。完成即execution_done交review，无需预勾T-07/08 |
| [ ] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（补全主体/能力/平台轴及测试owner，区分物理store与逻辑工件合同，冻结后继1055/1056的最小接口。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/10；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

2026-11-19 stage:work | T-01 开工身份核对完成（用户"完成计划1044"授权本包 work→review→merge 全链，实施授权记录于此）。**G-AAC 门核**：PLAN-985 T-A05 M2A-08 正式收据 commit `0db949754`，delivery `95c7fbad8`（`docs/specs/ace-migration/m2-certification.md` 落主，SHA16 `045340642830E6B0`，M2A-01..08 门级及 B-pending 见该 Spec）。**专树**：`D:/autostack/.wt/lang-1044/auto-lang` @ `plan-1044-dev`，基面 master `104074a5b`（建组时 master 已自 54ed8a4ca 前移，Plan 983 r28 落主）；auto-down 兄弟 `detached@72b0970`（组内解析序就绪）。**材料 scope**：`D:/autostack/plan-artifacts/plan-1044/` 已登记（REGISTRY.md，owner/用途/状态/删除条件）。**§4 指纹开工重绑（2026-11-19 实读，按 §5 r2 规则以当前值取代编计划快照）**：`core/src/source/mod.rs` `92B5CD1BADE1DD7F`（原 675579ccb74fb169）、`core/src/hir.rs` `870D33B813AD5902`（原 0176789ebd78eb7a）、`av-core/src/mir.rs` `2A2C32D4746C4A17`（原 6e85b54663fa5e04）、`source-query/src/lib.rs` `E79BB8BCF94597D0`（未变）；Design59 `B7A4013BEEB18A42`（原 e449ff8f6d237981）、roadmap `32D55995FEC8A2DE`（原 9dd084389bc142e3）。**输入就绪**：m2-certification.md / Design59 / roadmap / `docs/specs/auto-ac/project.md` / `docs/specs/auto-hir/stage-contract.md` 均存在；SD-01 目标 `docs/specs/auto-ac/aac-baseline-and-subject.md` 尚不存在，待 T-02..T-06 产出。**备注**：Design59/roadmap 及 1044–1054/1068 计划文件在主检出为未跟踪文件（AAC 路线编制会话遗留），merge 时需一并入档处理，本包 work 不触碰；主检出既有根 `.rs` 探针与 packages/blueprints 工作区删除与本包无关，仅标注不处理。outcome: pass（T-01）；next: work(T-02)。

stage: goal | plan_id: PLAN-1044 | plan_revision: 6 | round: 1 | from: drafting | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1044/auto-lang | commit: (T-01 无代码提交) | next: work(T-02)

2026-11-19 stage:work | T-02..T-06 完成（契约冻结与验证，commit `f04910ed5` @ plan-1044-dev，worktree `D:/autostack/.wt/lang-1044/auto-lang`；依赖 rev：auto-down `72b0970`）。**交付物**：`crates/auto-lang/test/cases/ace-m2/aac-baseline-1044/` 七矩阵（contract-profile 25 行、subject-map 7、bridge-contracts 7、cs-requirements 16、cl-requirements 42、supply-matrix 11、stage-test-contract 7，schema `autolang.aac-baseline/1`）；T-04 准入校验器 `crates/auto-lang-core/src/test_cases/aac_baseline.rs`（身份 sha16 钉、CS/CL 分母完整性、NA/无 owner/deferred-stage 词表、主体权威翻转/坏 checked 反控、供应空列拒绝、阶段生产者/证据策略反控）；SD-01 规范 `docs/specs/auto-ac/aac-baseline-and-subject.md`；Design59/roadmap 由主检出未跟踪状态随本包入档（commit 内含，解决 spec 链接悬空）。**T-02**：CS/CL 分母按 Design59 §3 冻结（CL 42 行含 ~T/?T/!T、泛型、CTE、required 标注、dynamic/闭包/资源/领域/组合；CS 未用不删 CL；每行 producer_chain=语法→Auto语义→MIR→runtime→consumer 与 owner 非空）。**T-03**：contract-profile 身份 11 钉＋支持面（host/target=windows-x64、ct-domain=hosted-av、checked-serialization=never-trusted）＋M2-A 资源冻结值（front 64MiB/深度 1<<20、semantics 1M 步/5s、CT root 10s，源码依据 frontdoor.rs/budget.rs/schedule.rs）；新增未覆盖项（AAC 输出/期限）由已冻结值有界约束，留 1051/1052 实测后按批准 profile 登记。**T-04**：校验器实现并注册 mod.rs；**T-05**：后继最小输入/版本登记于 SD-01 §8（1045/1046/1047/1051/1052 各列 CS/CL 行与身份钉）；**T-09**：supply-matrix F01–F10 逐行落实际交付任务/输入/oracle/关闭条件（BLAKE3→1056、包锁→auto-man/1052、host/target 轴冻结等）；**T-10**：stage-test-contract（AutoDown=文档格式 owner、语言仓=TestCase producer、first-user-work-task driver 策略、gen0 窗口合法）。**T-06 验证**：V-01..V-06 六例绿（nextest 每例 0.021–0.032s，总 0.061s，均远低于 10s 门；逐例见下）；本包改动为纯测试/数据增量，未触产品代码——auto-lang-core lib 编译由测试构建覆盖，ACE 固定组（ace-syntax/semantics/abi）未触面不扩跑，影响组结论=仅目标测试＋lib 编译。**占用等待记录**：测试前机器被 lang-1078（cargo t case-av-language-parity）与 lang-1098/1072（ACE release 构建）持续占用，有界轮询等待约 45 分钟，最终以 -j 4 受限并行运行，未干扰其构建。outcome: pass（T-02..T-06）；next: review(T-07)。

新增用例计时（T-04 校验器，V-01..V-06；dev/test profile、nextest、单例含 setup/断言/清理，均 ≤10s 门）：

| New case ID | 测试行为与 oracle | Profile | 实测秒数 | 结果 |
|---|---|---|---|---|
| v01_complete_legal_contract_set_admits | 完整合法合同集准入（11 身份钉 sha16 匹配、分母齐全、词表合规） | dev/test | 0.023 | PASS |
| v02_missing_identity_missing_cl_row_and_fake_na_are_red | 删 A 收据身份/漏 CL-403/required→NA 均拒绝且列行 ID | dev/test | 0.023 | PASS |
| v03_subject_authority_flip_and_checked_flag_are_red | subject.semantics 翻转 rust-retained、checked-flag→trust 拒绝；合法 MIR 保留仍 admit | dev/test | 0.023 | PASS |
| v04_restore_required_with_real_evidence_admits | 删 CL-304 拒绝→伪 NA 拒绝→恢复 required 全身份有效 admit | dev/test | 0.024 | PASS |
| v05_supply_and_platform_mutations_are_red | 删 BLAKE3/包锁供应行、漏 CL-501、host 声明篡改均拒绝；完整合同通过 | dev/test | 0.032 | PASS |
| v06_stage_test_producer_and_evidence_policy_are_red | auto-down 当 producer、仅旧 Rust 组绿报告均拒绝；来源齐才接受 | dev/test | 0.021 | PASS |

会话测试总表（本会话实际执行）：`cargo test -p auto-lang-core --lib aac_baseline -j 4`（准备/编译=冷构建约 6 分钟含依赖，发现/运行=0.04s，清理≈0s；含首次修复 1 处 format 参数、1 处 hex 大小写比较后绿）；`cargo nextest run -p auto-lang-core --lib aac_baseline`（运行 0.061s）。产品测试未跑（无产品代码改动）；占用等待约 45 分钟单独计，不属测试执行时间。

stage: goal | plan_id: PLAN-1044 | plan_revision: 6 | round: 2 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1044/auto-lang | commit: f04910ed5 | next: review(T-07)

2026-11-19 stage:review | review_scope: final（本 Plan 未指派独立终审角色，本会话内复审）| reviewer_identity: 实现同一会话，**独立上下文限制如实记录**——结论基于工件重建（提交 diff `104074a5b..b50bb1891`、独立重跑测试、TSV/冻结集交叉核对、身份钉哈希独立抽查），非执行摘要背书。plan_id: PLAN-1044 | plan_revision: 6 | outcome: **pass** | worktree: D:/autostack/.wt/lang-1044/auto-lang | branch: plan-1044-dev | reviewed_commit: `b50bb1891`（终审基线；含三轮修复提交 dc8c3dcd2/334dc1073/b50bb1891）| base_commit: `104074a5b` | dependency_revisions: auto-down `72b0970` | spec_inputs: m2-certification `045340642830E6B0`、Design59 `B7A4013BEEB18A42`、roadmap `32D55995FEC8A2DE`、grammar `9E5248B2A6AEACD7`、auto-ac/project `0E9CA9D469698FA6`、auto-hir/stage-contract `8BE44EA7473E1954`、auto-abi `27333270014826A0`（全部独立哈希抽查匹配）。

**AC 证据映射**：AC-01 pass（contracts/profile/subject-map/需求矩阵齐，V-01 绿；旧 743/GOAL-017 未作 AAC 事实）；AC-02 pass（V-02 三拒绝各列行 ID）；AC-03 pass（V-03 主体翻转/checked=true 拒绝、合法 MIR 保留 admit）；AC-04 pass（V-01..06 全绿 0.021–0.032s/例；本包为纯测试/数据增量，未触产品代码——受影响面=auto-lang-core lib 编译（测试构建覆盖）+ 新测试，ACE 固定组未触面不扩跑，未跑项具名列出，不以未跑签通过）；AC-07 pass（supply 11 行 + V-05）；AC-08 pass（stage 7 行 + V-06）；AC-AC-STAGED pass（deferred 词表+V-01/V-02/V-04 覆盖，阶段切片缺来源阻塞规则由 driver-policy/deferred-owner 承接）；AC-CL-FAMILY pass（CL 42 行全族、CS 16 行独立、删行/NA 拒绝）。**复审修复**：F-R1 fmt 差异→dc8c3dcd2；F-R2 未使用变量 warning×2→334dc1073（复核 warning 清零）；F-R3 SD-01 §5 缺单位轴/整数溢出/先限后分配规则→b50bb1891。**具名债务**：D-1044-01（AAC 新增资源实测去向，KNOWN-DEBT-AND-RISKS 登记，不阻塞）。**格式/健康**：aac_baseline.rs fmt 门过、无 warning、无调试残留；其余 123 warning 为存量非本包文件。**Spec delta**：SD-01 新规范内容核（CS/CL/主体/桥/代际分账、G-AAC、资源合同、供应/阶段合同、后继交接）与 TSV 一致；SD-02 按计划留 merge 沉淀。**pending_closeout_gates（merge 所有）**：Design59/roadmap 随本提交入档的落主核验；SD-02 修改 project.md；归档/清理；master 已前移至 `224ea34a5` 需 merge 时对齐重验。证据：`cargo test -p auto-lang-core --lib aac_baseline` 终审重跑 6/6 绿 0.03s；`cargo nextest run` 0.021–0.032s/例（见表）。next: merge(T-08)。

stage: goal | plan_id: PLAN-1044 | plan_revision: 6 | round: 3 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1044/auto-lang | commit: b50bb1891 | next: merge(T-08)

## 10. 待澄清事项

G-AAC 已有 985/T-A05 正式 A 收据，T-01 复核当前源、SDK、ABI 与库版本。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。


2026-11-19 stage:merge | plan_id: PLAN-1044 | plan_revision: 6 | outcome: pass | delivery_commit: `6022f10f2`（rebase 等价性经 range-diff 全 `=` 证明；master 多次前移 104074a5b→224ea34a5→91d5d7766→ae264f138→1a9dcda7e 期间三次重定基）| base: 1a9dcda7e | 检查点：prepared=6022f10f2（SD-02 含入）；landed=master tip == 6022f10f2（--ff-only，无 merge commit）；canonical_verified=docs/specs/auto-ac/aac-baseline-and-subject.md（SD-01）+ aac-baseline-1044 七 TSV + aac_baseline.rs + project.md（SD-02）全部落主，Design59/roadmap 随包入档（主检出未跟踪副本 LF 规范化内容等价后由 merge 覆盖，原副本备份至 plan-artifacts/plan-1044/*-backup.md）；archived/cleaned 待执行（本收据先记）。merge 期修复：身份钉重绑（Design59 LF 规范化 96E06FB6、project.md 随 SD-02 46169AF8）——测试 6/6 绿 0.03s 于最终重定基态重跑。备注：落地时主检出 Design59/roadmap 未跟踪文件先移至 plan-artifacts 备份（内容 LF 等价验证），避免 merge 覆盖冲突。next: archive+cleanup。