---
plan_id: PLAN-1054
status: executing
feature_name: AAC同源代际自编译与独立认证发行
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-24
plan_revision: 9
current_step: 1
total_steps: 10
depends_on: [PLAN-1053]
supersedes_spec_components: [docs/specs/auto-ac/project.md]
new_spec_components: [docs/specs/auto-ac/aac-certification.md]
touched_goals: [GOAL-002]
---

# [PLAN-1054] AAC同源代际自编译与独立认证发行

## 0. 变更摘要

### 2026-11-24 · T-10 遗留面收口：S 重新可编、Gen1 重新出具（r9 执行簿记，非规划变更）

work 会话（2026-11-23/24）在 r9 授权范围内继续 T-10：把 r54-56 批量分页转换的遗留面收干净，
**S（86 模块/1,619,040B）已由 Gen0 前端完整编译**，Gen1 重新链接并 boot（退出码 681978131 与旧 Gen1 相等），
`ace-aac-gen1` 9/9、`ace-aac` 26/26、`ace-aac-host` 22/22、`ace-aac-incremental` 20/20 全绿（新 Gen1 身份）。
期间实测定案：AV executor 对嵌套 list 元素访问按整个嵌套值计步，故**宿主/支撑/帧车道回退 flat**
（1045 宿主面、1056 codec/driver），页式存储只留在编译器前端与阶段 driver IO；
`paged_read_file/paged_write_file` 拆到 `aac/support/paged_file.at` 使纯存储模块无 host 依赖。
T-10 ④ 边界负控维持既有结论；⑤ 仍余 V-G1c（driver 无 std 包多包装载）与 V-G1d（gen-next 全链，
冻结墙具名 pending：模块准入 256KiB/语义步数/ABC 每函数表项/G2 mn 16MiB）。本轮不预勾 AC-G1。

### 2026-11-23 r9 · 用户裁定吸收 G1：新增 T-10 自举前端分页（范围扩批）

用户 2026-11-23 显式裁定选 (1)：把 T-02 实测判定的具名阻塞 G1（自举 1047 语义前端在 ≥~59KiB 输入上触发冻结合同 `MAX_LIST_ELEMS=65536`/`av.resource.list-elems`，平面 src/token/行存储超限；>64KiB 文件连装载都失败）**吸收为本 Plan 扩范围任务 T-10**（合同保持：MAX_LIST_ELEMS 不动、1046/1047 模块 pub 签名不动；根因/修复设计/验收见 §5 T-10 节与 §6 V-G1、§7 AC-G1，源头证据包 plan-artifacts/plan-1054/g1-frontend-paging-handoff.md）。G2（mn 16MiB verify 墙，64MiB tier 归 AGE elem-view owner）维持具名 pending 不吸收。T-02 全树 S 编译段依赖 T-10 完成后重跑 gen-next 续行；其余任务/AC/V/原义务保持。原 T-02 行内"G1"具名阻塞文本保留为历史，关闭责任移交 T-10。

### 2026-11-19 r8 · 接收AV-ONLY阶段与非AV检查恢复

用户已明确1078–1096只验AV，公共语料中AC/A2X等检查先以检查级#ignore暂缓。其未实现/未运行/失败不阻AV计划，不充PASS；本包进入对应后端开发/验证阶段后，逐check接手、移除本阶段标记并真实执行。共享源码、原oracle/条件/owner和本包完整验收门保持；不得要求AV计划先交非AVPASS。本轮仅修订规划。

### 2026-11-19 r7 · 公共语言语料按特性共源（仅规划）

按用户要求接[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)与1078的`IF-ACE-FEATURE-CORPUS/1`。Gen1/Gen2/Gen3对同一canonical CL源码分别编译运行，Case主体保持一份，各代工具/产物/错误/资源结果独立。CS自举源码和代际专项保原位置；不为各代再复制公共语言Case，不将共源数据视为共享PASS。AAC继续独立验收、非M2硬门。 原任务ID/数量/进度与已有证据边界保持，本轮不施工、不迁实际测试。

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包按原任务收实际生产任务的证据，安排已验阶段交接。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：收紧同源/跨代证据和可复现判据；明确本包可以签AAC而不是套用前序“不冒整个AAC”的模糊验收。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。以同源Gen1→Gen2→Gen3实际native链完成AAC独立验收，记录复现与依赖边界、仓外使用和资源/失败表现，不强加为M2发布门。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

以同源Gen1→Gen2→Gen3实际native链完成AAC独立验收，记录复现与依赖边界、仓外使用和资源/失败表现，不强加为M2发布门。 本包出口见AC-01..06。依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

