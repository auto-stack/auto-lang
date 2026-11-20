---
plan_id: PLAN-1046
status: archived
completion_kind: delivered
feature_name: AAC Auto语法运行时、源码投影与模块装配
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-20
plan_revision: 8
current_step: 9
total_steps: 9
depends_on: [PLAN-1044, PLAN-1045]
supersedes_spec_components: [docs/specs/auto-ac/project.md, docs/specs/stdlib/design/context-source-assembly.md, docs/specs/auto-syntax/grammar-contract.md]
new_spec_components: [docs/specs/auto-ac/aac-source-frontend.md]
touched_goals: [GOAL-002]
---

# [PLAN-1046] AAC Auto语法运行时、源码投影与模块装配

## 0. 变更摘要

2026-11-20 r8。9/9 完成。代码与四份规范已快进落主，计划已归档，专树与本计划验证材料已清理。

2026-11-20 r8。最终复审通过，R1046-01/02 关闭。按冻结提交验证原始反例、受影响分支及规范增量，进入 T-08 合入。

2026-11-20 r8。复审发现已返修并同步 1086 基面。T-04/T-05/T-06 冻结提交通过源码候选、共享 grammar、Windows AC 及四组回归，进入 T-07 复审。

2026-11-20 r8 复审第 1 轮。1051 Windows 文件宿主验收已闭。AAC 两种符号表导入缺少目录关系，已具名返修，重开 T-04/T-05/T-06。

2026-11-20 r8。共享 grammar 的 AAC 组合 profile 已覆盖 V-CL-FAMILY。Windows AC 正式宿主和 A2R 均通过真实目录的正反例。格式串、位置和受影响固定组已验，T-06 完成，进度 7/9，进入独立复审。

2026-11-20 r7。D-1046-04 证实公开 ACE 前门在共享 grammar 前处理格式串，原始字节直接走 grammar 漏掉实际接受的语法。本修订在现有 T-05 补齐 Auto 前门与原始位置映射，T-09 验编辑/恢复位置，T-06 重验受影响的 T-02/T-03/T-04 证据。保留原任务、验收、资源与阶段授权，进度 4/9。

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包承接项已补入 §8 原执行任务，复用 §6 真执行和独立反控；正式交接须标明原 required 行、实际源与产物、身份和结果。

2026-11-16 r4本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r3当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r2当前：拆清#in的语法/候选目录与类型/CT选择，补模块循环适用性及Unicode/恢复的精确边界。 保全部原目标/义务及任务ID；新增T-09，现9任务，全部未执行。r1文字为初版背景，冲突以本次§5–8合同为准。仅规划，不建树/写实现/探针/运行产品。

2026-11-14 r1。由Auto主体执行统一grammar的词法/语法、CST/span/edit、AST投影与模块/namespace/provider装配，完整涵盖冻结M2-A语法而不借用Rust parser运行。 本轮仅设计/需求审计与编计划，未实施、未建树、未编译、未创建测试/探针。9任务全部未执行，当前不具备开工授权；不把本草案当M2-A或AAC通过证据。

## 1. 目标

由Auto主体执行统一grammar的词法/语法、CST/span/edit、AST投影与模块/namespace/provider装配，完整涵盖冻结的最终M2 required语法而不借用Rust parser运行。 本包出口见AC-01..06。依据[Design59](../../design/59-aac-native-bootstrap-and-compiler-subject.md)及[AAC路线](../../design/strategy/aac-delivery-roadmap.md)。AAC独立后续（沿用v0.7归属），M2-A稳定后与M2-B可并行；非M2硬门。

非目标：不推进M2全产品默认替换，不迁AAV/所有A2X emitter，不要求自主机器码后端；不另建通用标准库/全仓测试框架，不使用旧AAVM/AA2R自举证据代AAC。CS源码用到的子集与CL可编译完整语言分别验，不因本包子集完成宣称完整AAC。

## 2. 架构方案

遵守Auto主体 `syntax/source→semantics/CT→checked HIR→Typed MIR→显式backend/AV服务`；Auto还拥有增量和发布策略。Rust可保留Cranelift/机器对象、消费侧安全复验、AV执行及显式runtime。服务不得接源码代做parse/bind/infer/实例或HIR lowering，序列化Checked标志不具权威。

实现位于 `experimental/ace/aac/syntax/`、`experimental/ace/aac/source/`，阶段验证入口位于 `experimental/ace/tests/aac-syntax/`。已核实影响路径见§4。共享grammar/HIR/MIR/ABI/固定组/SDK文件按实际归属更新。

## 3. 技术栈

Auto主体+新版按场景装配的Auto标准库，默认自有ABI/OS adapter；Windows x64 hosted+alloc首版，外部Rustbackend/AV与link工具显式声明。OS布局/编码适配不得隐含libc provider或把长度字符串普遍改成NUL终止。Linux/其他host扩面按Design59具名pending/后继，不改M2既有平台义务。

## 4. 需求分析与背景调查

授权：用户以 `/goal 完成计划1046` 授权在 auto-lang 专树实现、验证、独立复审和合入清理。r7/r8 沿用该授权，范围为现有语法、源码投影及目录装配。用户未另设预算或自动续行额度。G-AAC 开工证据及依赖身份见 §8/§9。下表为初版代码观察基面 `9a6e5bf68ff3618340ac4ecb1697fdbf57a06c19` 的历史取材，当前工作基面和来源如下。

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

**r7 来源。** 专树 `D:/autostack/.wt/lang-1046/auto-lang`，分支 `plan-1046-dev`，基面 `dcd4fdafd412be911dc44ba1fee911a916d9c198`，代码 `a09237e845125f9c539f31448a9928384a6ce841`。工作树只多一行原始格式串失败用例注册，供 T-05 接续。现行约束取自 [snapshot/CST 合同](../../specs/auto-syntax/snapshot-cst.md)（`311a0d047ae559ad`）、[AAC 基线](../../specs/auto-ac/aac-baseline-and-subject.md)（`c5eea8eb376ad839`）、[964 L-22](964-ace-m2-dynamic.md) 和冻结 CL-105 位置行。以下均为该树内容 SHA256 前16位。

| 路径 | 观察事实 |
|---|---|
| `experimental/ace/crates/core/src/source/parser.rs` `eb120cd796ea97b5`、`fstr.rs` `1b2b2fb6f5ce840a` | `parse_program_with` 按 profile 启用转换，Scheduler 兼容 `{ident}`；`fstr.rs` 给出文本、`$ident`、`${expr}`、转义及非法近邻的实际边界。没有原坐标映射出口。公开入口 944 字节正例产 3 项，943 字节截断例的改写坐标为 952..952，见 §9。 |
| `experimental/ace/grammars/auto-source.at` `f785e607c818003d`、`experimental/ace/aac/syntax/syntax_frontend.at` `2901afdc57650a1e` | grammar 解析普通字面量/表达式及调用，`parse_cst` 目前直接将源传给 `lex_all`、`parse_tokens`。模块目录亦消费该入口。 |
| `experimental/ace/aac/syntax/syntax_incremental.at` `8d26fed4b1b1967b`、`experimental/ace/aac/source/source_ast.at` `94755fc3e17ac86a`、`source_position.at` `f9846e56221c3c28` | `incremental_lex` 目前用绝对 token offset 做 64KiB 内的 8 token 收敛；`project_ast` 和 `LineIndex` 等价接口分别消费树跨度与原始 bytes。现有 native driver 的 edit 路径直接调用 `parse_tokens_n`，须与冷入口共用修订后的输入/发布约定。 |
| `crates/auto-lang/test/cases/language/text/interpolation-unicode.case.ad` `a2085cb8e25fcadc` | `text-nul-and-format` 保持原共享源码，包括 NUL、Unicode、表达式与相邻的两段调用。完整样本和截断近邻均由 §6 判据覆盖。 |

**r8 来源。** 专树 `plan-1046-dev` @ `fc3785af7b54c42b3a717d650be94beeed0bb4d5`，基面仍为 `dcd4fdafd412be911dc44ba1fee911a916d9c198`。本轮核对时 master 已到 `384609645cfc092da66e7d005ee1b052e0a547f2`。T-05 开工先比较共享 grammar/compiler/owner 触面，以实际进入专树的基面重绑产物与证据。§9 的 r7 真目录和临时 grammar 实测用于确定修复路径。

| 现行来源 / SHA256 前16位 | 对本修订的约束 |
|---|---|
| `experimental/ace/grammars/auto-source.at` `f785e607c818003d`，`experimental/ace/crates/syntax-grammar/src/lib.rs` `6d37993f970d7b68` | `Ctx::gate_allows` 只识别一个当选 profile；`fn-tail`、`ann-ty`、`ct-item` 和 AGE 项的裁剪交集不够。`profiles_927` 已测精确名单和 GrammarId 隔离；加具名 profile 须增量更新其期望，并守住旧 canonical bytes。 |
| [M2 冻结分母 L-35](../../specs/ace-migration/m2-denominator.md) `2f2bdc63b49f0f66`，[CL-601/602](../../../crates/auto-lang/test/cases/ace-m2/aac-baseline-1044/cl-requirements.tsv)，`crates/auto-lang-core/src/parser/decls.rs` `cbc25bdb24d97c9a` | `parse_fn_annotations` 实际识别 `c/vm/rs/test`、同块逗号和连续块、`api` 键值、`export(system)` 形；`pub` 走关键字前缀。字段位取实际 parser 与生产样本，如已有 enum 字段属性，由 T-05 逐行核边界。以冻结分母与现行 owner 的证据对账，旧名称不自动变成新的 `#[...]` 拼写。 |
| [M2 工具发现](../../specs/ace-migration/m2-tools.md)、[M2 Web 路由](../../specs/ace-migration/m2-web.md)、[C ABI 导出](../../specs/auto-lang/trans/overview.md) | 保留 test/api/FFI 元数据供真实消费者，语义、目标分流与执行沿 1047/1048 及 971/967/980/982 原阶段。ACE 已交工具入口的零参发现规则有自己的阶段合同。 |
| `experimental/ace/crates/native/src/link_driver.rs` `2829a5bc6f07a0a4` | `link_only` 通过 `Command` 将全部 COFF 路径交给 `rust-lld`。r7 标准入口已进入真实链接后报 OS 206。T-06 从实际 Windows 供给入口重放，后端服务/驱动侧分别核 1051/1052 和 Design59 的正式来源。 |

## 5. 详细设计

### 任务交接与验收纪律

每个Auto阶段driver和测试配置由**该包首次使用它的work任务**建立/登记实际入口（必要时注册Cargo target/现固定组）；T-06既跑该Auto driver又跑实际受影响原Rust职责组。1053只汇总整机适配，不能把早期Auto代码一直等到1053才测。Gen0编译driver可使用Rust前端；driver运行并处理被测源码/IR时，主体对应算法必须Auto执行。命名driver/fixture未实施前不声称已有命令或已PASS。

基础阶段和完整阶段通过同一schema/版本化接口连接，任务表的前置必须核正式已交输入。T-09 属于 work，排在 T-06 之前。AC-01覆盖原T-02..05全部输出，AC-07覆盖 T-09。AC-04要求本包全部V行及适用输出齐全。后续整机和其他host义务按原归属记录。

本包只解析#in/is/if/for、保留条件AST及公共/provider候选目录，不宣称完成一般CT求值、可见性或类型绑定。1047消费无CT或已知target常量的基础模块，1048提供语义装配/生成后重绑定，1050/1053关闭高级CT组合。模块依赖循环按M2-A区分：禁止use/重导出或值初始化环，合法函数互递归必须接受，不能见环全拒绝。

每个work批次的真实生产者/消费者、输入/输出和关闭条件见§8及§6对应V-01..04。阶段driver/fixture/adapter若未存在，由本包T-02..05创建并记录真实命令；阶段模块先由Gen0编成native测试driver，传入被测用户源码/IR并观察Auto算法结果，不能用Gen0重新编译被测用户源码冒Auto主体。Gen1整机及代际只在1053/1054验收。

契约：成功产物绑定源码/grammar/schema/pass/target/stdlib/ABI/桥/工具身份；失败/取消/资源超限独立分类，不能静默忽略/返回空成功。已交M2-A资源支持及更严预算不减，新增资源profile取1044已冻结值；若不适配需回具名供给owner或有界重新设计，不无限重试/扩大门/降低规模。

跨仓仅消费现有规范/SDK/域host/TestCase格式；若接口确需改变，work先具名真实owner后继、合同delta、生产与消费版本，再接已交结果；当前无外仓代码修改授权，也不挂空泛跨仓依赖。完整AAC签章不强加为M2前置；AAC独有实现缺陷按本路线关闭，涉及M2三端公共能力的缺口必须回Design59 具名实施任务或实际库/领域 owner并在M2最终前修复。

### r7 格式串前门与原始位置（D-1046-04）

- **作者源码与解析输入。** 原始 UTF-8 快照拥有全部 bytes、内容 SnapshotId 和原始行索引。由 Auto 前门按真实 profile 选择格式串拼写，先对 UTF-8 与编辑边界准入，再生成供既有 scanner/grammar 处理的普通调用输入。拟用 `__fstr(...)` 和稀疏有序区间映射，保留原样复制区间、生成符号和消费的原始分隔符；无格式串走恒等路径。`$ident`、`${expr}`、字面/转义、空串及已支持的 Scheduler `{ident}` 沿公开 Rust 前门和 964 行为。按 `SourceProfile` 及实际入口核 profile 归属，不从共享 grammar slug 或其他领域的预处理分支推断许可。表达式、调用和运算符继续走共享 grammar 与 Pratt。
- **两套坐标与产物。** 虚拟输入及其映射只在当前解析请求内产生并参与身份校验。普通文本和嵌入表达式的复制区间保原字节范围；生成符号保生成来源并锚定相应原始分隔符或真实零宽坐标。按文档序遍历公开 CST 的叶子，正宽区间精确铺满原始输入，零宽不吞字节。节点、声明原点、诊断端点一起转换，后续 `project_ast`、目录、`source_position` 消费同一原始坐标。AST 能区分原写 `__fstr` 调用与展开构造，并保留按源序的字面段、真实表达式子树及段的原始跨度。语义后继收到完整结构和段数据，求值、显示转换及格式算法沿原 owner。
- **同源编辑与恢复。** 在 `syntax_frontend.at` 共用转换/坐标校验；冷解析、目录及 `syntax_smoke.at` 中当前 `incremental_lex`/`parse_tokens_n` 调用收敛到该合同。编辑先在原始快照校验；受影响的格式串区域从有效词法状态前的 checkpoint 重建，结合原始区间与扫描状态判定收敛。原始位移与虚拟位移分别计算，旧映射、未闭合状态或跨原始/生成区间的未验证边不能嫁接。格式串外的独立合格声明仍有真实重扫/嫁接见证；不能证明收敛的编辑只走现有带明确原因的 cold 路径。恢复可返回一份完整的当前候选 CST 和原始坐标诊断，语义准入拒绝错误树。失败 edit、取消或资源拒绝保留上一次成功快照和产物，未准入结果不发布到持久或语义消费入口。
- **已验资源和诊断边界。** 原始 `text-nul-and-format` 全样本与截断、冷/热、Unicode 位置和原始 bytes 的实证见 §9。转换和映射分配计入同一 64 MiB 前端预算，语法深度 1,048,576 和已交更严边界保留；日常 native case 使用现有默认执行预算及每例 ≤10s。资源耗尽独立拒绝。组合 profile 触到的入口重放 §6 永久回归。

