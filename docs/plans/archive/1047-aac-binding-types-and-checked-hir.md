---
plan_id: PLAN-1047
status: archived
completion_kind: delivered
feature_name: AAC名字绑定、类型推导与受检HIR核心
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-21
plan_revision: 6
current_step: 8
total_steps: 9
depends_on: [PLAN-1046]
supersedes_spec_components: [docs/specs/auto-hir/stage-contract.md, docs/specs/auto-semantics/semantic-contract.md]
new_spec_components: [docs/specs/auto-ac/aac-semantic-authority.md]
touched_goals: [GOAL-002]
---

# [PLAN-1047] AAC名字绑定、类型推导与受检HIR核心

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包承接项已补入 §8 原执行任务，复用 §6 真执行和独立反控；正式交接须标明原 required 行、实际源与产物、身份和结果。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：明确基础与高级语义接口，补完整声明/常量/外部接口校验及后续实例、资源生产者的无环接缝。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。把源码绑定/基础类型求解及HIR生产与verify迁到Auto，建立单一源码语义权威和不可伪造的阶段边界，供高级语义扩面消费。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

把源码绑定/基础类型求解及HIR生产与verify迁到Auto，建立单一源码语义权威和不可伪造的阶段边界，供高级语义扩面消费。 本包出口见AC-01..06。依据[Design59](../../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**experimental/ace/aac/semantics/、hir/与tests/semantics/（拟新增）**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `experimental/ace/crates/core/src/source/resolve.rs` | 80eb12467732dd65… |
| `experimental/ace/crates/core/src/source/typecheck.rs` | 429649c4b44d91ca… |
| `experimental/ace/crates/core/src/descriptor.rs` | 2104646d76867f04… |
| `experimental/ace/crates/core/src/hir.rs` | 0176789ebd78eb7a… |
| `experimental/ace/crates/core/src/verify.rs` | cba93ead10ab8ab0… |
| `experimental/ace/crates/semantics/src/infer.rs` | 77d4008d3b4c5638… |
| `experimental/ace/crates/source-query/src/produce.rs` | f31afa3a37ae44f0… |

规范输入：`docs/specs/auto-hir/stage-contract.md`; `docs/specs/auto-semantics/semantic-contract.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：1046 AST/module目录、1044类型/ID/schema合同；M2-A AC基础数值/聚合/函数语义与972库签名；泛型/CT与高级效果分别交1048/1049。

**当前开工状态**：1046正式Auto AST接口及1044冻结HIR；本包有限核心出口不冒完整CL通过。 公共G-AAC须985/T-A04独立复核、T-A05已合入A收据/M2A-01..08；不等985-B、984全产品、986/987整包。frontmatter只列AAC包级最终依赖，任务级准备以§5为准，不用整包串行制造环。

2026-11-20 `/goal 完成计划1047` 授权 r6 的实施、独立复审、合同内修复、合入及清理。985 T-A04 独立通过绑定 `ecc282df1`，T-A05 交付 `95c7fbad8` 是开工基面 `defff852a0d5d6c232f23f391220e1d4b18bbdd3` 的祖先，M2-A Spec SHA256 前缀 `045340642830E6B0` 与收据一致。1044/1045/1046/1051/1067 已归档。1046 当前交付 AST 为 preorder 行、原始字节范围和 source/origin ID。采用当前落主 grammar、schema 和 Windows x64 hosted 输入重新绑定，保持 1044 的 64 MiB 前端工作量、1,000,000 语义步与 5,000 ms 语义墙钟。语法程序 `0C6630E1CA87BE15`、AST `B332BAC8A2FE410C`、目录 `0287F45EC5094596`。开工 SHA256 前缀依 §4 顺序为 `C46B73ABE185FAF3`、`D0A858B4CBB25C59`、`B40E02648FE6B0A9`、`D58AD64491C17285`、`B89D2D5E42AD8122`、`77D4008D3B4C5638`、`A7905F3F95A8E5CF`。grammar/HIR 自 1044 冻结点已有正式变更，后续各切片以实际入口重验，保原 CL 分母。

专树 `D:/autostack/.wt/lang-1047/auto-lang`，分支 `plan-1047-dev`；组内只读 `auto-down` 基线 `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`。经 `.retention/policy.json` 登记 `D:/autostack/plan-artifacts/plan-1047` 为在途材料唯一目录，含临时文件与可删除 debug target。T-02 将直接消费 1046 的 grammar/AST 行与源字节，预注册声明、作用域和签名后才绑定表达式；V-04 观察顺序和槽位为两列，避免恰好同序的正例掩盖错误。

## 5. 详细设计

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

基础出口负责无待决CT的名字/模块可见性、数值/聚合/函数/常量与foreign声明的类型和签名；遇未完成泛型/CT保显式pending工作项，不发布全量Checked。1048通过已声明回调返回实例/生成语义事实，1049返回高级效果/资源事实；最终whole-module准入必须完整而非把pending当Any。早期1055只消费基础已受检闭包，因此无需先等完整1048/1049。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-semantic-authority.md` | 新增AAC源码语义权威及Auto checked HIR准入；Rust实现仅reference，未声明跨进程Checked信任。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-hir/stage-contract.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |
| SD-03 | modify | `docs/specs/auto-semantics/semantic-contract.md` | 增补 `aac.semantic-facts/1` 的单次输入身份、语义事实和已有查询页边界 | 明确两个版本化入口各自的准入规则 | AC-03/05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](../1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-semantic-authority.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**基础类型与语义边界**，沿[Design59的CL族级映射](../../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-03/T-04/T-09；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-semantic-authority.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

**AC 语法同源验收。** 在 T-06 运行已交的同源 AutoDown 绑定、跨模块、类型及声明用例的 AC 检查，按用例原预期核合法行为和非法输入的阶段、错误及源位置。 按 [AV AutoDown 共用语料规则](../../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | 异型命名实参按不同于参数的顺序，含可观测副作用 | Auto bind/infer/HIR阶段driver（T-02创建） | 绑定精确且事件保持源码求值顺序；本阶段可观察HIR顺序，native执行在1050/1053复核 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 错返回类型、分支类型冲突、未绑定/不可见符号 | Auto solver/verify入口 | 在准确语义阶段具名拒绝，不发Any/HIR假Checked，不调用后端 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | 合法HIR改写成错owner/悬挂DefRef并置checked标志 | Auto HIR准入入口 | 所有伪造体拒绝；合法近邻通过，禁Rust源solver后合法源码仍产正确HIR | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-02/03/05 | 按冻结语法定义f(a:Int,b:Bool)，调用先求b参数记事件B、再求a参数记事件A；把b实参改成Int作为反例 | 本包V-01/02所列真实/拟新增入口，沿相同启用任务 | 合法参数映射a:Int/b:Bool，HIR求值顺序恰[B,A]；反例在type阶段拒绝，不产生Checked；后端执行轨迹由1050复验 |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-source ace-semantics`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | 声明和foreign语义接缝的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | 未决CT不可冒whole-module Checked；已知基础程序可供1055。foreign中错Bool/Int、布局/句柄归属必须前端拒绝，合法近邻保独立signature/顺序expected | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 本族可区分验证（新增合同，未实施）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | T-03/T-04/T-09创建/扩本包既有Auto阶段driver，最后由原T-06与独立review核；完整Gen1/代际入口分别归1053/1054 | 嵌套Option/Result/Future在参数/字段/返回/集合和跨模块流转，合法typed事实正确；把未解包值当T、错E/Future载荷或标注归属错位须在后端前诊断，类型标签擦除不得通过。 |

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | Auto binder+签名目录、阶段driver及命名实参事件轨迹；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；本包支持范围不冒整个AAC |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [x] **AC-AC-STAGED / work**：§9 记录 1044 真实准入与 required 负控、55 版无工具路径的独立核心发布与供给阻塞，以及 1067 原样 await 源在实际 AC 运行和 AAC 具名交 1049。阶段证据供 1068 复核，最终三端 CL 必验分母保持。

- [x] **AC-CL-FAMILY / work**：`semantic-native-68` 在正式 driver 保 Option/Result/Future/Iter、公共字段与跨模块参数、返回和集合的递归类型行，拒未解包返回、错 E/Future 载荷和标注归属。原样 `nested-result-option` 已取得现行 AC 真 Windows 42；实际 `await` 和产物见 1067 腿。原 CL 分母保留，整机与机器扩面由原后继验证。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1047/auto-lang`，分支 `plan-1047-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1047/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 2026-11-20 已核 §4 授权、交付祖先、输入指纹、Windows hosted profile 及材料登记；AC-01/04 |
| [x] T-02 名字与签名目录 | work | T-01 | 移植scope/DefRef/签名预注册/局部遮蔽、模块可见性、方法/实参映射，保持先按源码顺序求值再按参数绑定；生成可审计ID/来源，拒绝未绑定/重复/不可见。 修现 AC 的共享声明/可见性/模块及可变全局初始化顺序的绑定接面。 | §9 `semantic-native-16` 及同源真 PE，覆盖绑定阶段 V-01..04 的可观察成立部分；整个 AC/T-04..06 仍须各自验证 |
| [x] T-03 核心类型求解 | work | T-02 | 移植基础数值/布尔/文本/聚合/函数类型工作表与精确类型错误，保不可变facts与任务工作状态隔离；循环依赖按冻结规则拒绝或求解，不以Any填补推导失败。 基础类型表显式区分T、Option<T>、Result<T,E>、Future<T>和Iter<T>，支持嵌套/参数/返回/字段/集合/跨模块所需事实；高级效果交1049，不得用Any/i32擦类型。 | §9 `semantic-native-18/19/25` 的实测类型与算术边界；V-01..04 的 HIR/后续消费、V-05 完整 foreign 与 T-06 全门仍各自核实 |
| [x] T-04 HIR与语义verify | work | T-03 | Auto生成unchecked HIR并通过唯一verify入口构造checked对象；核owner/类别/DefRef/调用类型/控制流及基础效果资源约束，descriptor输入同样重验，序列化checked位无权威。 由共享 Checked HIR 固定全局读写与初值、跨模块布局及非法引用的诊断/span，机器存储归 1051。 | §9 的已有 HIR 证据与 69 重验，F-1047-01/02 均关闭。 |
| [x] T-05 语义反例与接口交接 | work | T-04 | 对坏DefRef/归属、错类型分支/实参/返回值、符号遮蔽及求解预算做反控；基础结果对独立expected和Rustreference，提交1048/1049/1050/1052使用的immutable事实接口。 | §9 的 55、61 阶段实测和 `aac.semantic-facts/1`，冻结每份源码指纹与事实字段。 |
| [x] T-09 声明和foreign语义接缝 | work | T-05；§5实际输入合同 | 补跨模块公开类型/常量/方法、别名/重导出、全局初始化和完整数值类型/字面量规则；现AC descriptor/HIR输入同样走Auto binder/verify；语言级规范化/常量处理或已有pass也在主体中，pass版本/来源/资源/effect保持不可推给机器后端，但不强加新优化器。foreign声明消费967规范化接口元数据，核参数/返回/所有权/回调签名与错误模型，生成事实留原owner，不重做Rust/C解析器。用统一接口把未决CT/实例/高级资源交其生产者。 声明标注元数据进入版本化事实并指向其实际语义consumer；泛型约束/CT生成声明与未决异步效果有明确1048/1049交接，不能静默删除。 修 required 标注/foreign 类型、调用签名与 ABI 验证接缝；同例覆盖当前 AC 与 AAC。 | §9 的 967 正反例和 68 定向阶段、声明、数值及同源执行收据。现 AC Unit 和窄整数机器差额交 1068/机器后端，证据保留。 |
| [x] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | §9 105 项阶段门、修复控制及 39d092c36 最终固定三组。 |
| [x] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | §9 r6 final pass 绑定 39d092c36 与 SD-01/02/03。 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 已合入、归档、移除专树与分支。唯一待项为已登记 scope 的删除与验收，执行器自动审批拦截。 |

## 9. 复审记录

2026-11-21 `stage: merge`，`PLAN-1047:r6`，outcome `blocked`（只余清理）。归档提交 `b89da215de40d53c558359aa6c25467dc9d094f3` 已在 master。两个工作树均以最新 `wt-guard.ps1` clean 后移除，语言 9.124s，辅助 0.179s，删除分支 `plan-1047-dev` 并确认组目录消失。材料 scope 归属、绝对路径和 reparse-point 校验均已核实。对 `D:/autostack/plan-artifacts/plan-1047` 的合并清理命令及独立、单一字面路径 PowerShell 命令均在执行前被自动审批以 `blocked by policy` 拒绝递归删除。目录仍在，审计 37,929,124,489 bytes。登记表保有该条，状态 `archived-cleanup-blocked`，已写交付和归档提交及恢复条件。T-08 保待办，后续只清理该目录、验证实际删除、刷新登记与收尾记录。没有具名证据持有。普通变更未启动 release 或部署。

归档提交 `b89da215de40d53c558359aa6c25467dc9d094f3`。清理前路径对账已核 `D:/autostack/.wt/lang-1047/auto-lang` 54,559 文件、18,528,758,337 bytes，组内只读 `auto-down` 1,318 文件、27,021,855 bytes；登记 `D:/autostack/plan-artifacts/plan-1047` 456,317 文件、37,929,124,489 bytes。三处 reparse-point 检查均 clean，未见相关运行进程。工作树内是常规 debug/runtime-dev Cargo 输出，其余 CDB、Rust、A2R、AV/AC 探针、外部提供方、固定组各轮收据及临时目录均在登记 scope。已把所需命令、身份、结果与异常边界折叠到本节。D-1047-AC-NARROW/UNIT 可从 canonical 考例重放，无唯一未保存件，未留材料持有。

`archived`，交付归档文件的 T-01..09 无重号，归档当时 8/9，T-08 余工作为物料与专树清理。主检出实际已验证本计划、三份 Spec、设计与债务回链的 18 处目标，原活动路径已迁移。检查日志 SHA256 6adcc7914148f7eb9f1a0c370e298594acb0ce974e1bf061efe8f371f81252f9。路线表只提交 1047 所在行，工作区其他变动保留。仓库全局 Spec lint 存有其他模块既有路径/编号告警，1047 的链接按归档实际位置逐条通过。

2026-11-21 `stage: merge`，`PLAN-1047:r6`。`prepared` 基于 `39d092c3603333caa5f065eea02f92f8cacfb32e`，Spec、导航与其三条 1044 内容 pin 的后继收据 `7bd65c763a797eab873ae7a7a96558b9dc33da97`。阶段生产者、依赖、输入和原 CL 不变，复用已通过的最终固定组。准备版 1044 六项 6/6、exit 0、0.067s，日志 SHA256 f05cef03087f1b6036acd04ec537cff6a7207161270eee7099d43bea6a66b605。`wt-guard.ps1` 在合入前输出 clean，rebase 无改写，`git merge --ff-only plan-1047-dev` 使 master HEAD 恰为该交付提交。主检出内容身份下重新编制原 1044 六项（rustc `--test --edition 2021`、编制 0.332s、exit 0），执行 6/6、0 跳过、exit 0、0.097s，日志 SHA256 035a9e05251f9eafb6842f6c9094d72ac11764da7c5ecc5dd0f5c5ec3f985a97。

`canonical_verified`，master 上 SD-01 `docs/specs/auto-ac/aac-semantic-authority.md` SHA256 b2b4074efa50d5871a14c8a6a922e77d4ac3eb7d6f4982bdca5f06fe9905fdda，review 草稿转受管 LF 并去尾空行；SD-02 `docs/specs/auto-hir/stage-contract.md` SHA256 de3bf1bc2602d8a350f691b2452d203833b9076651e0369807cebb14faff6463；SD-03 `docs/specs/auto-semantics/semantic-contract.md` SHA256 a0d25fc42e4d387554f7d8bce9d26ac785e1f6a1f7c89659b05cb85b70a3f1cd。Design59 与 AAC 路线由归档路径替换，沿旧 1044 门重验。未发布应用索引或改写旧 ledger。当前准备归档，随后清理。

2026-11-21 final re-review **pass**。PLAN-1047 r6，`plan-1047-dev`，`D:/autostack/.wt/lang-1047/auto-lang`，reviewed commit `39d092c3603333caa5f065eea02f92f8cacfb32e`，代码修复 `756b798501de7959ace794e85e4735d0918948d1`，基面 `a5922b88149c91515304f5f57430732ede1eecd1`。同一会话独立选例并从实际输入、输出和代码重建结论，未启用额外会话。依赖 1046 `dca20c884737485d4da6dc32b9399e4245718769`、1051 `1d949f30b52be1f1544b43e53c494a9a6d0a2a77`、985/A `95c7fbad85649bbfd8ebbbab315cf8f572f3f509` 都是祖先。核对本计划未写旧 ledger。新增 Rust 文件按 crate edition 2021 的 rustfmt 全过，既有语法测试全文件遗留格式保留，补齐本计划新增导出及用例格式。`git diff --check master...HEAD` 通过，检查收据后专树仍干净。编译日志有基面既有 warning，例如 `ConfigPairValue` 未用在 master 同一行，修复没有引入 Rust warning 或调试输出。

| 验收 | 最终证据 |
|---|---|
| AC-01 / V-01、04 | Auto 实参异型绑定与源序事件独立对照，真 AV/PE 12；交换求值顺序的负控真实返回 21。控制流、初始化、作用域及原型经本次修复回验。 |
| AC-02 / V-02、04 | 错 Bool/Int、返回类型、分支冲突、遮蔽与非法模块分别命中实际源码。资源、OS 与未决子门对旧产物的保留和清理实测。 |
| AC-03 / V-03 | 无 Cargo/Rust 查找环境的 native 阶段处理真实源码。错 owner、悬挂 DefRef、失配实参、无根环与伪造控制图从 descriptor 入口重验。修复后合法近邻及实际坏槽位另行重放。 |
| AC-04 / V-01..05 | 阶段三 target 和受影响固定三组全部执行，见下。确认 raw nextest 终态、父子身份、原条件与环境排除。 |
| AC-07 / V-05 | Auto 消费真实编制且准入的 967 页。Bool/Int、回调、C 布局、跨 provider 句柄、缺供应/合法近邻有独立结果。原样 AutoDown 绑定、模块、类型及原型检查、同源现行 AC 与差额见 §9。 |
| AC-AC-STAGED / AC-CL-FAMILY | 1044 正反控、原样 1067 await、原样嵌套 Option/Result、Future 组合和效果归属已执行。AC 窄数值及 Unit 的机器差额随原分母交 1068，阶段证据不替整机通过。 |
| AC-05 / AC-06 | 复审通过，最终 Spec 落地、身份重绑及安全清理由 merge 执行。 |

F-1047-01/02 关闭。69 载体准备 94.962s、SHA256 1ca0bd8f38f2213f316b56581bcbbc3027386423539a94d411cca17fde92e30a。四命名复验 4/4，执行 3.175s、exit 0，日志 SHA256 45eb99a8212d063433fcd9f75584ed384dce6555d1b3fc386307c84d095f61e4。新回归验证 Bool 条件失败才运行后续块、后续局部承接外层初始化、终结后语句拒绝、有效原型只发正式定义。原复审源码和反向 Bool 近邻以及原型在最小 OS 环境产受检 HIR，实际读回准入，独立参考重验并原生链接后 PE 各 exit 31、0、42，运行 0.060、0.051、0.061s，SHA256 分别为 3ad529c8e60fe46951f2a8f8934724550114e2aa060a05bb9633696336097e1d、82e6caf2e308a9baec067d831dfb1a6267a140cba63ce5d0888c3ce42a3a28de、b1e06fd92f616748f4aa3c9750586193d7ee74c0a02997f84366cf41f81126b4。把实参槽位 1 改成 99，Auto descriptor 具名拒 `hir.reference`。独立重放日志 SHA256 0433210e677cf1d4c3c0917ada5294cbab3ad5f8c092761fd4f87376cf7532e9。

`cargo nextest run --offline -p ace-aac-syntax-conformance --features aac-foreign-semantics --test conformance --test semantic_conformance --test semantic_foreign --status-level pass --no-fail-fast --test-threads 1`，105/105、0 跳过、exit 0，执行 93.530s、命令 94.516s，日志 SHA256 a52b377ea1138053d39fe4fe3632d2c13c7af57e404e9d0a20a6b42ab0899095。55 个新语义/外部用例均具名计时，最长借用文本 6.447s，新增早返回 0.747s。格式提交与前一源码提交逐文件去空白等价，Stage 载体原指纹未变，三受影响入口定向 3/3、2.160s、exit 0，日志 SHA256 2f03c0968647d41d8d58aa83c2f6d131b5dd595d8aff8059c7546c9919df020c。Rust 编制显式离线 debug，日常 wrapper 只有 prepared 路径，缺载体或身份变化拒绝。

最终固定组 `ace-fixed-prepared-05` / `ace-fixed-run-05` 保持 03 的命令、原配置和 16 MiB 栈，来源与构件状态一致。准备 12.259s 和父运行 36.860s、exit 0。`ace-source` 932/932（原条件跳过 1，执行 7.692s），`ace-semantics` 295/295（跳过 10，10.207s），`ace-syntax` 159/159（跳过 3，9.120s）。每组 discovery/run exit 0、child validation true、not_run 0。清理并在父命令计量，工具没有单列独立清理时长。准备清单 SHA256 59990ac92349d86b0711a7d7739c8cb8a84cfc949b93c495248531667b5a78a0，运行 SHA256 c8b6d3bbdec9a5aacb1d6ee75e0f3f5995e11727dd8758de7a8b6199d3537e9a。Linux 腿/WSL 裁定排除。

核验规范增量 SD-01 草稿 SHA256 1e64bbeeB2e6b8ff789b25ebb25c48c53e0e972c97c528f363e24d2c7652585d。SD-02 与 SD-03 分别基于 HIR 阶段表 SHA16 8BE44EA7473E1954 和语义页合同 SHA16 8D3E71AE5134D4A1，追加已实证 AAC 子面与新 Spec 指针。GOAL-002 保现状态。标准原主线、分母、机端缺口记录均延续。next: merge。

2026-11-21 `stage: review`，final scope，PLAN-1047 r6，outcome `needs_fix`。同一工作会话独立从原语料、代码与执行记录核对，未启用额外会话。`D:/autostack/.wt/lang-1047/auto-lang` 的 `plan-1047-dev` 干净，reviewed commit `6e6181610781d7a82f925475f58b6904131d8ed8`、base `a5922b88149c91515304f5f57430732ede1eecd1`。固定三组复核通过，54 个新增语义/外部用例均已计时且没有超过 10s，最长 6.356s。主要条目 AC-01/04 由 F-1047-01 暂挂，其他绑定、资源失败、来源重验、外部签名及同源分段结果沿已有收据。

**F-1047-01，T-04/T-06，基础条件控制流**。复审选择 `fn side(a int, flag bool) int { if flag { return a }; return 0 }`，入口 `fn main() int { return side(flag: true, a: 31) }`。68 真 driver `AAC1047_HIR=verify` 返回 `ERROR hir.unsupported 0 54..62`。Rust core-i32 支持此普通早返回，类型阶段能表示 Bool 与 Int、调用的源码顺序与形式槽位。现 `lower_block_prefix` 把首个 if 后的语句视为终结以后内容，`lower_blocks` 又要求显式 else。该邻例要求将有效的后续语句交未命中分支，保既有合法双终结分支、作用域/求值和错误拒绝。关闭证据是该真实源与真假两个路径、后续局部及直接/伪造 HIR 的独立重验。回 work，恢复 T-04/T-06，保其余证据。未提交 Spec 或合入。

**F-1047-02，T-04/T-06，已经绑定的签名原型**。同身份以前文已通过类型阶段的 `fn add(n int) int;`、无重复注解的 `fn add(value) { value + 2 }` 与 `main` 调用 40 组合运行 `verify`，却返回 `ERROR hir.unsupported 0 0..18`。原型只补签名，应由唯一同模块定义生产 HIR。关闭需受检产物只含两个实际函数、Auto descriptor 再验和正常 PE 输出 42，并保无主体原型的原错误。与 F-1047-01 共用已打开任务，不扩大到后续机器能力。

2026-11-21 work handoff，revision 6，冻结 `6e6181610781d7a82f925475f58b6904131d8ed8`，基面 `a5922b88149c91515304f5f57430732ede1eecd1`，专树干净。`python scripts/fixed-tests.py ace-source ace-semantics ace-syntax --prepare-only --debug-tools --out <scope>/ace-fixed-prepared-03` 制备 50.807s、exit 0。对应 `--run-prepared <scope>/ace-fixed-prepared-03/prepared.json --out <scope>/ace-fixed-run-03`，原 16 MiB 栈，父 28.619s、exit 0，源码 932/932（原条件跳过 1，执行 8.293s），语义 295/295（跳过 10，3.338s），语法 159/159（跳过 3，2.706s）。三阶段父子核验均 valid，各无 not_run 和 errors，日志保留实际命令。制备清单 SHA256 523bd17cba8c16a70bda349445d6639c5014c50f527dbf8d7e4b16aaa453f9be，总结果 SHA256 6c5a5a8e372ec2a30a6772adfc7b7337a99163848af80becd3ba9e07b5b6314c。Linux 腿/WSL 裁定排除。另有 104 项正式阶段、1056 Atom 17 项、1044 六项及同源 AV/AC 实证见下。独立复审核 §§5–8 及原 CL 边界，T-07/T-08 待执行。

2026-11-21 6e6181610781d7a82f925475f58b6904131d8ed8 冻结阶段三 target，语法 50、源语义 49、967 外部 5，合计 104/104、0 跳过、exit 0，准备 100.104s，执行 92.643s，命令 93.495s，日志 SHA256 1aa5362c5cc06569a4f18a23dfee3e0c528fb3fc7fb5fa5260f3310062ce69d0。另以此载体原样运行 1085 五组项目输入，三个合法项目有稳定 facts，两个非法项目分别 `bind.private 0 8..14`、`bind.module 0 4..17`，是现 AC 对应整条 `use` 范围内的符号/路径。全局正例由事实中读回 `app.state` 的 Int 初值 8..10。日志 SHA256 11c3b05acc5f6cb1dc2a39e043309da1a4c550f97c02bddbc0bc79beb4bc6221。固定组正在按此提交重制，T-06 待最终结果。

2026-11-21 声明原型与布局收口。9d00febd1cbeb509ef5f3f5c0e89d1b65c104643 在 Auto 中关联同模块的原型和函数体、用原型填写缺失类型、校验隐式尾值及冻结 `PROTOTYPE` 关联。原样 `forward-decl` 在现行 hosted AC 执行 exit 42（0.065s，PE SHA256 38dadf2dfa9519fecc5922cf01264868ee45b82270b97be83e862340dfc4920c），原样 `prototype-no-body` 无机器产物，Bind `source.unresolved` 命中 0..14，AAC 原样 `bind.unresolved` 同范围。AC 日志 SHA256 2b732489ef3fba3371a8aa2c5ec068eda704a6eaa4668c2df8248b66b296bf5c；67 定向 1/1、1.432s、exit 0，日志 SHA256 3b8ce09e1515186b6608c4fa48a08397e617ea01222d706d2b2d7720b600d11d。6e6181610781d7a82f925475f58b6904131d8ed8 将多载荷的后续位置成员也加入非间接布局图，递归第二字段在其类型字节范围拒绝。68 准备 100.104s，载体 SHA256 245e0fd1d90945b8273b2b9710acf1fc5be62d6b399ecf8a3025a397f9acd4b3；原型/布局交叉定向 2/2、1.939s、exit 0，日志 SHA256 ffedd941641527cc1e837605777858d1d48df6db0685fc1f1d9788fb5a3ffef8。

AutoDown `imports-globals` 原样多模块检查。按 `compile_package_product_auto` 保各包身份，沿普通 Auto 初始化链降 Typed MIR 交 1051 AV 及 NativeCollections。`multifile-call`、`dependency-reexport-alias` 各 AV 42、真 Windows PE exit 42，两 PE SHA256 1d1f52e4bbdb160df8555fa7fea47643f5eea4cea2f78189da403c6def08a34c、e798443a4bb9b4024247cf8aeadb20f956f25cc8b2a9afbcd2eb0833a4c3aee0。`private-import` 与 `missing-import` 现行 Bind 按 0..14、0..17 区分 source.module-private 和 source.module-missing。`module-global-once` 同源 AV 42，一全局、一初始化、两根调用；现行 NativeCollections 报 `mn.unsupported.type`（Unit）。AAC 同源全局事实及其他实测全局存储用例已有记录，原样的 Unit 机器差额保给 1068/机器后端核对。源码清单日志 SHA256 a98bccf822d03aef252f1d9f046672e03f45562d6a843ae578963c87c62df642，运行日志 SHA256 79f87e75e2deb4c7f472e605e5ea8195a5d46739b75b5763309ed360eb1e512d。

2026-11-21 枚举载荷验证 28f283fec97f5ef6c52419c249d408b9d99089da。把具名字段、位置元组、模式作用域和有序构造实参保存在 Auto 事实中。正式 `named-enum-payload` 原样 SHA256 7423949f8a8324f6db4cb0cbebb9ef63810e2e9f6cd4483b9f1bc421d4c7a273，经既有语言前门和 hosted AC 原生链接，真 PE exit 42（0.134s，SHA256 2cd1cd72c07054911ae2cfd78c6607a504891e4f06d25cfc19422b0d144eec75）。同一源码在 Auto 绑定和基础类型检查通过。实参反顺序仍保求值序与声明槽位 `[1,0]`，Bool 代 Int 在实参字节范围拒绝；位置 `(int, bool)` 按顺序解构。66 载体准备 107.602s，SHA256 afe4ac9c5e19ab428453381985479739417d54b8c6833f5953f444b0ceefba8f。具名、嵌套模式和递归布局三命名用例 3/3、执行 4.580s、exit 0，日志 SHA256 dbdc40e592dd44b23d97a82bdaff3cb21a2efc5ba05f7d076d818e667daf00e3。AC 原样日志 SHA256 486b18c3e0eecfc51339d303dabb532d424f802fe9f082e71ec5cee3211abd9b。

2026-11-21 数值、管道与来源重绑。64 版对原样 `wide-exact`、`mixed-width-explicit`、`overflow-is-not-wrap` 生成类型事实，错 shift amount `5u8` 在实参范围拒绝。原 1079 宽值经现行 AC 真 PE exit 42（0.197s，SHA256 4eca6fa47c196b3ed1fc7a20e6e0b9d68e1fa6e3d10f1bbcb406ac8f0b9e8a3f）；溢出真 PE exit 70（0.069s，SHA256 1b42491187bd86d681287b39b7cb0ff394fd14c47511dee9553c690b26c8d5b0）。混合宽度通过普通 Auto 源码前端，现行 native 在 `PLAN-972 narrow typed arithmetic awaits the numeric backend unit` 具名停止。原样数值命令日志 SHA256 48884fc70d71924c376238f088a517cc01233a2ea9f366e8769d2d78d49fd59f。64 全 target 首轮 103/104，单个语法测试的 Windows 进程返回 -1073741795。65 定向重放相同语法载体回绿，原样管道方法和错布尔参数反例一同通过，2/2、exit 0、执行 2.697s，日志 SHA256 7db006f70a48d01d63490029d9c76a9af99d6db0b6d2650ab61e224d30a8bee2；完整重放 104/104、0 跳过、执行 86.743s、exit 0，日志 SHA256 b17909789edb827910a04509365fb1455cadb769aa9e7b796aa8e9b31e65c242。65 载体准备 81.521s、SHA256 59cbdb7481210ced6aed14fe3f0967f3a6221d4ff5d69d7601fb026e49da9171。

`pipe-method-and-field` 同源 SHA256 6da304a6a8aacdf55334231c1d4d33ed602b94bd60dd9797fd5f75847d2dc568。普通 Auto 前门和 `lower_checked_auto_language` 把原样源降到实际 Typed MIR，经 1051 `ace-aac-bridge` 的 AV/NativeCollections 两个入口，AV 值 42，真实链接的 Windows PE exit 42，运行 0.066s，PE SHA256 96b0ca727f47d8db14f2fa1c8356b7ca3b98bd0224b8da24987adb1f49731bd7，桥日志 SHA256 067d2ed6f388cc9795d4299e74febeeb18b96e9392a53c1038c9a25b31eca429。AAC 绑定两次 `.add` 到同一接收者方法，并经基础类型检查拒绝替换的 `false`；机器桥证据使用现行 Auto 语言前门和共享事实消费面。该路径保留语料原预期，旧 `ac-native` scalar 检查原配置未改。当前 grammar SHA16 EEBE77C36BEE5A75 重绑后，1044 固定六项使用同一 exe 重放 6/6、exit 0、0.165s，日志 SHA256 671ff16fa43b90ff3b289fae7df0da46cdcbdb9aec59cb2606144c1082420de4。

2026-11-21 AutoDown 原样聚合语料。旧 AAC 组合语法把 `!?int` 首符识别成 scheduler `effect`，62 版在 19..20 误交 pending。63 版在 AAC profile 优先匹配完整返回注解，仍保留无返回类型的 scheduler `!`；Auto 主体为原样 `Ok(Some(42))`、`Some(42)` 与 `Err("bad")` 生成各自嵌套类型，在替换成 `Err(9)` 后于该调用原始范围拒绝。旧语法 native 载体首轮因指纹过期给出 49 个基础设施红；重建语法载体 55.854s，语义载体 75.508s，同源冻结三 target 重放 104/104、0 跳过、exit 0，执行 89.943s，日志 SHA256 cd4143fb33cd41f054fa816c759e3aba6d40184f543cc947dbe2bc3ba74e57ff。

用 `compile_language_source_with` 和既有普通 Auto 实例化流程编译同一原样 AutoDown 源，通过现行 `build_hosted_collections_checked` 原生链接执行。`nested-result-option` 真 PE exit 42（0.081s，SHA256 df1b0408690130561d7c1c44fca68aad3722c7938cb7f9e006b21d7aec0ac6bb），`statement-fallthrough` exit 0（0.078s，SHA256 7d1114f2e0633dd701bc60304e309b29029d6cead1f75954008239ddc462d700）。`nonexhaustive-expr-rejects` 在 23..39 `source.pattern-nonexhaustive`，与 AAC 的 `type.match-exhaustive` 命中同范围；`missing-call-argument` 在 63..68 `source.argument`，与 AAC 的 `bind.arguments` 命中同范围。旧 `ac-native build-source` 对部分普通 Auto 语法报更早的 Text 错，保其首次观察。当前 hosted 链对 pipe 方法的 `lower.method` 缺口与 AAC 同源前端仍开放，未改共享语料检查的 ignore 状态。

2026-11-21 T-05 证据 9bfb6d70eeec7f7450ef46e4a49e99c846847f8f。`aac.semantic-facts/1` 冻结绑定、类型、模块、初始化、声明标注、具名待办及已准入外部页。相同输入重跑报告一致，逐字节修改输入改变源码 BLAKE3。56 版源模式首轮将两种 AST 形态混同，57 分别按语句和聚合分支重放 1/1。58、59 Gen0 准备分别在缺失 import 和词法作用域处失败，修正后 60 跑绿 103/103，记录 wrapper 10.649s 超过试验成本线。61 将已有语义错误在未产生候选 HIR 前返回，合法文档和需优先保留形状诊断的绑定错误仍完整验证。`cargo nextest run --offline -p ace-aac-syntax-conformance --features aac-foreign-semantics --test conformance --test semantic_conformance --test semantic_foreign --status-level pass --no-fail-fast --test-threads 1`，准备 77.164s，执行 78.582s，命令 81.356s，104/104、0 跳过、exit 0。记录 wrapper 3.033s，快照一致性 0.645s，AutoDown 模式正反例 1.020s。driver SHA256 dc4acd99acd14d0854602db92d757f1d5f398562a5fcfbfa631dc7c747386f1d，日志 SHA256 3903508ec69105b7ee1748c4099878ea696a19aea297dd70b9ac5facc3e2169c。

2026-11-21 首轮固定组。专树用 `python scripts/fixed-tests.py ace-source ace-semantics ace-syntax --prepare-only --debug-tools --out <scope>/ace-fixed-prepared-01` 制备，同三组通过 `--run-prepared` 执行。语法 159/159（3 原条件跳过），源码 931/932（1 原条件跳过），语义 294/295（10 原条件跳过）。源码深链 host 栈溢出与语义字面状态审计红均在 16 MiB 同身份重放复现。用已准备的真实 `auto-case.exe run --suite crates/auto-lang/test/cases/ac-semantics/ac-semantics.suite.ad` 核对迁移的五例，修前和修后各 5/5，逐例 check、源和 oracle 摘要与 PASS 终态逐项相同，两份日志 SHA256 均为 33282e4d202fd51e42cd5a60be7b545cf9b3a4ec61efd43aacdf67ec0c86419d。b7d0847d51722ecc621f80e7e66ca05c83e90112 补回五条 `status: executed`，旧审计二进制定向重放 1/1，日志 SHA256 3766a4fc09873a98d8d768f611044fa7e17df6263299895b346c56c3e80d64cc。源码深链仍须在修复后、与最终冻结三组一起重验。Linux 腿/WSL 裁定排除，归专用 Linux 环境。

源码深链修复 d38cd462ed523ec9ed95568b4c3821d2f52a0f1d。用 CDB 定位到 `GenericExpand::resolve_typed_ctors_in_expr` 的重复递归栈帧，日志 SHA256 509d8754875574553bf71866148cec638a8412d9789cc98a9ec36c79c4d8e7b0。采用显式待处理表按 lhs/rhs 源序访问二元链，仍将构造器节点交原逻辑。首轮编译借用重叠后用整节点解构，原 `scalar_source::deep_logic_chain_reaches_checked_bundle` 1/1（1.014s，exit 0），日志 SHA256 776c148c28715e38cf58df93198957a14b4c09d41ad6104cfe1ec4b476c20e3e。`scalar_source::deep_`、`source_generics_963::`、`record_source::` 定向并集 71/71，执行 4.665s，exit 0，日志 SHA256 dc8a4e731cc139c30ab6bae20ddba6def94e53b1e7ea7d55773f741759efe799。样本和 16 MiB 栈配置保持原样。

2026-11-21 V-AC-STAGED 异步差额同源腿。直接提取 1067 永久用例 scheduler_typed_reply_crosses_the_await 的原样源码，SHA256 f0f2b9359e8bc2c958ac338be01b571d1eae7d292d3d0a6af37c2d72565c0dc3。用现行 ace_native::sched_runtime::prepare 显式准备 runtime-dev（1.445s，exit 0，ABI mn.sched.runtime/1，SHA256 918cec37665b021ca894966988ca3efb63dcbe28bbbda94cd9ca5c14fa825769），在 run-debug 的真实 Windows link/run 入口执行同一源码，PE exit 0、stdout true 换行、stderr 空，0.429s。PE SHA256 a0065d4e9cadcdf509ed4ec1b84cbbcf16b701e44a64a458ea0e90e1344969ee，日志 SHA256 e279c63a91a95bc007d8736b257aaed01bd2eb005f910f6c46c3843158aa95b3。Auto 阶段同源只发 PENDING 3，分别绑定 task、reply、await（await 为 135..140），留给 CL-301/302 的 1049 语义 owner，报告 SHA256 62359daa87bb5f7561a6916459a1f40686bf6334ecd0410fdd1200ce3ad2c081。结合 1044 六项合同检查及 55 的不同供给/保旧子门，已覆盖本包 AC-AC-STAGED。

变动的共享 Atom 入口定向回归：cargo nextest run --offline -p ace-aac-support-conformance --test atom_data --status-level pass --no-fail-fast --test-threads 1，原 1056 的 17/17 全绿、0 跳过、exit 0，执行 64.495s，日志 SHA256 715a3c47e8b47e13b1eb0ecb26cadfced8687a354e0e365fb24daadfa103b3c2。

2026-11-21 T-04 证据 cbce6faf965ec35dac00476fbafc741f294e07dd。Auto 生成 copyable 名义记录布局和构造、嵌套字段读取、调用 view，并用与 descriptor 重读相同的 verify 核字段范围、名义类型、构造映射、借用只在直接调用实参。50、51 首轮分别在 Gen0 准备时发现旧 helper 残留和 owned List 元素整体读取，修正后 52 运行 48/49，Rust 独立绑定发现 inline 字段分隔符不符。53 绑定通过，独立 verifier 发现需显式 view，54 的 Auto verifier 进一步拒绝先引用后定义的 view。55 调整产生顺序后，语法/语义/外部三 target 全跑 99/99、0 跳过、exit 0。准备 69.791s，nextest 运行 82.495s、命令 84.791s。记录正反例 wrapper 8.423s。driver SHA256 f33f71564e0d899fa3bdde50b90ced8d75497651b4aa6c9064ffa0ec445c6c77，日志 SHA256 4f3ad23ee8483138f0b4e298c3399b5bec2746c56ea12607e368f696ca24a226。正式 foreign-suite-formal-01 由仓内 prepare-semantic-foreign 预建 Rust、C 与第二 C 提供方，收进提交 52405db18974b6d79553d369946b3d1fb66a1243。语法单独尝试 03 与准备进程共同占用 Windows prepare-native.exe，编译因文件在用退出 101，完成准备后同一组并入 55 重验。

55 衍生消费和身份：最小 SystemRoot、临时路径和显式输入环境运行 native driver，没有 PATH、Cargo、Rust 前端或服务环境。scalar 合法近邻发布 core，record 合法近邻发布 text-record，源码 SHA256 84a5f08df7756251c3d22d4354ddd756bdb6cc936181fcb474e07dbb981516b8，Auto HIR SHA256 6ce5306753e460b7c86e17d2c96e8113b5343e59cbd89b3e723d35be52cf295e。该 Atom 独立下游复验后直接降 MIR，经 ace-aac-bridge 返回 AV 42，再真实链接、运行 PE exit 42、0.057s，PE SHA256 5e7592f25583d724dba119b4ae53e33633175ea29c01e86293ee6d06160485ca。缺 967 页时，同一 C 句柄源码停在 pending；提供正式页后产 typed facts，现行 AC 同源 exit 7 已在上条记录。两次外部阶段均保旧 artifact SHA256 2c70d66fa05eff05a35bed0a2d90dc465da68998aeb11495e7510fc803fd6140，清理 staging 路径。staged-supply 日志 SHA256 6c9415ce53edc6bb1ee9cf5b16c4805b7c74827972dd60562742c29d3fc11215。实际 async 差额/供给的 V-AC-STAGED 分支需另核。

967 C 布局同源 AC 验证。现行 C 前门编译 f0c_point 源 SHA256 1c13497f6dabee142782375244006c19a9c28c93de5fc24a4261e79ca441316d，PE exit 7、0.783s。仅交换 x/y 字段的候选 SHA256 6b26e924f02c0aa2c745d133bfad0cc28c611a1d1f487ddb352cbc8fced9cce8，在符号 f0c_point_sum 的 108..121 被当前 AC 准入前门拒绝，前门 source.undefined；同源 Auto 正例保类型事实，反例 foreign.layout 落同一符号。原样日志 SHA256 9f69741c853cd849519555f37fe38d578de1df85919c8788ef749d69210b74a2 与 62b9f94564aa5c5eb5d77d7a6e0f4a75113662991cd78654c71d83d46047cad5。

2026-11-21 V-AC-STAGED 合同准入。用本专树原样 aac_baseline.rs 构建 1044 六个测试（先 cargo build -p auto-test-case --locked --offline -j 4，2.459s，随后 rustc --test 指向该包真实 rlib 与本工作树，0.311s）。开工后上游九个设计、源码或 Spec 的真实字节前移，原 profile 在当前树最先被 Design59 旧 pin 拒绝，0/6，exit 101。提交 006937f8e3a3f01210c7276396bd4de2ac358e69 将九个 pin 绑回已落主实际输入。A 收据 045340642830E6B0 与 delivery 95c7fbad8 仍核祖先；阶段、资源及 CL-required 矩阵保持原合同。使用同一 exe 重放，6/6、0 跳过、执行 0.10s、命令 0.126s、exit 0，含删 A 收据、删 CL-403、required→NA、主体越界、失败证据边界。exe SHA256 49f1fef0ef8323097eb46885fc46a195028d3742be57297ecf996b00b848e02e，首红日志 SHA256 2755b9ead61e46c4a23117ccb0ca548124489e85cc9dce3833d105bdd9ac52cd，重放日志 SHA256 8e0a9b19aeb95e0b29f80171a9b2acd52e53c83221140584467ed7b3c249bf58。若沉淀现行 HIR Spec 导致其 pin 更新，归档前用同入口核准入。实际阶段缺供应和恢复同源有效的 driver 子门另验。

2026-11-21 在途检查点 f28048b7d5768f6a333e441a46febe5114477e0d。名义记录构造的检查为实参到字段的显式双射，字段注册沿原顺序。967 C 只读布局在源码类型核名字、列数、顺序与字段类型。真实 f0c_point 近邻通过；易序、改 Bool、同形异名分别在 f0c_point_sum 处报 foreign.layout。另用同一冻结 C fixture 派生并实编第二份具名 provider，二者各经 967 权威准入。第二 provider COFF SHA256 72c9531635d484fa2ec2fd833c1f2d2ff01e036f3cd95b34f9e707fd75156b09，合并的 20 条 SourceDecl SHA256 0d5f736419354b1ec11f9481fdc7cc76079e492e43faa2d8451e432d3b979609、BLAKE3 00070f21fcd6aee47ff62edb25d18796b58b1e2bba534e83a9cb5ee47f53e3d6。同提供方句柄通过，换成第一提供方实际签发的句柄在参数 span 拒绝。semantic-native-49 准备 71.794s、exit 0，双 target nextest 执行 37.537s、48/48、0 跳过、exit 0，布局/不同提供方测试分别 1.231s、1.197s。driver SHA256 5e8dba569a3214ce85ae044c923a400efb01465cd582209c52b33a736e24d23c，运行日志 SHA256 06a5e6157f08881defa53465c7627263dde788478abcce9e63c10b23a32727b5。

2026-11-21 在途检查点 02684661d93f952aeecdc7d5ed8123f7da1e1277。真实 967 C 提供方经 cl 预处理、编译与既有 HostAuthority 准入，输出 10 条规范化 SourceDecl。页 SHA256 6e75b4abfd94b61874ddd14d445a133466ec85d19c422f8d57d63b75ec02c334，授权 BLAKE3 256b41bda0c2a6933169e1324255a42b1ce024f2afa95861ff318b12e796f1c2，真实 COFF SHA256 ff919f2187972a5c424e1c5f077137913877e33b26b0a60444e2524a96b672b4。语义46首轮 44/46，观察到 Gen0 A2R 对引用局部 Str 的 return 比较先清空局部再比较。将判定先绑定布尔值后，47 恢复 Rust 路径（45/46）。实读 C 的 SourceDecl.logical 每条还含函数名，将 C/Rust 分别按其发行的 provider 命名空间核对后，48 准备 76.647s、exit 0，双 target nextest 命令 37.199s、执行 34.930s、46/46、0 跳过、exit 0。C 句柄与回调 wrapper 1.755s。driver SHA256 7481d68d8992991430cb9b287efc9971be77e0927b0e89210c56e0a721d64420，运行 SHA256 ec310ebf93ab8af688938fa502196b7d2456f44d98fa8558a2141746f272bfa0。外部句柄保持标准 Int 来源类型，同时 Auto 校验提供方签发的来源凭据。回调核命名函数与实际形参、结果类型，正例 / 普通 Int 伪作句柄 / 错回调参数 / 同签名间接回调均按独立判据观察。暂保外部整个 HIR pending。

967 同源 AC 当前前门独立观察。创建、读取、销毁源码 SHA256 dc47bf22a5990f261dba6e87a93d025a83abb56f09ff0bcdc2b4aab5033efe88，经现行 COFF 前门及实编译的 C 对象链接，PE exit 7，命令执行 0.598s，日志 SHA256 f0f93a11347188578014557ab6ccbe21ae95f3c07c66f99757dd54eb444cfb8d。仅将工厂调用换作 Int 7 的反例 SHA256 45dc7561749f9cbeb8d0cd4db6faaf72cd363e138024222ad7b18c466c698599，现行 AC 在真实提供方运行时 exit 70、0.524s，日志 SHA256 594c518524ce223c4105e865c983becf7888577b2ea94b13b2634d37f5a502b6。AAC 同语义反例在前端报 foreign.handle-owner，源准确到 h。两条路径各自保留诊断阶段，不据现行 AC 运行时拒绝认领 AAC 外部 lowering。

2026-11-21 文本消费定向验证绑定 d339324806cb9213f2b85278be1b58f92da400b4 与 semantic-native-45。相同 Auto Atom 经下游安全复验和 lower_checked_owned_with_init，交当前 ace-aac-bridge CompileTime 与 NativeObjects，以仓库 runtime-dev 静态文本库完成 Windows PE 链接和运行。UTF-8 输入 A中😀 加换行，源 SHA256 71c1e327555053879de8e088a3baae91209b6cfd1886dcab3b1757cb7506b8d9，HIR SHA256 f27eb8762ce85af3e6a2f21b29af56d7e26f34c20ab6bc443068853818bb8b00，AV 返回 4，PE 真实 exit 4 并记录 TEXT-SCALARS n=4。空串近邻源 SHA256 0f5ae03043f195b1d6768313c40064628bdfc6cadaef0ec4e0ac17ccd456a95a，HIR SHA256 e6a6cf6721a8aa31b4151e9663c248fa161c7dd5a4f40fb46f094bb07b50532d，AV 返回 0，PE 真实 exit 0 并记录 TEXT-SCALARS n=0。两个 PE 分别 0.167s、0.039s，SHA256 f94d5efca37f286bc66228c7b789bd81238bfdbbbb6a108f70781fb3ab185c4f 与 3ff68c26651666fcf01899f1c977ef3015f258d9ee9f44761637dfb75c0b531f。运行日志 SHA256 7ca5ea908cd21d3cee2083833bf3bdb4c0690716412254ccf98fd89c4703b2bf，runtime SHA256 603498a0d1c1cb30ad7e1e9e9fd5032ae2aaf833301e9d7faab941fbd979bf76。

2026-11-21 在途检查点 9af41a68173a5571320ad86c4b3ba215c85d6d0b。成对描述符准入读取实际源码目录并核对全局路径、访问符号、形参、可观察效果与启动函数。合法近邻通过。修改 setter 目标、降为纯效果、修改启动符号或替换模块路径均拒绝。semantic-native-40 准备 72.834s、exit 0。带 967 正式页的双 target nextest 命令 43.727s，执行 39.377s，43/43、0 跳过、exit 0。driver SHA256 ca70c0b3fd9c9a27aa5dc0cf80778b218bb8a04fb920513a24c3ed186d3942e3，日志 SHA256 81f956216ee6b13ef3a89abefa11464cd901bb491e70246427386df1f74e35aa。基础文本借用、布局与外部资源语义仍在途。

2026-11-21 在途检查点 d339324806cb9213f2b85278be1b58f92da400b4。Auto 从 UTF-8 源生产 str 借用、文本标量长度和 view 形参 HIR，当前仅准入静态文本和受检 view 的流转。Atom 准入校验域、text runtime 声明、借用方式、类型与从实际语句根可达的表达式。语义44首轮因夹具使用保留名 view 为43/44，修名后定向 1/1、7.768s。全局成对准入此前两轮耗时 12.965s 与 12.772s。Atom 解析和选描述符共用一次文档 arena，已知全局授权错误提前拒绝，所有成功候选仍校完整描述符。semantic-native-45 准备 72.950s、exit 0，双 target nextest 命令 34.065s、执行 32.101s、44/44、0 跳过、exit 0。全局反例 wrapper 3.464s，文本借用 wrapper 6.215s。另对脱离语句根、所有子引用自洽的表达式环定向负控，合法近邻通过，环拒绝，1/1、0.670s。driver SHA256 18656c2f149c81e8ab3555366a90675707b06ea3041e1f6b577505678c4a064f，双 target 日志 SHA256 3f87e01e2f33acfec3951f1faeb951aad18097d090bf583b8254fb4f48bfc4ff，表达式环日志 SHA256 7827917ce173f5a8ddb7edd467ddd461ad5512e194a6bc879d90ea8bb01ea05c。真实 AV/AC 文本消费、名义记录 HIR 与外部资源仍在途。

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（明确基础与高级语义接口，补完整声明/常量/外部接口校验及后续实例、资源生产者的无环接缝。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

2026-11-20 /goal 第二步在途检查点 `3efc6f347256e3dcd5bdda01b843e7c214fc5ee6`。实际输入走 PLAN-1046 的 Auto grammar/CST/AST，经 Auto 前向声明目录、词法块作用域、跨模块导入/公开重导出及有序实参映射。初始化依赖沿已绑定调用传播，全局读按依赖排序，独立用例验证跨模块初始化链和传递循环。直接编译 A2R Windows x64 dev driver，Gen0 只编阶段 driver。T-02 仍开着，方法接收者和完整签名、V-04 错类型邻例及 hosted AC 证据尚待关闭。

专树 `experimental/ace` 下 `cargo run --offline -p ace-aac-syntax-conformance --bin prepare-native -- <scope>/semantic-native-06 semantic` 退出 0，141.365 s。driver SHA256 `e9144bc83e85a06e2bc12c28d5e617d7f3562ef64f7255038d0d2753c6af52cb`；测试设置 `AAC1047_NATIVE_DIR` 指向该目录，`cargo nextest run --offline -p ace-aac-syntax-conformance --features aac-semantics --test semantic_conformance --status-level pass --no-fail-fast --test-threads 1` 退出 0。8/8，0 skip，Rust test 构建 2.810 s，测试（各例含退出与清理）4.832 s，父命令 8.394 s，发现与外层清理未拆。日志 `semantic-native-06-tests.log` SHA256 `c87ce29b9f12aecb891fa55fc97e026618aea9faa14b483e77aade478ea2dee1`。

| 本检查点新增用例 | 真实 oracle | A2R dev 秒 |
|---|---|---:|
| block_shadowing_initializer_sees_previous_scope | 初始化读取外层，内块与退出后引用分别命中局部和形参 | 0.333 |
| distinguishes_semantic_exhaustion_from_a_source_error | 两步预算资源拒绝，合法近邻保绑定事实 | 0.427 |
| forwards_and_maps_named_arguments_in_source_order | 前向调用实参 `B()`、`A()`，独立槽位 1、0 | 0.250 |
| orders_global_initializers_using_transitive_reads | 跨模块 Seed→Later→Buffer，读取成环具名拒绝 | 2.152 |
| rejects_duplicate_named_argument | 拒绝第二个 b，命中字节范围 | 0.234 |
| rejects_out_of_scope_reference_at_the_written_name | 同型名字出块后未绑定，命中字节范围 | 0.238 |
| rejects_private_import_and_import_cycle_separately | 私有导入拒绝、公开近邻通过、导入环另行拒绝 | 0.790 |
| resolves_aliases_and_qualified_recursive_calls_across_modules | 符号表、重导出别名及限定递归绑定同一个 DefRef | 0.392 |

当前仍为 work，进度 1/9，尚未送复审。早期制备中的 reserved 名称和 owned 容器 view-escape 红由改用普通标识符、整型目录和实际源跨度修闭。固定职责组为 T-06 验证，未在本检查点运行。

2026-11-20 /goal 类型检查点 `813fcd7d12683c99b6ad0c6bfe59de9852ebaa33`。新增 Auto 复合类型结构、声明/表达式推导、返回/分支及按槽参数类型检查。独立 A2R dev 制备 124.054 s，复用上表同目标 11/11 通过、0 skip，Rust test 构建 2.260 s，测试 4.262 s，父命令 7.338 s，发现/外层清理未拆。driver SHA256 `c2eb3d93e6a76353c5f8d7b5f99b22e62b820d667263aa83b1fd0eaeb80a17ab`，`semantic-native-07-tests.log` SHA256 `4d1813232082350aaa12264e492678918d32c157ca07093406f76c432a197c12`。新增参数槽错型的正反例 0.482 s、Option/Result/Future/Iter 嵌套结构和未解包反例 0.439 s、返回/分支正反例 0.533 s。后继方法切片制备 `semantic-native-08` 后同目标 12/12 通过、0 skip，测试 4.873 s，方法同名异 owner、字段类型及方法命名实参错型控制 0.520 s。该切片随下次提交重绑。

同一类型检查点正反例在 Windows x64 AC 真 PE 复跑：独立 dev 探针编译约 61 s；`prepare-debug` runtime 0.881 s，`run-debug` 读回 0.091 s、Cargo 调用 0，BLAKE3 `3fa2a017617c8076ee83ce5f3f76553838d2743e4a428ef66b4c2d24a435e898`。正式 `build_hosted_collections_checked` 的 Gen0 输入和链接分别 65.319/21.985 s，AC 产物 SHA256 `7e58717db9bb9d36f6effb1382506c682162439fd5dcfa65f9576baace588803`。材料 `ac-observe.py` 定向运行 5/5，退出 0，实际断言和进程 4.64/3.01/2.20/1.91/2.18 s（源码序与正确槽位、错实参范围、分支冲突、未解包返回、资源预算），父命令 14.266 s。测试输入和逐子项在 `ac-typed-cases.json`，该切片按已接 1051 供应验证。1096 尚未产最终同源全量收据，已交切片继续实施，未把 AC 定向结果记作完整 V-AC-STAGED 或最终 CL。

2026-11-20 /goal 标量 HIR 与声明检查点 `28b24bede4f28c464c97cd9826701f87db36a53e`。选择现有 `core-i32-draft` 作为首个真实产物切片，Auto 源码→类型事实→草稿→进程内唯一 verify→原 descriptor Atom；序列化无凭证，外部 Atom 经固定 descriptor 投影和同一语义 verify。逐项实证为同名方法接收者、i64 最小/u64 最大及越界、标注 owner、未决模板/CT/foreign 项、实参求值和绑定位、全局初始化、错误 owner / 悬挂 DefRef / 声称 checked / 错型参数或擦除算术溢出策略。Gen0 A2R 制备 72.284 s，产物 SHA256 `49380b9c438c45f2f6a130342844088548dd7340e4ba248717729621d9a8d45c`。

沿上文同 Nextest 入口冻结 `semantic-native-14`，首次 18/19，构建 2.480 s、执行 9.199 s，日志 SHA256 `be6e447697a2cf808fa5deb80fab92a6ae6806bce04e61c3a0606f2fabd47f17`。未决 CT 用例首个输入未通过现 AAC grammar，改用 1046 已验证的 `let v = #{ log2<2>(21) }`，定向复跑 1/1（0.297 s，18 未选），日志 SHA256 `5277da48ede0e6eb873a02a7f542fcdbc6af3979a16f199b72cbb1a185c11c7e`，driver 源和输入身份未变的另外 18 项保原收据。更早 Rust 参考曾拒 HIR 布尔常量，核 `source/adapter.rs` 后改为两个 i32 常量与比较节点，独立参考重验通过，取值实序 `B`、`A`，槽位 1、0。

真实同源 `language/calls/binding-pipeline.case.ad` 的 `av.parity.1080.named-binding` 从原唯一作者切出源码（SHA256 `88e33879ae1679b79451cda63c181cf07e2af8f06f123db590d99c543e949d78`），Auto 产生 Atom `57f41449f37b76e21430c34fbc1ec7b00c658aa11f63643345f7dee6dd665833`；独立 Rust 消费复验后由正式 `MnDriver::build_checked` 生成 PE `318cd1b8962388b19cc3cfbcb7617352ecfa1ac6dbd836264def7fa8fd904a49`。dev 探针构建 5.190 s、机器准备 0.357 s，真实运行 0.054 s、退出码 7 对原 oracle 7。此为同源定向证据，正式遍历其余 required 仍开放。当前 HIR 切片为单模块、标量函数参数和返回值、具名调用、算术与比较；共享全局/控制流 HIR、原 foreign 所有权与布局、完整规范与归档仍在合同中。

2026-11-20 /goal T-02 完成检查点 `6fe89c15f7b80c2b3944339a09ad921d0adf3435`。包＋模块名字绑定、发布可见性按源身份，跨包重导出和同名模块相邻；方法、参数签名/求值映射与初始化图已由实际 fixture 核实。首轮 21/22，跨包私有目标拒绝正确但落在上层转引处；按导入依赖先校核出口后校核使用者，复跑命中原非法重导出名称。A2R `semantic-native-16` driver SHA256 `dee2c79eb442cad161230239da4e5255f18d62bd384c33f05882aedd1f7df5eb`，同入口 Nextest 22/22、0 skip，测试 13.298 s，日志 SHA256 `efcb6944de7165636e58b7d7ea6b3ae49e121a2533c321f3c11064f27ce52af8`。新增跨包私有反例和同名控制 1.626 s，tuple 与间接签名 0.535 s，跨模块字段中嵌套 Future/Result/List/Option 合法同型与错 E 实参 0.639 s。

同轮主线已合 PLAN-1087，独立树清洁后 rebase 到 `c941db401`，四个提交 range-diff 均等值，当前 `47c904197`，原验证保留对应旧基面，下一次构建绑定新 grammar/backend。前置供给和 V-02/03 全链仍待相应任务。当前进度 2/9。

2026-11-20 在新基面验证 `9f40b9a63dff226fa5084378e54e1bd8b54a7726` 的 `semantic-native-17`。阶段组 23/23、0 跳过、实际运行 10.992s。新增终止分支两个方向通过，篡改为回边被 Auto 准入拒绝。driver SHA256 `62316c28957bc48e641827643a94369f322deee06e44c5ac66f3cd581c4bb5c8`，日志 SHA256 `98b68ec00b03e6e09c0892b4fee5159d6abaf5c8898a24adf886d03b8ce9e5a5`。

2026-11-20 在途检查点 `44019628a91078508c0175d8757fc77fb949e9d7`。真实 AutoDown `1080.untyped-old-form` 的调用实参求解两项未标注参数、尾值求返回类型，独立 Core oracle 接受生成的 HIR；后续错型呼叫拒绝。跨模块公开前向类型别名统一到同一目标，反向别名闭环拒绝。按 §9 同一显式 prepare/nextest 命令换用 `semantic-native-18`，准备 78.990s，nextest 整体 17.437s、运行 13.842s、25/25、0 跳过、均 exit 0。driver SHA256 `b0ef72fcc5eb42534167583d219c8ae9ab293016fb2b83328907b030884e7574`，日志 SHA256 `52fc69c4317b3439789caecad48e0dd3f40da09f754d7eb07c086948bcab4533`。仍以 2/9 记进度。

2026-11-20 在途检查点 `3100723c804a15bd325d5a945a654dbfbc9af407`。浮点字面量按目标宽度和最近偶数舍入的溢出分界准入，Rust 数值解析独立核实与分界相邻的 f32、同值 f64 及 f64 溢出；文本拼接推导 owned 文本，float 位运算在前端精确拒绝。该证据针对语义诊断，不作浮点 HIR 位载荷证明。`semantic-native-19` 准备 89.190s，nextest 整体 16.863s、运行 14.394s、27/27、0 跳过、均 exit 0。driver SHA256 `405f6f67028cbfa48fb6942ac95afde322e50f3c43eda50f5b2c4ba9f47f2c81`、日志 SHA256 `f395bb823dcc1622adb3f3a9e4910ea6c09878b7a823a708ecd47b44ce991789`。PLAN-1089 的独立 nextest archive 发生在本批 driver 准备期间，未同时运行本批阶段用例。

2026-11-20 在途检查点 `fbdcddc66aee547ffe63d836025a2dd3bc955997`。同一 Auto verifier 逐个核表达式的唯一求值位置及结构块的唯一入口。真源码正例与重读均通过，原描述符修改的无主表达式、共享求值及可正常终止的孤立块分别被拒绝。 `semantic-native-20` 准备 93.416s，nextest 整体 19.221s、运行 15.983s、29/29、0 跳过、均 exit 0。driver SHA256 `e87404d4b3fb8ad1655be05536ebf9c3f09b9504b27a605e069b977500c65b4e`、日志 SHA256 `55d7ea0f083cfac87e3560c3e608c21376bbaa710a84b517aab1a02708a0af49`。

2026-11-20 在途检查点 `1e76848f077ac401238708b7410fb9a9ecfc7776`。阶段 driver 继续使用 1045 的 `std.file.replace_atomic`，真实产生旧 Atom 后分别注入错返回类型、求解耗尽和 Windows 禁止删改共享的目标句柄。三个结果分别为 `type.return`、`resource.semantic`、`os.replace`，旧工件逐字节保留，临时文件均清除，解锁重试替换的 Atom 通过独立 Core verifier。首轮 `semantic-native-21` A2R 生成的 Rust 因返回借用文本 E0308 失败；改为报告写端直接消费输入，`semantic-native-22` 准备 68.108s，nextest 整体 19.555s、运行 16.779s、30/30、0 跳过、均 exit 0。driver SHA256 `cf70b9c847704c84341a6859d995262aabb5e397ab782e07458f625b706aeeb3`，日志 SHA256 `4c0980eb0d5fd033d0de9cde831874ec350a388548aaaeF4a757fb81cdad662a`。该证据是本阶段 V-02 故障保旧项，1052 仍负责整机发布。

同轮从 PLAN-967 实际 `ace_abi::rust_provider::build_source(f0_rich_provider_source())` 经 `f0_authority` 和 `ManualManifest::admit` 生成 4 个规范化 `SourceDecl`，构建同时产生实际 DLL/静态库。命令 `cargo run --offline --manifest-path <scope>/foreign-fixture-probe/Cargo.toml -- <scope>/foreign-967-f0-rich`，显式共用登记的 `target-ace` 和 `tmp`，exit 0、72.354s。规范化 JSON SHA256 `6cb87464c0a68fb127d98ae7756e9d11e5e4dd588b5331ee3edcc5f42539a8c4`、BLAKE3 `29c9d31745f220d4c7d654a9118e239ad16da469bd09d9ffb6a8738adb2c7276`，日志 SHA256 `7c86889e3fc2217bfac74e35b00d6ced956c5a13fdb5be1bc38d266c1be9f76f`。输入包含异型 view/i32 实参、Bool 返回、显式错误表及 noescape。该证据只关闭 T-09 输入存在性，Auto 消费与布局/句柄/回调正反例仍在途。

2026-11-20 在途检查点 `354d1ae079a9b13f9d9dc23142e8e899d1ab2301`，进度 3/9。Auto 求解器将真实列表字面量的嵌套 Future/Result/Option 元素类型带到集合与返回，异型 E 精确拒绝；原跨模块参数、字段、间接函数调用、浮点与整数宽度保留原预期。语义工作表的非零类型 ID 只在具体类型确定后写入，同一事实输出与工作燃料/错误列表隔离。 `semantic-native-25` 准备 79.889s，nextest 整体 19.421s、运行 16.212s、31/31、0 跳过、均 exit 0。driver SHA256 `9152edcd6757dce73d925ee54d2a263ea32e3407f22f1660c145ab8f94379c0b`，日志 SHA256 `346d807f1467a65c7432caabaeaf62551e25487dff2bc7547925f8936fe293a0`。

同一 driver 消费上述真实 967 规范化页，由独立 host 授权摘要准入。源级混合 view/Int → Bool、fallible 状态表可取得基础签名事实；错形参、错返回、篡改物理表摘要和未导入 provider 均在 HIR 前拒绝。暂不发布整模块 Checked，句柄/回调/作用域生命周期及 AC 同例仍需 T-09 后续验证。 `foreign-observe.py semantic-native-25 replay` 六项 6/6、exit 0，单项 0.148–0.176s，日志 SHA256 `cafcbad67a87162b6674eeacbf22330377f9bc9600a73cdf739aa2a12a8c395b`。首轮反例 oracle 曾因 Windows 文本写入将 LF 变成 CRLF 而错一字节；保留首轮结果，改用原始 UTF-8 字节重放，错实参为 `63..67`。首两轮 driver 准备分别暴露聚合 Gen0 的既有 std 装配集合无 JSON 和 stock JSON 的现代 for 语法尚未被该载体支持，最终使用只读取已验 SourceDecl JSON 字段的 Auto 游标，保留原规范化页及摘要。

2026-11-20 从 `c941db401` 重定基到含 PLAN-1089 的 `a003e2321`，十个在途提交 range-diff 全部等值。真实 967 准备与测试入口固化在 `098bc2d85571a808cd9072dcd90e3c7b187f4ac0`。准备 `semantic-native-26` 85.366s；`cargo run --offline -p ace-aac-syntax-conformance --features aac-foreign-semantics --bin prepare-semantic-foreign -- <scope>/foreign-formal-01` 38.528s；`cargo nextest run --offline -p ace-aac-syntax-conformance --features aac-foreign-semantics --test semantic_conformance --test semantic_foreign --status-level pass --no-fail-fast --test-threads 1` 整体 41.136s、运行 17.526s、33/33、0 跳过、各命令 exit 0。两份准备产物经 `AAC1047_NATIVE_DIR` 与 `AAC1047_FOREIGN_FIXTURE` 显式提供，测试过程没有构建 provider；规范化页与上一份生产的 SHA256 完全相同。driver SHA256 `3fbda991966b3e43c22f8fecbe0e47b63ba672b16ee11fa1ca8b93ae31199500`，日志 SHA256 `165d53ff098661db29f66b4ad4d1a9e2b9214c4e1473d10f0e497cea82bbd49f`。此复验不扩展已记录的通过范围。

2026-11-20 在途检查点 `b53aa35759405f0ee5601fd4794a8632acd805e1`。Auto 从真实语句生成标量局部绑定、读、带可变 place 的赋值与精确顺序，复用同一重读准入。独立 Core oracle 通过。交换初始化/赋值、篡改可变性分别得 `hir.initialization`、`hir.mutability`，两个终止分支合法读取祖先块绑定。首轮 `semantic-native-27` Gen0 拒嵌套的 owned place 文本参数，切换为显式局部 move； `semantic-native-28` 准备 81.913s，同一双 target nextest 整体 27.820s、运行 19.977s、35/35、0 跳过、各 exit 0。driver SHA256 `4af5573e28db0982a07ff2be989b18a671ab952b21bf3cf835b5e2b027669164`，日志 SHA256 `2114369640766d9f9b6f59bac52cd73230e0d01d56e35a653c8f6c12c2b0b511`。当前仍 3/9，T-04 模块全局、跨模块布局及基础借用/资源验收尚开。

2026-11-20 在途检查点 `80950c680ad10b6a7e8a0fe734a17046b45590c1`。共享 AutoDown 原例 `av.parity.1085.dependency-reexport-alias` 的三个实际来源经 Auto 绑定和求解进入同一 HIR，输出 `support.lib.answer`，Auto 重读和独立 Core verifier 均通过。 `semantic-native-29` 准备 72.944s，双 target nextest 整体 20.785s、运行 18.430s、36/36、0 跳过、均 exit 0；driver SHA256 `220179cb7bb01332b77871a67a41254422d986d698801a67f9252a79fbabb84b`，日志 SHA256 `bdca7a2ee8157e994f12f84e4ad115a9d6f4ac792960a1d92816631fe32ba6d7`。

复用同一 driver 与重建的 `ac-probe core-artifact`，直接把该原例三个源码文件的 Auto HIR 交真实 Windows AC，源文件 SHA256 依次为 `b7e4a9fb0922746beea9ae6f72595127172d9b455c3d1b0f8e600df0338b9a5e`、`58e3f65fb1b126869cb9ce78645ee5eea67182a66fadee5a8f9d95ddfe62cbfb` 和 `8a5b8fb6b40664b2b2ea9438c45bd0722d2a1a14eae0cc094355b022025ea585`。Auto HIR 0.257s，SHA256 `d12bdfd676c0f97e4786f628ff9450aa2c39d8187d7bd2bc1c970c6b6992ca8a`；AC 0.394s，PE SHA256 `9ff41823fa5a86f22a82d4bc212ac8d552326da9f0dae65130f65325a83ea8cc`，执行 0.089s，原预期与实际 exit 同为 42。重放日志 SHA256 `0080064732b89f84cffd1dd2da99b62f9a216cd5a441adc1852a25247365940a`。首轮 probe 的 inventory 带末尾空项，Auto carrier 返回 TextIndex trap；去除该空项后同产物重放成功，非法 inventory 的具名拒绝待补。

2026-11-21 在途检查点 `a977915734231359abfed6d7094bb43946f12470`。针对首轮跨包 probe 的空 inventory trap，调用分词前核空路径段与空行；对末尾空行、空清单、空包名/模块名均输出 `source.module`。同轮将类型求解的整数宽度判定接入 HIR 数值，保十六进制 42 和 `i32::MIN` 两个精确值；改写入站 Atom 为超过 u64 的负数时具名 `hir.type`。首两轮准备分别暴露 Gen0 限制 `List<u64>` 和 `U64→I32`，用已有宽度核验及负向 i32 逐位累积保留目标值，未扩大批准工具语义。 `semantic-native-32` 准备时间由日志时间戳观测约 69.715s，双 target nextest 整体 23.125s、运行 20.194s、38/38、0 跳过、exit 0。driver SHA256 `7a411dcdcf4b30c27e5ed7acdf1e087bec4ba82d50c867d81cf7633ccf1a0b3c`，日志 SHA256 `191f63da90d4b1d2d05f0b66a88b9464310cc825a74d48e4c17021294dc8c053`。

随后清洁专树重定基至已归档 PLAN-1088 的 `a5922b88149c91515304f5f57430732ede1eecd1`，14 个在途提交 range-diff 全等，检查点 `dcdbc65e6884bf57be4658176dd8349f88ee7aa7`；上述验证仍绑定各自原基面。

2026-11-21 在途检查点 `e60be7ceb37079a55d5c42e943bd8096c4cc047c`。`av.parity.1085.module-global-once` 使用原两个模块文件，核 `g3_appm5_state_n` 的 Int 类型、可变性、实际初值表达式 `40` 的 byte span 及三次读写的同一 DefRef。首轮 `semantic-native-33` 为 38/39，unannotated 顶层 var 经共享 AAC grammar 投影为 statement；绑定在显式包接入时将 program-scope var 归模块存储，再用原初值拓扑和类型表验证。复跑 `semantic-native-34` 准备 66.825s，双 target nextest 整体 22.554s、运行 19.933s、39/39、0 跳过、阶段 exit 0。driver SHA256 `0b66a484f54a5f02580689d2c3144009432d964118502f18339f9561c292908f`，日志 SHA256 `798f2c9f26c2e05e4e4fed41f0787d0ec9ce33a046920803b889223b507bde2e`。该证据建立 `aac.globals/1` 的类型化事实，实际全局 accessor HIR 和 AC 消费仍待实现/验证。

同基面显式核 1046 已有 `aac1046_grammar_data_identity_binding`，1 用例红、48 筛除、exit 100，首个漂移 profile `auto`。日志 SHA256 `8bfde39b93ecbabe912c2f9861613819de28af26dbc8dff87707104b00b77796`。不把此旧嵌入表作为当前源码语法签章，使用原 1046 生成器刷新后在新身份重验。

2026-11-21 在途检查点 `e271ab2f7bc365f5fdb5188edde315e45bfdd1c0` 修复真实输入依赖。现有 `generate-grammar` 1.177s、exit 0，八个 canonical profile 的数据 SHA256 从 `b0e6128e7f4368bbb2b82e728319857fbb27fb93dca0e2ab1dfe3655ef02e8bd` 变为 `9f72010b72b9270a5ffb5e39f3f80ed3c3ac0e623897bebecbdde7ddd3fbb03c`。单刷新版的 49 例为 26 绿/23 红，身份通过而 Auto decoder 拒 PLAN-1088 的 tag 4，明确不是语法数据生成失败。Auto 现在为该 token 验证引用与单次使用，检查相邻字节且拒绝空白/注释隔断，与原 Rust oracle 的比较区分显式类型实参与比较运算。没有删除原用例。

`syntax-identity-native-02` 准备 53.618s，完整语法 target 50 例执行 30.531s、整体 33.766s；49 个既有例全部通过，新增例因探针误用入口 `ast` 得 `UNKNOWN_CASE`，修正成现有 projection 入口后按同一 driver 定向 1/1、exit 0、1.656s；49 个既有例与新增例的证据合计 50/50，完整首跑 exit 100 保留。driver SHA256 `7e22aefc94eb64548871d4a2b8ede283f9d18df7b951987be16201088d07786a`。单刷新红日志 SHA256 `65563b668fcebf21cbf2f7c7b45f715263bf8a854572c69177d9c5576fbf8878`；完整复跑 SHA256 `d178d6f8f2eb6f4f87daaccc81b7fbca4b429fade0cb85a338668517b46d285d`，命名重放 SHA256 `59e8baca1635f2bd50a2f1b708250474ce99a55957cd86812c696b48b4a7cf8a`。

2026-11-21 在途检查点 `4819bbcbc664973a75b18405ac9eb6d6f635493d`。浮点十进制源码在 Auto 中直接达到目标精度，经 guard/sticky 与 ties-to-even 生成 f32/f64 原始位，Rust 标准解析独立比较 signed zero、普通分数、刚越 tie、f64 最小次正规数与负下溢。首轮 35 的位移拼装在 Gen0 解析失败，改用十进制余除构造指数位，不改输入/预期。以刷新后的八 profile 数据准备 `semantic-native-36` 62.952s，双 target nextest 整体 26.322s、运行 23.635s、41/41、0 跳过、exit 0；f64 次正规 0.484s，f32 和 tie 0.259s。driver SHA256 `bcc723a1b943a0a3d49498426a8232ab2092a772c54eac6fb2484a90781a84a6`、日志 SHA256 `0ec0a8470711453af7795da5f57f1c9088862d77c7d52faa7568175f84092222`。浮点位来自类型化事实，核心 i32/bool Atom profile 不扩大为浮点 HIR。

2026-11-21 在途检查点 `432d77be7111dd8d91c5cd872ec19e63e0d80dee`。真实全局标量读写和初值经 Auto 生成现行 get/set intrinsic 与启动函数；本地 HIR verifier 逐一核类型、owner、调用槽与可变性。首轮 37 的 41/42 保留缺失 owner 被误当 intrinsic 的诊断回归；按同时无 owner 与无 entry 区分 bodyless 后 38 阶段 42/42。38 的实际 AV 适配反查拒绝 core 域 `lower.domain`；调整全局形状由 Auto 发射现行 text-record 域，在其独立 Rust verifier 复核。39 准备 68.676s，双 target nextest 42/42、0 跳过、运行 23.049s、exit 0。driver SHA256 `dfee966435616ee38839dec662dd177f23dbf6a0c01f771e6799caddbefc530d`，日志 SHA256 `d64cf9139db696db5d43bd193e219bb95b946d4076fe9f289f04240cf1d4b688`。

实际 V-04 观察两条消费链，`global-av-observe.py semantic-native-39` 经当前 `ace-aac-bridge` 的 `CompileTime` 与 `NativeObjects`，Windows 真实链接与执行，Rust 只绑定描述符并从 Auto HIR 降 MIR。源 SHA256 `76b38dcc22e2b584785cf35014bdb2440c4925dcd3085e9f420bab802308ce5a`；0.351s 生成 Atom SHA256 `6ff7a16c4c0cafa2a0bea9fba30be0bdd8db7679cc14a6e2da47a45bd4c968be`。B 再 A，AV 值 12，真实 Windows AC exit 12（0.066s，PE SHA256 `c5c08e9166fef0c5c3072e3f34c0143d4edaf13f6dbc5f6ea03e09103284ef27`）。对照交换 eval_args 并同步映射到原形参，保持 Bool/Int 类型，AV 值 21，Windows AC exit 21（0.053s，PE SHA256 `318e65ff431ebe12a3a3cdebc74ee4c3289ed53ed03daeabdaccc408efb99a54`）；候选 Atom SHA256 `65cf48028fc657e563b8df34b867b2bd276ebecc896638554c1786e9e6d51ffe`。两例各一全局、一初值、两启动根帧，探针 exit 0。worker SHA256 `0d7868a430a798b1269a71c6e8b2181252b636fe0280b9138d8ca9ac9a42cfc6`，日志 SHA256 `83325b5f59fd5c47483356aee64a5717615ca4e6ae3871eb98d5cbe2c6f7c784`。text-record 候选的 Auto 成对重读及基础借用/布局的必要反控仍在途。

## 10. 待澄清事项

无须当前用户新增决定。1046正式Auto AST接口及1044冻结HIR；本包有限核心出口不冒完整CL通过。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
