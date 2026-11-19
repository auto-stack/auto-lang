---
plan_id: PLAN-1055
status: drafting
feature_name: AAC基础Typed MIR与CT启动闭环
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-19
plan_revision: 4
current_step: 0
total_steps: 8
depends_on: [PLAN-1044, PLAN-1045, PLAN-1047, PLAN-1051]
supersedes_spec_components: [docs/specs/auto-hir/stage-contract.md, docs/specs/auto-mir/architecture.md]
new_spec_components: [docs/specs/auto-ac/aac-base-mir-and-ct.md]
touched_goals: [GOAL-002]
---

# [PLAN-1055] AAC基础Typed MIR与CT启动闭环

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包按原任务收实际生产任务的证据，安排已验阶段交接。

2026-11-16 r3本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r2当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r1，本轮AAC规划复核增补。把原1050的早期Auto HIR→Typed MIR和本地准入独立成先交批次，使1048能实际通过AV执行CT，消除隐式1048↔1050启动环。 仅规划，8任务均未执行；不建树、不写实现/fixture/探针、不编译、不运行产品，不签M2-A/AAC。

## 1. 目标

把原1050的早期Auto HIR→Typed MIR和本地准入独立成先交批次，使1048能实际通过AV执行CT，消除隐式1048↔1050启动环。 基础profile是1047已受检的闭合单态纯计算函数：冻结M2-A中已交精确标量/布尔、局部/直接调用、分支/短路/循环及参数/返回映射；没有待决CT/泛型/高级资源义务。泛型先实例化为这些具体类型即可进入。未包含的owned/closure/dynamic/async/领域和高级CT输入保持逐行1050/1053必需去向，基础出口不能称全语言lowering完成。