### r8 共享 grammar 组合与标注所有者（D-1046-05）

- **具名组合 profile。** 在 `auto-source.at` 声明 AAC 的组合 profile，复用 `auto`、`age-package`、`packages-ct`、`packages-aggregates-ct` 与 `scheduler` 中经试验的规则准入并集。采用一个显式组合声明，由 `compile_profile` 按原 DSL 顺序裁剪同一套规则。组合列表的空项、未知名、重复和自引用须在 grammar 编译时受控拒绝；若允许组合引用组合，也验证环。普通 profile 的旧 Gate 可用集、已声明顺序、canonical bytes 和 GrammarId 逐项不变。组合产物有独立 name/GrammarId，并绑定实际 program bytes、schema/codec 与前门设置。格式串和领域方言按显式源策略保留，组合语法 gate 不隐式改变原有 mode 探测、目标配置或动态 CT 选择。
- **声明标注是共享 grammar 节点。** 确认 L-35 与已交消费者实际拼写后，按合法位置增加组合 profile 专属的 grammar 生产式。至少覆盖 `#[test]`、`#[api(method = "GET", path = "…")]`、`#[c,vm,rs]`、`#[export]`/`#[export(system)]` 及其合法/非法邻接。`#in/#if/#is/#for` 与 `#[...]` 由实际 token 形状分派，各自保留原 AST。每个出现保留名称、键值或位置参数、顺序和原始跨度，结构关联实际拥有的声明与规范已支持的成员/字段位置。参数类型及泛型约束沿现有 field role 进入 AST；标注落到无该语法的槽位须受控诊断。名称允许集、目标或语义检查沿现有阶段/消费者裁决；源码转换或目录装配均不得提前执行标注、抹去未选分支或发布 Checked。
- **正常入口及同源产物。** `tests/aac-syntax/src/bin/generate-grammar.rs` 制作 canonical 数据，`syntax_program.at` 解码及准入、`syntax_frontend.at` 选显式 profile，`source_catalog.at::catalog_directory` 对该包逐文件执行同一 Auto 前门，`source_ast.at::project_ast` 以结构边/field role 交下游。按真实组合 file 样本核 `sourceId`/origin、类型及标注 owner。只在独立测试 Oracle 使用 Rust grammar compiler、Rust reference parser 和实际标注消费者的已交事实。源包不拼接；不能遇到节点、属性或配置差异后轮换 profile 吞掉语法诊断。
- **可行性与资源。** 在 T-05 首个检查点将 r7 的 35,357 字节原型形状接到生产生成、Auto 程序解码和默认预算的 source-bound driver，确认 canonical bytes 和合法/缺 `]` 两模块入口。分别记录 table/first sets、解析、CST 重建及投影开销和实际错误阶段；范围或模块数按原 fixture。超过物理表容量时先从事实缩减冗余表或改有界表示，保留 §4 原预算、每例 ≤10s 和错误分类。该阶段结束前把 42 项格式串、源码位置、嫁接及目录的受影响证据重绑；受影响的原 Rust 职责组在 T-06 验证。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-source-frontend.md` | 记录 Auto frontend 权威、统一 grammar 身份、语法与模块准入，以及格式串原始快照/生成来源/坐标与恢复契约。补实际已交 AAC 组合 profile、标注及类型的语法 owner 路径。 | 使 AST 和诊断消费者可校验同源数据，保留 Rust reference 的正式独立入口 | AC-01/02/03/07、AC-CL-FAMILY |
| SD-02 | modify | `docs/specs/auto-ac/project.md` | 只在review验证后补本包已交事实与AAC新Spec指针；不得把整个目标态提前写成current | 保现行Rust事实及历史阶段，避免规划冒实现 | AC-05/06 |
| SD-03 | modify | `docs/specs/auto-syntax/grammar-contract.md` | 记录实际交付的显式 profile 组合声明、顺序、无效组合拒绝及 canonical 兼容。 | 把同一 DSL 的组合语法与普通裁剪身份对账落在规则程序权威来源 | AC-03/04、AC-CL-FAMILY |
| SD-04 | modify | `docs/specs/stdlib/design/context-source-assembly.md` | 增补 AAC 源码候选目录的实际接口和准入，链接其来源/语法规范及后继绑定任务。 | 说明 provider 文件如何逐文件保留原始来源、候选和条件臂，继续沿现有绑定入口选择实现 | AC-01/03/07 |

本轮不写canonical Specs；work准备delta，review核实际revision，merge再沉淀。

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](../1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-source-frontend.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本包语言族的实际补齐与验收（2026-11-16本轮检查）

本包负责**完整语法与类型/标注投影**，沿[Design59的CL族级映射](../../design/59-aac-native-bootstrap-and-compiler-subject.md)和1044原CL表，落实到原T-02/T-03/T-05；不能只靠“完整语言”总括句。语义实现/前端检查由本包原任务承担，现AC机器/runtime或库缺口交Design59 具名实施任务或实际库/领域 owner，取得真实正式源与结果后关闭。CS自用子集、编译器单线程实现和A期AC未实现均不缩最终CL；早期阶段允许具名后继，整包完整支持不能以pending代通过。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CL-FAMILY | modify | docs/specs/auto-ac/aac-source-frontend.md（实际交付后） | 旧泛称完整/M2-A有限适用→本族最终M2 required及实际任务/执行链/CS未用补验 | 用户要求检查是否真正补齐语言 | AC-CL-FAMILY |

## 6. 测试设计

**AC 语法同源验收。** 在 T-06 运行已交的同源 AutoDown 语法、源位置和模块装配用例的 AC 检查。按原用例观察合法源码、非法语法及对应诊断，保留解析阶段本身的定向检查。 按 [AV AutoDown 共用语料规则](../../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)执行，尚缺具体用例在 AV 全线交付后统一对账。

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

现有固定组为 `ace-syntax`、`ace-source`。T-06 在 `scripts/test-groups.toml` 登记缺失的本包 `ace-aac-syntax` 组并核对列名及真实选择，按实际触面补充共享 grammar、Rust 前门及必要消费者组。准备 driver 的成本和退出码单列，固定组运行使用已对账的源绑定产物。模块常规集合沿现行排除 `::performance::` 和 Windows 上 Linux/WSL 的规则，正式规模/持续压力只由 `cargo t ace-perf` 承担。1053 汇总整机入口。

work核定向反例+影响组；独立review按同树冻结源码复核，不以自报PASS为证据；merge用身份未变证据，不堆重复全仓轮次。记录列用例/实际执行/原条件排除数及准备、发现、运行、清理/总耗时；未执行/pending不充PASS。外仓保owner入口。未指定模型或额外agent，不擅自启动新会话。

### r2 必验补充

| Case ID | AC / owner_stage与任务 | 输入/状态 | 入口及启用任务 | 独立预期 | 边界 |
|---|---|---|---|---|---|
| V-05 | AC-07/AC-04；work / T-09 | 源码位置和模块分类补证的合法基线与具名错误/变化 | T-09建立或扩本包真实阶段driver/父准入入口 | 含CRLF与emoji的源码按既定单位定位且冷/编辑/恢复一致；函数互递归和值初始化环在语法合法时都产保留类别/来源的AST/图，本层不误拒也不产Checked；值初始化环的语义拒绝明确登记1047/1048必验，当前只验可区分结构/交接，不反等未来语义Plan才能完成 | 本包阶段；最终整机/代际去向1053/1054 |

### 格式串判据（r7 已验，受影响路径随 r8 重验）

T-05/T-09 扩展现有 `experimental/ace/tests/aac-syntax/native_driver.at` 和 `tests/conformance.rs`。在 ACE workspace 用 `cargo run --offline -p ace-aac-syntax-conformance --bin prepare-native -- <唯一绝对材料目录>` 制备源绑定 A2R driver，沿 `AAC1046_NATIVE_DIR`/`AUTO_PLAN_ARTIFACTS` 运行 `cargo nextest run --offline -p ace-aac-syntax-conformance --test conformance --test-threads=1 --no-fail-fast`，定向定位对应命名 case 后运行完整范围。T-06 按相同原始输入补 V-AC-STAGED 的实际证据。Rust 公开源入口只在独立测试器观察接受/拒绝，其改写文本及共享 grammar 可作为表达式结构 oracle。原始跨度和来源以输入的独立区间和快照位置 oracle 为准。

| Case ID | AC / owner_stage与任务 | 输入/状态 | 独立预期与区分性 |
|---|---|---|---|
| V-FSTR-01 | AC-01/03、AC-CL-FAMILY；work / T-05 | 原始 AutoDown `text-nul-and-format` 整例，另有 `$ident`/空串/转义、相邻表达式、注释或普通串中的 `f"`、手写 `__fstr` 及 Scheduler `{ident}` 和不支持 profile 近邻 | 原始 bytes 逐字节重建，AST 保留源码顺序、独立的运算/调用子树、生成标志和原始分隔符/片段区间；`left()`/`right()` 各出现一次且顺序正确。profile 接受性取其实际公开前门，普通串与显式调用保持原类。将格式串当单个不透明字面量、跳过某一表达式或将生成调用冒充作者调用均不能通过。 |
| V-FSTR-02 | AC-02/03/07；work / T-05、T-09 | 原例删最后声明 `}`；另取合法近邻分别删插值 `}` 或闭引号，CRLF 和 emoji 位于错误前，另设低预算与同预算合法邻居 | 公共 Rust 前门报告阶段/接受性；AAC 偏移取原始 EOF 或确切坏分隔符，范围在原快照，byte/UTF-8/UTF-16 位置由 Rust `LineIndex` 对原 bytes 核验。可查询候选树保持无损、错误及恢复标志，严格准入和发布拒绝坏结果。资源拒绝与正常语法恢复分开，低预算失败不替代正常预算断言。 |
| V-FSTR-03 | AC-02/04/07；work / T-05、T-09 | 同一快照在嵌入表达式内编辑和撤销，在含 emoji/CRLF 的前缀编辑使格式串右移，保留格式串之外的可复用声明；另试 UTF-8 字符中段坏 edit | 冷/热 CST、AST、原始诊断和坐标逐步对账；撤销恢复内容 hash 与语义结构，逻辑 source/project id 保持。成功热路径用实际扫描和嫁接事实证明格式串外仍可复用，跨未闭合构造的回退记录原因。坏 edit/资源拒绝后核原快照/树身份和正例可再次读取。 |

### 当前阶段交接验证

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

### 组合语言族判据（r8）

| V / AC | 原任务 / 真正入口 | 独立oracle和拒绝控制 |
|---|---|---|
| V-CL-FAMILY / AC-CL-FAMILY | work/T-05 在既有 `native_driver.at` 的目录入口加载同一份两个或更多 `.at` 文件并产可观察候选；T-06 及独立复审核完整结果。Gen1/代际入口分别归1053/1054 | 真实 `entry.at` 同时含泛型、Option/Result/Future 签名、CT 分支、实际 `#[api(...)]`，领域 `views.at` 含 Node 及被导入成员。用实际 provider 分类、独立 Rust 参考树及原源码切片核每文件 sourceId、参数/返回/类型字段、标注的真实目标与值、两个条件臂及域 AST。删标注 `]`、断类型修饰或把标注移到不准入位置各自拒绝，正常邻居可继续查询；删除类型/标注边或跨文件挪用 owner 的反控必须被断言抓出。 |
| V-CL-ANNOTATIONS / AC-CL-FAMILY、AC-01/03 | work/T-05 扩本包格式化输出与目录的既有 fixture，在 `experimental/ace/tests/aac-syntax/tests/conformance.rs` 校验 | 对照 L-35 实际消费者输入覆盖无参 `test`、连续块/`c,vm,rs`、`api` 键值、`export(system)` 位置参数、正式允许的成员/字段目标及参数类型归属。正例断言结构、原边与原字节，确切坏括号/错误槽位拒绝。标准可见性继续走 `pub`；符合词法但语义未知、目标错误的标注按原阶段交接并区分语法拒绝。用 1047/1048 能直接读取的名称、参数、owner、来源与受恢复状态，不以原字符串或工具后端执行冒领。 |
| V-CL-PROFILES / AC-03/04、AC-CL-FAMILY | work/T-05 在现有 `ace-syntax-grammar` 的 `profiles_927` 家族补组合验证，T-06 `cargo t ace-syntax ace-source` 及 `ace-aac-syntax` 固定组 | 按进入 r8 的基面记录全体旧 profile canonical hash，再比较 r8 同名单逐个等值；新 profile 有独立身份、原 gate 顺序、真实解码和声明名单项。坏组合引用不能落到普通 profile 或产半程序。以原 ACL/预算下实际新语法成功、坏输入受控、旧 profile 邻例正常证明输入确实被同一 Auto runtime 消费。 |

## 7. 验收标准

| ID | owner_stage | 可观察关闭条件 / 验证 |
|---|---|---|
| AC-01 | work | §6 V-01、V-FSTR-01 与§8各 work 产物逐项通过独立可观察验证，含原始 bytes、结构和来源。 |
| AC-02 | work | §6 V-02、V-FSTR-02/03 实际拒绝/失效/故障保旧与合法近邻结果均满足；资源拒绝、语言错误、OS/工具故障分类准确 |
| AC-03 | work | §6 V-03 与 V-FSTR-01/02 实际通过/拒绝符合其 oracle；Auto 权威、默认库依赖与可信边界有真实禁 Rust 源服务见证；支持范围沿本包合同 |
| AC-04 | work | T-06所有受影响固定职责组/必要消费者及本包全部V行完成，逐输入/工具/产物身份、退出码、expected和失败边界入计划摘要；pending完整列明，不以未跑签通过 |
| AC-05 | review | T-07独立审查本revision每项AC、漏项/未批准延后/workaround/编译警告/格式/debug及规范delta；不足返回needs_fix/needs_replan，未通过不merge |
| AC-06 | merge | 仅review通过且来源落主后沉淀已验证Spec、归档/清理；按真实支持面交后继收据，不给M2增加AAC硬门 |
| AC-07 | work | T-09 完整交付，V-05 和 V-FSTR-02/03 的合法/非法与失败近邻均满足独立位置、恢复及分类预期 |

- [x] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

- [x] **AC-CL-FAMILY / work**：V-CL-FAMILY、V-CL-ANNOTATIONS、V-CL-PROFILES、V-FSTR-01 和原T-02/T-03/T-05 的实际结果满足。族级CL、现有 profile canonical 兼容和源码所有者交接逐条对账。语义与runtime/库补齐正式来源齐，原独立review逐行核。

## 8. 执行步骤

专树 `D:/autostack/.wt/lang-1046/auto-lang`、分支 `plan-1046-dev` 已建，work/review/merge 沿同一树。计划材料使用已登记的 `D:/autostack/plan-artifacts/plan-1046/`，测试生成输出/SDK/仓外消费及外置 target 显式放该 scope，禁止 junction/symlink。T-04/T-05 返修和 T-06 重绑、T-07 复审、T-08 归档及清理均已完成；如当前工具提供原生 TODO，执行前登记下表未完成任务 ID。T-02/T-03/T-04 的已交实现保留，组合所涉及的共享语法、投影、目录和已有证据由 T-05/T-06 补证。

