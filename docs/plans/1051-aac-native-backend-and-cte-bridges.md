---
plan_id: PLAN-1051
status: drafting
feature_name: AAC native后端与CTE执行服务桥
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-19
plan_revision: 6
current_step: 0
total_steps: 9
depends_on: [PLAN-1044, PLAN-1045]
supersedes_spec_components: [docs/specs/auto-ac/native-backend-and-retirement.md, docs/specs/auto-abi/project.md]
new_spec_components: [docs/specs/auto-ac/aac-service-boundary.md]
touched_goals: [GOAL-002]
---

# [PLAN-1051] AAC native后端与CTE执行服务桥

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包承接项已补入 §8 原执行任务，复用 §6 真执行和独立反控；正式交接须标明原 required 行、实际源与产物、身份和结果。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：补服务层的ABI/平台轴、capability隔离和transport可行性硬条件；明确外部Rust/C源码工具与禁止Auto源码服务的区别。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。提供AAC到现Rust机器后端和AV的最小显式服务，选定生产transport，严格禁止把解析/类型求解/实例化/HIR lowering藏进桥内。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

提供AAC到现Rust机器后端和AV的最小显式服务，选定生产transport，严格禁止把解析/类型求解/实例化/HIR lowering藏进桥内。 本包出口见AC-01..06。依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**experimental/ace/crates/aac-bridge/（拟新增Rust薄adapter）及experimental/ace/aac/host/bridge/（拟新增Auto客户端）**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `experimental/ace/crates/native/src/backend.rs` | aa4b920d4c21c1aa… |
| `experimental/ace/crates/native/src/typed_pipeline.rs` | 674befd7c815c74a… |
| `experimental/ace/crates/native/src/typed_session.rs` | 29186796e89fe450… |
| `experimental/ace/crates/native/src/worker.rs` | 600020b158424c95… |
| `experimental/ace/crates/av-core/src/mir_verify.rs` | 9f1427164bd6a54f… |
| `experimental/ace/crates/av-core/src/abc_verify.rs` | 92cbeef13481701e… |
| `experimental/ace/crates/source-cte/src/executor.rs` | a970487d747940ca… |
| `experimental/ace/crates/artifacts/src/subproc.rs` | a5a1d17a62da34f9… |

