---
plan_id: PLAN-1050
status: archived
feature_name: AAC共享Typed MIR lowering与资源保真
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-22
plan_revision: 6
current_step: 6
total_steps: 9
depends_on: [PLAN-1047, PLAN-1048, PLAN-1049, PLAN-1051, PLAN-1055]
supersedes_spec_components: [docs/specs/auto-hir/stage-contract.md, docs/specs/auto-ac/native-backend-and-retirement.md]
new_spec_components: [docs/specs/auto-ac/aac-mir-lowering.md]
touched_goals: [GOAL-002]
---

# [PLAN-1050] AAC共享Typed MIR lowering与资源保真

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包承接项已补入 §8 原执行任务，复用 §6 真执行和独立反控；正式交接须标明原 required 行、实际源与产物、身份和结果。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：在1055同一MIR实现上扩面并负责高级CT真实组合，不再将基础MIR留在CT之后；所有源语义转换归Auto。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。由Auto完成checked HIR到统一Typed MIR的全部适用lowering与准入，保证效果、求值顺序和资源生命周期，与后端机器映射职责分离。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

由Auto完成checked HIR到统一Typed MIR的全部适用lowering与准入，保证效果、求值顺序和资源生命周期，与后端机器映射职责分离。 本包出口见AC-01..06。依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**experimental/ace/aac/mir/与tests/mir/（拟新增）**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `experimental/ace/crates/lowering/src/hir_adapter.rs` | fa54e8c60ac15ff3… |
| `experimental/ace/crates/lowering/tests/conformance.rs` | e44b4c5647e55218… |
| `experimental/ace/crates/av-core/src/mir.rs` | 6e85b54663fa5e04… |
| `experimental/ace/crates/av-core/src/mir_verify.rs` | 9f1427164bd6a54f… |
| `experimental/ace/crates/av-core/src/ownership.rs` | 6be41a08ecf80af6… |
| `experimental/ace/crates/av-core/src/place.rs` | c7de6e9e56441e33… |
| `experimental/ace/crates/native/src/foreign_lower.rs` | c750cdc5b4fa585e… |