| Task ID / 状态 | owner_stage | 依赖 | 实际影响/执行内容 | 输出与验法/对应AC |
|---|---|---|---|---|
| [x] T-01 开工身份核对 | work | G-AAC及本包§4/5最小前置、未来实施授权 | 重读本Plan当前revision和当前Spec，核已合入A基线与输入版本、实际路径/符号/供给就绪、唯一专树及材料登记；未就绪停在具名依赖不消费WIP | 记录worktree/branch/base与§4指纹变化、支持profile和后续任务入口；AC-01/04 |
|    [✅ 2026-11-20 /goal 授权开工] worktree `D:/autostack/.wt/lang-1046/auto-lang` @ `plan-1046-dev`，base=master@ec0c377e7；材料 `D:/autostack/plan-artifacts/plan-1046/` 已登记（REGISTRY.md）。G-AAC门核验：M2A-08已成立（985 T-A04 pass@ecc282df1、T-A05 fold落主95c7fbad8、M2A-01..08全闭，见985 §9 merge收据）；1044/1045/1043已归档；不等985-B成立。§4指纹重绑（基面ec0c377e7）：syntax-runtime/src/lib.rs 07328483e30aecd6（原5133b5cd漂移）、syntax-grammar/src/lib.rs 6d37993f970d7b68（原087b300b）、core/source/parser.rs 6a6233f7818deef4（原04782003）、modules.rs 87a482d8033bf548（不变）、packages.rs 95cc76cfdb247a96（原9aa1b33d）、cst/目录七文件在（frontdoor/kinds/lex/mod/project/shift/units）。主检出WIP=他计划簿记（docs/design/1081/1107），不消费。移植面核定：syntax-runtime执行引擎=program(546)/program_codec(920)/lexer(810)/parse(2776)/cst(404)/snapshot(644)/edit(467)/recovery(49)/position(303)/identity(97)≈5.6k行；autodown.rs(1309)为md文档层不在本包面；core/source/parser.rs为旧子集oracle。grammar数据：grammars/auto-source.at（rev3，25 profile），canonical bytes实测auto=16504B、age-package=21249B、packages-aggregates-ct=13787B（probe证据plan-artifacts/profile-sizes.txt）——hex内嵌可行，Rust侧grammar编译保留（T-02合同允许）。后续任务入口：T-02..05/T-09在aac/syntax/+aac/source/新增.at模块，conformance crate `experimental/ace/tests/aac-syntax`（沿1045先例布局，计划§2的aac/tests/syntax/路径按"常规机械实现由work选择"落为既有tests/惯例）+固定组ace-aac-syntax |
| [x] T-02 统一grammar与Auto解析 | work | T-01 | 消费同一版本化grammar定义，移植实际tokenization/parse/CST算法和预算，在本Plan建立Gen0编译的Auto阶段测试driver；Rust grammar数据生成可保留但输入源码解析必须Auto执行。 由最终CL语法行驱动解析，显式含~T/?T/!T类型及嵌套、async块/await/go/yield、泛型约束、CTE/标注及领域Node，不限AAC源码自己的语法。 修实际影响现 AC 的共享语法/projection 差额，保 AV/A2R 的相同源合同。 | Auto语法运行时及driver，首个正/反例阶段产物可独立检查；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
|    [✅ 2026-11-20 /goal work] `7cdb73c8d` 已交 Auto 程序表/词法/parse/CST/增量与原生 driver，D-1046-03 关闭。重定基后七 profile 编解码身份和完整 22 个已建用例通过，含 `~Iter<T>`、`?T`、`!?T`、AGE Node 及 scheduler 的 Task/await/go/yield。AST 投影归 T-03，组合语料/回滚与边界余量归 T-05，固定组及 AC 阶段签章归 T-06。详见 §9。 |||||
| [x] T-03 AST与来源映射 | work | T-02 | 实现全适用语法到Auto AST的投影，保sourceId/byte span/生成来源/错误恢复；完整语言含Schema/AutoUI/AGE语法节点，不引入renderer、不将未知域节点忽略。 保Option/Result/Future/Iter不同节点、标注与其声明/字段/参数归属及原span；不能parse后擦除类型修饰、标注或未使用分支。 | AST投影和diagnostic snapshot，冷解析与编辑增量结果相等；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-04 模块与provider装配 | work | T-03 | 实现模块/包/namespace/use/pub与#in的语法目录/条件AST：公共.at与at/c/rs/py候选逐文件parse，保选择条件和来源，不在本语法Plan执行一般CT或作语义绑定。1047基础绑定、1048语义选择与重绑定消费此目录；不拼源文本、不建第二binder。 修源模块及 provider 装配的公共接缝，交同源模块正反例给 1047/1068。 | 模块目录/来源关系及受控语法重复/缺失诊断及循环分类输入，语义可见性/初始化环留1047/1048；用§6 V-01..04的对应输入核独立预期，绑定AC-01/02/03 |
| [x] T-05 语法边界反例 | work | T-04；§4 r8 基面 | 先按 §5 将具名组合接入 `compile_profile`/`generate-grammar`/Auto 目录，验证原 artifact 身份及正反模块。补 T-02/T-03 组合缺口、L-35 生产式、owner 投影与边集，闭合 D-1046-05。继续逐 CL 真实语法生产式对账，覆盖坏 UTF-8、token 边界、深度/资源、恢复、坏编辑与模块循环。r7 的 D-1046-04 已关闭，受影响的冷/热路径与全部 required 语法族重验。 | 完整语法映射、资源拒绝及无Rust parse调用的运行证据；§6 V-01..04、V-FSTR 和 V-CL 中 T-05 负责的字节/结构/归属/编辑断言，绑定 AC-01/02/03/AC-CL-FAMILY |
| [x] T-09 源码位置和模块分类补证 | work | T-05；§5实际输入合同 | 以当前syntax-runtime/position.rs的byte/UTF-8/UTF-16、CRLF与emoji中点floor规则补driver；校验普通/格式串的冷、编辑、恢复诊断原始跨度。验证语法恢复后仍保错误状态、全部语法义务映射、合法互递归与非法初始化环的不同去向。保持逻辑source/project定位与内容SnapshotId分账。 | §6 V-05、V-FSTR-02/03 的位置和保旧预期及输入/产物身份；AC-07/AC-04 |
| [x] T-06 定向验证和交review | work | T-02..05、T-09 | 按 §6 登记/运行本包固定组及 `ace-syntax ace-source`、必要 consumer 和命名正反例，核父子真实终态/身份/原oracle/资源/清理。用正式 Windows AC 链接入口复核 V-AC-STAGED，沿 §4/§10 的实际 owner 核供给和当前切片的支持；诊断 response file 保留作定位证据。更改旧 profile/源码身份后重绑受影响的证据，准备 SD-01/02/03。 | `1abe595fc`。四个固定组 1470 项通过，源绑定 A2R 和 Windows AC 正反例重验。交独立复审，证据见 §9。 |
| [x] T-07 独立复审 | review | T-06 | 同一专树冻结revision独立追数据流逐AC核证据；搜漏项/延期/workaround，具名债务写KNOWN-DEBT-AND-RISKS，核warning/格式和Spec-impact | revision绑定pass或具名修复意见；AC-05，不能把本草案静态预检当代码复审 |
| [x] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、受影响证据、Spec增量，归档本Plan；所有输出路径逐项对账，必要证据折叠计划后清空材料scope，wt-guard clean再移树/分支 | 独立交付摘要/后继输入及清理结果；AC-06，M2/AAC总出口按各自合同 |

## 9. 复审记录

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r4：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r3：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14规划复核：stage=new；revision=2；静态合同已修订（拆清#in的语法/候选目录与类型/CT选择，补模块循环适用性及Unicode/恢复的精确边界。）；outcome=blocked（实施仍等待G-AAC及实施授权），next=在开工门后按§8执行，不是代码review通过。Design59 §9记录本次F01–F10发现及关闭owner；本轮未执行验证fixture。

未施工、未执行独立代码review、未合入。当前文档仅作需求/依赖静态预检，实施进度0/9；所有验证用例为拟议合同，具体结果在未来work/review/merge补齐。此前用户批准规划不等于本Plan代码实施授权。


2026-11-20 /goal work，r6 / T-02 嫁接修复单元，专树 `D:/autostack/.wt/lang-1046/auto-lang`、`plan-1046-dev`，基面 `ec0c377e7`，代码 `5cea8cf0f79300e9fd2ee2d664d1259bbf4b5863`。逐步快照在 150、200 步配对跑开启/关闭嫁接，两个截止点两组分别一致：第 151 步 i=37、11 帧、top rule=35、graft=0、pending=1、efhi=132；第 201 步 i=40、11 帧、top rule=30、graft=0、pending=1、efhi=154。两组检查的 arule/ai 物理长度 11，逻辑深度 10。本轮的 154 是边 arena 的逻辑前沿。以上截止点均在首次实嫁接前，历史 ListElems 触发点的精确链尚待核实。

完整正例 `cargo test --offline -p ace-aac-syntax-conformance --test conformance aac1046_edit_incremental_equality -- --exact --test-threads=1`，debug，无 ignore，退出 0，1 执行、0 红、17 个未选。新回归覆盖旧树声明实嫁接后的 CST、字段、byte span、诊断和复用计数；同 grammar / 原始 edit 的 Rust 参考实现判定实嫁接 1（前方的 b 与重扫区相交）。临时探针版修复重放 48.51s，移除本轮探针后同一用例 78.81s；后者 Rust 测试构建 1.32s，发现和清理阶段未单独计时。限步对照编译分别 2.15s 和 1.39s，测试 128.71s、146.81s，各为一个外层探针、两个 AV 对照。回归超过每新增用例 10s 门，保留首次及复验时间，T-02 继续执行，无 final review/merge 收据。

材料均在 `D:/autostack/plan-artifacts/plan-1046/`。`graft-cap-control.log` SHA256 `6de99b165e91896bdbf76f1a0e58eada187dd01d9cd5bd2b5e632fac0304420`；`graft-cap200-control.log` `43a00e7080b581712b9887461498eb0b4c6cb7195ff28b137117828d9f6c7b47`；`graft-clean-oracle.log` `27bd5bd3fe6c3da181a8b3f3cdf4959bf557d2f70ac6aff6f01b1a376e17a738`。



2026-11-20 /goal work，r6 / T-02 增量扫描单元 `3a7b88bdc9d67c3cd875f55e900ea817d565cb1b`。从可信 checkpoint 调用真实逐 token scanner，8 token 收敛即停止，64KiB 窗控制扫描，拼接输出只在收敛后写入；删除按新逻辑字节长度进行 fresh fallback。复验同一 AV 三函数 edit，Rust 独立比对 canonical CST、诊断、实际重扫量和复用数，1/1 PASS、17 未选，test 100.26s，Rust 构建 2.51s（首次开发编译检查定位参数名 `byte` 是关键字，关联日志 `bounded-splice-oracle.log`，1 个红用例 2.47s）。通过日志 `bounded-splice-oracle-replay.log` SHA256 `84cf6f3e8f3bfb69429d3ce77881e50b881eb3fb2b9c450331c41308cbcd4eab`。

同修订真实 A2R native 试车把编译与执行分离。首次生成 Rust 有 E0308（digest 的借用 List 误作 owned）及 E0382（grammar header move 后仍清理），相应 Auto 源改为自有工作列表及直接使用 header 输出变量。复验 emit 准备 57.382s、debug rustc 4.672s 成功；默认栈实机溢出，16MiB Windows 栈预留再编译 4.772s、执行 0.256s，程序 `value:0`。对真实 native 输出逐字节比对 Rust：3886 字节 canonical CST/诊断一致，`ok=1 fb=0 rescan=20 grafted=1`。原始 `native-probe/observed.txt` SHA256 `0f4528fc8452fc76fca88fdacae4978125273a0ad0cf915c0ad18e8819a1a560`，校验日志 `native-probe/oracle.log` `e4c32d51da9d0caf24e8292efede8fe459c5caa0a1ae9dff4190ad30df1c3cd8`。共用 digest 2049 字节树折叠反查 `cargo test --offline -p ace-aac-support-conformance --test conformance v01_blake3_tree_vectors -- --exact --test-threads=1`，1 PASS、0 红、8 未选，Rust 准备 1.35s、test 3.12s，发现/清理单独未测，日志 `native-prep-digest-control.log` `b20527215918f018065f396f9d44d8aaFC3c79f82f2a817378c4a7da8d1fbc09`。该试车只闭合编辑原生路径，通用 driver 与所有新增用例的 10s 门需正式重跑。



2026-11-20 /goal work，r6 / T-02 原生 conformance driver `f9288004260b2e2861b1f805f01fcbaad45e71b0`。独立准备 `cargo run --offline -p ace-aac-syntax-conformance --bin prepare-native -- <绝对材料目录>`，绑定 Auto 模块、stdlib 输入、准备源码、目标配置与 Cargo.lock，登记 rustc 版本和 exe 内容 hash。测试运行只读准备清单，启动 Windows/A2R 实际二进制并比对独立 Rust oracle、回收每例文件目录。目标 `cargo nextest run --offline -p ace-aac-syntax-conformance --test conformance --test-threads=1 --status-level pass`，18/18 PASS，0 skip，单例 0.074–0.791s，编辑嫁接例 0.270s，串行测试 3.945s，编译 Rust 测试器 1.27s，命令总时 5.728s，独立 Gen0/A2R driver 准备总时 65.822s（其中 Rust 准备工具构建 1.29s）。每例的 setup/child/assert/cleanup 均计入 nextest；发现/外层清理未拆。强制缺准备清单反控，编辑精确 case 立即拒绝，子退出 101，0.02s，无临时构建回落。测试日志 `prepared-native-verified-tests.log` SHA256 `b72f1fd66549063ddce6f8a0ba0dd29753465611d2ba59da358ac8041085fc74`，逐例见此证据。T-02 继续对账无损重建、资源负控和语言族。

D-1046-03 历史失败定点复现：从 `cf997b356` Git 对象恢复实际 Auto 源模块，保持三函数输入与字节 25 编辑。仅临时 AV 插桩在原 ListPush 资源拒绝路径附函数/指令/操作数槽，构建后恢复 executor 源文件。74.920s 后原环境报 `ListElems`：65536，func 311 / instruction 144 / slot 2，源跨度 329592..329617。用同输入组装产物的 resident unit 及 lowering slot 名表交叉定位，符号 `d_p3_appm12_syntax_parse_try_graft`、slot `t`（CST 节点 arena）。历史爆点在真嫁接的节点复制内，前 200 步探针对照均未到达这条复制路径。日志 `native-probe/graft-baseline-run.log` SHA256 `c724bf7d8290715ca40c29dc413da21d2bf2ba9f1a8da17ce36330424f294598`，`native-probe/graft-baseline-location.log` `be79a6f734c15fa508d817b01012d8595bfa8efabd79a76e4ebc63160bdb809f`。修复版 AV 与 prepared A2R 均通过独立逐字节 oracle，原 ignore 移除，唯一历史失败已定位，D-1046-03 关闭；未单独归因旧两层写入循环的所有失效机制。



2026-11-20 /goal work，r6 / T-02 关闭单元，专树仍 `plan-1046-dev`。补严格字节源入口、解析前 UTF-8 拒绝、深度与工作量拒绝、所有 parse/edit 路径的 CST 字节重建；V-04 对 CRLF 中的 α 做真实插入和删除，分别与 cold CST/spans 同源对账。首轮 21/22，AGE 参考侧需要上下文原始 token 字节，接入真实源码后该 case 0.386s PASS，完整 pre-rebase 重跑 22/22，最长 0.850s。随后将专树从 `ec0c377e7` 重定基至主线 `dcd4fdafd412be911dc44ba1fee911a916d9c198`，8/8 range-diff 等价，保留前证据的原提交身份，当前对应提交见 branch log。按真实主线共享 grammar 重生 7 profile，加入 scheduler 的 Task/await/go/yield，源码 SHA256 `f785e607c818003df7dc9c085c2564dbeb8eae69debf837dd03afcc2125d81d8`，表 SHA256 `3b7d091df50e9a0cfd19ed84522c897895b3d95d4a05097680dd0228ecfdd4f8`。