依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)和[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC在M2-A稳定后与M2-B可并行，独立后续版本/验收，非M2硬门。非目标：不重写AV/Cranelift/全部A2X，不迁全部产品，不改旧M2分母/旧oracle，不用旧AAVM证书证明AAC。

## 2. 架构方案

Auto主体拥有source语法/语义/实例/CT调度/HIR/MIR和增量策略，Rust只保机器后端/消费侧安全复验/AV执行及明确外部runtime。Gen0编译阶段driver可用其Rust前端，driver运行处理被测source/IR时算法必须Auto执行；1053/1054再核Gen1/2/3主体窗口。序列化checked标志没有信任权威。

拟新增或扩展：experimental/ace/aac/mir/基础实现、host/cte-entry/及tests/mir-base/（拟新增）；以后1050扩此同一实现，不复制第二份MIR。实际共享路径由供给owner单写，后继消费正式已交接口，不读其他树WIP。所有源码/工具/stdlib/grammar/ABI/schema/target/provider分轴标识，build/host/target/CT执行域分清。

## 3. 技术栈

纯Auto+新版core/alloc/必要hosted标准库，默认自身ABI与OS adapter；首个独立host/native基线Windows x64，其他host按原设计后继，不改变M2平台义务。需要公共库新增时按现覆盖合同，不复制通用stdlib或默认依赖libc；显式后端/AV/链接工具的外部依赖单列。

## 4. 需求分析与背景调查

本轮用户明确授权再次验证1044–1054，必要时补齐细化、先不施工。新增本Plan属于此规划范围，不是代码施工许可。G-AAC须M2A-01..08、985/T-A04独立复核与T-A05已落主来源稳定收据，并有未来实施授权；不等待整个985/B、984/986/987，也不给M2添加本Plan前置。

本轮只读观察auto-lang HEAD `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`。具体源/Spec取材：

| 来源（仓根相对路径） | SHA256前16位 |
|---|---|
| `experimental/ace/crates/lowering/src/hir_adapter.rs` | fa54e8c60ac15ff3… |
| `experimental/ace/crates/av-core/src/mir.rs` | 6e85b54663fa5e04… |
| `experimental/ace/crates/av-core/src/mir_verify.rs` | 9f1427164bd6a54f… |
| `experimental/ace/crates/source-cte/src/executor.rs` | a970487d747940ca… |

规范取材：`docs/specs/auto-hir/stage-contract.md`; `docs/specs/auto-mir/architecture.md`。旧overview/早期注释不是当前全语言事实；源/Spec冲突须具名解决，不能把规划写成实现。原正式输入在work时重新锁版本/hash；本表只读快照不替M2-A稳定收据。

## 5. 详细设计

frontmatter为本Plan完整交付前置；§8任务再细化本包批次。开工先核G-AAC和依赖已正式落主，未来每个driver首次使用任务建真实入口/命令及必要target/固定选择，T-06实际运行本包Auto资产和受影响Rust职责组；不等1053才第一次测试。

本包的需求矩阵、阶段profile与支持边界见§1，具体数据路径/owner/输出见§8，正反例与独立结果见§6。1044冻结现有更严预算、source/node/depth/bytes/CT fuel/输出/期限和计数算术，合法支持边界不窄化，超界明示资源拒绝。合法/非法/取消/恢复分别保真实终态，不能临时增预算或把错误变NA。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-base-mir-and-ct.md` | 新增本包已验证profile/接口/所有权与支持边界；完整AAC及M2边界不变 | 关闭Design59 §9的具体缺口 | AC-01..04 |
| SD-02 | modify | `docs/specs/auto-hir/stage-contract.md` | review通过后补本包当前事实和新Spec指针，保原历史/未交项；本轮不写current Spec | 避免目标态冒当前实现 | AC-05/06 |

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-base-mir-and-ct.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本轮语言完整性边界

1055基础LowerCt仍按原有界profile先交，完整语言执行由1050/1053关闭，不为“全语言”把基础CT依赖环重新接回。 当前CL目标取最终M2三端required及A后延增量，来源身份以阶段真实版本绑定，不以旧A有限面封顶。

## 6. 测试设计

下列全部是拟议必验case，不是已运行结果。用冻结M2-A真实语法/IR/API构造；字面数值/字节/阶段/事件由独立oracle给定，不复制被测输出。必要fixture的创建任务已列入口列；Rust参考与独立规范/向量同时核，不能仅两端相等即PASS。

| Case ID | AC / owner_stage | 输入/状态 | 入口/启用任务 | 可观察独立预期 | 边界 |
|---|---|---|---|---|---|
| V-01 | AC-01/02/03/04；work | f(x)=2*x+1，实参3；布尔恒假且RHS为trap | 本包Auto driver→1051 AV/native，T-02/04/05启用 | 两consumer各得7、false；trap计数0，Auto实际lowering进入；Rust source和HIR lowering禁止窗口内调用数0 | 本包profile，未实施 |
| V-02 | AC-01/02/03/04；work | 合法MIR把一个DefRef改为越界或改参数Bool/Int映射 | Auto verifier及1051准入，T-03/05 | 受控拒绝，codegen/AV执行本次计数0；恢复原MIR仍正常，不能把字节解码当验证 | 本包profile，未实施 |
| V-03 | AC-01/02/03/04；work | CT访问未授权IO或超fuel，随后执行合法返回7请求 | Auto CT入口→1051 AV，T-04/05 | 按M2-A权限/资源类别失败、无CT冻结值，句柄/子进程回收；下一请求仍7 | 本包profile，未实施 |
| V-04 | AC-01/02/03/04；work | 给基础入口送尚未闭合CT或高级owned/dynamic输入 | 本包profile准入，T-02/03 | 明确非本基础profile或pending，不伪造Checked、不自动送Rustsourcecompiler；登记1050/1053对应必需行 | 本包profile，未实施 |
| V-05 | AC-01/02/03/04；work | 用同一f(3)请求，在基础MIR里把常量1改为2 | 本包lowering事件及真实两consumer，T-05 | 原请求独立结果7；变更MIR实际结果8，原7的oracle必须使反控失败，证明未用fixture返回值或wrapper自报PASS | 本包profile，未实施 |

现有受影响候选组 `cargo t ace-lowering ace-av ace-native`（只读查`scripts/test-groups.toml`确认，未运行）；work核真实触面，选完整职责组与必要consumer，并实际运行本包Auto driver。普通模块排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不成为每Plan收尾门。记录实际case/执行/条件排除数，准备/发现/运行/清理/总时间及输入工具产物身份；required未跑不能PASS。

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

## 7. 验收标准

| ID | owner_stage | 关闭条件及验法 |
|---|---|---|
| AC-01 | work | §8所有work输出与本包profile逐项对应，V-01/05实际观察值/字节/控制符合独立预期；不以文件存在代行为 |
| AC-02 | work | V-02/03/04的错误/资源/权限分类、合法近邻与失败清理均满足，无隐式Rust源语义/legacy回落 |
| AC-03 | work | 本包真正生产/消费边界成立，Auto主体/标准库单源/机器服务职责不偷换，schema/ABI/阶段身份及后继输入可复现 |
| AC-04 | work | 全部V行和本阶段required输出/测试已实际关闭，所有真实命令/退出码/独立oracle/输入和产物身份摘要入计划；不能拿后续pending豁免本阶段required |
| AC-05 | review | 独立同树按revision核每个AC与完整数据路径、遗漏/延期/workaround、warning/format/debug及规范delta，pass或具名needs_fix/needs_replan |
| AC-06 | merge | review通过、落主身份核准后沉淀Spec、归档计划并按材料政策清理交接；不签M2或完整AAC |

- [ ] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

## 8. 执行步骤

拟唯一树 `D:/autostack/.wt/lang-1055/auto-lang`，分支 `plan-1055-dev`。材料唯一 `D:/autostack/plan-artifacts/plan-1055/`，开工登记owner/用途/在途状态；SDK/生成输出/target/仓外消费均显式在scope，不放仓内证据，不建junction/symlink。当前树/材料未建立。

| Task ID / 状态 | owner_stage | 依赖 | 输入/实际工作 | 输出/验法 |
|---|---|---|---|---|
| [ ] T-01 基线与供给核对 | work | G-AAC及frontmatter实际已交输入、未来实施授权 | 核Plan r1、§4当前Spec/source/SDK/库/ABI/grammar/工具及唯一树和材料登记，记录兼容profile与数据接口 | 输入版本/缺口/owner及后继任务入口；AC-01/04 |
| [ ] T-02 基础lowering接口 | work | T-01 | Auto实现同一MIR的CFG/block/value/place/类型/调用builder、精确运算/短路/分支/循环lowering及source映射；接1044 LowerCt接口，不依赖1048 CT调度器或完整1050。 | 基础MIR与阶段driver，消费1047 Auto checked闭包；对应V行和AC-01..04实际核验 |
| [ ] T-03 本地MIR准入 | work | T-02 | Auto检查类型、ID/DefRef归属、CFG/控制流、调用参数/返回与基础effects，只有合法体可进入1051安全复验；不信wire checked标志，不从Rust lower_checked生成主体MIR。 | Auto verifier及坏ID/类型/CFG、trap顺序反控；对应V行和AC-01..04实际核验 |
| [ ] T-04 CT受检调用入口 | work | T-03 | 在Auto侧把字面实参与目标函数组成受检call entry，必要wrapper先走1047再lower；值/预期类型、target语义及capability/fuel契约明确，1051只执行已定型请求。静态meta查询不算AV执行。 | Auto CT执行工厂，可注入1048；不搬Rustsource-cte整链进服务；对应V行和AC-01..04实际核验 |
| [ ] T-05 真实闭环与兼容交接 | work | T-04 | 阶段driver用Auto parser/binder→本包lower/verify→1051 AV及native运行，检查独立值/顺序/失败资源；记录schema/profile和1050继承入口。原1050基础lowering义务移交但保原1050最终兼容复验。 | 正式已交基础闭环，1048不必等1050即可真实运行基础CT；对应V行和AC-01..04实际核验 |
| [ ] T-06 定向验证与交review | work | T-02..05 | 实际运行全部V行、本包Auto阶段driver及受影响完整固定组/必要consumer，核输入/工具/产物身份、真实父子终态、资源和清理，准备Spec delta | 逐项证据摘要，AC-01..04；execution_done交review，不预勾T-07/08 |
| [ ] T-07 独立复审 | review | T-06 | 同一树冻结revision，逐AC/完整数据流独立核验；搜漏项/延期/workaround，具名债务登记并核warning/format/Spec-impact | review结论及修复条件；AC-05 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、沉淀已验Spec并归档；所有输出路径对账，证据折入计划后清scope，wt-guard clean再移树/分支 | 独立交付/后继版本与清理摘要；AC-06 |

## 9. 复审记录

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r3：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r2：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14 stage=new，revision=1；规划已完成，outcome=blocked（等G-AAC和未来实施授权），next=开工门满足后work。本轮只是文档审计/计划补齐，没有实现/测试执行/独立代码review/合入；进度0/8，Design59 §9记本次发现与责任分配。

## 10. 待澄清事项

无须本轮额外用户决定。具体实现/等价数据结构由work依据冻结合同选；若源/能力/预算/接口不成立，先回具名owner或有界重规划，不自行豁免义务。本轮先不施工，所有计划前置仍待正式验证。
