---
plan_id: PLAN-1053
status: executing
feature_name: AAC Gen1完整语言闭环与主体迁移核验
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-22
plan_revision: 8
current_step: 2
total_steps: 9
depends_on: [PLAN-1044, PLAN-1045, PLAN-1046, PLAN-1047, PLAN-1048, PLAN-1049, PLAN-1050, PLAN-1051, PLAN-1052, PLAN-1055, PLAN-1056]
supersedes_spec_components: [docs/specs/auto-ac/project.md]
new_spec_components: [docs/specs/auto-test-case/aac-language-parity.md]
touched_goals: [GOAL-002]
---

# [PLAN-1053] AAC Gen1完整语言闭环与主体迁移核验

## 0. 变更摘要

### 2026-11-19 r8 · 接收AV-ONLY阶段与非AV检查恢复

用户已明确1078–1096只验AV，公共语料中AC/A2X等检查先以检查级#ignore暂缓。其未实现/未运行/失败不阻AV计划，不充PASS；本包进入对应后端开发/验证阶段后，逐check接手、移除本阶段标记并真实执行。共享源码、原oracle/条件/owner和本包完整验收门保持；不得要求AV计划先交非AVPASS。本轮仅修订规划。

### 2026-11-19 r7 · 公共语言语料按特性共源（仅规划）

按用户要求接[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)与1078的`IF-ACE-FEATURE-CORPUS/1`。AAC的CL公共语言语料复用language Case，在同Case绑定实际Gen0/Gen1编译器与目标运行检查；CS编译器主体源码/专属IR探针仍归AAC原位置，不把同一CL源码再复制到cases/aac。按代际/编译器/SDK/源指纹分别记结果，不以Rust reference/emit替运行。 原任务ID/数量/进度与已有证据边界保持，本轮不施工、不迁实际测试。

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包按原任务收实际生产任务的证据，安排已验阶段交接。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：细化主体禁用窗口和完整语言/库/领域真实结果，明确早期阶段测试由各包执行，本包汇总而非首建全部测试。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。由冻结Rust AC真编出native Gen1 AAC，接通全部Auto主体与显式服务，以完整CL和真实标准库/领域接口验证原生编译器，而非仅编出自身子集。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

由冻结Rust AC真编出native Gen1 AAC，接通全部Auto主体与显式服务，以完整CL和真实标准库/领域接口验证原生编译器，而非仅编出自身子集。 本包出口见AC-01..06。依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**crates/auto-lang/test/cases/aac/仅保AAC自身CS/阶段IR专项，公共语言CL引用language特性Case；experimental/ace/aac/tests/integration/及其薄TestCase adapter（拟新增）**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `scripts/test-groups.toml` | 1b432bb63abff7fe… |
| `experimental/ace/crates/case-worker/src/lib.rs` | 51a524b0bac9eae8… |
| `experimental/ace/crates/native/src/bin/ac-native.rs` | 7ef4fadfe510a653… |
| `docs/design/55-ace-m2-complete-language-and-product-replacement.md` | 7e4f4034a526b890… |
| `docs/specs/auto-test-case/ace-m1-expanded-acceptance.md` | 18ad22dc70c1a7da… |

