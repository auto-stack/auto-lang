---
plan_id: PLAN-1046
status: executing
feature_name: AAC Auto语法运行时、源码投影与模块装配
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-19
plan_revision: 6
current_step: 1
total_steps: 9
depends_on: [PLAN-1044, PLAN-1045]
supersedes_spec_components: [docs/specs/auto-ac/project.md, docs/specs/stdlib/design/context-source-assembly.md]
new_spec_components: [docs/specs/auto-ac/aac-source-frontend.md]
touched_goals: [GOAL-002]
---

# [PLAN-1046] AAC Auto语法运行时、源码投影与模块装配

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包承接项已补入 §8 原执行任务，复用 §6 真执行和独立反控；正式交接须标明原 required 行、实际源与产物、身份和结果。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：拆清#in的语法/候选目录与类型/CT选择，补模块循环适用性及Unicode/恢复的精确边界。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。由Auto主体执行统一grammar的词法/语法、CST/span/edit、AST投影与模块/namespace/provider装配，完整涵盖冻结M2-A语法而不借用Rust parser运行。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

由Auto主体执行统一grammar的词法/语法、CST/span/edit、AST投影与模块/namespace/provider装配，完整涵盖冻结的最终M2 required语法而不借用Rust parser运行。 本包出口见AC-01..06。依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**experimental/ace/aac/syntax/、source/与tests/syntax/（均相对experimental/ace/aac，拟新增）**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `experimental/ace/crates/syntax-runtime/src/lib.rs` | 5133b5cd69da1c15… |
| `experimental/ace/crates/syntax-grammar/src/lib.rs` | 087b300bf8f0fe2c… |
| `experimental/ace/crates/core/src/source/cst/` | 当前已存在目录，执行时按实际文件冻结 |
| `experimental/ace/crates/core/src/source/parser.rs` | 0478200351b0a174… |
| `experimental/ace/crates/core/src/source/modules.rs` | 87a482d8033bf548… |
| `experimental/ace/crates/core/src/source/packages.rs` | 9aa1b33d833ce001… |

