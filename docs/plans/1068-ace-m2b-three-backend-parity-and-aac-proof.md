---
plan_id: PLAN-1068
status: executing
feature_name: ACE M2-B 三端能力对齐与AAC实证收口
author: [Codex]
created_at: 2026-11-16
updated_at: 2026-11-24
plan_revision: 12
current_step: 0
total_steps: 17
depends_on: [PLAN-1067, PLAN-1064, PLAN-974, PLAN-975, PLAN-978, PLAN-1044, PLAN-1045]
supersedes_spec_components: [docs/specs/ace-migration/replacement-scope-and-evidence.md, docs/specs/ace-migration/m2-certification.md]
new_spec_components: [docs/specs/ace-migration/m2-three-backend-parity.md, docs/specs/auto-ac/aac-std-closure.md]
touched_goals: [GOAL-002]
affects: [ace, auto-ac, stdlib]
---

# [PLAN-1068] ACE M2-B 三端能力对齐与AAC实证收口

## 0. 变更摘要

### 2026-11-24 r12 · 用户裁定：AAC CL 生产实现并入本包，1068 转 executing

用户 2026-11-24 在"新开实现计划 vs 把生产任务并入 1068"中选择后者：把 AAC 自举前端的
**CL 生产实现**（parser 列表迭代、Str/char 原语与泛型构造、HIR builtin callee、MIR/ABC
lowering、std 闭包与 `aac.pending-work/1` 面）作为本包新增 work 任务 T-11..T-17 / AC-11..AC-17 /
V-11..V-17 / SD-AACCL-01..04 承接；本包状态 `drafting → executing`，`plan_revision 11 → 12`，
`total_steps 10 → 17`。同期起草的 PLAN-1130（`docs/plans/1130-aac-cl-primitive-and-std-surface.md`，
master `9cd6a4cab`）按用户裁定**撤回**（文件删除，内容=本批任务；git 历史保留）。

输入：PLAN-1054 的逐模块门面清单（85 模块：66 解析干净 / 7 parser 缺口 / 6 checker 缺口 /
4 `aac.pending-work/1` / 1 `hir.unsupported`）与 1053/T-07 的 53 个 blocked CL 族。
归属变更：Design59 §3 原把 AAC 前端差额归 1046–1052，但这些计划均已 archived 且无在办 owner；
本裁定由 1068 直接承接生产（T-11 内同步补记 Design59 归属表），1054 在该批交付后重绑 S 身份续跑
T-02..T-09。原 T-01..T-10、AC/语言/库/UI/AGE/AAC 义务全部保留，原 0/10 进度不变。

### 2026-11-19。T-04 及 T-06/08 按 [AGE 交付表](../design/strategy/age-m2-three-route-strategy.md#后继交付与消费) 接收各端。1097 交宿主/W2，1074 交 W3，AGE 公共 AC/A2R 取 [1102](1102-age-ac-gallery-fusion-and-capabilities.md)/[1103](1103-age-a2r-gallery-fusion-and-capabilities.md)，两游戏取 [1100](1100-age-av-game-2d.md)/[1104](1104-age-ac-game-2d.md)/[1076](1076-age-a2r-game-2d.md) 和 [1101](1101-age-av-game-3d.md)/[1105](1105-age-ac-game-3d.md)/[1077](1077-age-a2r-game-3d.md)。保阶段任务及最终分母。

### 2026-11-19 r10 · 接收AV-ONLY阶段与非AV检查恢复

用户已明确1078–1096只验AV，公共语料中AC/A2X等检查先以检查级#ignore暂缓。其未实现/未运行/失败不阻AV计划，不充PASS；本包进入对应后端开发/验证阶段后，逐check接手、移除本阶段标记并真实执行。共享源码、原oracle/条件/owner和本包完整验收门保持；不得要求AV计划先交非AVPASS。本轮仅修订规划。

### 2026-11-19 r9 · 公共语言语料按特性共源（仅规划）

按用户要求接[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)与1078的`IF-ACE-FEATURE-CORPUS/1`。接收AV/AC/A2R语言结果及原→新迁移映射；按canonical Case/源码指纹/目标动作逐行核实，生成/构建/运行分开。各后端实施仍归Design59具名producer和1097，不在本包复制源码或实现第二前门。已有切片可先验，缺check/pending阻完整三端签章，不新增整包开工等待。 原任务ID/数量/进度与已有证据边界保持，本轮不施工、不迁实际测试。

### 2026-11-19 r8 · AC 同源语言验收

各实施计划复用 AV 的同一 AutoDown 用例验 AC。已有正式用例随任务验证，缺具体用例的行留到 AV 1078–1096 正式交付后统一对账。本包 T-02 收对应正式证据，T-06/08 核全量结果。

### 2026-11-19 r7 · 实施与验收交接

1068 负责冻结差额验收范围、接收正式证据、复跑 AV/AC/A2R 和 AAC 阶段实证并交 985/B。原 T-02/T-03 的源代码修改归 [Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)中的 AAC、库和 ABI 生产任务。原 T-04 的当时的图形实现及阶段收据归 [1097](1097-ace-ac-autoui-w2-and-age-host.md)。T-02/03/04 转为接收核验，各生产计划管理自己的实现进度。各旧变更摘要和复审记录保留当时口径。原 10 个任务与 0 进度保持。

### 2026-11-19 r7 · AV语言producer正式拆批

AV与AutoVM语言差额由[PLAN-1078–1096专线](../design/strategy/av-autovm-language-parity.md)逐包生产；本包T-01按原L/case行接收其实际已交切片，T-02 核实际生产任务交来的 AC/A2R 语言与运行证据，不另建AV前门或在AAC内复制语言实现。T-06/T-08最终全语言AV列必须接[1096](1096-av-parity-full-language-certification.md)的`IF-AV-LANGUAGE-PARITY/1`及同版原源/oracle/错误/资源来源；中间切片不冒完整AV齐平。

这是**阶段消费依赖**，不把1096整包设为本包开工、T-03已有系统库接线或T-04已交图形cap的总前置；1096也不反等1068/985-B、完整AAC或UI产品。原T-01..10、AC/语言/库/UI/AGE/AAC义务全部保留，进度仍0/10，仅更新producer归属与最终接收条件，不施工。

### 2026-11-19 · 980/981 后继接口