当前代码 `7cdb73c8d414c6ff60a4c14a5ded421b418dfe23`；`cargo nextest run --offline -p ace-aac-syntax-conformance --test conformance --test-threads=1 --status-level pass --no-fail-fast`，退出 0，22 执行/22 PASS/0 skip。新增验证范围（Win x64/A2R debug 每例 setup/运行/回收全计）：depth/work 近邻与拒绝 0.287s；Unicode edit/undo CST 及 byte span 0.383s；CRLF/emoji 无损字节与截断 UTF-8 入口拒绝 0.207s；Auto 复合类型、AGE 节点、scheduler 3 profile canonical 与诊断 0.836s。全目标单例 0.114–0.889s，串行测试 6.044s，测试 Rust 构建 20.39s，独立 native 准备总 81.679s；发现/外层清理未拆。日志 `prepared-native-rebased-1-tests.log` SHA256 `d420312ad83d58f99fc8d1728683dc03a2c7dd10f17f18ce73037e0129df965f`。进度 2/9，继续 T-03，同一专树，尚未发整包 work/review 收据。



2026-11-20 /goal work，r6 / T-03，`e724378c1` 与 `30b19c93b733a53b4bfe4d11f0d24de9101bac77`。Auto AST 顺序表保留 grammar 种类、父节点/字段角色、byte span、source id、origin id、恢复标记和 expected kind；完整领域节点随实际 grammar 结构交给消费者，CST 独立保留 trivia。冷解析与增量回放共用投影，Unicode 插入/撤销保持 source/origin 并与独立 fresh 树逐字节对账。三个新增原生 case：领域/泛型/坏形恢复原始 AST 与来源（0.687s）、对 Rust 正式类型投影核 Optional/Stream/Future/ResultBang 字段跨度（0.205s）、同源 AST edit/undo 与来源保持（0.336s）。完整目标 25/25、0 skip，测试 7.758s，最长 1.045s，prepared A2R 准备总 68.994s，发现/外层清理未拆。证据 `prepared-native-ast-edit-tests.log` SHA256 `b6072a7a31dddc8a1ce8f9bbf6e4c10bd697dc39cfe2bc9f45c1cad1060b93cc`。进度 3/9，继续 T-04。下游跨文件绑定和全部组合/边界输入由 T-04/T-05 处理。



2026-11-20 /goal work，r6 / T-04，`d7dd01753f6b5037b7e6c16cc2f0cf6e771c4f15`。共享 Auto syntax frontend 供目录按自身 UTF-8 文件独立造 AST。按 1043 的公共/at/c/rs/py 后缀保稳定目录源 id，对本包直接 use/re-export 构造边，保 #in/is/if/for 两侧的完整条件 AST 和导入引用，候选不做一般 CT 绑定；缺失/重复/导入环在当前语法目录阶段明确拒绝。正例用 Rust 1043 classifier 对照六个真实文件以及两个条件臂；反例与合法近邻定向覆盖缺失、同文件重复和导入环。测试首两次 native prepare 分别遇 Auto List<Str> view-escape 的绑定与 verifier 拒绝，改用现有 `remove` 移出所有权后通过。最终 `cargo nextest run --offline -p ace-aac-syntax-conformance --test conformance --test-threads=1 --status-level pass --no-fail-fast` 27/27，0 skip，测试 8.064s，新增目录正例 0.344s、反例与合法近邻 0.484s，全目标最长 0.902s；独立 native 准备 82.655s，发现/外层清理未拆。日志 `prepared-native-catalog-3-tests.log` SHA256 `979d459ddfcc973c46d7b9b9fd0f42e2637997aa954523ed4ef1325c3df1c5be`。进度 4/9，继续 T-05、T-09 和官方阶段/固定组验证。


2026-11-20 /goal work，r6 / T-05、T-09阶段检查点，`af3cfaffee9d6a4a80868921e722c41922d696f1`。共享 AutoDown 原始用例及截断近邻经 prepared A2R 接入。修复 FIRST 计算对合并位标志的误读、EOF 恢复的尾部 trivia 丢失，以及嵌套 Pratt 共用槽零起点导致 `xs[0] + xs[1]` 左操作数被右下标覆盖。全探针 37 运行、34 通过、3 失败，16.038s；新通过的五组为 basics/calls/collections/functions/numbers，记录 `prepared-native-pratt-1-tests.log` SHA256 `ffb6f459fe5538579521a5f1fea7e26c141844b17045de237dda60a920412975`。仅提交通过的范围；去掉探针模式后从同一工作树重新制备并执行 `cargo nextest run --offline -p ace-aac-syntax-conformance --test conformance --test-threads=1 --status-level pass --no-fail-fast -E '!test(/aac1046_corpus_(patterns|text|types)/)'`，退出0，34/34，11.815s。3项明确未计通过，保留在工作树：patterns 截断工作量拒绝，text 格式串在 raw Rust grammar 本身需预处理且 Auto 程序拒绝，types 截断恢复 CST 不同。证据 `prepared-native-source-bound-1-tests.log` SHA256 `373d06d8d88075baffdf09ed1156c9d92bf0cec5b6adc273d2716c76b6492eb4`。坏编辑回滚保持原内容 hash 和长度，合法近邻可编辑；CRLF/emoji 六点位置与 Rust `LineIndex` 双向一致。冷/编辑/恢复诊断位置贯通及初始化分类图仍属 T-09。任务进度保持 4/9。

同轮 AC 实机探针用 `AssemblyBackend::Ac` 和 `MnDriver::build_collections_checked` 编译实际 Auto parser，由标量输入的 CST/诊断字节与独立 Rust oracle 判退出。通常链接入口在 Windows 返回 OS 206（`native-probe/ac-minimal-syntax-3.log` SHA256 `50ce7eecde146273706dca3b530b6e328d4da8a0dade14cd6f539741910992ee`）。只在临时诊断构建中将原 `rust-lld` 703 参数、107113 字节经 response file 传入，实际 AC COFF 链接并运行，退出0，0.103667s；源文件 `link_driver.rs` 已恢复。`native-probe/ac-response-run.log` SHA256 `c9a1c8d441291f1992aab5d3a27307c46be9f12a6502074ed37d915f4c60d9b4`。这条诊断证明了纯标量切片的真实执行；常规链接入口、实际组合能力及阶段签章仍待验证。链接入口差额按 Design59 的 AC 后端供给核归属，不把诊断产物计为正式 V-AC-STAGED 通过。

2026-11-20 /goal work，r6 / T-05阶段检查点，`a09237e845125f9c539f31448a9928384a6ce841`。按 Rust 帧模型还原失败 Pratt 帧的完整入口状态，再向父帧传播；来自子规则的失败直接退出 Pratt 帧。恢复节点共用 CST 子节点范围计算，并去除 unwind 中对同一父帧的重复 hint。原预算下 patterns 和 types 的原始正例与截断近邻通过（分别1.584s、1.369s）。对照的 Rust CST 各漏一个位于根尾部首个 error 前的 pending 空格，输入独立确认 offset 194/198。oracle 仅在这两个截断样本补明确写出的原字节跨度/trivia 边，其他 CST 行、诊断和恢复跨度仍逐字节对照；Auto driver 另行检查完整字节覆盖及回放。差分 `recovery-structural-diff.log` SHA256 `bbaa24355effab2ce5d95dfdba7e36d43fb65de1400c2f075336a0df375a2120`。同源制备目标再运行 `cargo nextest run --offline -p ace-aac-syntax-conformance --test conformance --test-threads=1 --status-level pass --no-fail-fast -E '!test(aac1046_corpus_text)'`，退出0，36/36，15.105s，1项具名排除不计通过；新增全语料用例最长1.751s，发现/外层清理未拆。结果 `recovery-unwind-lossless-tests.log` SHA256 `449ee0aaa95ebd365d05a1e33b889299d04972f0201c7a398e5d98b05c168517`。仅需修复的格式串正/反例仍在工作树。

2026-11-20 /goal work，r6 / D-1046-04。以相同原始 `.case.ad` 源调用公开 `parse_program_with(SourceProfile::Auto)`：944 字节正例接受、3 项；943 字节截断例正式拒绝 `source.syntax`，所报 952..952 已落在改写文本坐标。命令 `cargo run --offline --bin format_front_oracle`，退出0，首次准备54.03s；`format-front-oracle.log` SHA256 `7dcc4b99fc35c81531740d6075579b84924a9e8160ea04d15a528dd331d32c86`。prepared A2R 默认近邻全探针首次 `34 PASS / 3 FAIL`，16.783s，日志 `prepared-native-recovery-unwind-1-tests.log` SHA256 `136f78d8b0aca9ba86f9be5d4ed23ff479807dd1d39dc740c671cb217dc3565e`。旧制备产物的诊断重放比较 16,777,216 与 67,108,864 WU；格式串正/反例均在 `project` 和 `parse` 写产物前拒绝，patterns 截断可产 AST 且调高预算能产 CST。该重放只用于区分失败阶段，正式验收预算保持；日志 `diagnose-recovery-budget.log` SHA256 `d73289a1036a37516417c160493e71712c9791a3516bc8713159f4ee2cef0341`。

`stage: work | plan_id: PLAN-1046 | plan_revision: r6 | outcome: needs_replan | worktree_path: D:/autostack/.wt/lang-1046/auto-lang | branch: plan-1046-dev | base_commit: dcd4fdafd412be911dc44ba1fee911a916d9c198 | code_commit: a09237e845125f9c539f31448a9928384a6ce841 | dependency_revisions: 1044/1045 archived; grammar, runtime and stdlib from pinned base | task_ids: T-02/T-03/T-05/T-09/T-06 | evidence: D-1046-04 and 36 prepared-native passes in §9 | blockers: raw-only grammar cannot meet lossless and public FStr admission together; Windows AC standard linker error 206 remains named evidence | next_responsible_role: new`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r6 | round: 1 | from: work | to: new | status: executing | outcome: needs_replan | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: a09237e845125f9c539f31448a9928384a6ce841 | next: new`

2026-11-20 /goal new。r7 只修订既有目标的实现路径。用同一树 `a09237e845125f9c539f31448a9928384a6ce841` 实读 Rust 公开格式串入口、AAC 冷/热路径、原始文本及冻结 Spec；确认唯一代码 WIP 为格式串反例注册。§5–8 已确定 sparse 映射优先、先走实际 cold 链的有界可行性见证与 T-05/T-09/T-06 的责任。原4项完成保留，受影响的历史证据需在 r7 运行重验。此步只改计划合同，未改实现、未执行 r7 验收。原 Goal 授权覆盖本修订。

`stage: new | plan_id: PLAN-1046 | plan_revision: r7 | outcome: pass | changed: T-05/T-09/T-06, AC-01/02/03/07, AC-CL-FAMILY, V-FSTR-01..03, SD-01 | preserved: 4/9, same source/compatibility/resource and delivery scope | next: work`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r7 | round: 2 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: a09237e845125f9c539f31448a9928384a6ce841 | next: work`

2026-11-20 /goal work r7，T-05 cold 路径检查点 `f939ded4fcceb64bd899ef924c2bf103f6ba7b20`。原始 AutoDown `text-nul-and-format` 全样本及截断例由 Auto 处理，生成格式串只在解析输入中归一化，公开 CST 从原始快照精确回放；独立 Rust 前门参与接受性对照，`desugar_fstrings` + Rust AST 拓扑对照，两个相邻调用在原始位置按源序出现，生成 `__fstr` 叶含独立 flag。初始路径在默认 16,777,216 WU 拒绝。生成 Rust 副本阶段观测定位三处开销（探针不进分支）：全样本 parser 约 66.14M WU、CST 拼串约 59.58M WU，截断触发 `saves=50672`。修正 memo 清表后计数和重复键、保存区随帧释放、AST/CST 平衡拼接后，全/截 parser 18.76M/33.86M；逐函数观测证明 `peek` 每次线性查 scanner trivia 是首因（全/截各 11.14M/20.14M WU 直接费用），改为 scanner 预编译 token trivia 位并复用已完全初始化的帧区后，默认预算全通过。诊断 `fstr-fuel-probe-3/full-profile.log` SHA256 `c8e811299419cc38c5b169c11c942542d09e5f48c2a97a98cb61f76e469dcB4e`，截断 `58c629f8ae476e0795c72c4975a1c07a1975f44831f739968ac5fd1fd9b1d6e3`。正常 prepared A2R 测试 37/37、0 skip，24.160s，格式串 9.156s，Rust 准备工具构建 1.19s。`prepared-native-fstr-4-tests.log` SHA256 `25b5d3aacf06963fbf14b8709c2eb524a635be16301d428f8046d5520c543deb`。

同轮 `16ee06bcb5389abfa4f0919c76c862ebf9368c09`。剪掉文件末声明闭括号的 943 字节近邻确认原始零宽 EOF 诊断；原作者缺插值闭 `}` 或引号也产无损错误 CST，原始字节诊断与公开 Rust 阶段对照。新增映射缓冲在写入前检查同一工作预算，128 bytes 控制保持资源拒绝。完整命令 `cargo nextest run --offline -p ace-aac-syntax-conformance --test conformance --test-threads=1 --status-level pass --no-fail-fast`（driver 独立 `prepare-native`、绑定源指纹）。38/38、0 skip，22.336s，格式串全文件 7.543s，分隔符/预算 0.562s，Rust 准备工具构建 1.33s；独立 native 准备总时、发现/外层清理未拆。`prepared-native-fstr-5-tests.log` SHA256 `94f240653608e5586ee5463245c59ce937c3e494e84ed97270e60fc24d6b3ed4`。仍需完成 V-FSTR-01 剩余 profile 近邻、V-FSTR-03 热路径及 T-09 consumer 位置和分类，进度 4/9。

2026-11-20 /goal work r7，`5a20cd937ecd6b3e0ad25fcc1c707b8a3365322d`。格式串热路径用原始换行窗口核旧 token、格式串范围和普通引号/注释状态，只归一化变更窗，分别平移原字节坐标和虚拟 token 坐标。初始版本 40/40，随后工作预算在组装新流、索引和标记之前准入，原始冷/热同源。完整 40/40、0 skip，30.491s；准备59.8724578s。格式串 edit/undo/emoji 前缀 4.510s，实际 `ok=1 fb=0` 且 `grafted>0`、归一化扫描量等于改动行；未闭合回退 `fb=1` 且不嫁接，原始部分树与冷对账。格式形态 0.768s，Auto `$ident`/`${expr}`/空串/转义、Scheduler `{ident}` 与 scalar 不准入，对照显式 `__fstr` 与评论中的 `f"`。首个形态样本的内联块注释被 Rust 正式前门拒绝为不支持的 statement，属 fixture 准入错误，改为正常行注释。`prepared-native-fstr-edit-2-tests.log` SHA256 `b1c35e0860b298c3a33f216202d6500779c33280f07c521685ca6cf6ba5d657c`。