已核实影响路径见§4；拟新增：**experimental/ace/aac/tests/bootstrap/、docs/specs/auto-ac/aac-certification.md（未来交付），包及原始证据仅在plan-1054材料区**。常规机械实现由work选择，不在未做试验前声称接口已存在。共享grammar/HIR/MIR/ABI/固定组/SDK文件由真实owner单写，私有AAC模块才可并行。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户本轮要求正式设计、需求审计及拆计划，明确先不施工；未来代码/试验需实施授权并满足G-AAC。没有用户指定预算/自动续行额度。当前源码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19`；本表指纹为编计划时文件实读值，目录仅核存在，开工重新绑定已合入M2-A source/SDK/stdlib/ABI/grammar/runtime与测试输入。

| 实际Spec/代码来源（仓根相对路径） | 内容SHA256前16位/事实 |
|---|---|
| `docs/design/59-aac-native-bootstrap-and-compiler-subject.md` | e449ff8f6d237981… |
| `docs/design/strategy/aac-delivery-roadmap.md` | 9dd084389bc142e3… |
| `docs/specs/auto-ac/artifact-contract.md` | ba5323ec912ba86d… |
| `scripts/test-groups.toml` | 1b432bb63abff7fe… |

规范输入：`docs/specs/auto-ac/project.md`，总览 `docs/specs/overview.md` 时间较早；具体模块代码/已交Spec核当前事实。Design59 §2/3是本轮静态审计，旧743/旧auto目录和已归档有限能力都不证明完整AAC就绪。frontmatter的GOAL-002仅关联ACE执行链成熟度，旧GOAL-017已经达成的是旧AAVM路线，不重开或据它认领AAC。

**外部依赖与跨仓owner**：1053真实Gen1/完整CL/主体收据、同一S、Gen0与服务/stdlib/ABI/grammar/runtime/linker冻结身份；原测试owner入口。Linux/其他host扩面具名后继，不自动算PASS。

**当前开工状态**：1053独立复审/正式交付，实际Gen1可用；代际源码S及其依赖冻结，不借旧AAVM/AA2R代际记录复用AAC证书。 公共G-AAC须985/T-A04独立复核、T-A05已合入A收据/M2A-01..08；不等985-B、984全产品、986/987整包。frontmatter只列AAC包级最终依赖，任务级准备以§5为准，不用整包串行制造环。

## 5. 详细设计

### AV-ONLY接收与本目标启用

1078的IF-ACE-FEATURE-CORPUS/1和1096的AV语言证书允许携带非AV#ignore/NOT_RUN行，接收时只核AV声明和迁移义务未丢，不以非AVpending回卡AV独立出口。本包实际验证自己的后端/代际时，由原生产任务逐check去掉AV阶段#ignore，保原平台/ignore与oracle，按当前同源/工具/SDK身份运行；失败属于本目标任务，必须修到本目标required齐才签其完整出口。1068/985的最终多端门及AAC独立门不接受忽略充PASS；AV阶段通过从不冒多端通过。

规范影响：公共作者/引用/选择/迁移协议由1078统一在拟`docs/specs/auto-test-case/feature-organized-corpus.md`沉淀；本消费者只在自身原SD/模块Spec记录实际Case引用、目标动作和正式证据，不再各写一套公共协议。AC-FC属于本包验收增量，原SD与目标义务全部保留。

### FC · canonical语言输入与旧义务承接

Gen1/Gen2/Gen3对同一canonical CL源码分别编译运行，Case主体保持一份，各代工具/产物/错误/资源结果独立。CS自举源码和代际专项保原位置；不为各代再复制公共语言Case，不将共源数据视为共享PASS。AAC继续独立验收、非M2硬门。

由原T-01/T-03/T-05/T-06/T-09按其职责消费/迁移对应行，保原source/expected/check及条件，重复仅在完整输入闭包/语义等价时合并，原义务→canonical Case/check映射不得丢；既有固定组/suite/worker读者按1078接口切换，不新建按后端/代际维护的Auto副本。缺字段/目标接口由1078和实际producer在原框架有界补接，不要求本包先等全AV/全AAC完成。

### r2 任务交接与验收纪律

本次授权为再次验证和细化规划，仍不施工。原r1代码观察指纹只作历史取材；r2静态审计观察主线 `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`，各仓Spec/代码/工作区规划可能继续变化，开工绑定实际正式M2-A收据及各输入内容，不把WIP或历史指纹当稳定签章。

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。§8的新增T-09/T-10若存在，属于work且排在T-06之前；总进度仍0，不预勾review/merge。AC-01覆盖原T-02..05全部输出，新增AC-07/08覆盖细化任务；AC-04要求本包**全部V行**及适用输出齐全，required未跑/缺口不能以“pending已写明”签本包通过。只有明确不属本阶段的后续整机/其他host义务可保具名去向。

每代可执行文件hash是测试输入身份，Gen1/2/3不同二进制的执行结果不能互相复用；相同代同输入未变证据可复用。G2/G3比较在固定source/stdlib/ABI/grammar/backend/runtime/link及确定性选项下进行，非确定字段须在对比前具名且局限metadata；MIR/机器指令、常量/重定位语义与程序结果差异不能被整段屏蔽或用“二进制不必相等”豁免。标准化仅限已说明的路径/时间戳/局部ID映射且引用关系完整。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### T-10 自举前端分页（r9 吸收 G1，用户 2026-11-23 裁定）

**根因（2026-11-22 实测终判）**：冻结合同 `av-core::contract::MAX_LIST_ELEMS=65536`（`av.resource.list-elems`，AV/AC/A2R 三后端共守，conformance 资源测试断言）下，自举 1047 语义前端以平面 `List<int>` 承载源字节（driver `src`）、token（syntax_lexer）、CST/AST 行（syntax_cst/source_ast/syntax_program/semantic_tree），三者各自独立触顶：70,036B 纯注释文件（行极少）=源字节 list 触顶（>64KiB 连装载都失败）；59,546B 密集代码（bytes<64K）=token/行 list 触顶；atom_data.at（66,236B，S 唯一超限模块）确定性 trap（backtrace 实测 push len=cap=max=65536）；Gen0 packages 车道（Rust 前端）编同文件正常（Gen1 boot 含 atom_data 即证据）——缺口专属自举前端规模性。附带矛盾：semantic_driver.at `admit_document` 准入 256KiB 与运行时 64KiB 合同不自洽，须一并收敛。源头证据包：plan-artifacts/plan-1054/g1-frontend-paging-handoff.md（dbg exe/delta-debug/合成探针全索引）。

**合同保持（红线）**：MAX_LIST_ELEMS 与 a2x-rust coll.rs 上限不动；既有报告格式与阶段 oracle 不动；既有固定组原 oracle/条件/owner 保持；不分流语言语义（List 仍是语言类型，只是前端内部存储分页）。模块间 pub 签名允许随分页类型（`List<int>`→页式 record）在**本 Plan 内联动调整**（S 全体一起重绑身份；Gen0 重备 Gen1 后链续行）——语义与报告行为零变化可原组回归证明。

**修复设计**：①行页表——syntax_cst/source_ast/syntax_program/semantic_tree 的行表改页表（每页 ≤32K 行，全局 row id = page<<15|index；stride/角色数学不变，仅 row 访问经页间接层）；②token 分页——syntax_lexer.at 的 toks 输出改页式游标（消费方 lexer 接口签名不变处尽量保，必要时有界扩签名并同步 1047 driver 与 1046 组）；③driver 分段装载——semantic_driver.at 的 src/ast/bases/roots 改分段 append + 页式累积，project 模式逐模块流式化；④准入收敛——T-10 中段先把 admit_document 收到 64KiB-1 使准入=合同（负控：65,536B 边界 ±1 case），分页全落地后恢复 256KiB 并配 >64KiB golden case。

**验收**：§6 V-G1a..e；1046/1047/1048/1050 受影响固定组原样绿 + 必要 consumer（gen1 视图/case-aac-gen1-language-parity 不回归）；新增大输入 golden 进日常组（≤10s/例）。**关闭**：atom_data.at 与全树 S 经自举前端真编译（mir→ABC），gen-next 全链产出 Gen2 native PE 且 boot 退出码与 Gen1 相等（681978131）。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-certification.md` | 新增AAC独立同源代际/完整语义/包与复现验收，保持独立后续版本边界，绝不作为M2或AAV/AACE整体验收。 | 明确本包实际主体/信任/支持边界 | AC-01/02/03 |
| SD-02 | modify | `docs/specs/auto-ac/project.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-certification.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**每代完整CL与CS分别验证**，沿[Design59的CL族级映射](../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-03/T-09；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-certification.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

### V-FC-CONSUMER（work / 原T-01/T-03/T-05/T-06/T-09，未执行）

同一canonical source变化必须使本包实际目标/代际依赖失效并用独立oracle重验；单目标oracle变异、遗留旧活入口、删原义务映射、零选中/缺required检查和emit冒run分别判红。公共语言输入去重不减少旧资源/错误/恢复/平台条件；typed IR/后端专项及外仓完整产品保原owner。实际结果按Case＋check＋编译器/目标/动作＋source/SDK/provider身份记录，不以AV或其它代结果回填。

**AC 语法同源验收。** 完整代际认证逐代复用同一版本 AutoDown 语料与独立预期，分别记录当前代的 AC 实际结果。现有用例可以先复跑，完整范围按 1096 交付后的语料对账。 按 [AV AutoDown 共用语料规则](../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

不变量：本包输出满足§1/2及其阶段身份/资源/所有权，合法近邻与非法/故障输入能区分；不是仅验证CLI exit0。未来测试以真实冻结的最终M2 required语法/IR/API构造，表内输入是语义案例定义，尚非已运行fixture。创建时应固定独立字面结果/诊断stage/span/事件/资源计数或文件hash，禁止把被测输出抄作expected。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 真实/拟新增入口及使其可运行的任务 | 独立可观察预期 | 覆盖边界 |
|---|---|---|---|---|---|
| V-01 | AC-01 / work；T-02..05按本行入口负责 | Gen1+S→Gen2、Gen2+同一S→Gen3，并各代完整CL/CS | 实际代际构建入口+1053 TestCase入口（T-02/03接通） | native代际来源精确、独立oracle全绿、无Rust源语义调用；分平台支持范围如实 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-02 | AC-02 / work；T-02..05按本行入口负责 | 替换S、漏一代/CL行或给出旧AAVM证据 | 独立AAC父门（T-05创建） | 整体拒绝，不能因各子报告exit0通过；合法完整身份链先产候选，真实T-07 review和落主身份齐后才可签AAC | 仅本包声明profile；全CL/代际完成以1053/1054为准 |
| V-03 | AC-03 / work；T-02..05按本行入口负责 | 换安装位置/去开发树/缺声明helper、不同路径重建 | 实际仓外包与复现入口（T-04） | 完整包真编真实程序；缺依赖具名错误；可控固定点与非确定字段报告一致 | 仅本包声明profile；全CL/代际完成以1053/1054为准 |

### 补充独立判据 V-04

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口 | 确切预期与边界 |
|---|---|---|---|---|
| V-04 | AC-01/02/03；work / T-02/03/05 | 固定源码树S的digest h；把Gen2收据source hash改为另一个值，其余子项保绿；另省Gen3链条 | 本包V-01/02所列真实/拟新增入口，沿相同启用任务 | 两种控制均整体拒绝；只有实际Gen1→Gen2→Gen3都绑定同一h且全CL/CS实际结果、真实review/落主身份齐才签AAC，M2阶段报告不变化 |

V-04是必验补充项，由对应work任务落为真实fixture；样例中的Int/Bool/f/id/CT写法为语义描述，实际拼写沿冻结的本版目标grammar/测试API，不另造语法。该判据和§8各批输出一同属于AC-01..04，不能因V-01..03绿就遗漏。资源/诊断/事件由各阶段真实生产点观察，不能用wrapper自报代替。

本包当前已存在的候选固定组：`cargo t ace-native ace-abi ace-artifacts`；work按真实改动选全部职责组及必要消费者，未触面不机械扩跑。模块常规集合排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不作每包日常门。新增AAC执行选择由1053在既有固定组机制登记，本包阶段driver的实际调用由T-02..05交付后写收据，不能现在报告存在新命令。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | 独立签章的身份和确定性反控的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | Gen2报告不能充Gen3结果；故意改常量或指令不能按metadata忽略；完整Gen2/3有界确定性与CL/CS、包、主体证据齐才允许签本Windows AAC，不签M2或AAV/AACE整体 | 本包阶段，未实施；最终整机/代际去向1053/1054 |

### V-G1 前端分页验收（r9 新增，work / T-10）

| Case ID | AC / owner | 输入/状态 | 入口 | 独立预期与边界 |
|---|---|---|---|---|
| V-G1a | AC-G1 / work | >64KiB 纯注释+单 fn（行极少，源字节 list 考验） | 自举 1047 driver（重备后） | 真编译通过（装载不触顶）；前 Trap=ListIndex |
| V-G1b | AC-G1 / work | 59,546B 密集代码（bytes<64K、token/行 list 考验） | 同上 | 真编译通过；前 Trap=ListIndex |
| V-G1c | AC-G1 / work | atom_data.at 全量 66,236B（S 唯一超限模块） | 同上 mir 模式 | 报告头自洽+ABC 产出 |
| V-G1d | AC-G1 / work | 全树 S project 模式 64 模块 | gen-next 链（Gen1 工具集） | ABC→bridge→COFF→link→Gen2 PE；boot 退出码=681978131=Gen1；父子产物 hash 不同 |
| V-G1e | AC-G1 / work | 合同负控 | 既有 conformance 资源测试 | `av.resource.list-elems` 仍断言 65536——合同未被暗中放宽；driver 65,536B 边界 ±1 准入 case 判别 |

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 本族可区分验证（新增合同，未实施）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | T-03/T-09创建/扩本包既有Auto阶段driver，最后由原T-06与独立review核；完整Gen1/代际入口分别归1053/1054 | Gen1/2/3各运行含自举源码未用特性的完整CL；缺当前代行、改runtime/source/grammar或用前代报告替代必须阻独立签章，不能只验证S→S成功。 |

## 7. 验收标准

- [ ] **AC-FC｜owner_stage: work**：V-FC-CONSUMER及§5 FC逐原义务满足，公共语言唯一作者/本目标真实动作与独立结果/迁移映射均可追溯；原独立review复核，merge只接真实落主来源。未交required目标保pending，不因共享目录、少文件或另一端通过而降门。

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | 真实Gen2/Gen3及身份链，不仅文件名或报告声明；同时§8所有work任务的输出都必须交付，不仅首批。 用§6 V-01及各work输出逐项核对，不用“文件已建”代行为 |
| AC-02 | work | §6 V-02实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03实际通过/拒绝符合其oracle，Rust保留范围/默认库依赖/可信边界无隐藏回落；work阶段只产本基线完整候选；T-07独立review和T-08落主身份/正式签章后才声明首版Windows AAC独立完成，不冒M2或AAV/AACE全体完成 |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅T-07真实独立review通过且落主源码/工具/包身份与受审输入一致（或受影响重验）后，T-08发布正式Windows AAC独立验收结论/证书，沉淀已验证Spec、归档/清理；不给M2增加AAC硬门 |
| AC-07 | work | T-09完整交付且V-05的合法/非法或失败近邻实际得到该行独立预期；不以文档/命名存在代行为 |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

- [ ] **AC-CL-FAMILY / work**：V-CL-FAMILY和原T-03/T-09实际结果满足，族级CL未因CS未用或A期AC不支持删减；语义与runtime/库补齐正式来源齐，原独立review逐行核。原任务数及0进度保持，本轮未执行。

- [ ] **AC-G1 / work**：V-G1a..e 全闭——自举 1047 前端真编译 >64KiB 输入、atom_data.at 与全树 S（project 模式）；1046/1047/1048/1050 受影响固定组原样绿+必要 consumer；合同负控（`av.resource.list-elems`=65536 仍断言、driver 准入与合同一致）如实通过；MAX_LIST_ELEMS 与模块 pub 签名零变化可 diff 证明。由原 T-10 落实，原独立 review 核；不提前计完成。

## 8. 执行步骤

拟专树 `D:/autostack/.wt/lang-1054/auto-lang`，分支 `plan-1054-dev`，一Plan一树沿work/review/merge顺序使用；当前未建立。计划材料唯一 `D:/autostack/plan-artifacts/plan-1054/`，开工登记owner/用途/状态，测试生成输出/SDK/仓外消费及target显式放该scope，禁止junction/symlink。仅永久实现/用例源码将来进实现树。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 记录worktree/branch/base与§4指纹变化、支持profile和后续任务入口；AC-01/04。✅ 2026-11-22 r8：开工门闭合——985/A T-A04 phase pass@ecc282df1＋T-A05 fold，M2A-08落主95c7fbad8（985 §9）；1053 archived（Gen1实链：experimental/ace/tests/aac-gen1，aac1053-gen1.exe boot＋aac1053-compile.exe A2R编译入口均已交付，compile阻塞已由a2x-rust lower_coll修复关闭）；1044–1052/1055/1056/1057–1060/1062/1066/1078/1094/1096 archived；1051/1067 archived。专树 D:/autostack/.wt/lang-1054/auto-lang @ plan-1054-dev，base c3e5a4fb2（含1053落主）；材料区 D:/autostack/plan-artifacts/plan-1054/ 已登记（.retention/policy.json scopes，in-flight）。§4指纹实读（c3e5a4fb2）：Design59=d13a42c8056bac9f（变）、roadmap=63cdd3d65c5cba2f（变）、artifact-contract=ba5323ec912ba86d（不变）、test-groups.toml=a75f0e2a9ab24af7（变，1053 gen1视图新增所致）；docs/specs/auto-ac/aac-certification.md 尚不存在（SD-01未来交付）。后续入口：prepare-gen1/gen1-worker（AAC1053_GEN1_DIR）、固定组 case-aac-gen1-language-parity、cargo t ace-aac-gen1；机器占用核查0进程 |
| [ ] T-10 自举前端分页（r9 吸收 G1） | work | T-01 | 合同保持修复：行存储分页——**缝策略（2026-11-23 侦察定案）**：行存取已集中于属主访问器族（syntax_cst nk/nstart/nlen/nfc/ncc/nexpected/nflags/ndecl；semantic_tree kind/row_count；lexer tok_*；driver 装载点），分页隐藏进访问器内部；新页式类型（record：chunks List<List<int>>+缓存 len，页 ≤32K 行）由新增 aac/support/paged.at 承载；直接点访问（实测 ~154 处核心+长尾）逐个改经访问器；syntax_lexer toks 分页；semantic_driver 侧 src/ast 分段装载+页式累积；admit_document 上限收敛（先收 64KiB-1 并配边界±1 负控，分页全落地后恢复 256KiB 并配大输入 golden）；**不动** MAX_LIST_ELEMS=65536/a2x-rust cap/AV 合同；模块 pub 签名随分页类型本 Plan 内联动调整（S 重绑身份，Gen0 重备 Gen1 后续行）。设计全文 §5 T-10 节。 | §6 V-G1a..e 全闭；1046/1047/1048/1050 受影响固定组原样绿+必要 consumer；新大输入 golden case 进日常组（≤10s/例）；AC-G1。证据折入本行。✅ 部分（2026-11-24 r72，commit d6c89f2eb）：**S 重新可编 + Gen1 重出**。r68 把 paged 文件 IO 拆到新 `aac/support/paged_file.at`（纯存储模块无 host 依赖，AV 装配面可再装）；r69 修完 6 处批量转换遗留（bridge client/invoke、host_diag、driver/boot 的 flat↔页式边界），Gen0 全树诊断 `S-COMPILE OK modules=86 bytes=1619040`；r72 依 AV 实测把**宿主/支撑/帧车道回退 flat**（根因：AV executor 对"元素是 list"的读写按整个嵌套值计步，`paged_push`/`paged_get` 在 `List<List<int>>` 上是 O(存储元素数) —— v03_frame_bytes_match_reference 在 64M/256M 步均失败），S 仍 OK。Gen1 重出（`gen1-r72`，prepare 285s）：`aac1053-gen1.exe` 704,000B，manifest fingerprint `ddaa3a31…`/exe `2fe37e2c…`/runtime `4b761dea…`，boot 退出码 681978131（=旧 Gen1）；固定组 `ace-aac-gen1` 9/9、`ace-aac` 26/26、`ace-aac-host` 22/22、`ace-aac-incremental` 20/20。余：V-G1c（1047 driver 无 std 包多包装载）、V-G1d（gen-next 全链；冻结墙具名 pending） |
| [ ] T-02 同源代际链 | work | T-01；全树 S 编译段依赖 T-10（r9） | Gen1编同一S得到Gen2，Gen2编同一S得到Gen3；记录实际父子编译者/产物hash/命令/退出码/目标，禁止中途偷偷换源码或调用Gen0源编译fallback。 | 真实Gen2/Gen3及身份链，不仅文件名或报告声明；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03。部分交付（2026-11-22，commit 1fa69ddc7）：代际构建驱动 `gen-next`（父代工具集 manifest 准入→全树 S project 编译→ABC→1051 bridge→COFF→link→子代 PE→boot 运行→身份 manifest）+ `probe-native-compile`（Gen0 窗 mn 车道判定探针）已交付并提交。实测判定两处具名阻塞（均有完整证据链，plan-artifacts/plan-1054/t02-generational-probe-design.md）：**G1**=自举 1047 语义前端在 ≥~54KiB 输入上触冻结 AV 合同列表上限 `MAX_LIST_ELEMS=65536`（`av.resource.list-elems`，av-core contract；driver 自身准入却允 256KiB，矛盾属实）——atom_data.at（66,236B，S 唯一超限模块）确定性 `trap:ListIndex`（a2x_runtime push 上限，带 backtrace 实测 len=65536/cap=65536；Gen0 构建的 aac1047-driver 同样复现=非 Gen1 构建问题；Gen0 packages 车道编同文件正常）；且 driver 全树平面累积模型在 64 模块下远超 64K，"Gen1 经 1047 driver 编全树 S"当前架构不可行。**G2**=mn 车道（MnDriver 实测，assemble 80.6s）拒lower全树 S+driver 产物（`d_p3_appm15_core_atom_bound_bind_body` ownership 分析 ~34,225,664B > 冻结 16,777,216B）；同 1053 §10 具名缺口（64MiB tier 归 AGE elem-view owner）。诚实边界：ABC→1051 bridge 服务路径对"编译驱动产物"的准入未实测（被 G1 挡在前端阶段，无法产出其 ABC）；G2 在 bridge 路径上的真实状态随 G1 解除后首测。native 代际编译工具的另一候选形态（A2R 车道）无 Auto A2R emitter，不可行。G1 根因细化（2026-11-22 三）：双 list 独立触顶实测——70KB 纯注释文件（行极少）trap=源字节 list；59,546B 密集代码（bytes<64K）trap=token/行 list；即 >64KiB 文件连装载都失败，前端平面存储距全树 S 规模差 1-2 个数量级。G1 已于 2026-11-23 r9 被用户裁定吸收为 T-10（本行原文保留为历史；关闭责任移交 T-10，完成后同输入重跑 gen-next 续行）；G2 关闭条件（具名 pending，不充 PASS）：AGE elem-view 交付后 64MiB tier 开放或 Design59 具名重设计；G2 解除后同输入重跑 gen-next 链条自动续行 |
| [ ] T-03 各代完整语义与故障 | work | T-02 | Gen1/2/3分别执行CS与完整CL适用测试、编译失败诊断、资源清理及冷/热/恢复/坏编辑修复；保独立oracle并检查每代服务/依赖身份。复用未变输入证据须身份精确。 各代都包含CS未用的~T/?T/!T、泛型/CTE/标注/领域与资源组合CL；当前代源码、库、grammar、runtime、实际编译者/产物逐行绑定，不能借上一代绿报告。 | 各代义务结果及实际错误阶段/资源轨迹；不能只跑自编译小子集；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-04 可复现与包验证 | work | T-03 | 控制时间戳/路径/随机种子/链接metadata等可复现输入，核语义IR/对象/输出和可控制范围字节固定点，任何剩余非确定字段具名解释。按978套路在材料区仓外安装/重定位AAC及显式runtime/helper，列host/target依赖。 | 复现边界和仓外真运行收据；不声称全工具链无libc或所有host已支持；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-05 独立总门与交付边界 | work | T-04 | 建立独立AAC验收汇总候选，缺代次/错源码/缺CL行/假Checked/未跑真实host/隐式Rust语义fallback都阻签；准备供T-07独立review核主体与全链的材料、支持范围/未交边界和后继清单。work只产unsigned候选，正式签章归T-08，在真实review与落主身份核准后进行。 | AAC独立验收unsigned候选及版本归属，不改M2分母、不切产品默认、不退役Rustreference；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [ ] T-09 独立签章的身份和确定性反控 | work | T-05；§5实际输入合同 | 核每代完整CL/CS及S的check/编译路径、无Rust源语义回落、真实包/host与producer身份。用前一代绿报告替代当前代、改source/stdlib/target/bridge、隐藏机器code差异、漏必需行/仅预览证书均要阻签；输出独立AAC支持范围及仅其他host等非本基线后继。 完整CL缺一族/一行、将A期AC有限面冒最终目标、runtime未交却named-reject充正例，均须阻AAC完整签章；实际三端M2证明仍由1068/985负责。 | §6 V-05实际结果及输入/产物身份；AC-07/AC-04 |
| [ ] T-06 定向验证和交review | work | T-02..05、T-09 | 运行§6命名正反例、实际影响固定完整组与必要consumer，核父子真实终态/身份/原oracle/资源/清理；摘要入本Plan并准备SD-01/02 | 实际命令/退出码/输入和产物hash、覆盖与缺口；AC-01..04。完成即execution_done交review，无需预勾T-07/08 |
| [ ] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量；核真实T-07 review及实际同源代际身份后发布正式AAC独立验收结论/证书，再归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

- 2026-11-24 stage: work | PLAN-1054 | r9 | outcome: pass（T-10 遗留面收口大额交付：S 86 模块 Gen0 全编、Gen1 重出 boot parity、四组全绿；非 T-10 关闭——V-G1c/V-G1d 未闭） | worktree_path: D:/autostack/.wt/lang-1054/auto-lang | branch: plan-1054-dev | base_commit: c3e5a4fb2 | code_commit: d6c89f2eb（r68 421733e03 / r69 6ef93ebaa / r72 d6c89f2eb） | task_ids: T-10（①②③ + 遗留面；④既有；⑤余 V-G1c/d） | evidence: g1-frontend-paging-handoff.md §"2026-11-23/24 续（r68–r72）"；关键数字：S-COMPILE OK modules=86 bytes=1619040 / Gen1 704,000B boot=681978131 / ace-aac 26+22+20、ace-aac-gen1 9/9；AV nested-list 计费实测（v03 frame 64M→256M 均超步） | blockers: 无外部阻塞——V-G1c 需 driver 的 std 包多包装载（计划内实现选择）；V-G1d 冻结墙（模块准入 256KiB、语义步数、ABC 每函数表项、G2 mn 16MiB）维持具名 pending | next_responsible_role: work（V-G1c→V-G1d→T-02 续行）

- 2026-11-23 stage: work | PLAN-1054 | r9 | outcome: pass（T-10 ①②进行中大额交付+中途检查点 848cb03ab；非绿态、非 T-10 关闭） | worktree_path: D:/autostack/.wt/lang-1054/auto-lang | branch: plan-1054-dev | base_commit: c3e5a4fb2 | code_commit: 848cb03ab | dependency_revisions: 1047 driver 预备 aac1047-prep-r9（Bind 全过） | task_ids: T-10（①闭合②进行中） | evidence: §5 T-10 节 + g1-frontend-paging-handoff.md（实施记录全链：Paged 基座/a2x-rust place_ref 臂/批转换/事故与修法/剩余 176 边界清单与确定性修法） | blockers: 无外部阻塞——剩余为确定性机械修；G2 仍具名 pending | next_responsible_role: work（T-10 ②边界统一→③④⑤）

- 2026-11-23 stage: work | PLAN-1054 | r9 | outcome: pass（T-10 ②③大额交付：Verify 全清零后 token/语义/mir 长尾全链分页落地，1046+1047 双 driver 编译零错；dense/bigcomment observe 全链绿，mir 探针实测中；非绿态、非 T-10 关闭） | worktree_path: D:/autostack/.wt/lang-1054/auto-lang | branch: plan-1054-dev | base_commit: c3e5a4fb2 | code_commit: <见 git log，r43-r56 共 7 个 wip 提交> | dependency_revisions: aac1047-prep-r9 + aac1046-prep-r9 双 driver（分页全链版） | task_ids: T-10（①②③闭合；④准入收敛与边界负控、⑤V-G1a..e 全链与受影响组、待续） | evidence: g1-frontend-paging-handoff.md 轮 43-57（7 错定位器/format_edit .len 真 bug/traplocate.sh 永久工具/paged_read_file 分块装载/paged_utf8_valid/KEEP 集合纪律/出生点与返回类型同转纪律/平面设计定界清单）；关键数字：176→7→0 Verify；dense(59,546B) 与 bigcomment(70,038B) observe 绿；A2X 计费实测线性大常数（探针窗 8G WU） | blockers: abc>60K 写出需 write_region_bytes std 增补（V-G1d 必触发，届时加）；corpus_text 门禁超时=master 同症环境账 | next_responsible_role: work（T-10 ④⑤→V-G1d gen-next）
- 2026-11-23 stage: new/revision | PLAN-1054 r8→r9 | outcome: pass（用户 2026-11-23 显式裁定吸收 G1 为 T-10 自举前端分页；范围扩批已授权并记录 §4/§0；G2 维持具名 pending） | next: work（T-10；T-02 全树 S 编译段候 T-10 完成后经 gen-next 重跑续行） | changed: 新增 T-10 任务行/§5 T-10 节/§6 V-G1a..e/§7 AC-G1；T-02 依赖列与 G1 关闭责任更新；total_steps 9→10

- 2026-11-22 stage: goal | plan_id: PLAN-1054 | plan_revision: r8 | round: 6 | from: work | to: work | status: executing | outcome: waiting（§10 裁定未答复；等待第 3 轮——阈值达，目标转 blocked 交还用户） | worktree: D:/autostack/.wt/lang-1054/auto-lang | commit: 1fa69ddc7 | next: user

- 2026-11-22 stage: goal | plan_id: PLAN-1054 | plan_revision: r8 | round: 5 | from: work | to: work | status: executing | outcome: waiting（§10 裁定未答复；等待第 2 轮） | worktree: D:/autostack/.wt/lang-1054/auto-lang | commit: 1fa69ddc7 | next: user

- 2026-11-22 stage: goal | plan_id: PLAN-1054 | plan_revision: r8 | round: 4 | from: work | to: work | status: executing | outcome: waiting（§10 裁定未答复；等待第 1 轮） | worktree: D:/autostack/.wt/lang-1054/auto-lang | commit: 1fa69ddc7 | next: user

- 2026-11-22 stage: goal | plan_id: PLAN-1054 | plan_revision: r8 | round: 3 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1054/auto-lang | commit: 1fa69ddc7 | next: work

- 2026-11-22 stage: work | PLAN-1054 | r8 | outcome: pass（T-02 部分交付+两处具名阻塞判定；非 T-02 关闭） | worktree_path: D:/autostack/.wt/lang-1054/auto-lang | branch: plan-1054-dev | base_commit: c3e5a4fb2 | code_commit: 1fa69ddc7 | dependency_revisions: 1053 工具集实备（prepare-gen1 双 exe 齐）；1047 driver 独立重备对照；ACE_AAC_COLLECTIONS_LIB runtime-release | task_ids: T-02（部分） | evidence: T-02 行内 + plan-artifacts/plan-1054/t02-generational-probe-design.md（P0-P2 全程：G1 根因=冻结 MAX_LIST_ELEMS=65536 合同上限 vs 自举前端平面累积（atom_data 66,236B 实测 push len=65536 backtrace；prefix-42/43 二分边界；driver 256KiB 准入矛盾）；G2 根因=冻结 16MiB verify 拒 34,225,664B（P2 实测 82.3s）；P1 链条工具 gen-next 本身就绪，输入侧被 G1 阻塞） | blockers: G1（前端行存储分页，owner=AAC 前端 1046/1047 后继具名任务）、G2（64MiB tier，owner=AGE elem-view/Design59）——关闭条件见 T-02 行；不充 PASS、不签链 | next_responsible_role: work（T-02 余量路由确认+T-03 准备面）

- stage: goal | plan_id: PLAN-1054 | plan_revision: r8 | round: 2 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1054/auto-lang | commit: 1fa69ddc7 | next: work

- 2026-11-22 stage: work | PLAN-1054 | r8 | outcome: pass（T-01 开工身份核对闭合；开工门就绪） | worktree_path: D:/autostack/.wt/lang-1054/auto-lang | branch: plan-1054-dev | base_commit: c3e5a4fb2 | code_commit: c3e5a4fb2（T-01零代码改动） | dependency_revisions: 985/A M2A-08@95c7fbad8；1053 archived@ff1599429系（reviewed 9550dacdf）；1044–1052/1055–1060/1062/1066/1067/1078/1094/1096 archived | task_ids: T-01 | evidence: T-01行内记录（§4指纹四件实读、gen1入口/固定组/机器占用0进程、材料区+policy登记） | blockers: 无——53个具名CL族缺口为985/A允许AC差额账，不卡T-02（§5规则） | next_responsible_role: work（T-02 同源代际链）

- stage: goal | plan_id: PLAN-1054 | plan_revision: r8 | round: 1 | from: work | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1054/auto-lang | commit: c3e5a4fb2 | next: work（T-02 Gen1+S→Gen2→Gen3 代际链）

- 2026-11-19 r8规划交接：明确AV路线只选AV，非AV#ignore由本目标实施阶段逐项恢复；原任务/状态/进度、完整门与原预期保持，本轮没有实现或运行。

- 2026-11-19 r7：`stage: new/revision`，`outcome: pass`（规划静态预检），`next: work`待后续指令/所需来源。新增FC/V-FC-CONSUMER/AC-FC，原T-01/T-03/T-05/T-06/T-09补接同源语言/迁移合同，任务/进度不变；未移动用例、修改expected、运行或合入。

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（收紧同源/跨代证据和可复现判据；明确本包可以签AAC而不是套用前序“不冒整个AAC”的模糊验收。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。

## 10. 待澄清事项

2026-11-23 裁定已决（用户选 (1)，r9 吸收 G1=T-10；本节原阻塞问题作废保留备查）：~~2026-11-22 T-02 阻塞态（须用户裁定二选一……~~ 当前无新增待澄清。G2（mn 16MiB verify 墙，64MiB tier 归 AGE elem-view owner）为具名外部 pending，非用户决定项；T-10 施工中的例行实现选择不反复请求确认。

无须当前用户新增决定（原 standing，2026-11-19 r8 前）：1053独立复审/正式交付，实际Gen1可用；代际源码S及其依赖冻结，不借旧AAVM/AA2R代际记录复用AAC证书。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
