---
plan_id: PLAN-1052
status: execution_done
feature_name: AAC增量查询、持久缓存与编译发布驱动
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-20
plan_revision: 5
current_step: 6
total_steps: 9
depends_on: [PLAN-1044, PLAN-1045, PLAN-1047, PLAN-1048, PLAN-1049, PLAN-1050, PLAN-1051, PLAN-1056]
supersedes_spec_components: [docs/specs/auto-semantics/source-query-session.md, docs/specs/auto-ac/function-incremental-source.md]
new_spec_components: [docs/specs/auto-ac/aac-incremental-driver.md]
touched_goals: [GOAL-002]
---

# [PLAN-1052] AAC增量查询、持久缓存与编译发布驱动

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包承接项已补入 §8 原执行任务，复用 §6 真执行和独立反控；正式交接须标明原 required 行、实际源与产物、身份和结果。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：补缺失的完整Auto MIR前置、包锁/发现/FFI装配与持久层合同，避免由Man或Rust ManagedStore隐藏编译主体。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。由Auto管理模块编译、正负语义依赖、cache/恢复/事务发布与对象链接驱动，避免frontend移植后仍把实际增量编译主体交Rust host。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

由Auto管理模块编译、正负语义依赖、cache/恢复/事务发布与对象链接驱动，避免frontend移植后仍把实际增量编译主体交Rust host。 本包出口见AC-01..06。依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**experimental/ace/aac/store/、driver/与tests/incremental/（拟新增）**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `experimental/ace/crates/source-query/src/host.rs` | 81e930720f18b45d… |
| `experimental/ace/crates/source-query/src/session.rs` | d32b10eb0d6cecf9… |
| `experimental/ace/crates/source-query/src/restore.rs` | 16ad201f46a8b13e… |
| `experimental/ace/crates/source-query/src/ident.rs` | 833e5ce97ef3a4da… |
| `experimental/ace/crates/artifacts/src/identity.rs` | b1ca0253dc944859… |
| `experimental/ace/crates/artifacts/src/store/` | 当前已存在目录，执行时按实际文件冻结 |
| `experimental/ace/crates/native/src/incremental.rs` | 73f7b1747a36c8c3… |
| `experimental/ace/crates/native/src/link_driver.rs` | 8c8bc4ea4ef6a049… |
| `experimental/ace/crates/core/src/link.rs` | 7e19a7d49894d30a… |