2026-11-20 /goal work r7，`fc3785af7b54c42b3a717d650be94beeed0bb4d5`。真实冷、编辑恢复 driver 调用原始快照行索引，CRLF/emoji 的原 byte、UTF-8/UTF-16 列和拒绝编辑保旧与独立 Rust `LineIndex` 通过。模块目录用来源/AST owner 行区分函数体和全局值初始化候选依赖，正例同时保留 `first↔second` 和 `alpha↔beta`，语义拒绝后者交 1047/1048。全量首次 42/42、0 skip，33.812s，但 `aac1046_corpus_text` 10.480s 超日常单例限，日志 `prepared-native-source-pos-1-complete.log` SHA256 `6ba5c5e6c000dda5339561af54b201fae7ffacee1837ffe1dc763995a0e972f2`。合并同一个正例的 AST 和原始字节重建观察到同一次真实解析，截断仍独立验证。最终同入口 `prepare-native` 94.2932099s，完整 `cargo nextest run --offline -p ace-aac-syntax-conformance --test conformance --test-threads=1 --status-level pass --no-fail-fast` 退出0，42/42、0 skip，26.257s。格式串全样 5.694s、分隔符 0.516s、编辑 3.742s、方言 0.778s、位置 0.343s、模块分类 0.231s，发现/外层清理未拆。`prepared-native-source-pos-2-tests.log` SHA256 `aa137753a0720114a5ef01a7a8326c7178d75541435cf0818ddcbb95b0ffbbc1`。V-FSTR-01..03 完成实际对照，D-1046-04 关闭；源码位置与分类输入 V-05 通过。T-05 的 CL 组合仍开着，按任务依赖保留 4/9。

2026-11-20 /goal work r7，V-CL-FAMILY 新证据。以现有 A2R `parse`、`catalog` 入口在同一份两模块输入中使用真实 `#[api(...)]`、泛型与 Option/Result/Future、`#in` 和 AGE Node。正式已装入的三个候选 profile (`auto`、`packages-ct`、`age-package`) 都给 `MODULE_ERR SYNTAX entry.at`。拆分定位：generic=age-package/packages-ct 绿，types=auto 绿，CT=packages-ct 绿，Node=age-package 绿；组合类型签名与声明标注在三者均红。原因证据为 `experimental/ace/grammars/auto-source.at` 的 `item`/`fn-tail`/`ann-ty`/`ct-item` 互异 profile 门控，及当前共享 grammar 无声明标注生产式。`cl-family-profiles.log` SHA256 `1dfbc364996c529e991b3768c24181c622285ad4d83aa700f78dd8398d7fb307`。

仅在材料目录上实测兼容路线：用共享 DSL 的具名组合 profile（既有五面门控并集）编译实际 grammar，34885 bytes、324 rules、125 tokens，独立 Rust runtime 可接受组合类型、CT 与 Node，标注仍12项诊断。再添加仅该 profile 的 `#[api]`/参数键值/真实声明 owner 原型，35357 bytes、331 rules，组合模块 `entry`、Node 与合法近邻零诊断；同源删 `]` 后12项诊断。命令 `cargo run --offline --bin cl_family_preflight` 和 `--bin cl_family_attribute_preflight` 两探针退出0。日志 `cl-family-union.log` SHA256 `3e5c11ce5c3e2ba7eb601c1d4dc03568f9181f74aa1afaad31041e296d758b54`，`cl-family-attribute-union.log` `e5315ec2e5ece8d73c3167a19985468a8e405dc0114ae61971069d9da229e7c3`。试验仅证明共享表设计可行，真实 AAC grammar 生成/解析、所有本版 required 标注语法、module/field/parameter 来源及组合验证仍需修订合同后按实际 owner 交付。

真实双模块目录对照 `cl-family-catalog.log` SHA256 `dd269523924e91cb7a0fb29885087a8c4a14c03392bd0a8311b863e57e829cfd`。r7 同轮常规 AC 重放：原探针宿主栈溢出，诊断宿主 worker 用 16MiB 栈以后，组装和前端编译均过，进入真实 `MnDriver` 后仍由常规 `rust-lld` 启动返回 OS 206。无生产链接改动或 response file 替换。`ac-minimal-r7-stack16m-replay.log` SHA256 `77fe8bf96cd598b1c36b33768044020e4b86fabf364dffc70fb1e5922ed22d19`，退出1。保留为 T-06 正式供给差额。

`stage: work | plan_id: PLAN-1046 | plan_revision: r7 | outcome: needs_replan | worktree_path: D:/autostack/.wt/lang-1046/auto-lang | branch: plan-1046-dev | base_commit: dcd4fdafd412be911dc44ba1fee911a916d9c198 | code_commit: fc3785af7b54c42b3a717d650be94beeed0bb4d5 | dependency_revisions: 1044/1045 archived; grammar/runtime/stdlib pinned in the worktree | task_ids: T-02/T-03/T-05/T-06/T-09 | evidence: D-1046-04 closed, 42 native A2R passes, r7 CL profile and attribute grammar preflight | blockers: D-1046-05 combined grammar/profile and attribute ownership contract; standard Windows AC linker supply recorded for T-06 | next_responsible_role: new`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r7 | round: 3 | from: work | to: new | status: executing | outcome: needs_replan | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: fc3785af7b54c42b3a717d650be94beeed0bb4d5 | next: new`

2026-11-20 /goal new r8。专树 `fc3785af7b54c42b3a717d650be94beeed0bb4d5` 干净，实读 §4 的共享 grammar gate、`profiles_927`、L-35、代码的 `parse_fn_annotations`、工具/Web/FFI 既有事实。以 r7 的正负原型定具名组合 profile 与源码所有者合同，补兼容、形态及组合判据。已见正常 Windows AC 的 OS 206，继续由 T-06 核实际供给，当前组合设计和实现可独立继续。此步仅改 Plan，未执行 r8 代码或验收；保留 4/9、原后继 owner 与授权。

`stage: new | plan_id: PLAN-1046 | plan_revision: r8 | outcome: pass | changed: T-05/T-06, AC-CL-FAMILY, V-CL-FAMILY/ANNOTATIONS/PROFILES, SD-01/03 | preserved: 4/9, original scope and budget | next: work`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r8 | round: 4 | from: new | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: fc3785af7b54c42b3a717d650be94beeed0bb4d5 | next: work`

2026-11-20 /goal work r8 检查点。专树从 `384609645` 合入了 master 的 PLAN-1084 动态对象基线（`23f0626ea`），组合实现 `5ad621b47`，再生成 `88db0e30e`。以进入专树的同一基面枚举 `declared_profiles` 并逐一 `compile_profile` 对照完整 canonical 字节，旧 26 个全部相同；新 `aac` 为第 27 个，36,932 字节，GrammarId `f6734f8a672f0c9238a143a53ba382afea27ab4493c3789b3a8e3a759e6f1081`，canonical SHA256 `c1a18b7765b383880760c6009580b7962a95a6a5ff87645716c47fcce4c66f5a`。明细 `r8-final-1084-compatibility.log` SHA256 `e663dcb0ff7571853eaf6d9e6963f5903740c6097e24912d9bd5ac07302dc998`。组合仅扩 gate，按 grammar 编写顺序保先后；坏引用、空/重复/自身/组合成员拒绝。刚更换基面，下面 45 项和有界 AC 结果是旧 `384609645` 上的第一次完整证明，最终基面重验单列。

在 `384609645` 基面以 `cargo run --offline -p ace-aac-syntax-conformance --bin prepare-native -- <绝对材料目录>` 准备 A2R driver，第一次运行前生成文件由 CRLF 归一成 LF，产物源指纹因此失效并实际拒绝（`prepared-native-r8-combined-2-tests.log` SHA256 `1e23bd8475dcdda3b2127ce0966d43bb335d223e7e493476255015f1e0c0b157`）。用归一后的源码重新准备 66.318s，`cargo nextest run --offline -p ace-aac-syntax-conformance --test conformance --test-threads=1 --status-level pass --no-fail-fast` 退出 0，45/45、0 skip、36.023s。真实 `catalog_directory` 分开读取两文件，按各自来源与 Rust 参考树对账，并拒绝坏括号、断类型与参数槽标注；单独验证字段与函数归属、标注参数、值泛型实参及 `#{...}`。全部原位置、恢复、格式串和嫁接用例同轮执行。格式串全样 5.839s、编辑 5.346s。`prepared-native-r8-combined-3-tests.log` SHA256 `8fc8eeff1721632004981f2276c138e08047e38c574f61e3898901876deadbd4`。

Windows AC 旧基面重放先消除了 OS 206：`link_only` 在 UTF-16 命令行上界过长时写入独占临时 response file，由原 `rust-lld -flavor link` 读取，并沿原发布/回滚路径回收。短路径、转义和长命令行的 Windows 命名单测通过，`r8-coff-response-unit.log` SHA256 `d48c3ff4cb0240dc1d0e0201a2373928369797f3850f4a9a9c031094f72404c4`。无截断诊断运行约 20s、stderr 约 5.96 MB；沿 text runtime 既有 trace 开关在此验收显式关闭逐操作追踪，保默认模式、收集摘要及正式 5s/1 MiB 限额。`cargo run --offline --release --bin ac_combined_bounded_r8` 用 `AssemblyBackend::Ac` 组装实际 Auto parser、调用 `MnDriver::ephemeral().build_collections_checked` 和 `run_pe`；含空格与中文的链接目录，2.398s，程序 4.053s、exit 0、stderr 517 字节，无残余 response file。二进制内的 Auto parser 比对同两份原始源码独立 Rust canonical CST（8,781 / 2,307 字节），同文件坏形必须产诊断。宿主 Gen0 前端 9.299s，首次 release 准备 4m55s。`ac-combined-bounded-r8.log` SHA256 `27fcd7e990bc38774a72d0314d49ebe612ff909094e3300655ef01a8aa2096ac`。目录、AST owner 与位置消费另由上述 A2R 测试执行。

2026-11-20 r8 更新基面复验，`23f0626ea` → `7e4a7f58437a92e0984ee40e4ed97624abf9aa11`。A2R 源绑定 `38a90a99ef86a3419ad5d5c5af0c9e7154ffe8e454d05465bce96e169a064249`，exe BLAKE3 `8c510c44c1d81e6106fbddde32245b1f194fdbe8bf45f7b4d8721d613e09ddef`，准备 158.130s，完整 conformance 退出 0，45/45、0 skip、34.393s。正例原字节、两文件及位置/错误的独立 oracle 不变，组合用例 1.323s；分别删类型边、删标注边和篡改为另一 sourceId，参考树比较均拒绝。`r8-1084-a2r-tests.log` SHA256 `9a5ea239ef7f544b753944c4a57eb9d23de62fa3b1d7373879223d523477db0e`。T-05/T-09 交付，D-1046-05 关闭。Windows AC 再跑 `ac_combined_1084_r8`，准备构建 2m54s、Gen0 前端 16.809s、正常链接 2.498s、进程 3.758s、exit 0、stderr 517 bytes。`r8-1084-ac-bounded.log` SHA256 `217388745622096777098f04a3c9d1688aced35530a71b800de45a9693e207d7`。

冻结分母 `cl-requirements.tsv` 本版 42 行。以下登记此阶段语法产物与 Design59 实际消费者的交接：

| CL | 1046 实测产物 | 后续证据生产任务 |
|---|---|---|
| CL-101..106 | `module_provider_and_conditional_catalog`、`module_catalog_missing_duplicate_and_import_cycle`、`catalog_distinguishes_body_and_initializer_edges`、`corpus_numbers/patterns/types/collections`、`positions_use_byte_and_utf16_units`、恢复诊断坐标。保跨文件、数据及聚合节点。 | 1047 T-02..04/T-09 绑定与初始化类别，1050/1051 数值、布局、执行 |
| CL-201..205 | `ast_type_roles_match_rust_projection`、`aac_combined_modules_preserve_source_owners`、`corpus_types` 和坏类型反控。保 `?T`、`!T`、泛型及嵌套节点、错误分支源码。 | 1047 类型，1049 T-02..05 语义与清理，1050/1051 对应执行 |
| CL-301..304 | `typed_age_and_scheduler_profile_parity`、`ast_type_roles_match_rust_projection`，`~Iter`、`task`、`go`、`await`、`yield` 及来源。 | 1067 正式调度，1049 T-03/04/09、1050 T-03/09、1051 机器供给，1068 联合验收 |
| CL-401..404 | `aac_preserves_const_arguments_constraints_and_ct_expressions`、`corpus_types/functions/collections` 及多模块类型树。保类型和值实参、`spec`、约束和成员字段。 | 1047 签名，1048 T-02 实例化与约束，1050/1051 布局和调用 |
| CL-501..505 | `#in` 的两个分支和路径各自可读，`#{ log2<2>(21) }` 源码可投影，截断产生原位置语法错误及保旧。 | 1048 T-03..05/T-09 求值与重绑定，1055 基础 LowerCt，1050 T-09 完整 LowerCt，1051 T-04 执行 |
| CL-601..602 | 无参、多标注、命名/位置参数、声明/字段/类型所有者，及错位置、坏括号、删边、错 sourceId 的反控。 | 1047 T-09、1048 T-03/T-09、1049 T-09，中间语义及 971/967/980/982 消费者的实际效果 |
| CL-701..705 | `corpus_collections/functions/calls/patterns`、parse battery、真实资源拒绝与故障保旧。输出原表达式、闭包/调用、数据和控制节点。 | 1049 T-02..05/T-09，1050 T-03/04/09，1051 机器执行 |
| CL-801..805 | `corpus_text`、格式串正反近邻、`source_bytes_and_lossless_reconstruction`、AGE AST、组合目录及 `#[c,vm,rs]`/`#[export(system)]`。 | 1045/1056 启动库，967 ABI，1051/1052 正式宿主，1097 AC UI/AGE |
| CL-901..906 | 源绑定 A2R 及 Windows AC 阶段的 Auto 解析、独立参考树、默认预算及 named negative。 | 1053 全 CL、1054 各代实证、1068 最终三端。Linux/WSL 依仓库裁定在专用 Linux 环境验。 |

T-06 固定组冻结于 `7e4a7f58437a92e0984ee40e4ed97624abf9aa11`。`cargo t ace-syntax ace-source ace-aac-syntax --prepare-only --out <scope>/r8-1084-fixed-prepared` 退出 0，110.579s。用同组 `--run-prepared <prepared.json> --out <scope>/r8-1084-fixed-tests`，四线程，父命令退出失败，总时 39.400s。`ace-syntax` 159/159，0 排除（发现 2.399s，执行命令 5.282s）；`ace-aac-syntax` 45/45，0 排除（发现 1.262s，执行命令 11.022s，最长用例 7.039s）；`ace-source` 930 过、1 红、1 原条件排除（执行命令 17.689s）。清理由工具随阶段合计。原红是 `scalar_source::deep_not_chain_reaches_checked_bundle`，20,000 `!` 的 Rust 源码固定用例，Windows stack overflow `0xc00000fd`。在同一专树暂离到干净基面 `23f0626ea`、同用例同配置重放，6.842s 同错，随后返回专树计划分支；专树的 16 MiB Rust 测试栈诊断亦同错。原红独立登记，保实际 FAIL 终态。`results.json` SHA256 `aa0deecb90bbe580dd416117bfa32fedf330042bb950006ec64c54ffaad917c3`，`r8-ace-source-23f-baseline.log` `9598a8298ee4514b4d032da42f438af14371536654f3a42165df6e81c985499f`，`r8-ace-source-16m-diagnostic.log` `fb2ef8998f8b74d73db4382fec7a360824a28b8211d352dc1a5acd1a7b1aa053`。