T-04 取 1071 与 AutoUI 104–108 的 W1，接收 1097 宿主/W2 和 1102、1104、1105 的 AC 域。T-06/08 接收 1074 W3、1103 公共 A2R 和实际内容/游戏交付，按原图形、业务、权限、资源、恢复及缺行反控核三端。

2026-11-16 r3语言完整性检查：区分AAC语义主体1046–1050与本包/1067的现AC机器/runtime补齐；1053/1054完整CL验证含AAC源码未用特性。本包T-02/05/06保三端全量与早期AAC实证，不反等完整AAC签章；原10任务/0进度/依赖保持，仅规划。

2026-11-16 r2当前：原T-04明确承接AC原生Node桌面，复用980/NATIVE A期AV＋Rust FFI宿主接口，交IF-1068-AC-DESKTOP供981/FUSION/AUDIT与985/B，不新增任务/计划；原10任务全未执行。

承接M2-A具名AC差额，冻结AV/AC/A2R同源完整能力表并补真实native接线，以AAC实际模块持续暴露语言/库/SDK组合问题，交985/B最终一致性门。完整AAC Gen2/Gen3签章仍独立；本包不反置985/A前置。

用户2026-11-16裁定：M2最终AV/AC/A2R能力一致，A2C完整对齐为增强项；M2-A允许AC具名差额，在M2-B结合AAC真实工程逐步补齐。本轮仅研究与规划，无源码/fixture/构建/运行/合入；全部0/10。归档968等不重开、不改历史收据。

2026-11-16 r4当前：T-04在UI-W1-AV正式后承接第二波完整AC UI/Widgets爬梯，独立交UI-W2-AC，不等本包三端最终/T-06/完整AAC或W3。T-06/07/08在W3及其余输入齐后汇总三端，10任务/0完成保持。

## 1. 目标

在本版明确平台/上下文下，AV/AC/A2R的语言、普通标准库、FFI与UI/AGE领域公共能力一致；尤其保留~T/?T/!T、泛型、标注、CTE及它们跨模块/资源/异步组合。AC合法能力不得以历史profile外或native.sched-unsupported作最终豁免。

同一目标平台的公共语言/库/领域语义对齐，不要求相同内部布局或调度实现。A2C保已有代码/用例/支持事实，缺完整对齐不阻M2；既有A2PY标准库义务、AV单执行器三provider、Linux1063延期规则保持。Linux写代码和完整用例，当期不测试；不能用Windows证据升Linux verified。

本轮只调整本版已确认required的语言/库/领域阶段，不重新加入已后置的Godot、JET/ARK等后续目标；生产TS/JS消费闭包沿当前范围。

## 2. 架构方案

沿985原逐行义务矩阵追加阶段/差额owner/证明来源，以共享前门/IR为唯一语义权威，核真实生产任务交来的 native lower/runtime/调用接缝证据。默认库Auto＋OS薄层；显式C/Rust引擎及各宿主依赖分别记录。AAC是同一语言/库的真实复杂consumer，修复回实际ACE/标准库owner，不在AAC私建替代stdlib或临时解释器。

本包M2-B实施门为985/T-A05已落主M2A-08收据及本包执行授权；允许A出口保留有owner/输入/独立expected/关闭条件的AC差额。frontmatter不写整个985，防985/B→本包→985全包环；也不要求AAC1054整包或Gen2/Gen3先完成。

## 3. 技术栈

既有Schema/CST→共享绑定/Checked HIR→MIR→AV/native Cranelift/A2X-Rust；Auto默认库＋受检OS原语，显式外库经967稳定ABI。未来唯一树`D:/autostack/.wt/lang-1068/auto-lang`、分支`plan-1068-dev`，材料唯一`D:/autostack/plan-artifacts/plan-1068/`。纯规划在主检出；跨仓只有真实修改需求与对应owner接续时加入同组树，禁止链接。

## 4. 需求分析与背景调查

授权为当前目标/阶段边界的设计和计划修订，未要求立即启动实现或新会话，无新增预算。起草读取HEAD `2961a2aa24cb8f6158e638786cb0972b812dd589`；来源为[总览](../specs/overview.md)、[Design55](../design/55-ace-m2-complete-language-and-product-replacement.md)、[AAC设计](../design/59-aac-native-bootstrap-and-compiler-subject.md)、[调度现状](../specs/ace-migration/m2-scheduler.md)及985。归档965/966/968与真实代码只证明其原支持范围，不代三端全能力。

[能力差额审计](../design/strategy/ace-capability-gap-audit-2026-11-19.md)和 985/A 正式后延账为输入。1067 已交原生异步基础，1064/1065/974/975 和 967 供各自正式库/系统/ABI 证据。1051 承接共享服务与当前 AC 的机器执行余量，相关源语义、IR 与库/SDK 改动分别落在生产任务。

T-01 重绑源码、规范、工具、ABI 与 SDK 身份。源或合同变化按实际影响重验。复用 TestCase/AutoDown/worker/固定组。M2A-08 已由 985/T-A05 正式落主，签发提交为 `95c7fbad8`。

## 5. 详细设计

### AV-ONLY接收与本目标启用

1078的IF-ACE-FEATURE-CORPUS/1和1096的AV语言证书允许携带非AV#ignore/NOT_RUN行，接收时只核AV声明和迁移义务未丢，不以非AVpending回卡AV独立出口。本包实际验证自己的后端/代际时，由原生产任务逐check去掉AV阶段#ignore，保原平台/ignore与oracle，按当前同源/工具/SDK身份运行；失败属于本目标任务，必须修到本目标required齐才签其完整出口。1068/985的最终多端门及AAC独立门不接受忽略充PASS；AV阶段通过从不冒多端通过。

规范影响：公共作者/引用/选择/迁移协议由1078统一在拟`docs/specs/auto-test-case/feature-organized-corpus.md`沉淀；本消费者只在自身原SD/模块Spec记录实际Case引用、目标动作和正式证据，不再各写一套公共协议。AC-FC属于本包验收增量，原SD与目标义务全部保留。

### FC · canonical语言输入与旧义务承接

接收AV/AC/A2R语言结果及原→新迁移映射；按canonical Case/源码指纹/目标动作逐行核实，生成/构建/运行分开。各后端实施仍归Design59具名producer和1097，不在本包复制源码或实现第二前门。已有切片可先验，缺check/pending阻完整三端签章，不新增整包开工等待。