规范输入：`docs/specs/auto-ac/native-backend-and-retirement.md`; `docs/specs/auto-abi/project.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：1044共享数据/预算/留Rust主体边界及1045 host/SDK；现native/av-core/runtime/967FFI owner，不直接复用mn-worker为已完整协议。

**当前开工状态**：可与1046并行；先T-02有界可行性决策，再T-03/04生产实现。候选不满足则本包明确needs_replan，不让消费者发明fallback。 公共G-AAC须985/T-A04独立复核、T-A05已合入A收据/M2A-01..08；不等985-B、984全产品、986/987整包。frontmatter只列AAC包级最终依赖，任务级准备以§5为准，不用整包串行制造环。

## 5. 详细设计

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

桥只接受Typed MIR/ABC、明确host imports及目标ABI事实；Rust/C工具可处理外部Rust/C头/实现源码，不能处理Auto用户源/AST代做语义。build/host/target/CT执行域、字节序、bool/整数/AutoStr/错误/句柄与layout指纹逐轴核。CT默认无IO capability，编译器host的文件/进程权限不可隐式继承；具体许可按冻结M2-A。纯进程内C ABI若无法满足硬截止/崩溃隔离/预算回收即不可选；可采用隔离worker内薄shim，不能以transport选择免除资源义务。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-service-boundary.md` | 新增有限backend/AV服务协议和进程内安全capability重建；保留现Rust后端/AV，不保留Rust源码语义/CT调度作为AAC主体。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-ac/native-backend-and-retirement.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-service-boundary.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**机器服务能力与AAC语义权威**，沿[Design59的CL族级映射](../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-03/T-05/T-09；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-service-boundary.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

**AC 语法同源验收。** 在 T-06 运行已交的同源 AutoDown 用例的 AC 检查。合法程序在真实机器产物上验值、错误和资源等适用判据，保留原机器服务与协议的定向检查。 按 [AV AutoDown 共用语料规则](../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | 同一标量及owned MIR的合法backend/AV请求 | 本包生产bridge→Cranelift/AV（T-03/04） | 真对象/真执行值与独立expected；请求无源码语义负担；保backend/AV/ABI版本账 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 伪Checked、坏类型/owner/schema/target与超长frame | bridge准入前门 | 拒绝且codegen/AV执行未进入；近邻合法请求绿，不能反序列化capability绕验 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | 请求中途超时/取消、child crash及输出超限 | 真实父子/FFI客户端生命周期入口 | 区分资源/外部故障；回收句柄/进程/拥有值、不污染下一请求、不发布半新对象 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-03/04/05 | 合法MIR返回整数7；复制其消息仅改一个owner引用为越界ID，再跑原合法消息 | 本包V-01/02/03所列真实/拟新增入口，沿相同启用任务 | 第一次真实执行得7；坏消息拒绝且本次codegen/AV执行进入计数0；第三次仍得7且资源归零，证明拒绝未污染服务 |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-native ace-av ace-comptime ace-abi`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | 执行域、capability与边界反控的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | 交换target/ABI或把host IO权授给CT必须拒绝；合法next请求仍返回7；纯in-process方案不能安全回收则transport试验判no-go，不以父exit0或超时语义错误冒PASS | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 本族可区分验证（新增合同，未实施）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | T-03/T-05/T-09创建/扩本包既有Auto阶段driver，最后由原T-06与独立review核；完整Gen1/代际入口分别归1053/1054 | 合法SCHED/平台请求在runtime未交时明确pending依赖；交 Design59 对应实施任务正式源后真native运行；拿named rejection充成功、偷调Rustsource编译或仅回协议envelope均必须被识别。 |

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | 有界试验决策与实际命令/错误/身份；若不可行阻止后续生产实现；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；本包支持范围不冒整个AAC |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

- [ ] **AC-CL-FAMILY / work**：V-CL-FAMILY和原T-03/T-05/T-09实际结果满足，族级CL未因CS未用或A期AC不支持删减；语义与runtime/库补齐正式来源齐，原独立review逐行核。原任务数及0进度保持，本轮未执行。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1051/auto-lang`，分支 `plan-1051-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1051/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [ ] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 记录worktree/branch/base与§4指纹变化、支持profile和后续任务入口；AC-01/04 |
| [ ] T-02 有界transport决策 | work | T-01 | 审计现typed_pipeline/worker真实签名，优先进程MIR/object协议、备选C ABI薄shim；同一标量和owned MIR各做成功/坏输入/回收试验，比较安全准入、所有权、部署、序列化/调用开销及1044预算。选一个生产transport，留被否决理由，不实现两个完整生产桥。 | 有界试验决策与实际命令/错误/身份；若不可行阻止后续生产实现；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-03 native backend服务 | work | T-02 | 接收版本化typedMIR/target/ABI/编译选项与依赖身份，执行安全verifier后走现Cranelift/对象路径并返回受限结果/诊断。不得接Auto源码/AST、调用Rustsourcecompiler或隐藏HIR→MIR。 后端/runtime尚缺的必需机器能力在本包 T-03/T-05/T-09 补齐，经共享 runtime 真执行，缺陷保原用例和独立诊断。 修现 Gen0 AC 的 `native/src/lower.rs` 和实际 runtime/codegen 缺口，包括 GlobalGet/Set、DynPack/Is/Take、ObjectNew/Get/Set、调度调用以及平台＋调度同产物。用现有 Gen0 生成的受检 MIR 和必要的 hand-built verifier 反例交早期机器服务，1050 仅下游消费。 | 薄Rust服务+Auto客户端，实际对象/链接输入及资源计量；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-04 AV CT执行服务 | work | T-03 | 接收Auto已定型可执行请求，重验MIR/ABC/host imports、运行AV并返回值/读集/资源/失败分类；实例与CT依赖调度仍在Auto，不能调用Rustsource-cte全服务重编源。 承接 CTE 受检执行服务与外部 ABI/host target 区分，按 967 正式 ABI 和前端元数据验证实际 native 调用。 | 受检AV执行尾及类型化结果/句柄生命周期；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-05 协议与生命周期反控 | work | T-04 | 测错schema/target/ABI、坏MIR/ABC/引用/ownership、checked伪造、超长消息/输出/超时/取消/进程崩溃/重入适用项，验证无半新输出和原有资源清理。合法资源模块真执行，不仅协议roundtrip。 对 1067 正式基础上的 owned-text 循环/收发及 ask/reply、外部取消、late callback 补 native 调度生命周期与诊断/源 span，含重复失败后恢复。需要解析、源规则或 MIR 生成改变时，由相应 1046–1050 任务随后消费服务并交同源证据。 | 桥准入/预算/清理证据及真实consumer集成入口，交1048/1050/1052；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-09 执行域、capability与边界反控 | work | T-05；§5实际输入合同 | 补消息/返回值类型/ABI布局/句柄/错误、frame长度/协议版本/重放、host-target交换、工具路径依赖及CT capability反控。未来proof拒绝记录分类与codegen/AV真正入口计数，不用wrapper声称未进入。选定transport的取消/崩溃/最后owner路径必须能实测且稳定恢复下一请求。 补实际支持算子/类型/域不一致、把原native拒绝误判PASS、服务偷调用Rust源码编译的反控；合法请求真产物执行，缺口补齐后同源重跑。 按 974/975、1064/1065 和 967 已交范围补现 AC 平台/net/数据/FFI/异步 dispatcher 整合，包括非放宽 profile 的实际 File/net/DB/FFI 与清理。原库 API/公共数据或协议实现缺口归其具名库 owner。验实际系统/协议/权限/ABI/取消，不以具名拒绝计可用。native 默认系统接口沿 Design56 的 Auto＋Windows 公开系统 API 或 Linux syscall；显式外库归已有 ABI/provider，Linux 留 1063 NOT_RUN。 | §6 V-05实际结果及输入/产物身份；AC-07/AC-04 |
| [ ] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 实际命令/退出码/输入和产物hash、覆盖与缺口；AC-01..04。完成即execution_done交review，无需预勾T-07/08 |
| [ ] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（补服务层的ABI/平台轴、capability隔离和transport可行性硬条件；明确外部Rust/C源码工具与禁止Auto源码服务的区别。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

## 10. 待澄清事项

无须当前用户新增决定。可与1046并行；先T-02有界可行性决策，再T-03/04生产实现。候选不满足则本包明确needs_replan，不让消费者发明fallback。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