Windows AC 位置验收重放 `ac_positions_pair_r8`。同 exe 用 CRLF/emoji 的合法格式串和缺 `}` 版本，实际 Auto `diagnostic_positions_case` 检查合法空诊断及坏原字节 `49..50` 的完整坐标行 `10 49..50 2 35 33 2 36 34`，后者来自宿主独立 Rust `LineIndex`。准备编译 1.790s、Gen0 前端 6.875s、链接 1.711s、执行 0.546s、exit 0、stderr 505 bytes。初次探针使用 `std.text.split` 物化报告，触发 `MN-COLL-TRAP RESOURCE 22`；单输入重放均过，同 exe 改按完整坐标行匹配后也过，实际解析、位置函数、输入、限额和 oracle 等值保留。`r8-1084-ac-positions.log` SHA256 `4e678a1a4d272919d6203cefca08a67c593b5aa7df5c00871cd1de6b217c0b63`，`r8-ac-positions-pair.log` `3128c8c2792954096a8ea96bcc4924df1233b22f7ca48c7008e3164b5cf53353`。

T-06 AC 文件目录差额复现。对真实 `entry.at` / `views.at`，使用 `assemble_stdlib_sys_inputs`、同一个 Auto `catalog_directory`、标准 `MnDriver::build_collections_checked`。完整 sys（`ac_catalog_1084_r8`）与保留 `sysplat_win/sysplat_lnx` 条件接口和直接文件依赖的最小模块闭包（`ac_catalog_sys_1084_r8`）均通过源码组装，标准 MIR verification 在链接前返回 `host.duplicate: too many host declarations in one module`；去掉平台模块的诊断反控则在 bind 报 `source.module-missing`。直接观测最小闭包 `lower_checked_closures`，其 MIR 有 55 个不同 platform host 符号，现 `MAX_TOTAL_HOST_IMPORTS = 24`。`collections_lower` 当前亦没有 `PlatformCall` 实现，独立 console/platform 算法与 PLAN-1045 的 AV+Win32/A2R 收据覆盖各自入口。关闭要求沿 Design59 供给归属，由 PLAN-1051/T-03/04/09 提供 collections 与 Windows file op 可同时消费的正式实现和源身份，再按本计划同目录正、反例运行真实 AC。当前可核的独立 V-AC-STAGED 语法和源位置已通过，使用文件平台供给的目录分支具名受阻。完整 sys 日志 `r8-1084-ac-catalog.log` SHA256 `715f3cce56a2a3f70692626536ebcb825fdda9453a0260d94997b2d53aa7f5d3`，最小闭包 `r8-1084-ac-catalog-sys.log` `0024106fc6944d9c7c947f543bbaac599310409b387c4b9a064c712a70a176cf`，MIR 声明账 `r8-ac-catalog-mir.log` `d67b7a01b61495312923558263fd87a836a82d9501a95f1f1d818c21fc4cd7e8`。

补充 native 链接消费回归。`cargo t ace-native --prepare-only --out <scope>/r8-final-native-prepared` 退出 0，147.576s；用该清单的 archive/tool 环境，`cargo nextest run --archive-file <archive> --workspace-remap <ace> --ignore-default-filter -E '<ace-native 固定表达式> and not test(/linux_elf_1040::|console_out_linux_1041::/)' --no-fail-fast --no-tests fail --status-level pass`，四线程，退出 0，330/330、19 未选，执行汇总 77.902s；命名长 COFF 测试 0.013s。Linux 两模块本身 `#[cfg(target_os = "linux")]` 在 Windows archive 中不存在，Linux腿/WSL=裁定排除，归专用 Linux 环境。Nextest 自动回收临时解包目录。产物 `r8-final-native-prepared/prepared.json` SHA256 `789a1c349e06103d9cb78acafe77ff8856fe1d94516ec53e5b384fbec5edcfde`，`r8-final-native-windows-tests.log` `50cde58e297e5d341c890c28515fd4cc677ff1f6180ce6e881a12d033fa90d04`。