由原T-01/T-02/T-06/T-08按其职责消费/迁移对应行，保原source/expected/check及条件，重复仅在完整输入闭包/语义等价时合并，原义务→canonical Case/check映射不得丢；既有固定组/suite/worker读者按1078接口切换，不新建按后端/代际维护的Auto副本。缺字段/目标接口由1078和实际producer在原框架有界补接，不要求本包先等全AV/全AAC完成。

### 配置和三端验收边界

[982/B-CONFIG](982-ace-m2-man-gen.md) 交实际新配置宿主及 IF-982-CONFIG-ACE。仅验实际使用接口，区分编译/启动使用的 AV 配置求值和 AC/A2R 目标业务执行。动态配置调用按原目标义务独立验。T-03/05/07/08 依据正式输入验配置变化、owned 值、错 SDK/缓存、错误恢复和禁 guest fallback。

### 正式证据输入与阶段

| 证据入口 | 来源 | 1068 关闭条件 |
|---|---|---|
| T-01 差额基线 | 985/A 后延账和原 required 矩阵 | 每行冻结 Case/source/平台/预期和正式生产任务，缺证留待验，最终差额为 0。 |
| T-02 语言 | 1046–1051 的 [实施归属](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属) | 核泛型、标注、CTE、~T/?T/!T、动态、对象、全局、闭包和资源/调度组合的正式结果，保 AAC 自身未用的 required 行。 |
| T-03 库、ABI 和配置 | 1045/1051/1052/1056、1064/1065/1067、974/975、967、982/B-CONFIG 按实际输入 | 收真 Windows OS、File/net/DB/FFI/profile 组合、权限、诊断与资源，保既有 A2PY、AV provider 和 Linux 1063 边界。 |
| T-04 UI 与 AGE | 1071/AutoUI 104–108 完整 W1、1097 W2、1074 W3，以及 AGE 交付表按域给出的 1075、1099–1105、1076/1077 | 接 IF-1068-AC-DESKTOP、UI-W2-AC、适用 AC/A2R 能力与同源 AV/产品实证。核 W2 独立覆盖，最终收同源三端。 |
| T-05 AAC 真模块 | 1044/1045 和所用 1046–1052 已正式交付阶段 | 复跑真实 Auto 编译器宿主及至少一个源码/模块处理链，验实际源版本、诊断、文件、符号和失败恢复；后续可纳入真实泛型/CT/构建模块。 |

生产方在原计划管理代码、任务状态、验证和阶段复核。1068 仅登记待接收/已接收/退回的验收状态，记录 source、规范和 compiler/ABI/SDK/执行身份、独立 oracle、结果及适用平台。失败退回具名生产任务，修复方重新交受影响证据，1068 按身份复验。发现账外 required 缺口时，归到对应共享实现或库/域计划后列入验收矩阵。

普通 AAC 与 B 验收准备按已交输入先行，图形 W2 由 1097 独立交 1074，T-06/07/08 于正式 W3 后完成图形三端并集。配置 982、OS 产品和完整 AAC 代际依各自既有阶段。frontmatter 的原基础依赖继续适用，1046–1052 及 1097/1074 取表中指定阶段证据。985/B 消费本包最终交付。1053 全 CL 和 1054 完整代际仍按 AAC 自己的门验证。

### 结果与支持声明

一行的通过=同一Auto源/等价合同输入在AV、真AC、真A2R运行，分别满足规范独立expected、错误/授权/取消/生命周期与源位置合同。不同平台对象/ABI、调度交错、内部布局与物理性能可不同；公共能力缺失不能藏在“适用格”里。外部OS/设备客观不适用须有真实规范理由，不能用AC未实现伪造平台排除。Linux验证仍按1063延期，逐行保实现与原例/NOT_RUN；本包不自动宣布Linux通过。

### AAC CL 生产实现（2026-11-24 用户裁定并入；原 PLAN-1130）

**目标面**：让自举 AAC 前端吃下全树 S 与 Windows std 闭包所需的 CL 面，把全树 project 编译从
"被 parser/checker/std 面挡住"推进到"越过 parse/bind/typecheck 进入 HIR/MIR 并可被 AV 消费"。
生产点只在本包（`experimental/ace/aac/**` 与阶段 driver），Rust 侧只作独立 oracle/消费端。

1. **parser（`for <binding> in <collection>`）**：共享 grammar 已在
   `experimental/ace/grammars/auto-source.at` 行 107 记有"集合族最小语句面（for-in 迭代）"；
   T-11 先判定是 `aac` profile 未含该生产、还是 `syntax_parse` 投影未接、还是形状不匹配，
   再改 profile/重新生成 `syntax_grammar_data.at`（**不手改生成文件**）或补投影。
   绑定为 body 作用域局部不可变；`List<T>`/`Str` 两类 iterable；range-for 语义不变。
2. **checker 原语与泛型构造**：补 `Str.char_at(int) -> char`、`char.to_codepoint() -> int`、
   `char.from_codepoint(int) -> char`、`List<T>.new() -> List<T>`（含嵌套泛型）的事实；
   未知方法仍 `type.unresolved`（负控）。
3. **HIR builtin callee**：primitive 调用无可指向声明行，HIR 增**闭集** `callee: "b:<name>"`
   形；既有程序 HIR 文本逐字节不变；`verify_core_document` 只接受闭集内名字，未知名具名拒绝。
4. **MIR/ABC 与 AV**：迭代 lower 成索引循环（ListLen/ListGet + 合成索引槽）；
   `char_at`/`to_codepoint`/`from_codepoint` 优先复用 `av-core` 合同已有 op，缺 op 按合同增补并配
   独立 conformance；泛型构造→LIST_NEW。不支持面必须落具名阶段码，不得 trap 或空成功。
5. **std 闭包**：project 车道按 `std/<stem>` 装载（`semantic_driver.at`）+ `gen-next` 物化
   `stdlib_assembled_files()+stdlib_sys_files()`（Windows 过滤 `*_lnx`）；S 的 10 个 std 模块
   （bytes/conv/env/file/path/process/syscommon/sysplat_win/text/time）绑定/定型可达；
   `aac.pending-work/1`（env/path/rand/time）与 `std/math` 的 `hir.unsupported` 逐条决定
   "实现"或"具名分期+阶段码"，不得静默通过。