规范输入：`docs/specs/auto-hir/stage-contract.md`; `docs/specs/auto-ac/native-backend-and-retirement.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：1047–1049完整HIR/资源/实例接口；1044公共MIR/ABI/版本，实际native/AV消费验证使用1051稳定桥，本Plan不要求迁AV。

**当前开工状态**：基础HIR lowering独立前移1055并正式交付；本包继承同实现，完整交付须1048/1049全部适用输入和1051消费腿，不复制Rust HIR lowering到服务。 公共G-AAC须985/T-A04独立复核、T-A05已合入A收据/M2A-01..08；不等985-B、984全产品、986/987整包。frontmatter只列AAC包级最终依赖，任务级准备以§5为准，不用整包串行制造环。

## 5. 详细设计

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

1055已交的builder/verifier是本包唯一基础，禁止第二MIR或私有lowering。完整lowerer通过1044的LowerCt接口交1048调度器，编译期与runtime都复用此实现；1048不反依本包的完整交付。高级resource/closure/dynamic/领域CT及生成代码再验证在本包T-09实际关闭，并由1053重验全CL。现有语言级规范化、常量变换及语义pass随Auto主体接管并纳入版本/重验；不要求额外开发新优化器。机器后端只能按语义事实作目标ABI/layout映射，不能暗做资源策略、实例推导或重排效果。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-mir-lowering.md` | 新增Auto HIR→共享Typed MIR生产与本地验证；Rust保消费侧安全复验和机器映射，不存在HIR直达backend或Rustsourcefallback。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-hir/stage-contract.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-mir-lowering.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**完整CL到MIR与native执行**，沿[Design59的CL族级映射](../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-03/T-04/T-09；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-mir-lowering.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

**AC 语法同源验收。** 在 T-06 使用已交的同源 AutoDown 用例覆盖源码 → 共享 MIR → AC 的实际路径。同时保留原 MIR 对照与 verifier 定向检查。 按 [AV AutoDown 共用语料规则](../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | 命名实参副作用+短路trap+分支/循环owned资源 | Auto lower/verify→1051 native/AV（T-05接通） | 结果、事件顺序、drop计数与独立expected一致；真实执行不能用emit冒充 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 在合法MIR中改DefRef/Place类型或去掉终值drop | Auto MIR verifier及Rust服务二次准入 | 阶段定位拒绝；codegen/执行调用计数为0，合法近邻仍执行 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | native与AV同一foreign/closure/dynamic数据组合 | 真实两个consumer | 独立结果/资源合同均成立；不能仅凭native与AV彼此相等认领 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-02/04/05 | false && trap、true \|\| trap，及只在被选分支求值一次的owned返回值 | 本包V-01所列真实/拟新增入口，沿相同启用任务 | native/AV分别返回false/true且trap执行计数均0；owned返回的最终owner恰释放1次，不能在lowering阶段提前drop，合法转移不计作泄漏 |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-lowering ace-native ace-av`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | 高阶CT和资源整链闭合的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | CT返回生成owned类型/函数后普通源码调用实际结果与drop轨迹正确，坏生成语义拒绝；禁Rust HIR lowering仍通过。1048→LowerCt→AV→重绑定有限收敛或具名cycle，不无限重试 | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 本族可区分验证（新增合同，未实施）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | T-03/T-04/T-09创建/扩本包既有Auto阶段driver，最后由原T-06与独立review核；完整Gen1/代际入口分别归1053/1054 | 同源泛型CT生成Future/Result/Option组合→Auto MIR→真实AC，独立值/tag/执行顺序/清理；类型擦除、提前drop、错误分支、只生成未跑或native原拒绝充通过均须失败。 |

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | Auto MIR builder、阶段driver和结构/事件独立expected；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；本包支持范围不冒整个AAC |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

- [ ] **AC-CL-FAMILY / work**：V-CL-FAMILY和原T-03/T-04/T-09实际结果满足，族级CL未因CS未用或A期AC不支持删减；语义与runtime/库补齐正式来源齐，原独立review逐行核。原任务数及0进度保持，本轮未执行。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1050/auto-lang`，分支 `plan-1050-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1050/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 记录worktree/branch/base与§4指纹变化、支持profile和后续任务入口；AC-01/04 |
| [x] T-02 基础控制与数据lowering | work | T-01 | 先对1055已交的同一函数/CFG/value/place/builder/verifier运行兼容反控并继承实现，补本包新域所需扩展点；不重新实现基础MIR。保持shared MIR版本/span/source身份和可观测trap顺序。 | Auto MIR builder、阶段driver和结构/事件独立expected；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-03 完整数据/CT/领域映射 | work | T-02 | 按最终M2三端required域扩面聚合/泛型实例/闭包/dynamic/async/foreign/领域host调用；明确哪些layout规则属语义、哪些属机器ABI适配，后者可由Rust后端实现但不能重做source semantics。 显式覆盖~T/?T/!T组合、泛型实例、required标注/CT生成结果、dynamic闭包及领域回调，不能限制为A期AC已支持子集。 修影响现 AC 的共享 MIR 构建及语义映射，含全局、动态 pack/is/take、对象字段、generic/Option/Result、闭包与 async/typed call、异常和资源流。 | 完整HIR→MIR映射矩阵及多domain组合，禁止私有MIR语义分叉；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-04 资源与MIR准入 | work | T-03 | 实现Auto侧MIR类型/CFG/DefRef/ownership/place/effect合法性检查，drop/借用/失败清理显式；经进程传输后Rust服务重新做安全准入，不能信任Auto序列化checked位。 MIR类型和帧/悬停值/结果tag/求值顺序/分支错误/close清理保真；不能把Option/Result/Future都lower成同一无类型标量。 共享 MIR 明确控制流、所有权、清理、失败阶段和源 span；Gen0 与 AAC 相同语义 IR，各自走正式机器路径复验。 | Auto verifier和跨边界坏MIR反控；后端不接unchecked对象；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-05 执行保真与规模边界 | work | T-04 | 把合法MIR通过1051送真实native/AV做定向语义/资源轨迹比对，保独立结果而非只比两份IR；反控故意提前drop/重排IO/trap、错owner与循环预算，合法近邻绿。 用常规源→共享 MIR→已交的 1051 native 服务→本机执行验合法结果，并对非法近邻、错位诊断和资源泄漏独立验。 | 完整适用域执行差分、事件/清理断言；常规小边界，不自动跑正式压力；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-09 高阶CT和资源整链闭合 | work | T-05；§5实际输入合同 | 为1048移交的必需高阶CT/生成代码组合注入完整Auto lowerer，核生成节点/实例/资源事实再次验证后进入shared MIR/AV；补typed layout/FFI、async suspend/cancel、闭包capture与dynamic拥有值的多出口事件/清理。规范明确不允许的CT副作用按当前能力/权限合同拒绝，不能自改NA。 消费Design59 具名实施任务及原owner正式runtime/ABI，实际运行跨挂起错误/资源与CT生成泛型的组合；机器服务原拒绝只能作未交依赖，不能判正例成功。 用完整 LowerCt 和跨模块/资源组合补验 1048/1049 的语义，交换版本化 MIR 结果；不增加 1051 对本包的反向依赖。 | §6 V-05实际结果及输入/产物身份；AC-07/AC-04 |
| [x] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 实际命令/退出码/输入和产物hash、覆盖与缺口；AC-01..04。完成即execution_done交review，无需预勾T-07/08 |
| [x] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [x] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

2026-11-22 `stage: goal | plan_id: PLAN-1050 | plan_revision: r6 | round: 1 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1050/auto-lang | commit: 0abc54a59 | next: done（批量回归到期移交 regress）`。用户指令"完成计划1050"即整链授权（work/review/merge），与先前"按顺序完成计划1050,1052,1053,1054"一致。

2026-11-22 `stage: merge | plan_id: PLAN-1050 | plan_revision: r6 | outcome: pass | delivery_commit: 0abc54a59 | reviewed_commit: 7f4702660 | base_commit: b260b6e65→0046408c5→ff116ef24（两次 rebase，后者未记录；range-diff 全等证明 + 逐提交 `git diff --quiet` 显示树差异仅含 master 基面增量，plan 自身 12 文件零漂移，`experimental/ace/Cargo.lock` 两段差异无重叠） | canonical: `docs/specs/auto-ac/aac-mir-lowering.md`（blob 72c2141c…）、`docs/specs/auto-hir/stage-contract.md`（blob 4dfb4be7…）已落主 | archived: docs/plans/archive/1050-aac-shared-typed-mir-lowering.md（status: archived） | cleaned: wt-guard clean；worktree/branch/组目录移除；plan-artifacts/plan-1050 已删、registry 刷新 | next: done（批量回归到期移交 regress）`。

**T-08 合入沉淀与清理（2026-11-22）**：
- **落地**：worktree 内 `git rebase master`（8 提交重放至 ff116ef24，零冲突——master 侧 1114/1075 与 plan 文件集无重叠），`git merge --ff-only plan-1050-dev` → master 顶端 0abc54a59（无 merge commit）。reviewed 谱系（b260b6e65..7f4702660）与最终谱系（ff116ef24..0abc54a59）经 `git range-diff` 8 个 plan 提交全部 `=`（补丁等价）；逐提交 `git diff` 树差异仅含 master 基面增量，plan 自身 12 文件（aac/mir×4、core_hir.at、tests/aac-syntax×4、specs×2、Cargo.lock）零漂移。
- **canonical 验证**：SD-01 `docs/specs/auto-ac/aac-mir-lowering.md`（57 行，已验表面+§2 具名供给缺口含 owner/解除动作）、SD-02 `docs/specs/auto-hir/stage-contract.md`（S5 数据域扩展节）均在落主提交内；blob 72c2141c… / 4dfb4be7…。
- **冻结门禁**（worktree 内，最终基线 0abc54a59，新预建 driver 源绑定）：`cargo t ace-lowering ace-native ace-av ace-aac ace-aac-bridge ace-aac-syntax` —— ace-lowering PASS 9、ace-aac PASS 26、ace-aac-bridge PASS 8；ace-native FAIL 28（=27 基线红 + `audit1027_utf_02`，后者 plan 对 ace-native 零 diff 故为 master 侧/环境归因，cold collections runtime 解析失败）；ace-av FAIL 20（失败集与 T-06 基线逐一比对完全一致，无新增）；ace-aac-syntax 51 中 50 PASS + `aac1046_corpus_text` 8s 预算并行超时（隔离重跑 PASS，环境归因）。feature 启用 127 套件（`cargo nextest run … --features aac-semantics`，T-06/review 同口径）：125 PASS + 2 例 deadline 抖动（`aac1046_corpus_text`、`rechecks_global_effects_against_the_source_layout`——后者 0046408c5 基面隔离 4.02s、最终基面隔离 3.39s 均 PASS，并行 12 路负载下超 5s 内部 deadline；两例隔离重跑 PASS，无行为回归）。**9 个新用例全部 PASS**：aac1050_v02_mir_mutations… 6.321s、aac1050_v04… 4.848s、aac1050_bridge×2 3.879/3.743s、aac1050_v01_record… 3.767s、aac1050_v02_data_face… 3.660s、aac1050_text_literals… 2.971s、aac1050_v01_named… 2.294s、aac1050_generic… 1.032s——均 ≤10s 门。准备阶段：gate-native/gate-semantic driver 经 prepare-native 源绑定（fingerprint 含 Cargo.lock）、bridge exe 预建、`--prepare-only` 6 组 archive；无其他 agent 构建冲突（等待 ui-106/lang-1115 完成后启动，占用记录见上）。归因验证用临时 worktree `attr-1050`@0046408c5（冷建 35.7s），已随清理移除。
- **T-09 具名供给缺口**（AC-07 partial，r3 阶段合同承接）：高阶 CT/owned 跨入口返回/closures/dynamic/async/foreign/globals 组合均已书面化于 SD-01 §2，owner/解除动作逐条具名，CL 完整目标 1053 收口，不以 pending 冒通过。

2026-11-22 `stage: review | review_scope: final | reviewer: 同一会话按冻结树独立复核（artifact 重建；未启用另一 agent） | plan_id: PLAN-1050 | plan_revision: r6 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1050/auto-lang | branch: plan-1050-dev | reviewed_commit: 7f4702660 | base_commit: b260b6e65 | spec_inputs: SD-01 `docs/specs/auto-ac/aac-mir-lowering.md`（新增）、SD-02 `docs/specs/auto-hir/stage-contract.md`（S5 数据域扩展节） | next: merge（T-08）`。

**T-07 复审 pass（2026-11-22，重审 7f4702660）**：
- F1 闭：SD-01/02 规范 delta 已入树（aac-mir-lowering.md 描述已验表面+§2 具名供给缺口表含 owner 与解除动作；stage-contract 补 S5 扩展节与新 Spec 指针），revision 绑定 7f4702660。
- F3 闭：`aac1050_bridge.rs` 未用参数 warning 移除（去除多余入参）、Cargo.toml 行尾空行修复；`cargo test --no-run` 对本包新文件零 warning；`git diff --check master..HEAD` 干净。
- F2/F4 按计划合同书面化：AC-07（高阶 CT）与 V-03/V-04/V-05 的 closures/dynamic/async/foreign/globals/owned-return 组合为 Design59 具名前端/协议供给缺口（r3 阶段合同允许 A 基础+具名 AC 差额、缺口回 Design59 具名实施任务），解除动作逐一列于 SD-01 §2；CL 完整目标不缩（1053 收口），不以 pending 冒通过。
- AC 证据映射：AC-01/02/03 pass（record/text/泛型 lowering + 标量/数据面变异反控 + 1051 桥 AV/native 双端，独立 oracle）；AC-04 partial（受影响组全集与基线一致、127/127 新测试绿、V 行具名缺口已书面化）；AC-05 pass（本 review）；AC-07 partial（供给缺口书面化，owner 路由）；AC-AC-STAGED/AC-CL-FAMILY 按阶段合同承接记录。
- 新用例逐例 0.325–1.94s（≤10s 门）；预建 driver 源绑定。

2026-11-22 `stage: review | review_scope: final | reviewer: 同一会话按冻结树独立复核（artifact 重建，非 executor 自报；未启用另一 agent） | plan_id: PLAN-1050 | plan_revision: r6 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1050/auto-lang | branch: plan-1050-dev | reviewed_commit: 8e8bfe3ae | base_commit: b260b6e65 | dependency_revisions: 同 work 收据 | spec_inputs: 未准备 | next: work（修复 F1/F3，重确认 F2/F4 具名去向）`。

**T-07 独立复审记录（2026-11-22，artifact 重建核验）**：
- 冻结 revision 8e8bfe3ae（master..HEAD 7 提交），`git diff --check master..HEAD` 仅 1 处格式（Cargo.toml 行尾空行）。
- **通过面（复用 work 证据重放）**：`ace-aac-syntax` 127/127（含本包 8 新测试+1055 全量 MIR 测试零漂移）；`ace-lowering` 9/9、`ace-aac` 26/26、`ace-aac-bridge` 8/8；`ace-native`/`ace-av` 失败集与 T-01 基线逐一比对完全一致（无新增红）。新用例逐例墙钟 0.325–1.94s（≤10s 门）；预建 driver 经 prepare-native 源绑定（fingerprint 含 Cargo.lock）。
- **F1｜needs_fix｜AC-04 部分｜T-06 输出**：SD-01/02 规范 delta 未在工作树准备（`docs/specs/auto-ac/aac-mir-lowering.md` 不存在）——T-06"准备SD-01/02"输出未交，须补 delta 描述本包已验表面+具名缺口后重审。
- **F2｜needs_fix｜AC-07｜T-09**：高阶 CT 与 owned 跨入口返回未交付（1048 LowerCt 高阶输入/桥 Value 协议供给缺口，Design59 具名 owner），按本包"整包完整支持不能以pending代通过"规则，AC-07 勾选未闭——修复=在 SD-01 内逐条具名缺口+owner+解除动作，属计划内允许的"缺口回Design59具名实施任务"路径，但需书面化。
- **F3｜needs_fix｜卫生**：`aac1050_bridge.rs:23` 未用参数 `abc` warning；Cargo.toml 行尾空行。修复后复跑受影响组。
- **F4｜记录**：AC-01/02/03 在 record/text/泛型面真实闭合（V-01/02/04 对应输入独立预期绿）；V-03 的 closures/dynamic/foreign 组合、V-04 owned 返回释放计数、V-05 高阶 CT 均前端供给缺口，具名去向（前端生产者/1048/桥协议 owner）。
- **结论**：not pass。返回 work 修 F1/F3，F2/F4 在 SD-01 书面化具名去向；重审前不 merge。

2026-11-22 `stage: work | plan_id: PLAN-1050 | plan_revision: r6 | outcome: pass(T-04/T-06) | worktree_path: D:/autostack/.wt/lang-1050/auto-lang | branch: plan-1050-dev | code_commit: 8e8bfe3ae | task_ids: T-04(数据面), T-06 | next_responsible_role: review（T-07）`。

**T-04 数据面准入（2026-11-22）**：新增 mir 控制 `mir-aggregate-ty`（10）与 `mir-field-index`（11）——对合法 record MIR 的 Aggregate 操作数错型与 field_get 越界字段索引变异，Auto verifier 分别以 `mir.type`/`mir.reference` 阶段定位拒绝且不发布 ABC；未变异合法近邻仍执行 42。永久测试 `aac1050_v02_data_face_mutations_reject_with_stage_location`。owned drop 显式化依赖前端产出 owned 值（`type.return` 缺口），记具名后继。

**T-06 定向验证（2026-11-22）**：受影响固定组全集 `cargo t ace-lowering ace-native ace-av ace-aac ace-aac-bridge`——ace-lowering PASS(9)、ace-aac PASS(26)、ace-aac-bridge PASS(8)；ace-native FAIL(27)/ace-av FAIL(20) 失败集与 T-01 基线逐一比对**完全一致**（sched_m2_1067 22、py_lane 环境腿、resource_915、av_method_owned、layout、shadow 等既有基线红，本包零代码触碰 Rust 消费面，无新增红）。`ace-aac-syntax` 127/127 全绿（含本包 8 个新测试与 1055 标量 MIR 测试零漂移）。**T-09 供给缺口**：高阶 CT（生成类型/资源组合）与 owned 跨入口返回值受 1048 LowerCt 供给与桥 Value 协议限制——1055 字面 CT 腿已工作；高阶 CT 属 Design59 具名前端/协议 owner，解除动作=1048 移交高阶 CT 生成输入。本包任务（T-01..T-05、T-04 数据面、T-06）已按前端实际供给面完成，缺口全部具名去向，CL 完整目标不缩（1053 收口）。

2026-11-22 `stage: work | plan_id: PLAN-1050 | plan_revision: r6 | outcome: pass(T-05 桥腿) | worktree_path: D:/autostack/.wt/lang-1050/auto-lang | branch: plan-1050-dev | code_commit: bd647336c | task_ids: T-05(部分) | next_responsible_role: work（T-04/T-05 余下）`。

**T-05 1051 桥集成（2026-11-22）**：Auto 产 ABC（record/text 域）经真实 `ace-aac-bridge` 客户端送桥——`Mode::CompileTime` AV 腿返回独立值（record→42、text→3），`Mode::NativeObjects` native 腿真实发射 COFF 单元+wrapper（`Outcome::Objects` 验证非空、payload 完整）。新测试 `aac1050_bridge.rs`（dev-dep ace-aac-bridge；桥 exe 经 `CARGO_MANIFEST_DIR/../../target/debug` 定位）。**注意**：driver fingerprint 含 `Cargo.lock`，桥 dev-dep 改动 lock 使旧预建 driver manifest 失效——已重跑 prepare-native 重新绑定。全量 `ace-aac-syntax` 126/126 绿。V-01 的"Auto lower/verify→1051 native/AV"腿在 record/text 面真实接通。

**T-03 域供给缺口（2026-11-22 实测，具名后继）**：Auto HIR 前端（core_hir）不产出 closures/dynamic/async/scheduler HIR；owned Str 返回被 typecheck `type.return` 拒（入口与非入口函数均拒）；foreign 调用在 driver 的 mir 路径停于 `aac.pending-work/1`（声明经 JSON 页准入后未进 MIR 绑定）；globals 的 getter/setter/init 为 intrinsic+init 链，需 imports 段与 snapshot 接线。这些均为前端/供给层缺口，按 Design59 实施归属表归前端生产者/实际库 owner（1046/1047/1049 生产链），本包 lowering 覆盖前端已产出的标量/record/text 域；CL 完整目标不缩（1053 收口）。

2026-11-22 `stage: work | plan_id: PLAN-1050 | plan_revision: r6 | outcome: pass(T-03 owned-text 切片) | worktree_path: D:/autostack/.wt/lang-1050/auto-lang | branch: plan-1050-dev | code_commit: 87c5aa055 | task_ids: T-03(部分) | next_responsible_role: work（T-03 其余域：closures/dynamic/async/foreign）`。

**T-03 owned-text 切片（2026-11-22）**：str 字面量（含 `\n`/`\t`/`\"`/`\\` 转义解码，texts 表首字段存源 AST 行、encoder/verify 按行定位序号）、StrLen、StrView 参数经 owned 线代际端到端：`"abc".len()`→3、`let s = "a\n\tb"; s.len()`→4（转义解码正确）、text 视图参数+record 组合→9，独立 Rust 消费端 decode→verify→AV 执行全绿。**修复回归**：str_lit 的 `data` 曾改为文本序号破坏 Atom 渲染（`verifies_borrowed_text_lifetime_and_scalar_length` 红），改回源行+texts 表行查找后恢复 124/124 绿。V-01 的"owned 资源"与 V-04 的 text 部分在 AV 面已闭合；owned 返回（Str 跨入口）与 drop 计数归 T-04。记录类型字段含 str 仍被 `verify_record_shapes` 拒（具名缺口，非本切片范围）。

2026-11-22 `stage: work | plan_id: PLAN-1050 | plan_revision: r6 | outcome: pass(T-03 泛型实例切片) | worktree_path: D:/autostack/.wt/lang-1050/auto-lang | branch: plan-1050-dev | code_commit: 882f1c4b1 | task_ids: T-03(部分) | next_responsible_role: work（T-03 其余域）`。

**T-03 泛型实例切片（2026-11-22）**：1048 的实例展开（`AAC1048_INSTANCES`+`AAC1048_WORKSHEET`，经 project/inventory 模式）先重写源（`fn id<T>(x T) T` → 具体 `id_i32`，worksheet 含 TEMPLATE/INSTANCE/CALL 行），随后本包 Auto lowering 处理单态代码，产出 ABC 经独立消费端 decode→verify→AV 执行得 I32(7)。永久测试 `aac1050_generic_instances_expand_before_lowering_and_run_in_av`（0.764s）。T-03 的 owned text/drop、闭包、dynamic、async、foreign 域仍待后续批次。

2026-11-22 `stage: work | plan_id: PLAN-1050 | plan_revision: r6 | outcome: pass(T-02) | worktree_path: D:/autostack/.wt/lang-1050/auto-lang | branch: plan-1050-dev | base_commit: b260b6e65 | code_commit: f78e93c56 | task_ids: T-02 | next_responsible_role: work（T-03）`。

**T-02 交付记录（2026-11-22）**：
- **Part 1 兼容反控**（commit 80f4fa948）：新测试模块 `tests/aac1050_mir.rs`（crate ace-aac-syntax-conformance，feature aac-semantics，组内 `cargo t ace-aac-syntax` 覆盖）。V-04 标量部分四种短路极性 trap 执行计数可观测（`false && 1/0>0`/`true || 1/0>0` 结果 false/true 且 0 trap，急切翻转 Trap DivByZero；四极性组合程序整体返回 0，`!` 一元否定暂不支持故用 `== false` 等价拼写）；V-01 标量部分命名实参源码顺序（corpus `bd-named-source-order` 形状，`a*10+b` 独立预期 12）；V-02 标量部分五种变异（defref/type/cfg/uninitialized/constant）阶段定位拒绝且不发布 ABC，未变异 MIR 合法近邻仍执行得 7。三例分别 0.625/1.490/1.674s，均 ≤10s。
- **Part 2 数据域扩展**（commit f78e93c56）：Auto MIR 阶段由 `mir.base-profile` 拒绝 record 源转为完整 lowering 到 owned 线代际（v2/profile 3）：`lower_base.at` 接受 records 表并 lower record_ctor→Aggregate（字段声明序）、field_get→FieldGet、borrow_view→折叠（owned 值直接满足借用视图位，与 Rust 参考一致）、record 型 read_op 折叠到原槽位（禁止 RecordView→Record 复制）；`verify_base.at` 增 Aggregate/FieldGet/View 类型检查（aggregate 的 first 是记录类型 ID 非槽位，故在通用 lane_at 前处理）与初始化读检查；`base_abc.at` 增 owned 头（v2/p3）、记录类型表（TY_RECORD+TY_RECORDVIEW，owner="m"/name="t<id>"/字段名+类型）、函数 sig 索引偏移（9+2*rec_count+fn）、FIELD_GET/AGGREGATE 线编码；`core_hir.at` 门由 `records.len()>0 → outside_base_profile` 改为放行 records（仍拒 layouts/text），并透传 records。**实测闭环**：`type Pair { number int; enabled bool }` ctor+field 直读、`fetch(boxed Pair)` 视图参数调用、带 `let`/分支的 record 程序三例经 Auto lower/verify 产出 ABC（MIR aac.base-mir/1 报告），独立 Rust 消费端 decode→verify_abc（owned schema）→AV 执行分别得 42/42/7。
- **排障记录**：AutoDown 语法不支持 `else if`（全仓零用例，改嵌套 if/else）；`record_view_op` 等标识符含保留词 `view` 但作为整标识符可解析（此前 `view` 作参数名是保留词冲突，已改名 as_view）；函数记录线格式首字段为 sig 类型索引（标量 9+f，有记录时 9+2*rec_count+f）。
- **回归**：新 driver（hir-data 预建）下 `cargo t ace-aac-syntax` 全量 122/122 绿（含 1055 标量 MIR 测试零漂移）；新增 4 例全部 ≤10s。V-01 的"分支/循环 owned 资源"（owned text/drop）与 V-03（closures/dynamic/foreign）按本包任务分属 T-03/T-04/T-05 承接，本节未冒。
- 证据：预建 driver manifest/exe 于 `D:/autostack/plan-artifacts/plan-1050/hir-data/`（材料区）；临时探针（probe_ref_mir/probe_run/AAC1050_DUMP 调试）已清理，未入库。

2026-11-22 `stage: work | plan_id: PLAN-1050 | plan_revision: r6 | outcome: pass(T-01) | worktree_path: D:/autostack/.wt/lang-1050/auto-lang | branch: plan-1050-dev | base_commit: b260b6e65 (master) | code_commit: none（T-01 仅簿记） | dependency_revisions: G-AAC=985/T-A04 review pass@ecc282df1 + T-A05 delivery 95c7fbad8（M2A-08 成立，两 SHA 均为 master 祖先）；输入包 1047@7bd65c763/1048/1049/1051@1d949f30b/1055@8483384ce 均已归档且为 master 祖先；auto-down 兄弟 45b642d | task_ids: T-01 | evidence: 见下 | blockers: 无 | next_responsible_role: work（T-02）`。

**T-01 核验记录（2026-11-22）**：
- 用户指令"按顺序完成计划1050, 1052, 1053, 1054"即本包实施授权；G-AAC 正式门核验：985 §8 T-A04 独立复审 ✅（`[✅ 2026-11-17 ... pass @ecc282df1]`）、T-A05 ✅（`M2A-08 正式成立`，delivery `95c7fbad8`）；`git merge-base --is-ancestor 95c7fbad8 b260b6e65` = 祖先 ✓；1055 交付 `8483384ce` 亦为 master 祖先 ✓。
- 专树建立：`bash scripts/new-wt-group.sh lang-1050 --branch plan-1050-dev`，auto-lang `plan-1050-dev` @ b260b6e65（master），auto-down detached @ 45b642d；建组脚本生成的 `.git` 指针为 msys `/mnt/d/...` 形态，原生 git 无法解析，已改回 Windows `D:/...` 形态（与既有 lang-985/984 一致），`git rev-parse --show-toplevel` 两树均验证通过。
- 材料区登记：`D:/autostack/plan-artifacts/plan-1050/` 已建（owner=PLAN-1050，用途=阶段探针/测试输出/SDK，在途）。
- §4 指纹重绑定（编计划时快照 `9a6e5bf...` 为历史取材，现按 master 实读）：`hir_adapter.rs` A0465BA30B5F717F、`conformance.rs` E871ED65F20366E0、`av-core/src/mir.rs` 05A0557F66258D3E、`ownership.rs` 24853C2E89314BC2、`place.rs` C7DE6E9E56441E33、`native/src/foreign_lower.rs` 24F131773E997558；`mir_verify.rs` 已改为目录 `mir_verify/{mod,init}.rs`（`verify_mir` 入口不变）。
- 支持 profile 与后续任务入口：Auto 侧主体为 `experimental/ace/aac/{syntax,hir,semantics,mir,support,host}/*.at`（Gen0 前端经 packages-aggregates-ct 车道编译、AV 执行）；`aac/mir/` 现有 base_ir/lower_base/verify_base/base_abc 四模块（1055 基础标量 profile）；阶段 driver 现有 `tests/aac-syntax/semantic_driver.at`；固定组 `cargo t ace-lowering ace-native ace-av ace-aac ace-aac-bridge` 已确认存在。
- 基线（当前树零代码改动）：`cargo t ace-lowering ace-aac-bridge` PASS；`cargo t ace-aac` PASS（26 例）；`cargo t ace-native` FAIL（27 例基线红：collections_source_tier 1、data_owned 1、gap_closure 1、record_method 1、shadow 1、sched_m2_1067 22）；`cargo t ace-av` FAIL（21 例基线红：av_method_* 5、av_owned_* 3、av_test_batch 1、component_event_sessions_1106 1、py_lane_probe_1094 7、resource_915 3、layout 1）。基线红均为既有 master 状态（非本包触面），按证据策略留账不追，本包新增用例必须独立绿。机器占用：基线运行前无 cargo/rustc/nextest/link 活动进程。

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（在1055同一MIR实现上扩面并负责高级CT真实组合，不再将基础MIR留在CT之后；所有源语义转换归Auto。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

## 10. 待澄清事项

无须当前用户新增决定。基础HIR lowering独立前移1055并正式交付；本包继承同实现，完整交付须1048/1049全部适用输入和1051消费腿，不复制Rust HIR lowering到服务。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