规范输入：`docs/specs/auto-ac/project.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：1044–1052必要交付；985/A全分母、原TestCase/AutoDown格式和独立expected，领域host/SDK来自原M2-A owner。外仓无代码迁移依赖。

**当前开工状态**：所有主体/服务/驱动输入都已正式交付，Windows x64 Gen0 toolchain/SDK可复现；不要求M2-B完成或产品改默认。 公共G-AAC须985/T-A04独立复核、T-A05已合入A收据/M2A-01..08；不等985-B、984全产品、986/987整包。frontmatter只列AAC包级最终依赖，任务级准备以§5为准，不用整包串行制造环。

## 5. 详细设计

### AV-ONLY接收与本目标启用

1078的IF-ACE-FEATURE-CORPUS/1和1096的AV语言证书允许携带非AV#ignore/NOT_RUN行，接收时只核AV声明和迁移义务未丢，不以非AVpending回卡AV独立出口。本包实际验证自己的后端/代际时，由原生产任务逐check去掉AV阶段#ignore，保原平台/ignore与oracle，按当前同源/工具/SDK身份运行；失败属于本目标任务，必须修到本目标required齐才签其完整出口。1068/985的最终多端门及AAC独立门不接受忽略充PASS；AV阶段通过从不冒多端通过。

规范影响：公共作者/引用/选择/迁移协议由1078统一在拟`docs/specs/auto-test-case/feature-organized-corpus.md`沉淀；本消费者只在自身原SD/模块Spec记录实际Case引用、目标动作和正式证据，不再各写一套公共协议。AC-FC属于本包验收增量，原SD与目标义务全部保留。

### FC · canonical语言输入与旧义务承接

AAC的CL公共语言语料复用language Case，在同Case绑定实际Gen0/Gen1编译器与目标运行检查；CS编译器主体源码/专属IR探针仍归AAC原位置，不把同一CL源码再复制到cases/aac。按代际/编译器/SDK/源指纹分别记结果，不以Rust reference/emit替运行。

由原T-01/T-03/T-04/T-05/T-06/T-09按其职责消费/迁移对应行，保原source/expected/check及条件，重复仅在完整输入闭包/语义等价时合并，原义务→canonical Case/check映射不得丢；既有固定组/suite/worker读者按1078接口切换，不新建按后端/代际维护的Auto副本。缺字段/目标接口由1078和实际producer在原框架有界补接，不要求本包先等全AV/全AAC完成。

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

Gen0编译AAC源码S时允许其Rust源码前端；禁用Rust源码语义的观察窗口从Gen1启动并接收被测Auto输入开始，Gen1/2/3均禁parse/bind/infer/实例/HIR lowering/sourceCT fallback，安全MIR复验不误判。1053汇总各包已交driver/fixture及当前TestCase薄adapter，不以旧Rust模块组绿替代Auto模块执行。CL按host/target/provider适用矩阵及合法/非法输入分别记录，NA必须是冻结规范所定；此独立首版仍Windows x64，不改变M2完整平台义务。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-test-case/aac-language-parity.md` | 新增Gen1实际native AAC与CS/CL/主体/桥四类义务的独立准入；M2-A证据只作输入映射，不直接沿用为AAC通过。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-ac/project.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-test-case/aac-language-parity.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**Gen1编译完整语言而非仅编自己**，沿[Design59的CL族级映射](../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-03/T-04/T-05/T-09；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-test-case/aac-language-parity.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

### V-FC-CONSUMER（work / 原T-01/T-03/T-04/T-05/T-06/T-09，未执行）

同一canonical source变化必须使本包实际目标/代际依赖失效并用独立oracle重验；单目标oracle变异、遗留旧活入口、删原义务映射、零选中/缺required检查和emit冒run分别判红。公共语言输入去重不减少旧资源/错误/恢复/平台条件；typed IR/后端专项及外仓完整产品保原owner。实际结果按Case＋check＋编译器/目标/动作＋source/SDK/provider身份记录，不以AV或其它代结果回填。

**AC 语法同源验收。** 完整 CL 认证以 AV 补全线正式交付的同源 AutoDown 语料为语言验证输入，逐 Case 运行 Gen1 生成的真实 AC 产物，照原 oracle 验成功、失败及资源。现有用例可先验证，缺项在 1096 交付后对账。 按 [AV AutoDown 共用语料规则](../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | 全部CL适用普通源码+库操作和领域真实host样本 | Gen1 native AAC→实际native产品/host（T-03/04使入口可运行） | 逐义务独立expected全绿且保适用性；Rustreference一致不是唯一oracle | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 禁止Rust源码服务并植入会被Auto语义拒绝的非法源 | Gen1真实构建入口+服务调用审计 | 正例正常、非法源在Auto相应阶段拒绝；任一Rust源码语义调用使主体门红 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | 漏必需用例/工具身份错/worker假PASS/emit冒run | 实际TestCase父准入（T-03接线，T-05反控） | 这些控制均导致总门失败；合法完整收据可通过，仅签Gen1阶段 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-03/04/05 | CL分母保一条必需资源程序，真实结果期望7/drop=1；仅将其报告改成emit-only或省略该行 | 本包V-01/03所列真实/拟新增入口，沿相同启用任务 | 真实编译执行得7且drop=1才计PASS；emit-only/漏行两种控制整体红，不能借Rustreference或旧M2收据填该AAC观察 |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-source ace-semantics ace-comptime ace-lowering ace-native ace-abi ace-artifacts`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | 主体观察窗口及整机组合的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | Gen0编S正常；Gen1禁Rust源码服务仍处理全部CL；故意在helper中请求一次Rust绑定使主体门红。缺一必需域/库操作或只有emit而无run，整机门红，M2-B无需改默认 | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 本族可区分验证（新增合同，未实施）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | T-03/T-04/T-05/T-09创建/扩本包既有Auto阶段driver，最后由原T-06与独立review核；完整Gen1/代际入口分别归1053/1054 | 让AAC自身源码不使用async/某标注但CL包含，Gen1必须编成AC并执行独立expected；删该行、以旧A适用集替CL、只emit、Gen0冒Gen1或共同错误均阻完整通过。 |

## 7. 验收标准

- [ ] **AC-FC｜owner_stage: work**：V-FC-CONSUMER及§5 FC逐原义务满足，公共语言唯一作者/本目标真实动作与独立结果/迁移映射均可追溯；原独立review复核，merge只接真实落主来源。未交required目标保pending，不因共享目录、少文件或另一端通过而降门。

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | 真实Gen1 exe/身份清单与无Rustsource服务证据；自举壳/转译壳不计；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；本包支持范围不冒整个AAC |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

- [ ] **AC-CL-FAMILY / work**：V-CL-FAMILY和原T-03/T-04/T-05/T-09实际结果满足，族级CL未因CS未用或A期AC不支持删减；语义与runtime/库补齐正式来源齐，原独立review逐行核。原任务数及0进度保持，本轮未执行。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1053/auto-lang`，分支 `plan-1053-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1053/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 记录worktree/branch/base与§4指纹变化、支持profile和后续任务入口；AC-01/04。已核（2026-11-22）：专树 `D:/autostack/.wt/lang-1053/auto-lang`@`plan-1053-dev`，base `9a34e2eee`；材料scope `D:/autostack/plan-artifacts/plan-1053/` 已登记 `.retention/policy.json`；§4指纹3/5不变（case-worker/ac-native/ace-m1-expanded-acceptance），`scripts/test-groups.toml` 与 design/55 已演进（历史取材，不阻塞）；1044–1052/1055/1056 依赖全部 archived。**开工门核实已闭（2026-11-22 更正）**：985/A 的 T-A04 独立phase复审 pass（r29，reviewed ecc282df1，独立子代理上下文）＋T-A05 A批fold pass，M2A-08 正式成立并落主 `95c7fbad8`（ff-only线性，A阶段Spec `docs/specs/ace-migration/m2-certification.md` 在delivery内；985 目标会话 round3 outcome pass done）——早前 T-01 初判"T-A04未执行"系只读 r29 §9 首条 mid-session 记录所致误判，已更正。机器占用注意：lang-1116 构建期须错峰。T-02 据此启动 |
| [x] T-02 Gen0构建与主体归属 | work | T-01 | 用绑定M2-A Rust AC/标准库/后端工具真编同一AAC源码树S为native Gen1，记录源码/工具/桥/ABI/依赖hash；审计服务入口与动态调用，Gen0编译S的窗口允许Rustparse等；从Gen1接受被测请求开始，Rustparse/resolve/infer/HIRlower/sourceCT入口禁止访问。 | 真实Gen1 exe/身份清单与无Rustsource服务证据；自举壳/转译壳不计；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03。已交（2026-11-22）：新 conformance crate `experimental/ace/tests/aac-gen1`（`gen1_modules()`=全树S 60模块；`support/driver.at` 因 stem 冲突改挂 `support_driver.at`（S内无 app.driver 引用）；`hir_schema_data.at` 按冻结schema合成；`boot_lib.at`+`gen1_main.at`=整机boot入口（入口unit profile拒typed/空表字面量，i64折入普通模块）；`prepare-gen1` bin=sys装配+Gen0窗口编译+`MnDriver::build_hosted_collections_checked` 真链接PE。产物 `aac1053-gen1.exe` 752,640B 真运行（runtime观测 allocs=frees=5379 live=0）；manifest=fingerprint `71a04efe…`+exe blake3 `f0846d4e…`+runtime `a058a96e…`+rustc 1.98.0。固定组 `ace-aac-gen1`（test-groups.toml+nextest-full bound=4）PASS 3/3（boot parity 0.58s，逐例≤10s）。boot退出码=wrap32 FNV折减=681978131，与Rust独立契约镜像逐位相等（常量引契约+sha2参考+blake3轴镜像；AV整树参考不可行：单一lowered函数ownership分析~34MiB>冻结16MiB verify预算，按1046 Rust前门同源oracle先例）。服务审计：exe外部管道仅collections runtime+Win32 shim、bridge client（spawn5+私有目录帧交换，后端服务）、CT invoke（钉pin信封）、host file/process（OS服务）；S无模块消费Rust parse/bind/infer/HIR-lower/sourceCT。兄弟组回归 support/host/incremental 67/68，`v05_host_diag_scenario`红=master复放同红（既有红非本批触面，归批量回归triage） |
| [ ] T-03 测试义务与固定组接线（部分交付，余量：corpus recipe 接线+逐check接手） | work | T-02 | 复用 `crates/auto-test-case`、`crates/auto-lang-core/src/test_cases/{mod,ac_av}.rs`、`vendor/autodown-test-parser/PROVENANCE.md`及ACE case-worker的现合同，创建CL逐行用例/去向、CS模块调用与新反控，复用各前序包已交阶段driver/选择，只补整机AAC薄adapter及固定组汇总；把实际native AAC命令/阶段产物配置成可执行入口，名称在实现时登记。 逐最终CL语言/库/领域行绑定AAC实际编译入口→AC真实产物→执行oracle，给AAC源码未用特性建永久语料；与AV/A2R正式producer分别对独立expected。 | 永久AAC语料与可运行固定入口；原旧义务不减、未观察不得PASS；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03。已交（2026-11-22 部分）：①**可执行入口登记**：`aac1053-gen1.exe`（整机boot，mn真PE）+ `aac1053-compile.exe`（整机编译命令=1047语义驱动复用为 app::main，A2R+rustc 车道，env 合同 AAC1047_INPUT/HIR(modes verify|mir|…)/AAC1055_ABC；AC(mn) 车道受冻结 verify 预算阻、整机缺口已具名记 §10/T-09）；双身份 manifest 准入（`prepare-gen1` 一次产出）。②固定组 `ace-aac-gen1` 4/4 PASS（boot parity/PE身份/stale拒/compile入口：正例 mir 报告头+ABC 长度自洽+双跑字节一致、反例 ERROR+阶段码）。③共享接面修复：a2x-rust moved_locals 分支/循环感知（`013c4fde5`），1046/1047 prepare-native 车道恢复。④语料勘察定案：language 共源语料27文件/ac-native check 带 AV-ONLY #ignore，recipe 接线设计入 plan-artifacts/t03-survey-design.md。余量：ac-native recipe adapter 实接（case-worker/test_cases 侧）→ 逐 check 去 ignore 接手；CS/阶段IR专项语料（cases/aac） |
| [ ] T-04 全语言/库与领域执行 | work | T-03 | Gen1编译运行全部冻结CL的AC适用输入，覆盖普通源码/完整标准库调用、泛型CT、dynamic/闭包/async/foreign/资源及Schema/UI/AGE最小真实host执行；每项保独立expected，Rustreference仅补对照。 显式执行~T/?T/!T与泛型/CTE/required标注的跨模块/owned资源组合，及完整Node/FFI消费；不能只自编译、只parse/emit、只跑happy-path或仅编一个代表例。 | CL全量结果与host/provider/生命周期记录；不凭parse/emit代run，不迁整个产品；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-05 反控与完整性 | work | T-04 | 实际运行漏CL行、假PASS worker、错工具身份、隐藏Rust源码fallback、emit冒run及服务重做语义反控；核CS/CL/主体/桥各分母、阶段错源/预算错误和缺平台pending。 增加删未被CS使用的async/标注行、改为A期native不适用、所有端共同错误和以Gen0替Gen1源码前端的负控，均须阻完整CL。 | Gen1本阶段独立可复核收据；Gen2/3保持未执行，不签整个AAC；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-09 主体观察窗口及整机组合 | work | T-05；§5实际输入合同 | 建立Gen0-build与Gen1-request明确分窗及真实Rust源码入口阻断/计数控制；对全部Auto主体模块、生成源码、CT/库、FFI ABI、包闭包及原M2-A领域最小host逐行映射。保原完整UI/AGE作者源兼容义务，不能只测试一个Hello或把领域前端移出主体当runtime。 用来源/调用审计区分Gen0构建AAC与Gen1处理测试程序；1053完整CL不得因Design59 具名实施任务未交而记NA/通过，阶段切片可先正式交接。 | §6 V-05实际结果及输入/产物身份；AC-07/AC-04 |
| [ ] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 实际命令/退出码/输入和产物hash、覆盖与缺口；AC-01..04。完成即execution_done交review，无需预勾T-07/08 |
| [ ] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

- 2026-11-22 stage: goal | plan_id: PLAN-1053 | plan_revision: r8 | round: 5 | from: work | to: work | status: executing | outcome: pass（T-03 切片：共享发射器修复 + Gen1 compile 入口交付） | worktree: D:/autostack/.wt/lang-1053/auto-lang | commit: cce104008 | next: work（T-03 余量：corpus recipe 接线与逐 check 接手）

- 2026-11-22 stage: goal | plan_id: PLAN-1053 | plan_revision: r8 | round: 4 | from: work | to: work | status: executing | outcome: pass（gate 更正后继续） | worktree: D:/autostack/.wt/lang-1053/auto-lang | commit: 9a34e2eee | next: work（T-02）

- 2026-11-22 stage: goal | plan_id: PLAN-1053 | plan_revision: r8 | round: 3 | from: work | to: work | status: executing | outcome: waiting（连续第 3 轮，转 blocked） | worktree: D:/autostack/.wt/lang-1053/auto-lang | commit: 9a34e2eee | next: user

- 2026-11-22 stage: goal | plan_id: PLAN-1053 | plan_revision: r8 | round: 2 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1053/auto-lang | commit: 9a34e2eee | next: user（§10 无新答复；连续 waiting 第 2 轮）

- 2026-11-22 stage: goal | plan_id: PLAN-1053 | plan_revision: r8 | round: 1 | from: work | to: work | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1053/auto-lang | commit: 9a34e2eee | next: user（985/A 开工门闭合后再入 work T-02）

- 2026-11-22 stage: work | plan_id: PLAN-1053 | plan_revision: r8 | outcome: blocked-on-named-dependency（T-01 完成， verdict=未就绪） | worktree_path: D:/autostack/.wt/lang-1053/auto-lang | branch: plan-1053-dev | base_commit: 9a34e2eee | code_commit: 9a34e2eee（无代码改动） | dependency_revisions: 1044–1052/1055/1056 archived；985 executing 6/14（A批work收口 r29，T-A04/T-A05 未执行，M2A-08 未落主） | task_ids: T-01 | evidence: T-01 行内记录（指纹3/5不变、test-groups.toml/design55演进、机器占用 lang-1116、材料scope已登记） | blockers: 985/A 开工门未闭——须 985 T-A04 独立复审 pass 且 T-A05 落 M2A-08 至 master 后本包方可绑定A基线开工；具名依赖属 PLAN-985 自身范围，本目标不代行 | next_responsible_role: user decision → work（T-02）

- 2026-11-19 r8规划交接：明确AV路线只选AV，非AV#ignore由本目标实施阶段逐项恢复；原任务/状态/进度、完整门与原预期保持，本轮没有实现或运行。

- 2026-11-19 r7：`stage: new/revision`，`outcome: pass`（规划静态预检），`next: work`待后续指令/所需来源。新增FC/V-FC-CONSUMER/AC-FC，原T-01/T-03/T-04/T-05/T-06/T-09补接同源语言/迁移合同，任务/进度不变；未移动用例、修改expected、运行或合入。

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（细化主体禁用窗口和完整语言/库/领域真实结果，明确早期阶段测试由各包执行，本包汇总而非首建全部测试。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

## 10. 待澄清事项

2026-11-22 语料清扫具名缺口登记（T-03/04 实证，sweep 测试内冻结清单守卫——新增 blocked 探针即红，须先在此具名）：①`av.parity.1080.pipe-method-and-field`→`hir.unsupported 20..53`，即 1047 归档已开放的 pipe 方法 `lower.method` 缺口（owner：Design59 具名实施任务/1047-1050 后继）；②`av.parity.1092.with-unconstrained-type-parameter`→`type.unresolved 28..29`，未约束泛型形参未解析（owner：1048/1050 实例需求求解后继；属 985/A 允许的具名 AC 差额账）。已跑通：`av.parity.1078.baseline-value` exit 14 过全链。关闭条件：对应 owner 交付正式来源后同输入重跑 sweep 自动转 ran。

2026-11-22 更新——**阻塞已关闭（本包在约内修复）**：根因 = a2x-rust `lower_coll.rs` 的 `moved_locals` may-kill 集合控制流不敏感——单分支内 `.move` 把根标记死，兄弟分支与汇合后误拒；循环体 kill 泄出循环头。误杀实例：`source_catalog.import_paths` 的 `if prefix.len() == 0 { prefix = part.move } else { prefix = prefix + "." + part }`（诊断增强后定位到根名+span）。修复（plan-1053-dev `013c4fde5`）：if 两分支以保存集合各 lowering、分支末集合**按交合并**（pre-if kill 保留）；循环体恢复 pre-loop 集合（零迭代可靠；真实 use-after-move 仍由上游冻结 MIR ownership 验证拒绝）。验证：`prepare-native` native+semantic 两模式恢复链接（aac1046-driver/aac1047-driver）；`ace-aac-gen1` 4/4 PASS（compile 入口取消 ignore 并交付：aac1053-compile.exe=A2R 车道整机编译命令）；aac 家族 ace-aac/ace-aac-host/ace-aac-incremental PASS，ace-aac-syntax 51/51 隔离绿（`aac1046_corpus_text` 组内 8.2s 红=同机争抢越过该用例 8s harness 预算，隔离 5.65s 绿、4 线程 libtest 绿，按环境归因记债务候选，非本修复语义变化）。1115/1116 的 A2R 语料车道为本修复的消费者复核面（记入 T-06/review 选择理由）。原记录保留备查：
~~2026-11-22 具名外部阻塞（T-03 发现，owner：a2x-rust 共享接面修复……~~（原文归档于 git 历史）

2026-11-22 更正与关闭：T-01 初判"985/A 开工门未闭"系误判——完整核对 985 §9 后确认 T-A04 phase复审 pass（r29/ecc282df1，独立子代理）＋T-A05 fold pass，**M2A-08 已成立并落主 `95c7fbad8`**（985 目标会话 round3 done）。开工门实已闭合，无须用户在 §10 二选一；用户已授权的实施指令有效，T-02 启动。下列更正前的问题记录作废保留备查：
~~2026-11-22（T-01 核得，需用户决定）：本包开工门为 985/A 正式来源（§5"985/A正式来源仍为开工门"），但 PLAN-985 当前 status executing 6/14——A批 work 已收口（r29，ecc282df1）而 **T-A04 独立复审未执行、T-A05 未落 M2A-08**，A基线未正式冻结落主。按 T-01 合同"未就绪停在具名依赖不消费WIP"，T-02 及以下未启动、零代码改动、专树与材料scope保留待续。 unblock 二选一：（1）先完成 985/A（用户授权跑 985 的 T-A04→T-A05，落 M2A-08 后本目标自动续跑 T-02）；（2）用户裁定本包可针对 985/A 未落主的 work 批次基线先行开工（等于修订开工门合同，属用户决定，须显式授权）。~~

原 standing 记录（2026-11-19 r8前）：所有主体/服务/驱动输入都已正式交付，Windows x64 Gen0 toolchain/SDK可复现；不要求M2-B完成或产品改默认。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