边界：Gen2 native PE/代际签章仍归 PLAN-1054 T-02；G2（mn 16MiB）归 AGE elem-view owner；
53 个 CL 族的 oracle/行不删；Linux host 不扩。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-CFG-HANDOFF | modify（拟，原owner） | `docs/specs/ace-migration/m2-three-backend-parity.md` | 原候选接收→实际配置新AV来源/宿主与guest边界、错误恢复/原平台，真实交付后才写已支持 | 982生产一次，本包只核实际消费者/安装退役阶段；A收据保留 | AC-CFG-HANDOFF、原AC与最终review |
| SD-PARITY | add | docs/specs/ace-migration/m2-three-backend-parity.md（新） | 分散适用域→AV/AC/A2R最终完整公共能力与AAC实证/未用补验 | 用户三端最终要求 | AC-01..04 |
| SD-EXIT | modify | docs/specs/ace-migration/m2-certification.md、docs/specs/ace-migration/replacement-scope-and-evidence.md | A必须AC全齐→A基本可用/具名后延，B三端差额=0；A2C完整对齐增强项 | 用户阶段与目标修订 | AC-01/04 |

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-DESKTOP | modify | docs/specs/ace-migration/m2-three-backend-parity.md | 接收 1097 真实 AC 会话、宿主及产品的正式证据 | 领域能力交付与最终三端对齐均可追溯 | AC-DESKTOP |
| SD-CL-OWNER | modify | docs/specs/ace-migration/m2-three-backend-parity.md | 证据回 Design59 对应生产任务并验全部 CL，含 AAC 自身未用的行 | 覆盖与实际代码版本一致 | AC-CL-OWNER |
| SD-AC-WAVE2 | modify | docs/specs/ace-migration/m2-three-backend-parity.md 及 m2-certification.md | 接 1097 W2 及 W3 后验三端 | 阶段独立交付 | AC-AC-WAVE2 |

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AACCL-01 | modify | `docs/specs/auto-ac/aac-source-frontend.md` | 增补 `for <binding> in <collection>` 语句面（aac profile）、AST 角色与无损投影；range-for 语义不变 | parser 缺口是 S/std 第一墙 | AC-12 |
| SD-AACCL-02 | modify | `docs/specs/auto-ac/aac-complete-semantics.md` | 增补 Str/char 原语方法与 `char.from_codepoint` 静态、泛型构造事实；未知方法仍拒绝 | checker 缺口 | AC-13 |
| SD-AACCL-03 | modify | `docs/specs/auto-ac/aac-mir-lowering.md` | 增补 builtin callee 的 HIR 形、迭代/原语 MIR/ABC lowering 与拒绝分类 | 生产面落地 | AC-14/15 |
| SD-AACCL-04 | add | `docs/specs/auto-ac/aac-std-closure.md` | 新增编译车道 std 闭包装载、Windows 平台选择、`aac.pending-work/1` 分期规则 | 目前无规范描述该车道 | AC-16/17 |

## 6. 测试设计

### V-FC-CONSUMER（work / 原T-01/T-02/T-06/T-08，未执行）

同一canonical source变化必须使本包实际目标/代际依赖失效并用独立oracle重验；单目标oracle变异、遗留旧活入口、删原义务映射、零选中/缺required检查和emit冒run分别判红。公共语言输入去重不减少旧资源/错误/恢复/平台条件；typed IR/后端专项及外仓完整产品保原owner。实际结果按Case＋check＋编译器/目标/动作＋source/SDK/provider身份记录，不以AV或其它代结果回填。