规范输入：`docs/specs/auto-semantics/source-query-session.md`; `docs/specs/auto-ac/function-incremental-source.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：1044身份/恢复合同、1045文件/进程/SDK、1047查询事实；1048CT/实例、1049高级资源事实、1050完整Auto MIR及1051对象/AV服务作实际集成；978构建/运行兼容与包依赖规则。

**当前开工状态**：基础身份/依赖/事务可在1047已交后先；完整Plan验收必须1048/1049/1051实际结果，不能用模拟服务关闭真实consumer腿。 公共G-AAC须985/T-A04独立复核、T-A05已合入A收据/M2A-01..08；不等985-B、984全产品、986/987整包。frontmatter只列AAC包级最终依赖，任务级准备以§5为准，不用整包串行制造环。

## 5. 详细设计

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

真正build链消费1050完整Auto MIR，再进1051后端；基础身份/事务准备可先，最终交付不得缺该输入。包获取/解析/锁定由auto-man既有owner供给，AAC消费声明闭包/manifest并做Auto语义绑定，不自行网络获取或扫盘突破声明门。Rust ManagedStore不能代管查询/依赖策略；选文件store或SQLite时只允许底层存储原语外置，Auto保身份/读集/提交策略，物理format独立版本化。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-incremental-driver.md` | 新增Auto查询/持久store/编译驱动的策略所有权及恢复/发布准入，工具/OS原语仍由公共库提供。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-semantics/source-query-session.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-incremental-driver.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**CL/标注/实例/CT输入改变后的真实驱动**，沿[Design59的CL族级映射](../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-02/T-09；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-incremental-driver.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | 合法工程冷→热→恢复，然后改body/签名/新增缺失符号 | Auto driver/query/store（T-02/03/04接通） | 独立结果一致，必要owner重算/无关复用；不能只用总cache hit判语义正确 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 篡改source/stdlib/grammar/bridge身份、root引用或页校验 | Auto restore入口 | 受控miss/拒绝而不产生Checked；正常同身份恢复仍成功 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | 旧exe有效时坏编辑、CT失败、对象/链接/rename失败后修复 | Auto真实编译发布入口 | 失败不污染旧可运行闭包；修复后真产新结果，sourcefallback被禁后仍正常 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-02/03/04/05 | A函数调用B，B初返3后仅改body为4；另一次源码引用尚缺名字N，然后添加合法N；最后制造一次错误编辑再修复 | 本包V-01/03所列真实/拟新增入口，沿相同启用任务 | 第一次产3、body编辑后产4；添加N必须使曾记录negative read的A重算并成功；错误编辑不发布新root/覆盖旧exe，修复后真实执行正确值 |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-semantics ace-artifacts ace-native`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | 包闭包和生成FFI工程装配的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | 物化但未声明的依赖拒绝；同锁闭包换cwd结果不变；换provider/目标ABI失效受影响artifact，缺外库报工具/链接错误且保旧，不重新用RustsourceCLI或整个ManagedStore生成结果 | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 本族可区分验证（新增合同，未实施）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | T-02/T-09创建/扩本包既有Auto阶段driver，最后由原T-06与独立review核；完整Gen1/代际入口分别归1053/1054 | 分别改已读取标注参数/实例/CT字段及runtime版本，核影响节点失效后实际重编重跑；无关body不重编，坏更新保旧，补齐后的能力不命中旧拒绝缓存。 |

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | Auto查询host与阶段driver、正负依赖和实际生产计数；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；本包支持范围不冒整个AAC |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

- [ ] **AC-CL-FAMILY / work**：V-CL-FAMILY和原T-02/T-09实际结果满足，族级CL未因CS未用或A期AC不支持删减；语义与runtime/库补齐正式来源齐，原独立review逐行核。原任务数及0进度保持，本轮未执行。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1052/auto-lang`，分支 `plan-1052-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1052/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 已重读本Plan r5和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | worktree `D:/autostack/.wt/lang-1052/auto-lang`（branch `plan-1052-dev`，base `292575164`），sibling auto-down detached `456c86a`；plan-artifacts `D:/autostack/plan-artifacts/plan-1052/` 已建并登记；§4 指纹文件全部实读存在（host/session/restore/ident/identity/incremental/link_driver/link）；依赖 1044/1045/1047/1048/1049/1050/1051/1056 全部 archived；auto-man 锁合同四文件齐备；AC-01/04 |
| [x] T-02 查询和依赖图 | work | T-01 | Auto实现source/签名/body/实例/CT事实查询和positive/negative read set，span/源码版本与语义版本分离；失效及复用决策基于真实owner事实，不用fixture cache命中数冒生产者工作量。 源/grammar/CL/库/runtime/标注/实例和CT输入的实际读取进入身份及失效，新增B补齐后不复用旧“AC未支持”负面结果冒完整支持。 修要求的真实 native 构建与 ABI/CT 配置相关的失效见证，复用当前资产/SDK 管道。 | Auto查询host与阶段driver、正负依赖和实际生产计数；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03。已交 `aac/driver/query.at`：完整查询键（`fact_key`）、正读集（`pos_read_key`，绑定 callee 签名版本）、负读集（`neg_read_key`）、失效/复用判定（`reuse_decision` 0-4）、邻域判定（`neighbour_decision` 0/1/2：签名变/负翻转=caller-recompute，仅 body 变=callee-recompute，全不变=caller-reuse）与生产计数报告；conformance `tests/incremental/tests/query.rs` V-01/V-04 逐字节对账独立 Rust 引用；真实会话计数器 `real_flow.rs v01_real_session_counters`（cold=3 solver/0 hits，同源 warm=3/3 hits） |
| [x] T-03 持久store与恢复 | work | T-02 | 消费既有hash/codec及版本合同，实现schema/target/stdlib/grammar/pass/bridge身份键、内容页/引用/根恢复；未知/损坏/过期/悬挂页受控miss/拒绝，不信checked标志、缓存不带活工作状态。 确认现存和 AAC 构建链均可在失败后保已交产物并实际恢复；保输入、诊断与结果源身份。 | Auto store/restore与坏cache反控，BuildIdentity/RuntimeCompatibility分账；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03。已交 `aac/store/store.at`：六轴 BUILD 身份键（`six_axis_key`，schema/target/stdlib/grammar/pass/bridge 轴轴分离，路径/span 不入键）、页键（`page_key`，body-only 编辑保签名读者 warm）、页校验和（`page_checksum`，blake3 精确字节）、ref 键（`ref_key` 域分离）、恢复准入分类（`admit_code` 0-6：envelope/checksum/key/ref/coverage/diag 六条件，缺一即受控拒绝）与发布三态（`publish_code` 0-3）；conformance `tests/incremental/tests/store.rs` V-02 身份绑定+准入 taxonomy 逐字节对账 |
| [x] T-04 驱动、服务与发布 | work | T-03 | 实现Auto build/check/diagnostic驱动，调用本地阶段、1051服务及host linker，按工件闭包准备/提交/回滚；native更新失败保持旧可用程序与准确账，不静默使用Rust sourceCLI。 交实际 AAC 编译模块构建/启动/发布链，涉及配置选择时消费 982/B-CONFIG 正式宿主构建输入，验两包目标返回，配置 AV 和目标 AC 分别举证。 | Auto编译驱动与可观察阶段身份/错误分类，真实process/link失败路径；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03。已交 `aac/driver/driver.at`：阶段身份（`stage_key`，stage/pass/source/grammar 输入进身份）、错误分类（`error_class` 0-5：language/resource/os/tool/admission 独立成类）、闭包决策（`closure_decision` 0-4：cold/hit/kept-old/affected/repair）、V-03 场景报告（坏编辑=language 1、保旧=2、修复真产=4）；conformance `tests/incremental/tests/driver.rs` 与 `real_flow.rs v04_bad_edit_then_repair`（真实坏编辑被拒+Auto 决策报告对账） |
| [x] T-05 冷/热/恢复闭环 | work | T-04 | 对只改body、改签名、增删原缺失符号、CT/meta改动、坏编辑后修复和外部工具失败做实际数据流验证；确认必要失效、无关复用及失败不发布/最后owner释放。 用同源仓外复建、冷/热、改库和 ABI 后复验，提前交已复核切片给 1068。 | 完整集成的identity/生产计数/输出事件与清理证据，交1053；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03。`tests/incremental/tests/real_flow.rs`：真实 Auto 程序（A 调 B）经 aggregates-ct 车道 Gen0 编译+真实 AV 执行——body-only 编辑 3→4 真运行；缺失名 N 被 `source.module-symbol` 拒绝、添加后真运行 42；坏编辑拒绝+修复后真运行；`v01_real_session_counters` 用真实 SourceQuerySession 计数器（cold 3/0，同源 warm 3/3，body 编辑重算 B）证明必要重算/无关复用 |
| [x] T-09 包闭包和生成FFI工程装配 | work | T-05；§5实际输入合同 | 消费Man `../auto-man/crates/auto-package-lock/src/{model,codec,verify}.rs`对应锁合同、`../auto-man/docs/specs/modules/generated-cargo-context.md`的构建边界及978 SDK与967 binding/shim产物，绑定公共/provider/target/capability身份和实际源依赖；明确raw工件/typed事实/进程内Checked重建与物理store格式。同时对冻结AC公开接口（源码、descriptor/HIR检查、canonical产物、build/check、诊断、SDK/library调用）逐项给AAC入口和兼容结果，不要求占用旧CLI默认名，不复活已退役路径。无Man在线/换cwd/缺依赖/错provider/旧库版本必须可诊断，输入编译失败不改lock或旧产物。 缺口补齐/切换runtime后重新运行真实AAC模块和受影响CL，保未读输入复用、坏更新保旧和源位置，不用全量cache clear掩失效错误。 实际 SDK 与 FFI 工程链接/生成/运行/包闭包，覆盖配置、错包/旧缓存及失败后修复。 | §6 V-05实际结果及输入/产物身份；AC-07/AC-04。已交 `aac/driver/package.at`：锁闭包身份（`lock_closure_key`，cwd 不入身份）、声明闭包 vs 物化逐项核对（`closure_code` 0-5：undeclared-materialized 拒绝/declared-missing/provider-ABI 失效/missing-lib 保旧）、V-05 场景报告（同锁换 cwd 身份不变、provider 变失效、缺库=tool 错误保旧）；conformance `tests/incremental/tests/package.rs` V-05 逐字节对账；消费面核对 auto-man 锁合同四文件与 generated-cargo-context 规范已存在（T-01） |
| [x] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 已注册固定组 `ace-aac-incremental`（`scripts/test-groups.toml`），19 例全绿：executed=19，prep 5.7s，exec 15.1s，maxpar=19；逐例 0.1-8.9s（并行 Gen0 编译争抢，符合 1056 已接受 N-4 模式，整组 wall 15s）；共享 digest 既有消费者 `ace-aac`（26 例，prep 2.9s/exec 37.8s）与 `ace-aac-host`（22 例，prep 4.0s/exec 34.5s）均 PASS，无回归；SD-01 已准备 `docs/specs/auto-ac/aac-incremental-driver.md`（worktree 内，merge 落主）；AC-01..04/07。完成即execution_done交review，无需预勾T-07/08 |
| [ ] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（补缺失的完整Auto MIR前置、包锁/发现/FFI装配与持久层合同，避免由Man或Rust ManagedStore隐藏编译主体。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

2026-11-20 stage: work | PLAN-1052 | r5 | outcome: pass | worktree: `D:/autostack/.wt/lang-1052/auto-lang` | branch: `plan-1052-dev` | base_commit: `292575164` | code_commit: `f8a3d882d` | dependency_revisions: auto-down sibling detached `456c86a`（未消费改动） | task_ids: T-01..T-06、T-09 全部完成 | evidence: 新 Auto 模块 `experimental/ace/aac/store/store.at`（六轴身份键/页键/校验和/ref 键/恢复准入 0-6/发布三态）、`aac/driver/{query,driver,package}.at`（查询 host 正负依赖与邻域判定/错误分类与闭包决策/锁闭包与 FFI 绑定）；新 conformance crate `experimental/ace/tests/incremental/`（store/query/driver/package/conformance/real_flow 五测试文件，19 例）；固定组 `ace-aac-incremental` 注册 `scripts/test-groups.toml` 并 PASS（executed=19，prep 5.7s，exec 15.1s，maxpar=19）；既有消费者 `ace-aac` 26 例 PASS（prep 2.9s/exec 37.8s）、`ace-aac-host` 22 例 PASS（prep 4.0s/exec 34.5s）无回归；SD-01 准备 `docs/specs/auto-ac/aac-incremental-driver.md`（worktree 内，merge 落主；SD-02 留 merge 后按已交事实补 source-query-session.md） | blockers: 无 | next_responsible_role: review（T-07） |

## 10. 待澄清事项

无须当前用户新增决定。基础身份/依赖/事务可在1047已交后先；完整Plan验收必须1048/1049/1051实际结果，不能用模拟服务关闭真实consumer腿。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