`stage: work | plan_id: PLAN-1046 | plan_revision: r8 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-1046/auto-lang | branch: plan-1046-dev | base_commit: 23f0626ea | code_commit: 7e4a7f58437a92e0984ee40e4ed97624abf9aa11 | dependency_revisions: PLAN-1044/1045/1067 archived; PLAN-1051 drafting r6 | task_ids: T-05/T-09 passed; T-06 pending | evidence: 26 canonical profile streams unchanged, ace-syntax 159 pass, ace-aac-syntax 45 pass, ace-native Windows 330 pass, AC syntax/diagnostic source oracle pass, ace-source base red reproduced | blockers: Windows AC catalog_directory requires PLAN-1051 collections+platform file host support; same positive and negative directory inputs must rerun | next_responsible_role: PLAN-1051 T-03/T-04/T-09, then PLAN-1046 work T-06`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r8 | round: 5 | from: work | to: blocked | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: 7e4a7f58437a92e0984ee40e4ed97624abf9aa11 | next: user`

2026-11-20 /goal round 6 外部前置复核。上一轮推进了 T-05/T-09 并首次实证 D-1046-06。本轮 master `de373d9d2` 自 `23f0626ea` 无 ACE 或 sys 运行路径变更，PLAN-1051 仍是 `drafting` r6、0/9，无计划分支、专树或正式供应收据。专树仍为洁净的 `7e4a7f584`。本次没有新增验收证据，沿用 §10 的双文件正反例解阻条件。

`stage: goal | plan_id: PLAN-1046 | plan_revision: r8 | round: 6 | from: blocked | to: blocked | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: 7e4a7f58437a92e0984ee40e4ed97624abf9aa11 | next: user`

2026-11-20 /goal round 7。主分支仍在 `de373d9d2`，PLAN-1051 仍是 `drafting` r6、0/9，无分支、专树或正式交付收据。专树 `7e4a7f584` 洁净。D-1046-06 连续三轮存在，待正式 Windows AC 文件宿主落主后，以原双文件正反例重跑 T-06。

`stage: goal | plan_id: PLAN-1046 | plan_revision: r8 | round: 7 | from: blocked | to: blocked | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: 7e4a7f58437a92e0984ee40e4ed97624abf9aa11 | next: user`

2026-11-20 /goal resume r8。用户确认 1051 已完成并继续。核实 1051 r6 服务代码与规范落主、计划已归档；本树同步到 `68fd4c59222c77a5de0963ff5827f1353abd7b4f`。Rebase 21 原提交，其中 20 内容等价，组合 grammar/member 与 fixed groups 冲突同时保留原 `aac` 和上游 `auto-packages`/`ace-aac-bridge`。生成表提交 `6f81d19ff`，基面所有 27 个旧 profile 完整 canonical bytes 相同；第 28 个 `aac` 37,165 字节，GrammarId `c7c66e540c36fca698dfc37e83815133088781d7f2b6aea94c23a9913ba8c7d1`。`r8-resume-range-diff.txt` SHA256 `ffe53b0b825fbea806a281e3612720085986389a2ee41c07fa46b0f59783e935`；`r8-resume-profile-compat.log` `94dc80cb374717ae58ac2789b469065453416eee934d974f247c47c37e437d8f`。

D-1046-06 关闭。普通 `build_collections_checked` 重放仍触发原 `host.duplicate`（`r8-resume-ac-original.log` SHA256 `0b212f22c7689b87eedb4d466e6fd3882e0b66b2a14131ab5d11efde761c9afa`），1051 的 `MnDriver::build_hosted_collections_checked` 正式接收同一 `assemble_stdlib_sys_inputs` CheckedModule，按入口函数闭包分单元准入并接真实 Win32 platform shim。保原 verifier 和上限。首次含报告写盘的探针超 5s，诊断复跑在建好空输出文件后返回 Windows `0xc00000fd`；改用 1051 的原生 `StrCmp`，保持完整值对照。准备 `runtime-dev`，运行用 `ACE_FIXED_TEST_RUNTIME_MODE=run-debug`、固定 5s/1 MiB，`cargo run --offline --bin ac_catalog_equal_r8`。真实 Windows 文件在中文和空格目录，复用同一个 PE，逐次更换原 `entry.at` bytes。正例完整 6,672 字节的目录报告必须与已预备 A2R 一致，两文件 AST 分别与独立 Rust 参考按源码、owner、跨度、字段和 flags 对账，两支条件 `USE` 验两行。正例 4.077046s 退出 0，缺标注 `]` 1.915737s、断返回类型 2.070426s、标注挪进参数槽 1.922465s，三坏形均以精确 `MODULE_ERR SYNTAX entry.at` 退出 11。Rust 探针构建 5.94s、Gen0 前端 53.283s、COFF 链接 13.598s，运行逐例计入写换输入、子进程与断言，未单拆发现/清理。运行时 BLAKE3 `3e901c9890bb0b9a5b43f6a48c9113065d709c45eec7072ffaaf1b063dcab345`。`r8-resume-ac-hosted.log` SHA256 `b24bc1bae3477292f88b826893e848cc278d15673a88a672dbcef7fb322bb418`，最终退出 0 的 `r8-resume-ac-equal.log` `44fc8fdd56b5aa2ea65e99abcbd083e40b79c03a8f66e67cbec775108c76437b`。

A2R `prepare-native` 52.054s 完成。首次重绑完整 conformance 43 过 2 红，运行 29.218s。两红为 `corpus_patterns` / `corpus_types`，旧测试预期假定特定 `{` 是根下错误叶；上游 grammar 现在在原位置识别 `{`，末尾截断仍报错。`a27a7c3c2` 移除旧的固定位置参考树补写，对同一完整/截断输入直接比较整棵 Rust 参考树，实际 native 入口持续校验原字节重建。命名复跑 2/2、2.157/2.072s、合计 4.229s，Rust 构建 1.37s，43 未选。初红 `r8-resume-a2r-tests.log` SHA256 `3eb26481340a188c6823a28b56b6249141c951f5b9980953ff2567ea69c8f9f1`，复跑 `r8-resume-recovery-direct-oracle.log` `da8342253169522d0b7cb6d005ba38f1e400aae142e8ebeae66523618b7c3fa4`。本轮未新增单测；T-06 尚需重绑受影响固定组。

T-06 共享语料 AC 复验走 1051 hosted 同一 PE，实际载入 8 份原始 AutoDown 输入的完整及截断近邻，以及 CRLF/emoji 位置，共 17 项。`ac_corpus_resume_r8` 用 `parse_case` 比较完整 canonical 和诊断，7 组非格式串用独立 Rust 源语法输出；格式串用真正 A2R 的原字节分层对照，Rust 独立判原始合法/截断及坐标。初跑执行通用 AC 的 5s 上限，15/17，14 项完整输出（0.831–2.639s）、位置项 3.097s 通过，两个长格式串在生成大文本报告时超时。诊断重跑延长仅定位，原 canonical 两份实际等值退出 0，6.329/7.797s。调研串行报告后，按 `tests/aac-syntax/src/native.rs::execute_report` 本计划既有每 child 8s 限额、1 MiB 捕获和每例 10s 纪律，原表和生产预算保持。完整样本的报告 42,574 字节等值 5.652s；截断文本报告重跑仍超 8s，改直接逐节点消费实际 parser 的 CST，961 节点的 depth/kind/flags/两端原始 span/expected/顺序字段全对，逐 byte 重建 943 原始 bytes，且比较原末 EOF 的完整诊断。截断这套结构对照 7.343s 退出 0。完整同结构探针 989 节点 4.060s。用实际声明、原始输入、独立坐标和同一字段全部对照，待交 17 项遂各有有界运行证据。语料准备 Gen0 47.460s/链接 10.343s，紧凑结构准备 42.343s/链接 10.710s，额外 8s replay 的 Rust 观察工具构建 2.07s。首轮 `r8-resume-ac-corpus.log` SHA256 `27cde0c5325e40942dc4fed95d7f8eb40023958eb3934eb95aa7874b97067c19`，结构 `r8-resume-ac-format-compact.log` `171eda67d13fa712c369b8563f0945c3d9a6ac1a91b6f56cb97dc506c89f7da2`，重放 `r8-resume-ac-format-eight-second-check.log` `e6aea9c29f8bbeb2cb0a5614328800d9c4efe3f2da09f691f96cc6cb12ba1552`。试改报告渲染只在临时探针验证，因 AC trap/超时未采纳。

固定组准备恢复记录。默认准备阶段会启动 release 工具，核实后终止自有准备树，改 `--prepare-only --debug-tools`。四组合用一个准备清单时，`ace-native` 的后置共有 `mn`/`mn-collections` 工具锁定文件覆盖 `ace-source` 的第一次锁定，正式运行 fail closed `PREPARATION_REQUIRED`。按不同工具列表用 `cargo t ace-syntax ace-source ace-aac-syntax` 与 `cargo t ace-native` 分别 `--prepare-only --debug-tools --out <绝对材料路径>` / `--run-prepared <清单> --out <绝对材料路径>`；固定组选集、Linux/WSL 原裁定、性能过滤保持。首次等待其他工作树编译 485.418s，debug 重试准备/执行分别 30.582/30.402s，拆分后 source 准备等待 363.709s，均另计。拒绝 `r8-resume-debug-fixed-tests.log` SHA256 `da2c23e2f0c7da6fe07c6631ce363f9d9bcabb81c6185ee9a746583e54bc3c61`，拆分等待 `r8-resume-split-fixed-wait.log` `222142af05fed46209470fba1079ff35f869c89ac3a5404cc06544d4e6a22066`。

`a27a7c3c295c8fb248a776edc0fd46ae6d7929ca` 的 source 分组准备 3.794s，闸命令 36.069s，父和实际选中子项全部通过：`ace-syntax` 159/159，0 排除，发现 2.144s、执行 5.386s；`ace-source` 932/932，1 原条件排除，发现 1.692s、执行 14.558s，上一基面 `deep_not_chain_reaches_checked_bundle` 此基面 0.490s 已通过；`ace-aac-syntax` 45/45，0 排除，发现 1.265s、执行 10.958s。清理随工具合计，未独立拆。`r8-resume-split-source-tests/results.json` SHA256 `64cc76bd26725812d07833b95fd5524fc3021b9d4503d717b3823b1b668b5945`。

同轮 `ace-aac-syntax-conformance::conformance` 45 项（`aac1046_` 前缀省略，含 case 内准备/运行/清理，三位小数秒）：

- profile 和 probes：`grammar_data_identity_binding` 0.250、`decode_all_profiles` 1.132、`decode_truncated_refused` 0.168、`decode_dbg_probe` 0.218、`hex_bytes_roundtrip` 0.138、`byte_len_probe` 0.326、`bucket_probe` 0.116、`raw_probe` 0.117、`scan_probe` 0.082、`stage_probes` 0.164。
- CST/编辑：`parse_parity_scalar` 0.148、`parse_parity_battery` 0.937、`threefn_parity` 0.293、`truncated_stream` 0.174、`singlefn_midexpr` 0.144、`midexpr_eof` 0.151、`source_bytes_and_lossless_reconstruction` 0.198、`depth_and_work_budget_admission` 0.353、`edit_incremental_equality` 0.339、`identity_change_restore` 0.139、`edit_unicode_span_and_undo` 0.435、`rejected_edit_retains_snapshot_bytes` 0.219。
- 投影/目录：`typed_age_and_scheduler_profile_parity` 0.852、`ast_source_and_domain_projection` 0.679、`ast_type_roles_match_rust_projection` 0.236、`incremental_ast_retains_origin_and_byte_spans` 0.323、`module_provider_and_conditional_catalog` 0.509、`module_catalog_missing_duplicate_and_import_cycle` 0.562、`catalog_distinguishes_body_and_initializer_edges` 0.240、`aac_combined_modules_preserve_source_owners` 1.604、`aac_attributes_keep_shapes_and_field_targets` 0.565、`aac_preserves_const_arguments_constraints_and_ct_expressions` 1.050。
- 共享语料/格式/位置：`corpus_basics` 0.381、`corpus_calls` 1.228、`corpus_collections` 0.798、`corpus_functions` 0.644、`corpus_numbers` 2.083、`corpus_patterns` 2.233、`corpus_types` 1.761、`corpus_text` 6.940、`format_delimiters_report_original_bytes` 0.697、`format_edits_match_cold_with_reuse` 4.958、`format_modes_keep_only_authored_interpolations` 0.771、`recovered_diagnostic_positions_match_raw_snapshot` 0.468、`positions_use_byte_and_utf16_units` 0.410。

同冻结提交 `ace-native` 独立 debug 准备 3.470s，固定组执行父命令退出 0，330/330，Nextest 19 未选，汇总 84.770s，发现 6.254s、执行驱动 92.140s、整命令 98.447s。Linux/WSL 两个实际模块由 Windows `cfg` 排除，仍归专用 Linux 环境。`r8-resume-split-native-tests/results.json` SHA256 `e83f414bd175dbeff0bdb14bf799b2ee97247557268e720adde4376cf1bedb5d`，`0/run.stderr` `0b7ef17487519c38fdf6f169eaeba47730eb3f4d2d77dacd3bb71eb9c9263313`。四组 Linux 原始条件与试验前后身份一致。前端与 native 的各受影响职责均验证，候选交接按 §5 SD-01..04。

V-AC-STAGED 的分母反控采用 PLAN-1044 正式 `6022f10f2` 冻结收据，已按实际源码核 `v02_missing_identity_missing_cl_row_and_fake_na_are_red` 和 `v04_restore_required_with_real_evidence_admits`。该版本的校验器和 42 行 CL 分母本树字节未改。A 基础收据是 `95c7fbad8`，未交宿主时旧基面同目录具名失败，独立语法已通过。1051 落主后同原始目录正反例实跑通过。复用的 V-02/V-04 是 A 收据分母反控，当前产品身份以新 grammar、AC runtime 和本轮产品收据核验。§6 其余 work 条件的当前结果与旧版已交验收边界见 §9。

`stage: work | plan_id: PLAN-1046 | plan_revision: r8 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1046/auto-lang | branch: plan-1046-dev | base_commit: 68fd4c59222c77a5de0963ff5827f1353abd7b4f | code_commit: a27a7c3c295c8fb248a776edc0fd46ae6d7929ca | dependency_revisions: PLAN-1044/1045/1067/1051 archived, 1051 native delivery 1d949f30b | task_ids: T-06 done; T-01..05/T-09 reverified | evidence: real hosted Windows AC 4 catalog checks plus 17 corpus/position checks, ace-syntax 159, ace-source 932, ace-aac-syntax 45, ace-native 330, 27 unchanged old profiles | blockers: none | next_responsible_role: independent final review T-07`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r8 | round: 8 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: a27a7c3c295c8fb248a776edc0fd46ae6d7929ca | next: review`

2026-11-20 T-07 最终复审第 1 轮，结论 `needs_fix`。在实施会话内从冻结提交、正式产物和独立选择的反例重建结论，按 §6 未另开审查会话。专树代码洁净，`git diff --check 68fd4c592..a27a7c3c2` 通过。复核了 V-02/V-04/V-05、V-FSTR-01..03、V-CL 现有分项和四组冻结收据。AC-02、AC-07 和当前 AC-CL-FAMILY 分项的证据仍成立。AC-AC-STAGED 的 1051 文件宿主正反例仍有效。AC-01/03 有模块目录反例，AC-04 需重绑受影响的目录用例与固定组，AC-05 本轮未通过，AC-06 待复审。

**R1046-01（P1，T-04，V-01/V-03）**：`catalog_directory` 的 `import_report` 只遍历 `use-path`，AAC 组合 grammar 同时接受 `age-use-sym-item`（`use beta: y, z`）和 `age-use-module-item`（`use app.beta: y, z`）。参考前门 `core/src/source/cst/frontdoor.rs` 将二者分别投影为每成员二段/三段导入。同一冻结 A2R 入口 `AAC1046_CASE=catalog`、`AAC1046_PROFILE=aac`、`AAC1046_PACKAGE=app`，输入 `alpha.at` 与可选 `beta.at` 执行八例。存在 beta 时，点分控制输出 `USE 0 -1 beta.y`，两类符号表导入各有 AST 但没有 `USE`。删除 beta，点分控制返回 `MODULE_ERR MISSING beta`，两类符号表导入均返回 `CATALOG`。alpha↔beta 双向点分返回 `MODULE_ERR IMPORT_CYCLE`，符号表形返回 `CATALOG`。修复用现有 AST 字段取模块名和成员，保原顺序、来源、条件和 provider/public 规则，经同一依赖图；真目录入口覆盖两类正形、缺失和循环近邻。

**R1046-02（P3，T-05，AC-05）**：新增 `tests/aac-syntax/tests/probe.rs`、`rustprobe.rs` 只输出临时诊断。`Cargo.toml` 设置 `autotests = false` 且只注册 `conformance`，两文件无引用。移出交付源码。`link_driver` 回收失败信息属于异常路径生产诊断。冻结组日志无本包引入的未处理 compiler warning。

复审取证 `r9-catalog-import-shapes/results.json` SHA256 `aa5dcd18bd7c43799d581653d93fedec51884476ff05aa7bd1c65db02930415d`，同目录保原输入和完整报告 stdout/stderr。driver `prepared-native-r8-1051/aac1046-driver.exe` SHA256 `0fa30298a62259e2cf692f960d394a6b026cfbc303ba54fe410d01ff1ab3c8ac`。末次提交仅改 Rust 参考测试，Auto driver 输入未变。八例每例进程退出 0 且 `value:0`，产品结果由报告字段判读，0.150–0.757s，child 限额 8s。SD-01..04 的目标路径已核，SD-01/04 的目录契约待修复后据实际行为复核。原 T-06 的 1466 个已通过检查按对应提交保留。

`stage: review | review_scope: final T-07 | reviewer_identity/limitations: Codex in implementation conversation, independently selected inputs and replayed frozen artifacts, no separate reviewer | plan_id: PLAN-1046 | plan_revision: r8 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1046/auto-lang | branch: plan-1046-dev | reviewed_commit: a27a7c3c295c8fb248a776edc0fd46ae6d7929ca | base_commit: 68fd4c59222c77a5de0963ff5827f1353abd7b4f | dependency_revisions: 1044/1045/1067/1051 archived, 1051 native delivery 1d949f30b | spec_inputs: SD-01..04 audited, accepted draft pending repair | acceptance_results: AC-01/03 failed catalog controls, AC-04 affected evidence pending, AC-02/07 and AC-AC-STAGED and AC-CL-FAMILY existing controls pass, AC-05 needs_fix, AC-06 pending | pending_closeout_gates: affected T-06 then T-07/T-08 | findings: R1046-01, R1046-02 | evidence: r9-catalog-import-shapes/results.json and raw reports | next: work T-04/T-05/T-06`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r8 | round: 9 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: a27a7c3c295c8fb248a776edc0fd46ae6d7929ca | next: work`

2026-11-20 /goal round 10 返修检查点。R1046-01/R1046-02 在旧基面修复并提交 `2b06d8ac5`，冻结的 `ace-aac-syntax` 48/48，复审相同八例全过。新目录沿 AST 中的模块名和成员字段为两类 AGE、`#in` 符号表生成逐成员路径。公共无条件导入共享缺失/循环检查，外包、条件和 provider 仍可查询。两个未注册的诊断文件随修复移除。初次、第二次 Gen0 准备发现 owned `Str` 需要明确 `.move`，分别 48.738s/46.554s 退出 1，修正后同入口 96.285s 退出 0。旧基面 hosted 探针在源前端后因 `PREPARATION_REQUIRED` 停于未匹配 runtime-dev 身份，未执行 PE；新基面将使用固定组返回的 runtime-dev 产物及内容摘要。

工作期间 PLAN-1086 落主。24 个 Plan 提交无冲突重定基到 `a3655eb2ff2638cdbecc002b7bfd74122d6ab43a`。正式 Rust 扫描器在含 `..` 的 program 中保留数字紧邻的区间标点。同步 Auto 扫描器和共享 grammar 数据，提交 `1abe595fc8aa7d2d5790657eda1d8aa8c0366656`，不改 API/验证预算。新基面 27 个旧 canonical program 逐个等值，组合 `aac` 37,315 bytes，GrammarId `c46d445a028a6652f626daeb4919fc0df402b5f5b953aad9cc2fad266697a784`。新基面 driver 制备 58.050s，原始八例重新通过，两个 profile 的紧邻切片/十进制与完整 Rust CST 对账，四条定向 Nextest 20.919s 整命令退出 0。固定组和 hosted AC 正在同冻结身份重绑。

2026-11-20 /goal round 10 完成返修和 T-06。冻结 `1abe595fc8aa7d2d5790657eda1d8aa8c0366656`，基面 `a3655eb2ff2638cdbecc002b7bfd74122d6ab43a`。据受影响源码重跑四组共 **1470 PASS**，未见本包新增未处理 warning。原测试选择保留，Windows 不执行 Linux/WSL；`ace-source` 1 项原排除；`ace-native` Nextest 19 项原条件未选。

| 本轮固定组 | 执行/结果 | 制备 | 发现 | 执行命令 | 整命令 |
|---|---|---|---|---|---|
| `ace-syntax` | 159/159 | 三组合并 116.284s | 2.457s | 5.371s | 三组合计 37.341s |
| `ace-source` | 932/932，1 原排除 | 同一制备 | 1.650s | 14.325s | 同上 |
| `ace-aac-syntax` | 49/49，0 排除 | 同一制备 | 1.140s | 11.338s | 同上 |
| `ace-native` | 330/330，19 原条件未选 | 18.823s | 6.193s | 93.265s | 100.626s |

组命令在专树根目录用 `cargo t <组> --prepare-only --debug-tools --out <scope>` 和同组 `--run-prepared <prepared.json> --out <scope>`，`NEXTEST_TEST_THREADS=4`，每波前核实无其他构建/测试进程；常规清理计入父命令。`r10-base1086-source-tests/results.json` SHA256 `581a9b8bb780bd6011564083cb17f0ff55aac11398408eee3d71a0a9337f14f5`，`r10-base1086-native-tests/results.json` `3b165071910d4fd600fcf0bc5a5d7b7550cee63bdf3859afbbbc8f8d8bacbb94`。新 A2R 同一显式 dev 准备，与原复审八输入逐报告对照均过，`r10-base1086-review-replay/results.json` `9344710bc14c4d334c2e33210e0cbaf2ecb1a04ac046f060978c6c9826865635`，driver SHA `7096da91353b55f9ee93950079c301ed3120725eb13b1cd4f706b576e85d13d3`。定向过滤四例均过，45 当次未选，编译 16.790s、Nextest 执行 2.580s、整轮 20.919s，完整固定组随后覆盖全部 49 例。

| 新固定用例（`ace-aac-syntax::conformance`） | 观察值 | Debug A2R 固定组实测 |
|---|---|---|
| `aac1046_symbol_list_members_follow_source_order` | 两种本包成员顺序、外包成员候选及外包没有本地文件 | 1.050s，PASS |
| `aac1046_symbol_list_imports_reject_missing_public_module_and_cycle` | 两种写法均返回精确 `MISSING beta`、`IMPORT_CYCLE`，含点分回边 | 0.957s，PASS |
| `aac1046_symbol_list_imports_keep_provider_and_conditional_candidates` | 两条件臂的成员顺序、guard AST id、provider sourceId，未选依赖不参与公共图 | 0.353s，PASS |
| `aac1046_adjacent_numeric_slice_bounds_match_shared_lexer` | `auto`/`aac` 两面上紧邻区间和十进制与完整 Rust CST 同值 | 0.625s，PASS |

Windows AC 专项使用 `ace_native::collections_runtime::prepare` 独立制备 `runtime-dev`，以 `run-debug` 重开同源身份且 `cargo_invocations=0`。复制经 SHA256 对账后，探针再次校验 BLAKE3 `3e901c9890bb0b9a5b43f6a48c9113065d709c45eec7072ffaaf1b063dcab345` 并调用 `build_hosted_collections_checked`。独立 debug 观察工具准备 50.372s，runtime 准备 0.401s，等待 PLAN-1087 的 `case-av-language-parity --prepare-only` 及 AV worker（PID 81476 等）让出资源 61.077s。`r10-base1086-hosted-runtime.json` SHA256 `ac835a52f176106f56780d1e04088fd75fd5f30ba74ba800eb9e1b1e4fd99d36`。

| Windows AC 专项 | 结果 | Gen0 / COFF link | 实际用例 | 整命令 |
|---|---|---|---|---|---|
| `ac_catalog_equal_r10_1086` | 8/8，正式 `std.file` 真实目录、整报告及两个来源 AST 对照 | 42.545s / 8.247s | 原四例 3.508/1.775/1.945/1.815s；符号表正例、限定正例含紧邻切片、缺失、环分别 2.541/2.821/1.574/2.600s | 71.109s |
| `ac_corpus_r10_1086` | 15/15，七组原 AutoDown 全文/缺尾 `}`、CRLF/emoji 复核 | 32.463s / 8.771s | 原始完整 canonical 及诊断 0.804–2.271s，坐标 3.022s | 64.231s |
| `ac_format_structure_r10_1086` | 2/2，989/961 个节点逐字段、原始 bytes 重建与完整 EOF 诊断 | 35.948s / 7.453s | 完整 3.903s、截断 6.500s，原本包 8s child 限额 | 60.397s |

AC 专项未单拆发现/清理，测时含实际文件写入、子进程和断言。日志 `r10-base1086-ac_catalog_equal_r10_1086.log` SHA256 `58bf089de0ff4c81ca090e6b6ca4acab3d4835355d627567a0b01d237ae6b137`、`r10-base1086-ac_corpus_r10_1086.log` `8ab752e2b25e43588ab72f7fdbd962f233cfa2f1fb92f295abb9c003e652cd50`、`r10-base1086-ac_format_structure_r10_1086.log` `f3079b1ba7a30754a8898da6bf0a430e26ddd948a38e9153b4e4e9600615f785`。源码对账 `r10-base1086-profile-compat.log` `4cc44f78fb30942903299b26572bf9e4881bbd5eaafee14da6c00d5ba186f502`、`r10-baseline-range-diff.txt` `1665bc67433105c23478cfd3fe3cf9c9e3aca369d5a246086ad788226eb22b8c`。

R1046-01 → 两类 AGE 生产式的逐成员公共边及 `#in`/provider 候选 → `source_catalog.at` 共用目录路径和以上三条永久回归 → 原八例与真实 Windows AC 正反例均关闭，待独立复审。R1046-02 → 无未注册打印型探针 → 两文件已从冻结树移除，唯一 conformance target 实执行 49 例，待复审。AC-01/02/03/04/07、AC-AC-STAGED、AC-CL-FAMILY 的 work 证据齐，1044 分母反控身份不变，代码/产物按新基面重绑。SD-01/04 在验收通过后陈述 AST 字段和三类逐成员候选、公共缺失/环边界。SD-03 记录共享 grammar profile 组合与 `..` 按当前 profile 表启用的数字切分。T-07、T-08 待执行。

`stage: work | plan_id: PLAN-1046 | plan_revision: r8 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1046/auto-lang | branch: plan-1046-dev | base_commit: a3655eb2ff2638cdbecc002b7bfd74122d6ab43a | code_commit: 1abe595fc8aa7d2d5790657eda1d8aa8c0366656 | dependency_revisions: 1044/1045/1067/1051/1086 archived | task_ids: T-04/T-05/T-06 done after repair and rebind | evidence: eight frozen A2R replays, four frozen groups 1470 pass, 25 real hosted AC checks, 27 canonical base profiles unchanged | blockers: none | next_responsible_role: final review T-07`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r8 | round: 10 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: 1abe595fc8aa7d2d5790657eda1d8aa8c0366656 | next: review`

2026-11-20 T-07 最终复审第 2 轮通过。按 §6 在实施会话内复核，未另开审查会话。复核使用独立选择的输入、已提交实现、真实可执行体、父子验收日志和规范源；未修改产品代码。`git diff --check a3655eb2f..1abe595fc` 通过，工作树洁净。1086 已落同一基面，1044/1045/1067/1051 的已交来源仍成立。

| AC | 复审判定 | 本次核验和保留证据 |
|---|---|---|
| AC-01 | PASS | V-01/V-04/V-FSTR-01 的已发布源投影和冷/热断言；原八个 A2R 目录输入重放，新增同入口别名重导出与本包/外包并存、两臂与 provider 的正形；AC 双文件报告 6672 bytes 与每文件独立 AST 对账。 |
| AC-02 | PASS | V-02/V-FSTR-02/03 的原失败近邻、回滚和真实嫁接断言在冻结 AAC 组通过；复审直接输入无效 UTF-8 得 `SOURCE_ERR UTF8`；Windows AC 的原始格式串完整/截断及保留诊断证据通过。 |
| AC-03 | PASS | 原缺失/循环负例和合法邻居，另验 `use app.beta: y` 与点分回边成环；候选外包、未选臂、未选 provider 不进入公共图。实际 A2R/AC 采用 Auto 主体；错误语法、资源限制和文件宿主准入的冻结原结果齐备。 |
| AC-04 | PASS | 两个 `--debug-tools` 准备/执行清单的源身份一致，四组 1470 PASS，`ace-source` 1 原排除及 `ace-native` 19 原条件未选如实保留。核 49 个 AAC 原始 nextest `ok` 与逐项耗时，最长 6.770380s。全组证据不在本轮重复。 |
| AC-05 | PASS | R1046-01/02 已关闭；`conformance.rs` 是唯一签入的 AAC Rust 测试入口，显式 `prepare-native` 执行 Rust 编译，测试仅核准备摘要并运行已制产物。检查格式、未处理新增 warning、无遗留 findings 或 legacy ledger 写入，冻结 SD-01..04。 |
| AC-06 | 待 merge | 在合入前复核冻结门禁有效性和主基面，落主验证规范源，归档后清理专树与材料。 |
| AC-07 | PASS | V-05/V-FSTR-02/03 的 CRLF/emoji、诊断原始位置、合法互递归和值初始化边分类已核冻结测试与 AC 诊断。 |
| AC-AC-STAGED | PASS | 保 1044 原分母反控收据及阶段归属。原 Windows AC 目录正/反例依 1051 `runtime-dev` 和真实平台文件能力，在同源冻结程序通过。 |
| AC-CL-FAMILY | PASS | 27 个基面旧 profile 的 canonical 等值，第 28 个组合独立；核 T-05 的类型、标注、原边、两个来源和拒绝控制；三次扫词的数字/双点/`..=`/无区间 program 近邻保持 profile 边界。 |

| 本轮复审实际执行 | 结果 | 制备/发现 | 子程序合计/最长 |
|---|---|---|---|
| `python <scope>/r11-independent-review.py` 的 `catalog` 路线 | 原八例及三例独立兄弟场景，11/11 | 复用冻结 A2R，无编译 | 2.282s / 0.273s |
| 同一命令的 `lex` 路线 | `auto` `1.25..=3` 起点 0,4,7；`aac` `1.25..3` 起点 0,4,6；`scalar` 无 `..` token 保持一个 [0]，3/3 | 同上 | 0.167s / 0.086s |
| 同一命令的 `parse-source` 路线 | AAC 坏 UTF-8 在源入口拒绝，1/1 | 同上 | 0.016s / 0.016s |

整轮子程序 2.464s，工具整轮 2.692s（含工具启动和 fixture I/O，发现及清理未单拆）。无新增永久用例。复审收据 `r11-independent-review/results.json` SHA256 `e1e4a454bb9bfae0136472c47451f0d21cedaf5e74bb77058139bd4f56f60926`，产品 exe SHA256 `7096da91353b55f9ee93950079c301ed3120725eb13b1cd4f706b576e85d13d3`。

SD-01..04 冻结材料 `r11-spec-delta.json` SHA256 `b7dad5a9decf6744406a016d3e081ec71aa7a01ff0f4d11dadd0ec91ed2f8f9a`。三处现行版核 SHA256 和目标内容，新增规范路径在主检出尚不存在。增量分为前端契约（原字节/编辑/来源/语法及候选边界）、项目导航（已验证切片）、规则程序（组合非递归、保原声明顺序、profile 绑定的数字区间处理）和装配（保条件及 provider 候选，绑定归后继）。四个准确 postimage 随冻结材料提供，合入时再验，包括新规范、项目、grammar、装配 SHA256 分别 `78d0c22ccd045a60b9ba0a5b76258811093d4bb5739ab536dcf3f800c4c3c9f3`、`a5ad36ccc03e3d77c4e34555340e59062e0a44e9af1fd71d8622798debe0206d`、`9a2337249817a8283f58ffe9518b158155ecfdec8b5f10ff03214d2214d1f970`、`2ff8c2cffe5701b41def879cfd7554a803043952ea45fa9fb386c933a0049f37`。已核 frontmatter 三个现行路径、一个新增路径及 `GOAL-002`。

`stage: review | review_scope: final T-07 | reviewer_identity/limitations: Codex in implementation conversation, independent inputs and frozen artifact audit, no separate reviewer | plan_id: PLAN-1046 | plan_revision: r8 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1046/auto-lang | branch: plan-1046-dev | reviewed_commit: 1abe595fc8aa7d2d5790657eda1d8aa8c0366656 | base_commit: a3655eb2ff2638cdbecc002b7bfd74122d6ab43a | dependency_revisions: 1044/1045/1067/1051/1086 archived | spec_inputs: SD-01..04 frozen r11-spec-delta.json b7dad5a9decf6744406a016d3e081ec71aa7a01ff0f4d11dadd0ec91ed2f8f9a | acceptance_results: AC-01/02/03/04/05/07 and AC-AC-STAGED and AC-CL-FAMILY pass, AC-06 merge-owned | pending_closeout_gates: frozen affected fixed union validity check, canonical landing, archival and cleanup | findings: R1046-01 and R1046-02 closed | evidence: 15 independent A2R observations and unchanged frozen round 10 receipts | next: merge T-08`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r8 | round: 11 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1046/auto-lang | commit: 1abe595fc8aa7d2d5790657eda1d8aa8c0366656 | next: merge`

2026-11-20 T-08 合入与归档检查点（PLAN-1046:r8，delivered）。

- `prepared`：最终复审代码 `1abe595fc8aa7d2d5790657eda1d8aa8c0366656`，基面 `a3655eb2ff2638cdbecc002b7bfd74122d6ab43a`。更改仅为四个已审核 Spec 的提交 `dca20c884737485d4da6dc32b9399e4245718769`；实现、依赖、组配置与输入均未变化。`cargo t --list` 和 `cargo t ace-syntax ace-source ace-aac-syntax ace-native --select-only` 核选集，逐项验 round 10 真实执行证据、同一源身份、当前 toolchain marker 与产物摘要。复用 `ace-syntax` 159、`ace-source` 932、`ace-aac-syntax` 49、`ace-native` 330 的冻结通过证据。原排除保留。核对记录 `r12-frozen-gate.json` SHA256 `8a48b7d330af43b8b32c4e85855a36110b3f1cc5c503362e6f5ea4162b690e3d`。
- `landed`：绝对专树路径已核，`pwsh -NoProfile -File D:/autostack/wt-guard.ps1 D:/autostack/.wt/lang-1046/auto-lang` 返回 clean。`git merge --ff-only plan-1046-dev` 退出 0，`master` 当时精确等于 `dca20c884737485d4da6dc32b9399e4245718769`；无 merge commit，主检出其它计划的工作区状态保持。
- `canonical_verified`：四个路径的工作区与交付提交内容逐字节相同，摘要与复审 postimage 一致，检查 0.325s。Spec 名称、摘要见前一轮。原工程带真实 `std.file` 的 Windows AC 仍采用已验冻结证据。落主后执行 `python <scope>/r12-postland-smoke.py`，同一产品可执行体在 11 目录、3 词法、1 坏 UTF-8 检查全过，子程序合计 2.470s、最长 0.284s、整命令 2.529s，退出 0。发现、清理未单拆，未新增永久用例。`r12-postland-smoke/results.json` SHA256 `f1e09e601ce40835290e0455d33162d4b41d11922d4d84217d97224a6236d88a`。
- `archived`：当前归档目标 `docs/plans/archive/1046-aac-syntax-source-and-modules.md`。计划正文和四处来链修正至真实归档路径，已检查本文件相对路径、任务和 frontmatter。交付既成，最后一项 T-08 清理待证据落账。

清理前以登记 `REGISTRY.md`、当前进程及不跟随链接的盘点确认：`D:/autostack/.wt/lang-1046/` 仅专树 `auto-lang`，产物主要在 `target`、`experimental/ace/target`、runner target。原有 `ace-native` 889 测试辅助代码在专树内产生的 `docs/plans/attachments/889/` 含已完成本次门禁的临时驱动和存储，共 7,395 文件 / 386,534,277 bytes，其验收摘要已保留，随专树删除。`D:/autostack/plan-artifacts/plan-1046/` 包含外部 `native-probe` 的调试目标、真实 PE/源码及工具日志并受本计划登记；早期简写 `plan-artifacts/profile-sizes.txt` 的实际文件在此 scope。逐一核计划命令的源树、外部 target、PE 工作目录、组输出和材料根，根目录、兄弟组和主检出附件中无额外同号输出；无具名未关闭 hold。

2026-11-20 T-08 收尾核验（AC-06 PASS）。归档检查点 `f2c30750553413e83be4d1765a9c46f826ac9cb0` 已在 `master`。专树的绝对路径、分支和交付祖先关系均一致，移除前重新运行 PowerShell `wt-guard.ps1` 返回 clean。`git worktree remove D:/autostack/.wt/lang-1046/auto-lang`、`git branch -d plan-1046-dev` 退出 0，空组目录已移除；`git worktree list --porcelain`、分支列表和磁盘三处确认已无该专树、分支和组。构建中间件随专树删除，测试落在原 889 辅助路径的本次临时文件一起清理。

`cleaned`：已核材料目录的注册 owner、解析绝对路径、进程占用与 reparse point，最终盘点 706,618 文件、18,030,825,187 bytes。自动执行策略拦截带 `-Force` 的命令；按核验过的固定绝对路径执行 `Remove-Item -LiteralPath 'D:/autostack/plan-artifacts/plan-1046' -Recurse` 成功退出 0，`Test-Path` 确认目录不存在。`.retention/policy.json` 的 `scopes.plan-1046` 已刷新为 `deleted-at-merge` 和 `cleaned`，剩余 0 bytes，无具名保留。主检出原有其他计划的文档改动独立保留。归档路径和设计导航已验证；受影响代码和四份规范已在默认分支，后继按已交语法与候选契约消费。

批量回归核 `docs/plans/.last-batch-regression.json`，上次覆盖 ID 1098，收据时间 2026-11-20T00:00:00Z，本次不到 48h。此后新归档 1046/1051/1081/1082/1083/1084/1085/1086/1106/1107/1109，无 ID 超过 1098 且可被 5 整除；批量门未到期，其已知基础设施问题沿原登记。本轮复用 1470 项原始固定组证据，实际落主后冒烟 15 项用时 2.529s，无新增永久用例。

`stage: merge | plan_id: PLAN-1046 | plan_revision: r8 | outcome: pass | reviewed_commit: 1abe595fc8aa7d2d5790657eda1d8aa8c0366656 | delivery_commit: dca20c884737485d4da6dc32b9399e4245718769 | canonical: SD-01..04 exact SHA256 verified | archive: docs/plans/archive/1046-aac-syntax-source-and-modules.md | archive_checkpoint: f2c30750553413e83be4d1765a9c46f826ac9cb0 | cleanup: worktree, branch, empty group and registered artifact scope removed; registry refreshed | batch_regression: not due | task_id: T-08 done | result: 9/9`

`stage: goal | plan_id: PLAN-1046 | plan_revision: r8 | round: 12 | from: merge | to: done | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1046/auto-lang (removed) | commit: dca20c884737485d4da6dc32b9399e4245718769 | next: done`

## 10. 待澄清事项

**D-1046-03（已关闭）**：历史版本 `cf997b356` 真嫁接，在 `try_graft` 的 `t` CST 节点 arena 追加达到 65536 上限（§9 失败位点和产物槽名相互印证）。提交 `5cea8cf0f` 原子验证整个复制范围，重定位每条边的真实子节点 id，位移表与计数分隔，候选限定在重扫区外，提交后恢复同一根列表。原三函数样本带真嫁接的 CST/byte span/诊断/实际复用计数过 AV 与 prepared native RUST oracle，原 ignore 已移除，prepared native 用例 0.270s。

**D-1046-04（已关闭）**：`f939ded4f`、`16ee06bcb`、`5a20cd937`、`fc3785af7` 交格式串、映射、受控错误、真实增量和坐标消费。原始共享集、正反近邻、触发空间/编辑失败与被使用的语法树通过 V-FSTR-01..03 的默认预算验证，原版本证据及实测计时见 §9。V-AC-STAGED 仍按 T-06 正式验收。

**D-1046-05（已关闭）**：r8 组合 grammar、声明/字段标注和目录来源按 §9 通过 V-CL-FAMILY、V-CL-ANNOTATIONS、V-CL-PROFILES，旧 26 个 profile canonical 字节逐一匹配。

**D-1046-06（已关闭）**：消费已落主的 1051 Gen0 hosted collections 正式入口，真实 Windows `std.file` 加载原双文件目录。完整正例与三个原坏形在同一真实 PE 上通过，见 §9 绑定收据。T-06 的固定组已重验，上一基面 `ace-source` 的深递归红项当前已通过。


无须当前用户新增决定。1044/1045已交；不复制旧legacy lexer作为新的语言真值。 1044负责冻结具体schema/源码需求/资源profile，1051/T-02负责transport有界决策；重大支持范围/预算/仓修改边界变化另按用户授权处理，例行实现选择不反复请求确认。后续源与工具变化按真实影响更新收据，不复用失效身份。