**AC 语法同源验收。** 按 [Design59 共用语料规则](../design/59-aac-native-bootstrap-and-compiler-subject.md#ac-复用-av-的-autodown-语言语料)接 AV 1078–1096 的 AutoDown 作者源。相同 .case.ad、Case ID、Auto 源码和独立预期分别记录 AV 与 AC 检查。正例验 AC 机器产物实际执行，负例验原阶段和诊断。T-02 先收已有用例的结果。T-06/08 在 1096 正式交 `IF-AV-LANGUAGE-PARITY/1` 后对账同版全量语料，任何必需行仍缺用例或 AC 结果均保留待验。

| V / AC / owner | 真实输入与独立关闭oracle |
|---|---|
| V-CFG-TARGET / AC-CFG-HANDOFF / T-03/05/07/08 | 使用982实际配置选择两份源包（拟fixture由原owner建成，业务返回7/9），宿主新AV配置身份/值分别指向正确包，AC/A2R目标实际运行各对7/9；切断guest AV仍执行、错配置/旧cache/缺新配置桥须拒。不得用配置AV的成功冒目标业务或动态配置API已支持。 |


### V-CFG-HANDOFF（本轮新增接收控制，未执行）

沿[配置设计九行Case](../design/ace-config-document-evaluation.md#6-验证合同与旧用例映射)，在本包所用真实consumer/原测试入口验合法动态值及坏源/错SDK-context/零旧求值/失败保旧/修复下一次read；把route C legacy config冒ACE、只核same_sdk、缺source或仅用静态投影则拒绝。原V全部保留，不新造runner。实际未用配置的域不增加虚假依赖；原平台/ignore/输入规模保持。

### V-AGE-CAP-HANDOFF（新增分阶段控制，原V全部保留）

按§5原任务冻结实际cap/owner/源/平台与各端oracle：原UI波基线齐且新cap pending可交对应UI-W1/2/3，但不能签cap/AGE全齐；cap合法来源可逐端交，删cap/错源/把AV冒AC或仅emit冒A2R必须红，恢复真实产物可通过。用于新增cap的typed接口须先有独立合法/错误/owner测试，消费方真实GPU/交互/资源/包另验；缺模型/Forge产品不影响基线。AC-AGE-CAP-HANDOFF由本表原owner及最终review核。


下表均为未实施合同；原T任务将其接到真实入口，保旧输入规模/判据/资源义务。三端分别对独立expected，不仅相互比较；确定性输出逐字节，异步合法交错按因果/完成/清理合同判，不强求同一时钟值或线程排布。

| V / AC / work owner | 实际输入与入口 | 可区分oracle与负控 | 覆盖边界 |
|---|---|---|---|
| V-01 / AC-01 / T-01、06 | 985原义务表、A后延账、全部原源，逐行triad/平台/阶段/source | 漏~T/泛型/标注/CT/公共库/领域行、以未实现当不适用或删后延行必须拒绝 | 复用原矩阵，不造第二registry |
| V-02 / AC-02 / T-02、03 | 泛型模块返回List<Option/Result<owned>>、闭包、CT生成/标注消费、真实File/net/DB/FFI；各端真执行 | 独立值/字节/错误stage/span/资源，异步仅合法因果trace；共同错误、只emit或AC转AV须红 | 既有A2PY std/AV provider另保；A2C缺完整面不阻M2 |
| V-03 / AC-02/03 / T-04 | 同源UI/AGE状态/typed事件与host ABI；真实native guest handler及两实例关闭 | CPU状态/事件/析构独立断言，所有权/错ABI运行前拒；只有AV窗口或A2R图片不代AC | 域owner真实renderer/窗口门仍保，不能缩成序列化文件 |
| V-04 / AC-03 / T-05 | 正式AAC实际模块/宿主链编为AC运行；Unicode/嵌零、模块依赖、合法与坏源、失败后修复 | 独立阶段AST/符号/诊断/输出文件，源码及producer身份；手写替身、Rust源码回落与自举自证应被识别 | 不要求Gen2/3；AAC未用行仍须V-02/03 |
| V-05 / AC-04 / T-06、07、08 | 冻结三端完整影响组、仓外SDK/恢复与B收据；同源改库/语法/ABI后恢复 | 完整差额=0，原错配/静默fallback/删行/伪PASS负控拒绝；平台支持标签真实，旧输入规模不变 | Linux NOT_RUN归1063，不用延期豁免Windows或删三端能力 |

现有候选固定组：`cargo t ace-source ace-semantics ace-lowering ace-av ace-native ace-a2x ace-runtime`，调度用`case-ace-m2-scheduler-968`，库/FFI/SDK按真实触面加入`ace-stdlib stdlib ace-foreign-contract ace-abi ace-artifacts`。T-01 核实际组 ID/选择，所需 native/AAC/GUI 入口由相应生产任务交付到原组后接收，不将拟名当现成命令。普通组不带Linux/performance；正式性能仅显式专项，保原成本和必要集成门。

### 显式复验

| 验证 | 合同 |
|---|---|
| V-AC-DESKTOP / OWNER / PRODUCT | 宿主两项取 1097，PRODUCT 取 1102/1104/1105 的实证。T-04 核机器业务、实际窗口/事件/对象寿命与坏 ABI。T-07 在 978 候选 SDK 上按原独立 GPU/输入/资源/失败反控复验真实仓外产品。 |
| V-CL-OWNER | AAC 某一阶段 PASS 且未使用 required async/标注时，缺对应同源真执行用例或产物即拒最终三端；复用独立 expected 并检查来源。 |
| V-AC-WAVE2 / NOCYCLE | 验 1097 已正式完整交 W2 的覆盖/反控、1074 实际消费 W2 的身份。留 required AC 行或缺正式 W3 时，T-06/08 最终汇总仍失败。 |

### V-AACCL（2026-11-24 并入的 AAC CL 生产验证，未执行）

入口=cargo t ace-aac-syntax（先 `prepare-native` 双 driver + `AAC1046_NATIVE_DIR`/`AAC1047_NATIVE_DIR`）、语义车道 `cargo nextest run --offline -p ace-aac-syntax-conformance --features aac-semantics --lib --tests --config-file ../../.config/nextest-full.toml --test-threads 1`、`cargo t ace-aac-gen1`（`AAC1053_GEN1_DIR`）、`cargo t ace-aac`；全树清单工具按 T-17 落材料区。

| V / AC / owner | 输入 | 独立 oracle 与负控 | 边界 |
|---|---|---|---|
| V-11 / AC-11 / T-11 | master 现 S/std 闭包 85 模块 + 1054 清单 | 逐模块阶段/span 与 1054 清单一致或差异有解释；改动不得改变既有固定组 oracle | 仅测绘与决策，不施工 |
| V-12 / AC-12 / T-12 | `for x in <List>`、`for c in <Str>`、空集、嵌套；缺 in/range 误用/绑定重名 | 合法形 CST 无损且绑定角色正确；非法形 source 阶段具名拒绝；range-for goldens 不变 | aac profile；不改其它 profile 字节 |
| V-13 / AC-13 / T-13 | 三类原语调用 + `List<T>.new()`（含嵌套）；`char_at(Str)`、`from_codepoint(bool)`、未知名 | 正确事实/类型；非法 type 阶段具名拒绝；未知方法仍 `type.unresolved` | 不引隐式转换 |
| V-14 / AC-14 / T-14 | 上述合法形 | 既有 HIR 文本逐字节不变；新形进闭集；未知 builtin 拒绝 | HIR schema 只增闭集 |
| V-15 / AC-15 / T-15 | 迭代求和、char_at/索引、codepoint 往返 | MIR/ABC 产出且 AV 执行值=独立字面 oracle；资源/失败分类正确 | base profile；op 增补走 av-core 合同 |
| V-16 / AC-16 / T-16 | S 的 10 个 std 模块 + 4 个 pending 面 | 绑定/定型可达；pending 面逐条具名（实现或分期+阶段码），不得空成功 | 平台面按 Windows；Linux 不扩 |
| V-17 / AC-17 / T-17 | 全树 85 模块 project | 越过 parse/bind/typecheck 进 HIR/MIR；剩余拒绝为完整具名清单（阶段+code+span），零 trap；固定组无新增红 | 不含 Gen2/G2 |

## 7. 验收标准

- [ ] **AC-FC｜owner_stage: work**：V-FC-CONSUMER及§5 FC逐原义务满足，公共语言唯一作者/本目标真实动作与独立结果/迁移映射均可追溯；原独立review复核，merge只接真实落主来源。未交required目标保pending，不因共享目录、少文件或另一端通过而降门。

- [ ] **AC-CFG-HANDOFF / work**：§5本包实际配置consumer/阶段的正式来源和V-CFG-HANDOFF闭合，区分宿主AV与目标业务、原A与完整B；不以未交接口/上游状态或peer-equal代真实consumer与独立oracle。原工作任务负责，原final独立复审核验。

- [ ] **AC-AGE-CAP-HANDOFF / work**：V-AGE-CAP-HANDOFF及§5原生产者与本包消费的逐 cap 义务闭合；UI三波与新图形cap来源分开、无循环、原验收不减，新cap不成为985/A新增前置。阶段R/fold不冒整包final。


| ID | owner_stage | 完成谓词 |
|---|---|---|
| AC-01 | work | A后延每行有B实现/验证去向，最终同平台三端required差额=0；V-01闭合 |
| AC-02 | work | 语言/库/FFI/领域真实triad满足独立oracle，V-02/03闭合，不能以fallback或适用域缩减过门 |
| AC-03 | work | 实际AAC模块及资源/失败链真AC运行，未使用特性有补验；V-03/04闭合 |
| AC-04 | work | 真候选/恢复/来源及完整矩阵通过，供985/B/986/987消费；V-05闭合 |
| AC-DESKTOP | work | T-04 接 1097 的 V-AC-DESKTOP*，核 V-03 真实 AC机器业务＋同一Rust/Iced宿主、事件/owner/错误/包/域格齐；IF-1068-AC-DESKTOP正式可供981/985使用，Hello/AXR0/AV或仅CPU结果不代验 |
| AC-REVIEW | review | 独立上下文沿真实producer/consumer核全部AC/V/SD/差额，不用本包自报PASS；缺项退回对应生产任务重交 |
| AC-LAND | merge | 已审验收结果、Spec/来源正式合入，收据供985/B；归档及实际所有输出scope清理、wt-guard/树/分支收尾完成 |

- [ ] **AC-CL-OWNER / work**：V-CL-OWNER满足，AAC语义/现AC机器补齐与完整CL来源分清，CS未用行有实际补验；原T-02/T-05/T-06/T-08负责，原独立review核，原10任务和0进度不变。

| ID | owner_stage | 标准 | 核验 |
|---|---|---|---|
| AC-AC-WAVE2 | work | UI-W1-AV之后，全部UI/Widgets AC机器执行/真实窗口/事件/生命周期/错误/资源独立oracle通过可正式交UI-W2-AC，第三波仍pending；最终T-06/08另核三端 | T-04 核 1097 阶段收据及 V-AC-WAVE2/NOCYCLE；原最终 T-09/10 保留 |

### AAC CL 生产验收（2026-11-24 并入）

| ID | owner_stage | 完成谓词 |
|---|---|---|
| AC-11 | work | T-11 决策记录落盘（生成器/形态/路径），85 模块清单刷新并与 1054 证据可追溯；V-11 闭合 |
| AC-12 | work | `for <binding> in <collection>` 在 aac profile 实现并投影；V-12 全绿，range-for 与既有 parse goldens 零回归 |
| AC-13 | work | 原语方法与泛型构造事实落地；V-13 全绿，未知方法仍拒绝 |
| AC-14 | work | HIR builtin callee 闭集 + verify；V-14 全绿，既有 HIR 文本零变化 |
| AC-15 | work | 迭代/原语 MIR/ABC lowering + AV 独立执行；V-15 全绿，拒绝分类正确 |
| AC-16 | work | S 的 std 闭包在 project 车道可绑定/定型；pending 面逐条具名；V-16 闭合 |
| AC-17 | work | 全树 project 编译越 parser/checker/std 面（MIR/ABC 或完整具名清单）；`ace-aac-syntax`/`ace-aac-gen1`/`ace-aac` 无新增红；V-17 闭合 |

上表证据同时受既有 AC-REVIEW（T-09）与 AC-LAND（T-10）覆盖；AAC CL 生产不另设 review/merge 任务。

## 8. 执行步骤

原 T-01..T-10 全部 0/10；2026-11-24 并入的 AAC CL 生产 T-11..T-17 为 0/7，证据接收与最终验收均待执行。

- [ ] **T-01｜work｜AC-01**：核 985/A 与原 required 矩阵，冻结原源/Case、独立判据、适用平台与生产任务。缺证仅登记验收状态。
- [ ] **T-02｜work｜AC-02**：按 §5 对照原语言和组合行收 1046–1051 正式输入，核当前 AC 真执行、共享源/产物、诊断与资源，交 V-02/V-CL-OWNER 的语言部分，登记已有 AV AutoDown Case 的 AC 同源验收结果和待对账行。
- [ ] **T-03｜work｜AC-02**：按 §5 收库、调度、配置与 ABI 的正式结果，核真实 File/net/DB/FFI/OS、权限、资源、错误和配置宿主/目标区分。
- [ ] **T-04｜work｜AC-02/03**：消费 1097 正式宿主/W2、1102/1104/1105 的 AGE AC 和各端域来源，核真机器 guest、UI/Widgets 全分母、生命周期与资源，保 V-03、V-AC-DESKTOP* 和 NOCYCLE。
- [ ] **T-05｜work｜AC-03**：消费 1044/1045 及所用正式 AAC 模块，复跑实际宿主和源码/模块处理链，按 V-04 验预期与来源；独立核源码矫正后恢复。
- [ ] **T-06｜work｜AC-01/02/04**：在既有入口逐行复跑 AV/AC/A2R，含 AAC 自身未用的语言、库、FFI、领域和组合；图形并集待 W3。A2C、A2PY、AV provider 按原范围取证。
- [ ] **T-07｜work｜AC-04**：用 978 同版候选 SDK 在仓外复验真实构建/运行、依赖、ABI/配置变化、坏输入、旧产物和恢复。图形含真实 Windows AC 产品；Linux 记 1063 NOT_RUN。
- [ ] **T-08｜work｜AC-01..04**：核 V-01..05、1096 正式同版验收分母与每个适用 AC Case 的实际结果、对应 W1/W2/W3 与域证据，完整 required 差额为 0 后冻结范围、同版身份及正式结果，交 985/B。
- [ ] **T-09｜review｜AC-REVIEW**：独立核生产证据、三端运行与逐行 oracle，反查缩分母、AAC 子集冒完整支持、fallback、领域和资源遗漏。
- [ ] **T-10｜merge｜AC-LAND**：正式合入验收结果、支持范围及证据簿记，交 985/B/986/987；沿原流程清理阶段材料和工作树。
- [x] **T-11｜work｜AC-11**：AAC CL 基线与 parser 缺口决策（**master 现有前端**：256KiB 模块准入/64K 列表；std 闭包单模块 ≤41KB 全部可测，S 的 341KB grammar_data 留 T-17）。重绑 master S/std 闭包身份并重跑可达模块门面清单（std 18 + S 中 ≤256KiB 的模块）；判定 `for x in <collection>` 属 profile 缺生产 / 投影未接 / 形状不匹配，落 `plan-artifacts/plan-1068/aaccl-gap-decision.md`（含 grammar data 生成器命令）；同步在 Design59 §3 归属表补记 1068 承接。验证：清单与 1054 证据逐条对齐、`prepare-native` 双 driver rc=0。
  - [✅ 已完成] 2026-11-24：决策记录 plan-artifacts/plan-1068/aaccl-gap-decision.md；master@7bb90b1f7 双 driver rc=0（316s）；83 项门面扫描（26 bind / 1 source.syntax / 6 source.projection / 5 type / 4 pending / 1 hir / 8 大模块 	rap:ListIndex）；or x in xs 在 aac 面 source.syntax，for-cond/for-range 可解析；grammar 加 aac/重排 alt 均不改变解析行为，Rust 参考 auto/aggregates 解析同源 OK → 缺陷归自举解析器分派/回退或 aac 裁剪，T-12 首步用 Rust+aac 表二分定位；生成器=generate-grammar bin（未改 grammar 时逐 profile 复现字节一致）。
- [ ] **T-12｜work｜AC-12**：parser 列表迭代。按 T-11 结论改 aac profile / grammar 源并重新生成 `syntax_grammar_data.at`（不手改数据文件），或补 `syntax_parse` 投影；新 golden 覆盖 List/Str/空集/嵌套与缺 in、range 误用、绑定重名三类负控。验证：`cargo t ace-aac-syntax`（预期 51/51 或仅既有并发账）+ 语义车道 parse 用例 + V-12。
- [ ] **T-13｜work｜AC-13**：checker 原语与泛型构造。`types/typecheck/binding.at` 增 `Str.char_at`、`char.to_codepoint`、`char.from_codepoint`、`List<T>.new()`（含嵌套）事实；`paged.at` 与 `syntax_grammar_data` 两个 S 内实例单独定位。验证：V-13 + 错参/未知名负控。
- [ ] **T-14｜work｜AC-14**：HIR builtin callee 闭集。`core_hir.at` 增 `callee: "b:<name>"` 形与 verify 白名单；既有程序 HIR 文本逐字节不变。验证：语义车道 HIR 用例 + V-14。
- [ ] **T-15｜work｜AC-15**：MIR/ABC 迭代与原语 lowering。`base_ir/lower_base/verify_base/base_abc.at` 落地索引循环与 char/文本原语；缺 op 时按 `av-core` 合同增补 + conformance。验证：`run_base_mir_case` + AV 独立 oracle + V-15。
- [ ] **T-16｜work｜AC-16**：std 闭包与待办面。project 车道 `std/<stem>` + `gen-next` 物化（Windows 过滤 `*_lnx`）；S 的 10 个 std 模块绑定/定型；`aac.pending-work/1`（env/path/rand/time）与 `std/math` 逐条实现或具名分期（阶段码 + 受影响 CL 行）。验证：V-16 + `cargo t ace-aac ace-aac-gen1` 无新增红。
- [ ] **T-17｜work｜AC-17**：全树 project 门与固定组。**前置**：全树需要 PLAN-1054 T-10 的分页前端 + 512KiB 准入（该批在 1054 分支，尚未落主）；1054 交付前本任务只做"std 闭包 + ≤256KiB S 模块"可达门并如实记 pending。1054 交付后补 85 模块 project 编译，落 MIR/ABC 或完整具名拒绝清单（阶段 + code + span，零 trap）；扫描脚本落材料区；记录执行/排除/耗时。验证：V-17 + `cargo t ace-aac-syntax ace-aac-gen1 ace-aac`。

T-01 冻结验收分母。T-02–05 按实际输入签收，T-06/07 最终复验，T-08 汇总，T-09 独立复审，T-10 正式交接。发布差额归零以原 required 行逐条有证为准。

T-11..T-17 为 AAC CL 生产实现（2026-11-24 并入）：T-11 决策/基线 → T-12 parser → T-13 checker（可与 T-12 并行）→ T-14 HIR → T-15 MIR/ABC → T-16 std 闭包 → T-17 全树门；其证据随 T-08 汇总、T-09 独立复审、T-10 落主交接，供 PLAN-1054 重绑 S 身份续跑 T-02..T-09 消费。

## 9. 复审记录

- 2026-11-24 stage: work | PLAN-1068 | r12 | T-11 执行 | outcome: pass（AC-11 达成：决策记录+基线清单+身份落地） | 证据：`plan-artifacts/plan-1068/aaccl-gap-decision.md`、`t11-scan-master.txt`、`t11-drivers/`；关键事实：master@7bb90b1f7 无分页前端，≥50KB S 模块 `trap:ListIndex`（8 个），`for x in xs` 在 aac 面 `source.syntax`，grammar 已有 for-in 规则（aac 表含名），Rust 参考 auto/aggregates 解析同源 OK；结论=缺陷在自举解析器分派/回退或 aac 裁剪，T-12 首步二分定位 | next: work（T-12）

- 2026-11-24 stage: new/revision | PLAN-1068 | r12 | outcome: pass（规划并入，非实现/认证 PASS） | 用户裁定：AAC CL 生产实现并入本包，`drafting → executing`；新增 T-11..T-17 / AC-11..AC-17 / V-11..V-17 / SD-AACCL-01..04；原 T-01..T-10 与 0/10 进度保持；原 PLAN-1130 撤回（内容=本批任务，git 历史保留） | next: work（T-11 起；先重绑 master S/std 身份再改生产面）


- 2026-11-19 r10规划交接：明确AV路线只选AV，非AV#ignore由本目标实施阶段逐项恢复；原任务/状态/进度、完整门与原预期保持，本轮没有实现或运行。

- 2026-11-19 r9：`stage: new/revision`，`outcome: pass`（规划静态预检），`next: work`待后续指令/所需来源。新增FC/V-FC-CONSUMER/AC-FC，原T-01/T-02/T-06/T-08补接同源语言/迁移合同，任务/进度不变；未移动用例、修改expected、运行或合入。

### 2026-11-19 AV语言producer拆批交接（r7；仅规划）

`stage: new/revision`；`outcome: pass`（依赖/范围静态预检，非实现复审）；`next: work`待后续实施指令。§0/原T-01、T-02、T-06、T-08接1078–1096正式AV语言来源，最终`IF-AV-LANGUAGE-PARITY/1`不能以代表profile代签；不改变其它端原AC，不新增整包开工等待，不形成1068↔1096环，原0/10保持。未施工/测试/合入。

2026-11-19 r7 规划交接。T-02/03/04 调整为证据接收，原实现由各生产计划核验交付。以下为历史记录。

### 2026-11-17 CONFIG-ACE-B规划交接（r6；非实施验收）

stage: new/revision | outcome: pass（规划合同预检） | 本轮用户仅授权补承接/设计/接口/验收及排程；未work/建树/运行测试/探针/构建/合入/归档/派agent。GAP-982-CONFIG-ACE由982原B-CONFIG两仓端到端承接，能力本身保持未实现；本包按§5–8现任务生产或接收，不新增Plan/任务，不改已验A收据。下一步须未来实施指令，982先B-CONFIG-I全链前探/接口与原输入冻结，再V→独立R→正式phase交接；其他owner按所用正式来源接。

取材和静态核验：7计划123原任务ID/顺序/勾选与状态/进度保留，语义revision各增1；仅982补已交967/979直接基础依赖。9个配置判别Case/4个能力AC、typed接口/值语义/预算/生命周期/原用例映射与Spec候选已列；24节点30边阶段图无环，新增本地链接可达，18份源码/Spec/归档/计数器身份不变，目标文档git diff --check通过。旧无owner快照/route C/执行回执作历史保留，不把规划PASS当真实能力或独立产品review。M2A-08 r29/95c7fbad8冻结不变。

### 2026-11-17 三路线修订的规划预检（不是实施验收）

本轮仅文档：统一strategy/架构、四个引擎AV新合同与四个新版AutoForge合同；保本包原任务ID/进度/全部旧AC/V/SD，新增阶段义务见§5–7。静态核对8新Plan共74任务、9既有Plan共140原任务ID、阶段DAG 44节点/90依赖无环、新增本地链接均可达，16份取材源码/规范/已归档合同身份未改变；四仓文档diff格式检查通过。981并发实施会话的T-01勾选保留，不计本轮交付。下方历史记录原样保留。

只说明规划合同可审阅；未运行Cargo/GUI/模型/构建/产品测试，未建实施树、未resume暂停工作、未合入/归档/签章。所需正式接口/最低负载/真实oracle仍须未来work→独立review→merge实际验证；原已存在历史失效链接不作为当前入口，新增链接已单独核对。


2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r3：核对AAC后补SD/V/AC-CL-OWNER与真实分账；保已有AC桌面详细合同、原任务/依赖/0进度。本轮仅规划检查，无实现/探针/运行，既有语言支持事实未改。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r1：用户已裁定三端最终对齐及A→B延期边界；本稿可供规划审查，未实施、未测试、未签M2/AAC。静态核验记录另补，实际work仍核阶段门和来源。

2026-11-16 stage:new/revision | PLAN-1068 r2 | outcome:pass（规划/静态交接，非产品PASS） | changed:T-04/07、AC-DESKTOP、V-AC-DESKTOP*、SD-AC-DESKTOP | preserved:drafting/0-of-10及原任务/依赖/三端与AAC原义务 | next:future work先M2A-08和精确正式输入；当前仅规划，不启动树/实现/运行。

本轮规划静态预检：9份计划/设计路线文档；四份Plan共87个原任务ID/总分母与依赖列表保持，仅980的NATIVE I/V/R因新增合同重开（15/29→12/29），历史收据未删。新增本地链接/锚点、0–10章节、验收/用例/原任务映射、表格与行尾检查通过；A入口→A出口→AC切片→B产品/终验的阶段图无环。候选固定组名按scripts/test-groups.toml只读核实，新UI/AC入口或case明确由work落实，均未运行。本记录是文档预检，不是产品或独立实现复审PASS；未改canonical Specs/源码/测试资产/取号计数器，未实施/建树/构建/运行/合入/清理/恢复目标。

2026-11-16 stage:new/revision | PLAN-1068 r4 | outcome:pass（规划可执行合同，非实现/认证PASS） | 原10任务/0完成保持；T-04明确W2全集与独立阶段交接；T-06/07/08后收W3，保留AAC语义owner/真实模块/完整CL补验原r3合同 | next:由原owner依新阶段/真实来源接续；本轮只规划，不实施/建树/测试/合入/清理/恢复目标。

本轮规划静态预检：14份计划/设计/路线文档，8份Plan共230个当前任务；仅980新增B-AV-I/V/R三项并按完整W3范围重开原RUST三项，旧AXR0收据保留；981仅核正原SOURCE三项已交的滞后计数。全部原任务ID/其余勾选、各Plan状态与整包依赖列表保留；新增本地链接/锚点15处、章节/表格/元数据计数/验收任务映射及行尾检查通过，A最小→A出口→W1→W2独立交接→W3→OS→候选/安装/退役阶段图16节点18边无环。原GUI/Widgets/Notes-Clock/语言/库/产品义务有具名阶段与原owner，不以延期冒PASS。仅文档预检，未实施/建树/运行测试或窗口/签阶段/合入/清理/恢复目标，未写源码、测试资产、canonical Specs或取号计数器；他会话并发修改不由本轮回退。

## 10. 待澄清事项

2026-11-24：T-11..T-17 的专树沿用本包 §3 的 `D:/autostack/.wt/lang-1068/auto-lang` @ `plan-1068-dev`
（当前尚未建立，T-11 开工时创建；材料区 `D:/autostack/plan-artifacts/plan-1068/`）。原 PLAN-1130
已按用户裁定撤回，其清单（85 模块门面扫描）作为本批 T-11 输入。AAC CL 的 AV runtime op 增补
（若 T-15 触发）须按 `av-core` 合同走并具名交接，不在本包私改其它计划冻结面；G2（mn 16MiB）
仍归 AGE elem-view owner，Gen2/代际签章仍归 PLAN-1054。

T-01冻结实际差额而非仅设计清单；T-04按现产品owner确定已有ABI可复用或需新增的最小AC接缝；T-05选实际已交AAC模块及有界工作负载，不能臆造未存在入口。当前仅规划，不启动实现或AAC。

日常等价实现选择由T-01/02有界调查决定，不为已授权方向重复申请确认。未提供平台环境保持NOT_RUN；计划结构检查不能代实际运行证据。
