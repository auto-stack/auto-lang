---
plan_id: PLAN-1051
status: archived
feature_name: AAC native后端与CTE执行服务桥
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-20
plan_revision: 6
current_step: 8
total_steps: 9
depends_on: [PLAN-1044, PLAN-1045]
supersedes_spec_components: [docs/specs/auto-ac/native-backend-and-retirement.md, docs/specs/auto-abi/project.md, docs/specs/auto-ac/native-async-scheduler.md, docs/specs/ace-migration/m2-stdlib-network.md]
new_spec_components: [docs/specs/auto-ac/aac-service-boundary.md]
touched_goals: [GOAL-002]
---

# [PLAN-1051] AAC native后端与CTE执行服务桥

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包承接项已补入 §8 原执行任务，复用 §6 真执行和独立反控；正式交接须标明原 required 行、实际源与产物、身份和结果。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：补服务层的ABI/平台轴、capability隔离和transport可行性硬条件；明确外部Rust/C源码工具与禁止Auto源码服务的区别。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。提供AAC到现Rust机器后端和AV的最小显式服务，选定生产transport，严格禁止把解析/类型求解/实例化/HIR lowering藏进桥内。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

提供AAC到现Rust机器后端和AV的最小显式服务，选定生产transport，严格禁止把解析/类型求解/实例化/HIR lowering藏进桥内。 本包出口见AC-01..06。依据[Design59](../../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

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

本包执行[1068](../1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-service-boundary.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**机器服务能力与AAC语义权威**，沿[Design59的CL族级映射](../../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-03/T-05/T-09；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-service-boundary.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

**AC 语法同源验收。** 在 T-06 运行已交的同源 AutoDown 用例的 AC 检查。合法程序在真实机器产物上验值、错误和资源等适用判据，保留原机器服务与协议的定向检查。 按 [AV AutoDown 共用语料规则](../../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

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

- [x] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核。

- [x] **AC-CL-FAMILY / work**：V-CL-FAMILY和原T-03/T-05/T-09实际结果满足，族级CL未因CS未用或A期AC不支持删减；本包消费的正式源码、runtime 和库与真实产物见 §9，原独立review逐行核。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1051/auto-lang`，分支 `plan-1051-dev`，一Plan一树沿work/review/merge顺序使用；已建立并用于本包验证。计划材料唯一 `D:/autostack/plan-artifacts/plan-1051/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 记录worktree/branch/base与§4指纹变化、支持profile和后续任务入口；AC-01/04 |
| [x] T-02 有界transport决策 | work | T-01 | 审计现typed_pipeline/worker真实签名，优先进程MIR/object协议、备选C ABI薄shim；同一标量和owned MIR各做成功/坏输入/回收试验，比较安全准入、所有权、部署、序列化/调用开销及1044预算。选一个生产transport，留被否决理由，不实现两个完整生产桥。 | 有界试验决策与实际命令/错误/身份；若不可行阻止后续生产实现；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-03 native backend服务 | work | T-02 | 接收版本化typedMIR/target/ABI/编译选项与依赖身份，执行安全verifier后走现Cranelift/对象路径并返回受限结果/诊断。不得接Auto源码/AST、调用Rustsourcecompiler或隐藏HIR→MIR。 后端/runtime尚缺的必需机器能力在本包 T-03/T-05/T-09 补齐，经共享 runtime 真执行，缺陷保原用例和独立诊断。 修现 Gen0 AC 的 `native/src/lower.rs` 和实际 runtime/codegen 缺口，包括 GlobalGet/Set、DynPack/Is/Take、ObjectNew/Get/Set、调度调用以及平台＋调度同产物。用现有 Gen0 生成的受检 MIR 和必要的 hand-built verifier 反例交早期机器服务，1050 仅下游消费。 | 薄Rust服务+Auto客户端，实际对象/链接输入及资源计量；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-04 AV CT执行服务 | work | T-03 | 接收Auto已定型可执行请求，重验MIR/ABC/host imports、运行AV并返回值/读集/资源/失败分类；实例与CT依赖调度仍在Auto，不能调用Rustsource-cte全服务重编源。 承接 CTE 受检执行服务与外部 ABI/host target 区分，按 967 正式 ABI 和前端元数据验证实际 native 调用。 | 受检AV执行尾及类型化结果/句柄生命周期；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-05 协议与生命周期反控 | work | T-04 | 测错schema/target/ABI、坏MIR/ABC/引用/ownership、checked伪造、超长消息/输出/超时/取消/进程崩溃/重入适用项，验证无半新输出和原有资源清理。合法资源模块真执行，不仅协议roundtrip。 对 1067 正式基础上的 owned-text 循环/收发及 ask/reply、外部取消、late callback 补 native 调度生命周期与诊断/源 span，含重复失败后恢复。需要解析、源规则或 MIR 生成改变时，由相应 1046–1050 任务随后消费服务并交同源证据。 | 桥准入/预算/清理证据及真实consumer集成入口，交1048/1050/1052；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-09 执行域、capability与边界反控 | work | T-05；§5实际输入合同 | 补消息/返回值类型/ABI布局/句柄/错误、frame长度/协议版本/重放、host-target交换、工具路径依赖及CT capability反控。未来proof拒绝记录分类与codegen/AV真正入口计数，不用wrapper声称未进入。选定transport的取消/崩溃/最后owner路径必须能实测且稳定恢复下一请求。 补实际支持算子/类型/域不一致、把原native拒绝误判PASS、服务偷调用Rust源码编译的反控；合法请求真产物执行，缺口补齐后同源重跑。 按 974/975、1064/1065 和 967 已交范围补现 AC 平台/net/数据/FFI/异步 dispatcher 整合，包括非放宽 profile 的实际 File/net/DB/FFI 与清理。原库 API/公共数据或协议实现缺口归其具名库 owner。验实际系统/协议/权限/ABI/取消，不以具名拒绝计可用。native 默认系统接口沿 Design56 的 Auto＋Windows 公开系统 API 或 Linux syscall；显式外库归已有 ABI/provider，Linux 留 1063 NOT_RUN。 | §6 V-05实际结果及输入/产物身份；AC-07/AC-04 |
| [x] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 实际命令/退出码/输入和产物hash、覆盖与缺口；AC-01..04。完成即execution_done交review，无需预勾T-07/08 |
| [x] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

2026-11-20 T-01 开工收据（r6）。`/goal 完成计划1051` 授权本包实施、独立复审、范围内修复、合入归档及清理。G-AAC：`git merge-base --is-ancestor 0db949754 master` exit 0；985 T-A04 phase PASS 与 T-A05/M2A-08 正式交付 `95c7fbad8`（M2A-01..08，具名 B 差额保留）。1044、1045、1056 已归档，1067 调度 `b28399cc9` 落主，967/974/975/1064/1065 均已归档。基面 `89f70228289c314ac24b74ff96b89f15d1360167`；专树 `D:/autostack/.wt/lang-1051/auto-lang`，分支 `plan-1051-dev`；只读依赖 auto-down `72b097006f34db3a871449f10e213820d51abef6`。材料 scope `D:/autostack/plan-artifacts/plan-1051/` 及专用 debug target 已登记。支持范围沿 1044 Windows x64 hosted+alloc、单请求串行、hosted AV CT；64 MiB 前门、1M 步/5 s 语义、10 s CT 根预算及更严分层上限。

§4 源指纹重绑（SHA256 前 16 位）：backend `AA4B920D4C21C1AA`、typed_pipeline `674BEFD7C815C74A`、typed_session `EC9965FDABE202F7`、worker `600020B158424C95`、mir_verify/mod `3CB1B3884C7D5A09`、abc_verify `F5B43BB2E81C2BA0`、source-cte executor `D6E3F08E258B9B31`、subproc `A5A1D17A62DA34F9`。语法 `E7A6401A7C7EC881`、MIR `74F0FDAF7C053565`、M2-A 收据 Spec `045340642830E6B0`、hosted SDK profile `B5555421EC092949`、1056 codec profile `7259CAF1E31A54C2`、ABI Spec `27333270014826A0`。T-02 待验证入口：ABC codec/深度 MIR 校验→现 Cranelift COFF 与 AV，Windows ProcessTree bounded executor；检查 1045 Auto 客户端可达的实际传输，及无输入源编译的调用链。D-1045-01 属 A2R argv 已知问题，当前 Windows native 与 AV 实验不消费该腿。T-01 静态前置通过，产品测试尚未运行。

2026-11-20 T-02 有界可行性收据（r6，基面 `89f702282`）。选择 Windows Job 隔离的一次一请求子进程，有界二进制帧传入已定型 IR；复用 `auto_tool_execution::run_bounded` 的 pipe/Job、cap+1 回收证据。可从 Auto 一侧使用已交文件/进程原语启动本机 supervisor；当前试验由 Rust 父程序执行，Auto native 客户端验证留 T-03/04/09。纯 C ABI 进程内共享地址空间，不能单独回收死循环/阻塞或隔离进程终止，故不做第二套生产传输。ABI 布局、跨域权限、实际取消和 native 同源调度组合仍需后续任务验证。

实验命令：`cargo build --manifest-path D:/autostack/plan-artifacts/plan-1051/transport/Cargo.toml --target-dir D:/autostack/plan-artifacts/plan-1051/probe-target --offline`；`cargo build --manifest-path experimental/ace/Cargo.toml -p ace-text-runtime --profile runtime-dev --locked --offline --target-dir D:/autostack/plan-artifacts/plan-1051/runtime-target`；`aac-transport-probe.exe verify <ac_text_runtime.lib> <probe-output-ipc>`，最终 exit 0。Rust 1.98.0。源码 SHA256 `F7A07657F3F8A496B1AB4EA0A0B77AEFEC29FA3097009D5AAA1D4E0EED0433E3`，exe `8E91948C0D592229459C9F17277116E6A65159E8899462A7DD878BDFC1E84E73`，runtime lib `AD7610C6CDEEBA1A9FEB38F26371CE824FFC3E72DAB4B1BADC461C6645CAEC1C`，完整日志 `D:/autostack/plan-artifacts/plan-1051/transport-run-3.log` SHA256 `C1F36235E40CA5CFA0AD2C7323395D7EFA6A94EA0C7A890533D57640C8D13063`。

- 标量帧 107 B，AV 得 7（2 步、1 个 root frame，59.57 ms）；机器服务产 189 B COFF，真实 PE 得 7（IPC 361.29 ms）。含 NUL 与 Unicode 的 owned 帧 156 B，AV 得 UTF-8 字节数 9、真实帧 1、存活 owned 文本归零（22 步、55.03 ms）；机器服务产 550 B COFF，真实 PE 得 9（IPC 333.77 ms）。
- 修改 owned Drop 槽位为 63 和标量 Const 索引为 63，分别得到 `mir.slot-ref`/ `mir.const-ref`；同一坏 owned 帧经机器服务拒绝，未创建暂存/对象目录。同一标量重跑得 7。长度前置 cap+1 拒绝；stdout 4,096 B 帽观察到 4,097 B 后回收；50 ms deadline 实际 50.52 ms、cleanup completed=true；child abort 返回 Windows `0xc0000409`，与语言诊断独立。
- 首次编译 probe 错用私有 CaptureCaps，改为同一 bounded executor 观察 PE；第一次 owned 测试发现 ABC decoder 写的是 wire schema 而 verifier 要求逻辑 MIR schema，按请求元数据选择固定 MIR schema 后，合法近邻通过。AV 旧执行入口不维护 process-wide execution_seq，正向采用真实 root_frame_commits；T-05/09 需覆盖 actual production entry 的拒绝差值。
- 预算沿既有 MIR/ABC 输入 ≤4 MiB，机器 backend 单对象、对象和总量沿 16/64 MiB，CT 仍取根剩余且最高 10 s。探针 4 KiB capture 与 50 ms deadline 是负控参数。全局表/初始化序列不在标准 ABC wire，生产接面传入独立 typed manifest 再重验；不能以 ABC 能编码 Global* 指令冒充全局表。输出聚合与整请求时限最终登记仍须 T-06 用真实阶段样本重测（D-1044-01）。

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（补服务层的ABI/平台轴、capability隔离和transport可行性硬条件；明确外部Rust/C源码工具与禁止Auto源码服务的区别。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

2026-11-20 stage: work | PLAN-1051 | r6 | outcome: in-progress | worktree: `D:/autostack/.wt/lang-1051/auto-lang` @ `plan-1051-dev` | base: `89f702282` | code commits: `0699292d4`、`66f72c962` | tasks: T-01/T-02 完成，T-03/T-04/T-05/T-09 只验阶段切片，6 条 work/最终门中的其余义务仍开 | next: work（共享全局/动态/对象、CT/FFI、调度组合与平台域）

- `0699292d4`：AQ01/AR01 双向帧长度与 BLAKE3 绑定、任务 ID/请求摘要校验、typed MIR 深校后实际 Cranelift COFF/AV；Auto 客户端经 Windows 文件/子进程返回结果。Gen0 hosted collections 按已选调用闭包验证每个函数单元，再用真实 Win32 shim 和显式准备的 runtime 链接。原始整模块准入报 24 宿主声明上限；单元准入后定位到未选平台分支的 open_bytes；限定实际可达调用闭包后，真实 Windows PE 成功。第一次成功仍有 111 个文本值存活，Auto 客户端改用字节直接编码十进制，复跑 native `MN-COLL live=0`，`allocs=16 frees=16`。AV host 1M 默认步数失败，使用 PLAN-1045 已记录的 compiler-host 执行预算验证 client 臂，非放宽 CT Worker 限额（本轮 AV host 审计 3,142,326 步，exit 0，Windows host-region 0 残留）。
- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-aac-bridge --test conformance --test hosted --offline --target-dir <scope>/probe-target -- --nocapture` exit 0。准备 27.45s；conformance 3/3，0.39s（对象与坏槽位 0.253s，target/host 权限拒绝 0.327s，owned AV 重验/恢复 0.394s）；显式 native 集成 1/1，2.95s（Gen0 产 PE→实际启动 worker→Auto 观察 CT 值 7→owned 文本/暂存路径归零，单例 2.938s 含清理）。发现、独立清理耗时未拆测。两项初始产物 SHA256：bridge exe `86E5ADD56127A7F41CD1ED72517793EEB0CF83896316433A7CEB2B3B67E9229B`，prepared collections runtime `4FF597CE061B37E31B5FCAA704E3DB98452D6B366E5C22929D2631495F6B4C4D`。日志 `bridge-frozen-integration.log` SHA256 `D185D3CACE0E7F2D289AD76270308BEB2EAC38D0441536759C6701EF4826788E`（路径均在材料 scope）。
- `66f72c962`：在同一 bounded executor 加 opt-in caller 取消、Job 树回收和取消后续请求。初次 heartbeat 探针误选 ProcessOnly，后代继续写；修复 oracle 为实际的 ProcessTree，真实后代已打点后取消，心跳停止、捕获丢弃、下一请求仍返回 7。根 `cargo test -p auto-tool-execution --test exec_lifecycle aac_cancel_bounded_exchange_stops_owned_descendant` 1/1，0.388s（重编 0.58s；34 例未选）。ACE `--test conformance` 4/4，0.41s（重编 5.84s；取消/重跑单例 0.177s，其余 0.272–0.411s）。两次 exit 0；`bounded-cancel-retry.log` SHA256 `974CD58A9093E3B30363F0B1021CEC1D8BC305C09137695D5B531F9F5922E389`，`bridge-cancel.log` SHA256 `0D0F1F6F679B5EB12EAD64A7C1117E3CDDF0717DB01663B5584BDAA3E5796867`。
- 尚未关闭：真实 CT 外部 ABI/host authority、程序状态完整语义、Gen0 AC Global/Dyn/Object 和调度+平台全组合、owned 调度/late callback、native File/net/DB/FFI、协议重放/码生成入口差分、同源语料和固定组并集。Linux/WSL 按仓规留 1063。仅实验和阶段实现，未把验证子集签作 T-03..05/T-09 或本计划 PASS。

2026-11-20 stage: work | PLAN-1051:r6 | outcome: in-progress | code_commit: `81e72ce80` | T-03/T-04 追加已验切片：版本化请求独立携带类型化全局表与初始化序列，核名字/类型/引用/常量写入权限/初始化函数后再构造进程内 credential。原生 COFF 壳定义标量槽并在入口前按序真实调用初始化器；同一来源 CT 由 AV ProgramSnapshot 执行。测试两端均得 7，缺全局表时两端提前拒绝 `global.reference`，错初始化引用拒绝 `bridge.initializer`。原生目前验证非拥有标量载体，组合 owned 及动态对象仍需关闭。

- 同一原生 PE 执行实际 Gen0 调度程序（stdout `go:\n7\n`），同时执行 Win32 页大小查询（返回值与独立 host GetSystemInfo 核对）；共享 MIR 修正平台/控制台组合准入，保有类型、声明与 authority 校验。该切片仅启用无共享状态的页大小、分配粒度、SleepMs 系统调用，未计 stateful 平台/调度并发、网络或回调通过。首次并存反控报 `host.profile`；改正共享准入和配套 shim 后测试通过。
- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-aac-bridge --test conformance --test hosted --locked --offline --target-dir <scope>/probe-target -- --nocapture` exit 0。准备 3.77s，conformance 5/5（0.55s，CT 全局 0.434s）；原生显式集成 3/3（3.20s，标量全局 0.639s、调度/平台同 PE 0.594s、Gen0 Auto client 3.182s，含各自清理）。`bridge-global-sched-integration.log` SHA256 `ECB11FED49ABE089690EECDC6255797021F294A393629C97CF5CBD325844D054`。新模块顺序和 Windows 结果实际跑过；未运行受影响完整并集或 Linux/WSL。


2026-11-20 stage: work | PLAN-1051:r6 | outcome: in-progress | worktree: `D:/autostack/.wt/lang-1051/auto-lang` | branch: `plan-1051-dev` | base: `a7ce84eff` | tip: `eb5eaa58a` | T-04 已关闭，T-03/T-05/T-09 新增已验切片 | next: work

- 上轮提交 `136d6f92f` 及本轮 `9348b9135` 重放至已含 PLAN-125 debug 门禁的主线后分别为 `00d3a896d`、`eb5eaa58a`。提交前的目标 Rust 文件格式与 staged diff 检查均 exit 0。T-04 沿受检 ABC → AV 真实执行，返回类型化值、可观察资源和读集。967 F0 DLL 以批准 manifest、库 hash 和纯长度型 view 准入，9 字节含 NUL，foreign_calls=1、活跃 owned 文本=0，错宿主/配置 digest 提前拒绝。Auto native 客户端真实运行，绑定响应 ID/hash 并读取 i32=7，交换文件和 owned 值均清理。
- T-05 最初 owned 调度出现 exit 72。共享分配账本串行化，降层每次成功写入 owned 槽时释放旧值，Gen0 1067 循环插值两次实际得到 a=1 和 a=2。owned 消息与 ask/reply 得 9，重复 reply 被释放。两次错型均 exit 70，源 function 1、bytes 54..61，下一合法运行恢复。实机 Job 在观察到处理函数入口后取消，reap 后无 publishing，下一实际 CT 得 7。普通调度 PE 的 kernel32 import allowlist 通过。
- T-09 真实 Win32 W16 文件路径读 [7,0,255]，成功关闭 exit 7，省略 close 的同程序 exit 72。WinSock 监听与回环经真实 PE，accept 静默窗 -10、文本与 raw bytes、错误 origin -3、重复关闭 -6，监听器在 drain 释放。官方 967 F0 经授权 native COFF 真调用；SQLite 3.46.0 用 975 canonical Auto 作者源，事务回滚后 SUM=9，未打开执行 exit 70，同一 manifest 传入 CT 域被 `bridge.foreign-call-frame` 拒绝。Windows 未运行 Linux/WSL。
- V-02/V-04 经生产 admit 在两个后端实际分派点核增量。合法输入生成 COFF 或整数 7，坏引用、伪造 Checked、版本、摘要、长度及尾随字节进入增量=0，合法重跑通过。真实响应换 request ID 或相同 ID 的 request hash 均在父消费入口拒绝，当前结果通过。
- `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-aac-bridge --lib --test conformance --test hosted --locked --offline --target-dir <scope>/probe-target -- --test-threads=1 --nocapture` 首次 exit 101，编译 8.83 s，unit 2/2=0.01 s，conformance 5/5=0.91 s，显式原生 15/16=8.50 s，整命令 18.472 s。Gen0 client 字段直取的 `.move` 调整成局部变量后单例 exit 0，准备 1.33 s，单例 4.057 s，命令 5.748 s（15 名过滤）。发现和独立清理时间未拆测，单例计时含清理。日志 SHA256 `09121835BCA69F491EFF366109CAA8F125E117A1B8ED5CC3D954BCD679EB1993`、`0A8F294A830019D795FFB75E66F6E9F11C333FB111D7BAC67D2C9FE254D87B96`。scheduler runtime-dev 支持库 SHA256 `01E80D5BAAACA7E2DB386F087A803F2E3F5798E0EB1B95D6DB6A73A339D7D0C4`。
- 新增用例耗时（debug/test，含 setup/cleanup）：伪造和越界提前拒绝 0.004 s，重放 0.001 s，循环插值 0.239 s，owned 循环/错型/源 span/恢复 1.157 s，File close/leak 0.659 s，WinSock 回环 0.433 s，native F0 ABI 0.342 s，canonical SQLite 0.626 s，运行中的 scheduler 外部取消 0.469 s。
- 下一项同源缺口：1085 的 `av.parity.1085.object-global` 需要有所有权的全局对象。当前 collections worker 仍拒绝程序状态（`bridge.collections-program-state`），此前的 canonical 对象切片覆盖局部对象。T-03 仍开，需扩展共享 collections 的全局 cell 及清理并重跑同源用例。固定组准备正在执行，最终判据与独立复审待完成。1050 是下游。


2026-11-20 stage: work | PLAN-1051:r6 | outcome: in-progress | commit: `0d665fc6f` | T-03/T-05/T-09 已验，T-06 固定组准备中

- 原样从 1085 正式 AutoDown 取 `av.parity.1085.object-global`，经同一 Gen0 packages 前门、正式 init 序列及受检 MIR 编出真实 Windows PE。源例返回 42，runtime `MN-COLL live=0`。把初次读改为 take 后紧接原读，实际 exit 70 / `MN-COLL-TRAP code=34`，同样零残留。先前普通单文件前门错接全局例导致 `source.nested-fn-no-capture`，改用正式 packages 前门后实际定位 `StrCmp` 机器降层缺口，复用文本字节比较补齐。被注册的程序全局 frame 在正常和 trap 路径复用现有运行时清理。全局同源例准备 4.98 s，运行及清理 0.666 s，exit 0，日志 SHA256 `4CEFECCC22333231062199FF99BC1821904B4BA8DDCB748044B958CB38AF5928`。新 collections 运行库 SHA256 `83F287FA6A51769273BF2F520FD52F8E0D0D9168F0703ECFBE279A116015AE20`。
- 真实文件发布入口在目标已存在时保留旧字节，坏摘要无产物和暂存残留，后继合法响应以正确 ID/digest 发布并返回 7。显式错 ABI 拒绝与 target/CT ambient-IO 拒绝同轮复验。聚合响应超过协议帽时，worker 在输出前改发绑定同一请求的资源诊断。
- 同一冻结记录的 `cargo test --manifest-path experimental/ace/Cargo.toml -p ace-aac-bridge --lib --test conformance --test hosted --locked --offline --target-dir <scope>/probe-target -- --test-threads=1 --nocapture` exit 0，准备 2.06 s，2/2 lib、6/6 conformance、17/17 集成，分别 0.01 s、0.88 s、11.09 s，整命令 14.140 s；0 忽略，发现与独立清理未拆测。新增文件原子交换反控 0.060 s，canonical 全局/移走负控 0.667 s；最慢单例 Gen0 Auto 客户端 3.116 s。证据 `aac-hosted-frozen-r6.log` SHA256 `F500774B37B9EEB546C4B33C89B88825B2C6696803E253C7F5D5C67AE836F4B5`。编译中的 av ELF 和已有 native cache/emit 警告为基线，对应未改代码；目标文件格式与 staged diff 检查通过。
- V-AC-STAGED 输入，985/A 正式快照 + 1044 的 42 条 required CL 保留原 A 账。CL-701/703/704 使用 1084 局部对象和 1085 全局原样源、失败 ownership 近邻。CL-301/302/304 以 1067 的正式 Windows 原生 lane、Gen0 插值/任务及主动取消为执行证据。CL-503/504 用受检 AV 与 967 已批准纯 F0 证明执行域、权限、读集及失败恢复。CL-802 绑定钉版 967 F0 及 975 SQLite 与 1065/974 Win32 原操作，证据为真实 Windows 二进制和系统结果；源级高阶组合继续按 1048–1050、领域回调按 1097、三端最后验收按 1068 原归属；此处交机器和 CT 尾，不计为整机或 Gen2/3 收据。
- T-06 六固定组 `auto-tool-execution ace-aac-bridge ace-native ace-av ace-abi ace-comptime`。ABI/AV 共享 verifier、native 机器及运行库影响直接，comptime 是 CT 消费端，root 工具组覆盖 Job 取消。初次准备在发现已有正式源的全局对象缺口后主动终止，六组未执行，产物留 scope 内。补齐后在 `0d665fc6f` 重新准备 debug tools；Windows Linux/WSL 分支沿现有 cfg 排除，归 1063。

2026-11-20 stage: work | PLAN-1051:r6 | outcome: in-progress | commits: `0d665fc6f` → `b44b68536` → `27b4bc5dd` | T-06 重放与 Auto SDK 启动器验收

- 首轮 debug `cargo t auto-tool-execution ace-aac-bridge ace-native ace-av ace-abi ace-comptime --prepare-only --debug-tools --out <scope>/frozen-prepared-0d665fc6f` 退出 0，80.305 s；同一组六 `--run-prepared <scope>/frozen-prepared-0d665fc6f/prepared.json --out <scope>/frozen-run-0d665fc6f` 退出 1，208.452 s。executor 50/50、bridge 8/8、comptime 136/136。Native 312/329（19 条选取后排除），16 个 typed-entry 无 CRT debug 链接缺符号和 1 个 Win32 spawn access violation。AV 1489/1491（23 条选取后排除），JSON 975 真 PE 出现 `FREE-INVALID`，64 KiB A2R 例 37,077 ms 超原限。ABI 303/328（8 条选取后排除）；25 个实际失败 ID 与 PLAN-1109 在 `f8311d8cf` 同工作区实测的基线完全相同，`Compare-Object` 差异为空，A2C SDK runtime 内容 `0a4f68e3…` 与已认证 `bd260a8b…` 不符，沿原债务 owner。本轮不授权改认证指纹。首轮 `results.json` SHA256 `D752CF4AEA34159E29EFA21C898133918FF3F6662D21EA45F99CA1654E404A7C`。
- `b44b68536` 修复 borrowed StrSlice 被当 owning 槽清理、debug typed-entry 复用 freestanding memory runtime 和 process adapter 的 Win32 输入结构零初始化。`runtime-warm` 以 `prepare-debug` 准备，运行用 `run-debug`。四次串行定向重放全部退出 0：`ace-native --test typed_entry_tier --test typed_entry_incremental_tier` 23/23，运行 27.00 s、总 42.957 s；`ace-native --test conformance sys973_win_proc_spawn_capture_timeout_exit` 1/1，0.73 s、总 5.152 s；`ace-av --test conformance json975_packages_coverage_av_mn_a2r` 1/1，1.47 s、总 26.557 s；`ace-av --test conformance bstr_1064_v01_large_64k_a2r` 1/1，1.84 s、总 2.137 s，原 64 KiB 输入和 10 s 原断言保留。四份 `repair-case-{1..4}.log` SHA256 分别 `E4219AA2049A5181B29AF65E606D942C34B79799D719194A05C1B8277FF2351A`、`9F037F0E73033538C505FE8FB027EDEAD3E20C3004D1A8FB661378D081B75AA7`、`3E6E82F353B62E25179215B7A7DA38D9C349FD9D30889F0DD9E191CB1FE366CE`、`BF11BD666D2BEABDEF6C7C22A28342A24741D8E79BD90F2BFCA8B967FD68B925`。
- Windows 首轮误纳 `sys973_t09_target_selection_linux_binds` 的 Linux-target 纯绑定用例，没有启动 Linux 产物或 WSL。`b44b68536` 按裁定把该用例编译选择限定为 Linux host；原 Linux ELF 和 console 专项 host cfg 继续保留。Windows 不计这些腿通过。
- `27b4bc5dd` 将 SDK 启动器独立提供的 `ACE_AAC_FOREIGN_CONFIG` 和 `ACE_AAC_FOREIGN_CONFIG_BLAKE3` 接入真实 Auto `--exchange`。两值必须同时存在，缺完整授权时拒绝，worker 用显式 argv 并移除这两个继承键。真实同一个 Gen0 PE 三次观察为无授权拒绝、单字段拒绝和批准的 F0 返回 9，三次均无交换文件残留且 `MN-COLL live=0`。同一确切命令再跑 `--lib --test conformance --test hosted -- --test-threads=1 --nocapture` 得 2/2、6/6、18/18，合计 26 例，退出 0，墙钟 21.955 s；显式 hosted 15.68 s，最长单例（F0 三次含 setup/cleanup）5.377 s，标量 Auto 客户端 2.894 s。`sdk-authority-hosted.log` SHA256 `68F7E5B2E1E901678E41E09EEAA7103696BF06B3E84FA08564E3D578B0949BA5`。
- D-1044-01 本桥使用的预算登记。实际标量文件交换 906 B 请求、387 B 响应、61.268 ms；Gen0 native 标量请求 943 B，F0 请求 2,124 B。沿已有上限冻结 4 MiB 全请求、64 KiB metadata、16 MiB 单 COFF、64 MiB 聚合响应、64 KiB stderr。机器服务上限 5 s，CT 不超过现 root 剩余且最多 10 s；超限诊断与半新发布反控保留。整机编译和缓存发布总预算仍归 1052/1053。

2026-11-20 stage: work | PLAN-1051:r6 | outcome: pass | commit: `2bd0ad08619b83a6b532578225972458d8724261` | base: `a7ce84effb8aa9bc5eaa3a0598ddae6d43a7cc28` | authorization: `/goal 完成计划1051` | next: review

- 冻结六组 `27b4bc5dd`，以 `NEXTEST_TEST_THREADS=8` 保留完整选取和断言。准备 23.779 s（退出 0），执行含发现 186.185 s（退出 1）。分组为执行器 50/50、native 329/329、AV 1490/1490、comptime 136/136、bridge 8/8、ABI 303/328。两侧发现均退出 0，分组发现/进程执行墙钟分别 0.346/5.729 s、1.419/47.843 s、1.958/56.500 s、1.509/17.298 s、0.371/0.849 s、1.068/49.517 s；进程退出除 ABI=100 外均 0。原条件排除分别 0、19、23、19、0、8；Windows 纯 Linux target case 已不入选，Linux/WSL 归 1063。清理未独立计时。64 KiB A2R 原断言 1.842 s；没有修改规模、阈值和选择。`prepared.json` / `results.json` SHA256 `3E41794B900C462CA27CC24F98C15378CF7C5CB1C7971587B8E0E2E034649E2B` / `5946F5E8897AF6D0A80EDB9992AAE0070BA6CB6FD6B391E2B2B6DD1ACAF24A84`，配置 `c6a23e7944077afa56e6ac419b4e0a62a4136b47a7370240a086741406de3265`，source state `2ccc7fc87bbd94e47db440b24c37ac9acf0f92bd26a598dd6763cf0cb57c17f0`。
- ABI 原样 25 个失败 ID 与 PLAN-1109 冻结和 `f8311d8cf` 基线清单逐条相等；A2C runtime/认证 pin 实施未改，仍交原 owner，其余 303 例保原结果。PLAN-1051 的 967 provider 正负例通过。保留真实 FAIL 记录，沿 PLAN-1109 相同基线归因办理交接。
- 两个原准入漏洞被真实 CT 消费器重现。普通执行体可再次调用初始器、Bool 槽可接收 `GlobalSet` 的 i32 终态，二者原来均得到值 7（重放各 0.31/0.29 s，退出 101）。`2bd0ad086` 在通用准入阶段拒绝 initializer 直接调用、closure 捕获、sched spawn/go 引用，以及不合型 store 结果，合法 initializer 和独立 canonical 1085 全局仍工作。同命令复测 2 lib、6 conformance、18 hosted 全通过，退出 0，准备 1.88 s、运行 0.01/1.00/21.98 s、整命令 25.134 s；命名 CT 检查 0.286 s、最慢新例 9.558 s，未独立拆清理。红/红/绿日志 SHA256 `8F0F9AD69A3CE60D15DEE4071969A8E41035B043135DD1F076F00711A5526DC9`、`14D2CD50327D389DF3CBC05699B2C2BB5AEBB444597A3D8139D660D127A9BB20`、`FCCC2873B1152A13B090BED08CC36AD0AEA8561FEF04520C55AD24B1C9633AE3`。
- `2bd0ad086` 仅触本 bridge 的 `execution.rs` 与 `conformance.rs`，前述余五组机制、依赖、配置和输入不变。按同一 `cargo t ace-aac-bridge` 的 prepare/run-prepared 入口实跑 8/8（0 排除，准备 2.593 s、运行整命令 1.935 s、nextest 0.491 s，两次退出 0），`results.json` SHA256 `25956B71A434DF2CBAABE37343B8FB5441B897DEF82E6A35C9420639F21240CE`。
- 术语更正：1044 CL 表共 42 行，41 条 required，CL-906 是既有具名 Linux/其他 host deferred。`cl-requirements.tsv` SHA256 `60192D3985696B6DB264CDF06BDF1D1C4163BD26D9264F6A569D1D68E6BE70A8` 与基线无差异。现有正式源和生产桥的同源结果按 T-03/05/09 交接，语言主体组合和独立完整代际消费保原 owner。无本包内未关闭的 work 反控，准备的 Spec 增量进入复审。

stage: goal | plan_id: PLAN-1051 | plan_revision: r6 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: `D:/autostack/.wt/lang-1051/auto-lang` | commit: `2bd0ad086` | next: review

2026-11-20 stage: review | scope: final | PLAN-1051:r6 | outcome: pass | reviewer: 同一 Codex 会话的冻结只读核查，未启用另一会话/agent | worktree: `D:/autostack/.wt/lang-1051/auto-lang` | branch: `plan-1051-dev` | reviewed_commit: `2bd0ad08619b83a6b532578225972458d8724261` | base: `a7ce84effb8aa9bc5eaa3a0598ddae6d43a7cc28` | dependency auto-down: `72b097006f34db3a871449f10e213820d51abef6` | next: merge

| 核查 ID | 实施面和独立可观察证据 | 结果 |
|---|---|---|
| AC-01 / V-01 | `worker` 的实际进程派发到 `execution`，`read_frame` 先限长后分配，Typed ABC 还原后重建进程内准入，不经过 Rust 源码编译路径。`hosted` 中 1084、1085 原样源由 Gen0 准备，随后验证真实 COFF/PE、全局初始化、值 42 和运行时清零；Auto 客户端自身在 PE 中构造 AQ01 和读取 CT 值。 | pass |
| AC-02 / V-02、V-04 | 核请求拒绝与 `EXECUTION_ENTRIES` 实际增量；坏尺寸、摘要、类型、owner 引用、伪 Checked、重放和启动函数/结果槽的反控；良好请求 7、拒绝零进入、后续仍 7。`Outcome::verify_payload` 重验分段和各内容摘要，旧文件在失败中保留。 | pass |
| AC-03 / V-03 | `run_bounded` 明确指定 Job 与清理截止。对照期限、超限、崩溃分类，以及运行中的真实子孙和调度程序取消后不出现 late reply，下一 CT 执行；源 span 和运行时文本/所有权/句柄审计对照各子终态。 | pass |
| AC-04、AC-AC-STAGED | 用原始 nextest streams 和已归档 1109 基线逐 ID 对账，完整 Windows 六组选取，五组全绿；ABI 的真实 25 个 FAIL 全部是原 A2C runtime pin 债务。`git diff` 核最后修订只触 bridge 私有代码/用例，该组按固定入口复跑；CL 42 行一字节未变（41 required、一行原 deferred），已交本包实际输入、剩余 owner 与窗口见 §6/9。 | pass，保留基线红 |
| AC-07、AC-CL-FAMILY | 比对 `Domains::windows`、批准 provider 配置路径和 digest、逐声明/shape 复验及 CT capabilities。真实 Gen0 F0 无授权、部分授权拒绝，完整授权返回 9，File、WinSock、SQLite 事务、活跃取消、文本循环和同源全局获得执行证据。纯网络/系统/外部库三端整机义务保原 owner。 | pass |
| AC-05、AC-06 | 冻结树干净、`git diff --check` 退出 0。没有 Plan-owned 未处置 warning/调试输出或 ledger 写入。既有警告源行与基面相同；旧 A2C pin 由已登记的原 ABI/C runtime 债务管理。26 条定向执行全部通过且新增单例最长 9.558 s。规范增量已核，实际落主、归档和清理交 merge。 | review pass，merge 待办 |

Spec 输入 SHA256（依次 native-backend、ABI project、scheduler、network）`3547178050B20180AC5C89AFD76ECAF30219D8A84AA94300CD37DB8218EA6AB7`、`27333270014826A06545115B530AC109643ADBE3D0664536A28A9E500C3D4F8A`、`350F5E0E72F084F00C7F1887E7B7B3C8629A975DCD050928E2A9B5241EFEB74A`、`A8338B81E6B5F0EE9462DEE4EC4B3851D2F02480CCD057E07384452461799911`。冻结 `review-spec-delta.md` SHA256 `CD6B6DE7113EADDA8B4D45C073BE50FE5E5E82E450E61057B54CFFD8035FD39C`。SD-01 记录 Windows Job 帧和预算、明确启动/类型重验及 Auto CT 消费。SD-02 记录沿共享 Cranelift 的对象接缝和显式全局。SD-ABI-1051 用 SDK 授权绑定实际 967 形状和库字节。SD-SCHED-1051 记 MIR owned 交接、逐次替换释放、兼容源 span 和 drain/外部取消。SD-NET-1051 记共享台账上的 Windows WinSock 组合。GOAL-002 保原关联。没有本包未批准延期；基础与全代际的范围仍按原合同。

stage: goal | plan_id: PLAN-1051 | plan_revision: r6 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: `D:/autostack/.wt/lang-1051/auto-lang` | commit: `2bd0ad086` | next: merge

2026-11-20 stage: merge | PLAN-1051:r6 | checkpoint: prepared | reviewed: `2bd0ad08619b83a6b532578225972458d8724261` | delivery: `1d949f30b52be1f1544b43e53c494a9a6d0a2a77`

专树 rebase master 无改写，delivery 与复审提交差异为已核五处 Spec。专树 clean，`wt-guard.ps1` clean。复用已冻结五组通过结果及 ABI 25 条既有红的逐 ID 归因，最终 bridge 8/8 再验；没有新一轮编译作为沉淀门。此前留存的 A2C runtime pin 债务依 PLAN-1109 批准过的同基线归因，由既有维护 owner 继续跟进。

2026-11-20 stage: merge | PLAN-1051:r6 | checkpoints: landed, canonical_verified | delivery: `1d949f30b52be1f1544b43e53c494a9a6d0a2a77` | default: master

`git merge --ff-only plan-1051-dev` 从 `a7ce84eff` 快进，落主 tip 等于 delivery。主检出以冻结 debug `conformance` 驱动完成真实文件交换冒烟，`file_exchange_preserves_existing_bytes_and_admits_the_next_request` 1/1、退出 0、0.061 s，906 B→387 B，原子交换 0.060 s。冒烟日志 SHA256 `6058B263BA8E3C5316AF71E6E915EAC10DA80AD6AEDC0EDF510CF644D5D3A3A9`，worker exe `F79272B6FAB6B1329EB85F0BAAFC79F658962FB25A64596234943D7AFDD2D56B`。代码与 Spec 所属路径干净。

| canonical source | 落主工作文件 SHA256，与专树审阅值相同 |
|---|---|
| `docs/specs/auto-ac/aac-service-boundary.md` | `AEC9F98C4849A7F769F876762DA48E268944AC8AD844DB29491BA59E3260BE4A` |
| `docs/specs/auto-ac/native-backend-and-retirement.md` | `B87475E18D65D9BBBCC7C15E398C29FF9A593576C1A8515172621D56FD9CAED0` |
| `docs/specs/auto-abi/project.md` | `8B75935B41FA18825F4BE739D2ADDF66AA6428268E619F97A7C96ED9F0F7D3A4` |
| `docs/specs/auto-ac/native-async-scheduler.md` | `F1441A1FA097C7A57FF590A39C1060FACDD1330D0F87594212E11FDE94EC3125` |
| `docs/specs/ace-migration/m2-stdlib-network.md` | `62F23DABB60F19F175A3C468493FCDB5057BDDC407D52BB5D6DE6846B729948D` |

清理前路径对账（文件数 / 字节数）：`D:/autostack/.wt/lang-1051/auto-lang` 132268 / 21422526981，内含 Cargo 默认 target 和 debug archive tools；同组只读 `auto-down` 1294 / 26841767；`D:/autostack/plan-artifacts/plan-1051` 54122 / 16788324740，含显式临时变量、预备 provider、外置 target、协议与所有运行日志。实际命令未使用其他安装/产物根，无具名保留。待归档位置与引用核验后守闸移除三根。

2026-11-20 stage: merge | PLAN-1051:r6 | checkpoint: archived | completion_kind: delivered | archive: `docs/plans/archive/1051-aac-native-backend-and-cte-bridges.md` | cleanup: pending

归档门核真实位置和 `archived`、原 T-01..T-09 唯一且清理前完成 8/9、交付祖先和五处 canonical diff、计划文档相对链接与两个 Design59 锚均有效。同步其它工作树的在途文档无需纳入本计划提交。

2026-11-20 stage: merge | PLAN-1051:r6 | outcome: blocked | checkpoints: landed, canonical_verified, archived, worktrees_removed | delivery_commit: `1d949f30b52be1f1544b43e53c494a9a6d0a2a77` | archive_commit: `c8c487996772dcec86ae73330cdeced0a2c3aeb4` | next: cleanup only

专树及只读 auto-down 兄弟树、材料根分别经 `wt-guard.ps1` 检查 clean。无在用产物进程。两棵 worktree、`plan-1051-dev` 与空 `lang-1051` 组目录均已实际移除并复查。材料根 `D:/autostack/plan-artifacts/plan-1051` 的目录精确解析、归属、无链接及数量已核。执行工具以 `blocked by policy` 拒绝对该单一材料根执行批量删除，逐文件校验删除和字面固定路径删除也被拒绝；材料根仍存在。登记状态标记为 `cleanup-pending`；待该目录移除后完成 T-08 与 cleaned 收据。已交代码/Spec/归档不用重跑。

主检出无本计划代码、Spec 或归档外的待处理变化；保留并行的既有文档 WIP。`ace-aac-bridge` 未在 PATH 安装，交付验证为 debug，不涉及应用部署。周期回归收据覆盖 1098，距今 11.09 h，收据后新增归档号 1051 不触发五倍数，当前不到期。

stage: goal | plan_id: PLAN-1051 | plan_revision: r6 | round: 3 | from: merge | to: user | status: archived | outcome: blocked | worktree: 已移除 | commit: `1d949f30b` | next: 仅剩 plan-artifacts 根清理

## 10. 待澄清事项

代码、规范及归档已交付。须由具备删除权限的执行环境移除单一材料根 `D:/autostack/plan-artifacts/plan-1051`，再核对目录不存在、刷新登记与归档 SHA256，并完成 T-08。当前工具的自动执行策略拦截该删除，详见 §9。