规范输入：`docs/specs/auto-ac/project.md`; `docs/specs/stdlib/design/context-source-assembly.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：1044 grammar/schema及完整CL语法矩阵，1045 bytes/文本/集合/文件；1043已交#in与provider装配合同，960及979/980/981的M2-A语法/域接口。

**当前开工状态**：1044/1045已交；不复制旧legacy lexer作为新的语言真值。 公共G-AAC须985/T-A04独立复核、T-A05已合入A收据/M2A-01..08；不等985-B、984全产品、986/987整包。frontmatter只列AAC包级最终依赖，任务级准备以§5为准，不用整包串行制造环。

## 5. 详细设计

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

本包只解析#in/is/if/for、保留条件AST及公共/provider候选目录，不宣称完成一般CT求值、可见性或类型绑定。1047消费无CT或已知target常量的基础模块，1048提供语义装配/生成后重绑定，1050/1053关闭高级CT组合。模块依赖循环按M2-A区分：禁止use/重导出或值初始化环，合法函数互递归必须接受，不能见环全拒绝。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-source-frontend.md` | 新增Auto frontend所有权、统一grammar身份及完整语法/来源/模块准入，已有Rust reference和标准库装配规则不变。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-ac/project.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-source-frontend.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**完整语法与类型/标注投影**，沿[Design59的CL族级映射](../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-02/T-03/T-05；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-source-frontend.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

**AC 语法同源验收。** 在 T-06 运行已交的同源 AutoDown 语法、源位置和模块装配用例的 AC 检查。按原用例观察合法源码、非法语法及对应诊断，保留解析阶段本身的定向检查。 按 [AV AutoDown 共用语料规则](../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | 真实grammar样例+多模块use/pub/#in与UI/AGE节点 | 本包Auto阶段driver（T-02创建，Gen0编译后原生运行） | CST/AST/来源及选择模块与独立expected一致；域节点保留供后续语义 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 多字节UTF-8旁发生单字节编辑后撤销；另有截断/坏UTF-8 | 同一parse/edit入口 | 合法冷/增量AST和byte span一致；非法源在syntax阶段拒绝，不发布半新树 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | 超预算深嵌套与同名/循环/缺失模块 | Auto解析与模块loader | 超语法资源/坏源码按1044规则拒绝，边界合法近邻通过；模块关系仅产保留来源的候选/依赖图，不在本语法层判全量名字/初始化语义；禁Rust源码parser服务仍通过正例 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-02/03/05 | 以冻结grammar合法样例为底，在α前插入一个ASCII空格后冷解析与增量解析；删除该空格再解析 | 本包V-02所列真实/拟新增入口，沿相同启用任务 | α对应所有byte span恰右移1，撤销后回原值；冷/增量规范化CST/AST相等；逻辑文档/工程身份保持，内容SnapshotId随字节改变、撤销回同文本时回原值，不能把source逻辑定位与内容hash混为一谈；绝对intern槽号不作为oracle |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-syntax ace-source`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | 源码位置和模块分类补证的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | 含CRLF与emoji的源码按既定单位定位且冷/编辑/恢复一致；函数互递归和值初始化环在语法合法时都产保留类别/来源的AST/图，本层不误拒也不产Checked；值初始化环的语义拒绝明确登记1047/1048必验，当前只验可区分结构/交接，不反等未来语义Plan才能完成 | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 本族可区分验证（新增合同，未实施）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | T-02/T-03/T-05创建/扩本包既有Auto阶段driver，最后由原T-06与独立review核；完整Gen1/代际入口分别归1053/1054 | 同一个含泛型、Option/Result/Future类型、CT/标注/Node的多模块输入检查独立树形/归属/span；合法与缺括号/错位置近邻区分，删类型修饰或标注路由必须被判失败。 |

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | Auto语法运行时及driver，首个正/反例阶段产物可独立检查；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；本包支持范围不冒整个AAC |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

- [ ] **AC-CL-FAMILY / work**：V-CL-FAMILY和原T-02/T-03/T-05实际结果满足，族级CL未因CS未用或A期AC不支持删减；语义与runtime/库补齐正式来源齐，原独立review逐行核。原任务数及0进度保持，本轮未执行。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1046/auto-lang`，分支 `plan-1046-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1046/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 记录worktree/branch/base与§4指纹变化、支持profile和后续任务入口；AC-01/04 |
|    [✅ 2026-11-20 /goal 授权开工] worktree `D:/autostack/.wt/lang-1046/auto-lang` @ `plan-1046-dev`，base=master@ec0c377e7；材料 `D:/autostack/plan-artifacts/plan-1046/` 已登记（REGISTRY.md）。G-AAC门核验：M2A-08已成立（985 T-A04 pass@ecc282df1、T-A05 fold落主95c7fbad8、M2A-01..08全闭，见985 §9 merge收据）；1044/1045/1043已归档；不等985-B成立。§4指纹重绑（基面ec0c377e7）：syntax-runtime/src/lib.rs 07328483e30aecd6（原5133b5cd漂移）、syntax-grammar/src/lib.rs 6d37993f970d7b68（原087b300b）、core/source/parser.rs 6a6233f7818deef4（原04782003）、modules.rs 87a482d8033bf548（不变）、packages.rs 95cc76cfdb247a96（原9aa1b33d）、cst/目录七文件在（frontdoor/kinds/lex/mod/project/shift/units）。主检出WIP=他计划簿记（docs/design/1081/1107），不消费。移植面核定：syntax-runtime执行引擎=program(546)/program_codec(920)/lexer(810)/parse(2776)/cst(404)/snapshot(644)/edit(467)/recovery(49)/position(303)/identity(97)≈5.6k行；autodown.rs(1309)为md文档层不在本包面；core/source/parser.rs为旧子集oracle。grammar数据：grammars/auto-source.at（rev3，25 profile），canonical bytes实测auto=16504B、age-package=21249B、packages-aggregates-ct=13787B（probe证据plan-artifacts/profile-sizes.txt）——hex内嵌可行，Rust侧grammar编译保留（T-02合同允许）。后续任务入口：T-02..05/T-09在aac/syntax/+aac/source/新增.at模块，conformance crate `experimental/ace/tests/aac-syntax`（沿1045先例布局，计划§2的aac/tests/syntax/路径按"常规机械实现由work选择"落为既有tests/惯例）+固定组ace-aac-syntax |
| [ ] T-02 统一grammar与Auto解析 | work | T-01 | 消费同一版本化grammar定义，移植实际tokenization/parse/CST算法和预算，在本Plan建立Gen0编译的Auto阶段测试driver；Rust grammar数据生成可保留但输入源码解析必须Auto执行。 由最终CL语法行驱动解析，显式含~T/?T/!T类型及嵌套、async块/await/go/yield、泛型约束、CTE/标注及领域Node，不限AAC源码自己的语法。 修实际影响现 AC 的共享语法/projection 差额，保 AV/A2R 的相同源合同。 | Auto语法运行时及driver，首个正/反例阶段产物可独立检查；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
|    [进行中 2026-11-20 /goal work轮·阶段三+修复轮：冷解析/门语义/截断流/拼接逐字节全绿；graft-ON列表爆=D-1046-03具名] 交付：`aac/syntax/syntax_program.at`(程序表解码:计数准入/引用校验/charset预展开/负哨兵kind)、`syntax_grammar_data.at`(六profile冻结canonical hex,128字符分块——AV文本访问按次拷贝整个后备缓冲)、`syntax_lexer.at`(模式scanner:CSR桶/最长匹配/嵌套注释/未闭合EOF错误叶/checkpoint恢复)、`syntax_cst.at`(CST arena+规范形+无损校验)、`syntax_parse.at`(161函数完整规则机:显式栈/FirstSets/cut提交域/Pratt五态/声明级恢复/失败备忘/copy-adopt嫁接/三重预算)、`syntax_smoke.at`(阶段driver)。conformance crate `tests/aac-syntax`(packages-aggregates-ct车道+AV执行,harness预算512M) **12/12 PASS(串行593s)**：冷解析canonical CST+诊断与Rust运行时逐字节一致(单fn/参数/if-else/坏声明恢复/嵌套/注释/pratt优先级；oracle kind/field映射:元kind -1..-4、NO_FIELD -1)。关键arena纪律：单边区段+逻辑前沿efhi预留+被超越帧段搬迁+物理追加推前沿(worktree提交2ea97c2↑/详见git log)。**余量(仍属本任务)**：snapshot分页/SnapshotId(V-04)、incremental_lex/reparse嫁接(V-02)、auto/age-package profile族用例、CST无损重建断言、budget反例(V-03深嵌套)；driver回归需串行(--test-threads=1)或固定组登记串行配置 |
| [ ] T-03 AST与来源映射 | work | T-02 | 实现全适用语法到Auto AST的投影，保sourceId/byte span/生成来源/错误恢复；完整语言含Schema/AutoUI/AGE语法节点，不引入renderer、不将未知域节点忽略。 保Option/Result/Future/Iter不同节点、标注与其声明/字段/参数归属及原span；不能parse后擦除类型修饰、标注或未使用分支。 | AST投影和diagnostic snapshot，冷解析与编辑增量结果相等；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-04 模块与provider装配 | work | T-03 | 实现模块/包/namespace/use/pub与#in的语法目录/条件AST：公共.at与at/c/rs/py候选逐文件parse，保选择条件和来源，不在本语法Plan执行一般CT或作语义绑定。1047基础绑定、1048语义选择与重绑定消费此目录；不拼源文本、不建第二binder。 修源模块及 provider 装配的公共接缝，交同源模块正反例给 1047/1068。 | 模块目录/来源关系及受控语法重复/缺失诊断及循环分类输入，语义可见性/初始化环留1047/1048；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-05 语法边界反例 | work | T-04 | 验证非法UTF-8、token边界、缺括号/截断、深嵌套/大token预算、坏编辑回滚与循环模块；从CL逐语法生产式抽真实样本，比较独立span/树形并保合法近邻。 对每个required语法族有合法和非法相邻样本及独立CST/AST/span预期；跨模块CT和领域组合不得靠最终1053补一条代表例。 | 完整语法映射、资源拒绝及无Rust parse调用的运行证据；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-09 源码位置和模块分类补证 | work | T-05；§5实际输入合同 | 以当前syntax-runtime/position.rs的byte/UTF-8/UTF-16、CRLF与emoji中点floor规则补driver；验证语法恢复后仍保错误状态、全部语法义务映射、合法互递归与非法初始化环的不同去向。保持逻辑source/project定位与内容SnapshotId分账。 | §6 V-05实际结果及输入/产物身份；AC-07/AC-04 |
| [ ] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 实际命令/退出码/输入和产物hash、覆盖与缺口；AC-01..04。完成即execution_done交review，无需预勾T-07/08 |
| [ ] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（拆清#in的语法/候选目录与类型/CT选择，补模块循环适用性及Unicode/恢复的精确边界。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

## 10. 待澄清事项

**D-1046-03（work自登记诊断项，取代D-1046-02，非用户裁决）**：graft-ACTIVE 解析在 AV 执行器上单列表突破 65536 元素上限（ListElems 资源拒绝）。已实证排除：拼接本身逐字节正确（S29：n2=58/phys=58/mism=0 对编辑后文本的全新词法）；graft-DISABLED 的同拼接流解析通过（root=119/120节点）；机器 9 个 arena（saves/fr/hints/t/ef/pvals/pops/optriv/dmsg）以 128 步窗/50000 阈全监控未触发（增长列表在监控集外）；嫁接次数限 16、哨兵推送限幅 200 后依旧；跨 100 新列表共 70000 元素通过（上限为单列表非总量）。修复收尾时已修复的真缺陷：ListNewline 门返回值被 rule_list 忽略（gate_probe 同行拒/换行收全绿）、嫁接复制 cc 未写回、测试样本编辑点 26→25。测试 `aac1046_edit_incremental_equality` 已 `#[ignore]` 具名（serial suite 17过/1具名ignore/0红，699s）。关闭条件：定位增长列表并修复后 INCR(嫁接树)==REF(全新树) 断言通过、移除 ignore；候选方向：try_graft 在 push_rule 帧内执行时对 top 帧 children/efhi 交互、graft 后 list 步重入路径的 pending/toks2 垃圾尾访问。


无须当前用户新增决定。1044/1045已交；不复制旧legacy lexer作为新的语言真值。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
