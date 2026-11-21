---
plan_id: PLAN-1055
status: archived
feature_name: AAC基础Typed MIR与CT启动闭环
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-21
plan_revision: 5
current_step: 8
total_steps: 8
depends_on: [PLAN-1044, PLAN-1045, PLAN-1047, PLAN-1051]
supersedes_spec_components: [docs/specs/auto-ac/aac-source-frontend.md, docs/specs/auto-ac/aac-semantic-authority.md, docs/specs/auto-hir/stage-contract.md, docs/specs/auto-mir/architecture.md]
new_spec_components: [docs/specs/auto-ac/aac-base-mir-and-ct.md]
touched_goals: [GOAL-002]
---

# [PLAN-1055] AAC基础Typed MIR与CT启动闭环

## 0. 变更摘要

2026-11-21 r5。用户明确授权 1055 在 auto-lang 同仓补齐 r4 探针确认的前端缺口。T-01 从依赖核对扩为交出所需 Auto 语法及 Checked HIR，T-02 仍由同一受检生产者消费。保留原 AC、V、基础 profile 和下游义务。

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包按原任务收实际生产任务的证据，安排已验阶段交接。

2026-11-16 r3本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r2当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r1，本轮AAC规划复核增补。把原1050的早期Auto HIR→Typed MIR和本地准入独立成先交批次，使1048能实际通过AV执行CT，消除隐式1048↔1050启动环。 仅规划，8任务均未执行；不建树、不写实现/fixture/探针、不编译、不运行产品，不签M2-A/AAC。

## 1. 目标

把原1050的早期Auto HIR→Typed MIR和本地准入独立成先交批次，使1048能实际通过AV执行CT，消除隐式1048↔1050启动环。 基础profile是1047已受检的闭合单态纯计算函数：冻结M2-A中已交精确标量/布尔、局部/直接调用、分支/短路/循环及参数/返回映射；没有待决CT/泛型/高级资源义务。泛型先实例化为这些具体类型即可进入。未包含的owned/closure/dynamic/async/领域和高级CT输入保持逐行1050/1053必需去向，基础出口不能称全语言lowering完成。

依据[Design59](../../design/59-aac-native-bootstrap-and-compiler-subject.md)和[AAC路线](../../design/strategy/aac-delivery-roadmap.md)。AAC在M2-A稳定后与M2-B可并行，独立后续版本/验收，非M2硬门。非目标：不重写AV/Cranelift/全部A2X，不迁全部产品，不改旧M2分母/旧oracle，不用旧AAVM证书证明AAC。

## 2. 架构方案

Auto主体拥有source语法/语义/实例/CT调度/HIR/MIR和增量策略，Rust只保机器后端/消费侧安全复验/AV执行及明确外部runtime。Gen0编译阶段driver可用其Rust前端，driver运行处理被测source/IR时算法必须Auto执行；1053/1054再核Gen1/2/3主体窗口。序列化checked标志没有信任权威。

拟新增或扩展：experimental/ace/aac/mir/基础实现、host/cte-entry/及tests/mir-base/（拟新增）；以后1050扩此同一实现，不复制第二份MIR。实际共享路径由供给owner单写，后继消费正式已交接口，不读其他树WIP。所有源码/工具/stdlib/grammar/ABI/schema/target/provider分轴标识，build/host/target/CT执行域分清。

## 3. 技术栈

纯Auto+新版core/alloc/必要hosted标准库，默认自身ABI与OS adapter；首个独立host/native基线Windows x64，其他host按原设计后继，不改变M2平台义务。需要公共库新增时按现覆盖合同，不复制通用stdlib或默认依赖libc；显式后端/AV/链接工具的外部依赖单列。

## 4. 需求分析与背景调查

本轮用户明确授权再次验证1044–1054，必要时补齐细化、先不施工。新增本Plan属于此规划范围，不是代码施工许可。G-AAC须M2A-01..08、985/T-A04独立复核与T-A05已落主来源稳定收据，并有未来实施授权；不等待整个985/B、984/986/987，也不给M2添加本Plan前置。

2026-11-21 用户通过 `/goal 完成计划1055` 授权 r4 的执行、独立复审、合约内修复及合入归档收尾。T-01 核实 985/T-A04 独立复审与 T-A05 的正式 M2A-08 已落主（`95c7fbad8`）；直接依赖交付 `1044@6022f10f2`、`1045@cc8053e32`、`1047@7bd65c763`、`1051@1d949f30b` 均为工作树基面的主干祖先。1047 Checked HIR 与 1051 消费接口的实际能力仍按 §8 逐项核对。

同日用户答复“授权 1055 补齐前端供给并继续”，覆盖 r5 对同仓 AAC 语法和 Checked HIR 生产者的增量。本次使用原工作树、原材料 scope 和验收门。1046/1047 的归档仍作为原始交付来源，新增支持随 1055 合入。

本轮只读观察auto-lang HEAD `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`。具体源/Spec取材：

| 来源（仓根相对路径） | SHA256前16位 |
|---|---|
| `experimental/ace/crates/lowering/src/hir_adapter.rs` | fa54e8c60ac15ff3… |
| `experimental/ace/crates/av-core/src/mir.rs` | 6e85b54663fa5e04… |
| `experimental/ace/crates/av-core/src/mir_verify.rs` | 9f1427164bd6a54f… |
| `experimental/ace/crates/source-cte/src/executor.rs` | a970487d747940ca… |

规范取材：`docs/specs/auto-hir/stage-contract.md`; `docs/specs/auto-mir/architecture.md`。旧overview/早期注释不是当前全语言事实；源/Spec冲突须具名解决，不能把规划写成实现。原正式输入在work时重新锁版本/hash；本表只读快照不替M2-A稳定收据。

T-01 在 r4 基面复核 Windows x64 MSVC / hosted-av / AutoABI。以下为工作树提交中的 Git blob 前 12 位，工具为 rustc 1.98.0（`88d9e12ae`）、Cargo 1.98.0（`797e8a9bc`）。

| 来源 | Git blob 前 12 位 |
|---|---|
| `docs/specs/ace-migration/m2-certification.md` | `151de4df0135` |
| `docs/specs/auto-ac/aac-baseline-and-subject.md` / `aac-hosted-profile.md` | `8aa5a763c265` / `c140a3f045cf` |
| `docs/specs/auto-ac/aac-semantic-authority.md` / `aac-service-boundary.md` | `0727c405d3ae` / `e7208a9b808e` |
| `docs/specs/auto-hir/stage-contract.md` / `docs/specs/auto-mir/architecture.md` / `docs/specs/auto-abi/project.md` | `d03c9ecad761` / `855fb0767a38` / `c833e4925278` |
| `experimental/ace/grammars/auto-source.at` / `experimental/ace/Cargo.lock` | `467273f66d15` / `fa2609a69c32` |
| `experimental/ace/aac/hir/core_hir.at` / `core_atom_bound.at` | `4eff339565a4` / `d698ed3a4f51` |
| `experimental/ace/aac/host/bridge/client.at` / `experimental/ace/crates/aac-bridge/src/execution.rs` | `a7468befaf54` / `4ca68ff6e208` |
| `experimental/ace/crates/av-core/src/mir.rs` / `abc.rs` | `12078c6e59a6` / `e06f6376505c` |

## 5. 详细设计

frontmatter为本Plan完整交付前置；§8任务再细化本包批次。G-AAC 与依赖的正式落主核对见 §4。T-01 在同树补齐可独立复验的基础前端供给，然后交 T-02。每个driver首次使用任务建真实入口/命令及必要target/固定选择，T-06实际运行本包Auto资产和受影响Rust职责组；不等1053才第一次测试。

T-01 使用既有链路：统一 grammar → `syntax_parse.at` / `source_ast.at` → `typecheck.at` → `core_hir.at` / `core_atom_bound.at`。检查条件循环在 grammar 程序与 Auto parser 间的诊断，保持 grammar 内容绑定。在同一 HIR 生产者及其 Atom 读回验证中承载循环、惰性逻辑和精确标量，保类型宽度、求值顺序、预期 trap 策略和源范围。T-02 只消费实际通过 Auto verifier 的闭包，随后通过既有 Typed MIR/ABC 契约接 1051。r4 的实跑诊断见 §9。

本包的需求矩阵、阶段profile与支持边界见§1，具体数据路径/owner/输出见§8，正反例与独立结果见§6。1044冻结现有更严预算、source/node/depth/bytes/CT fuel/输出/期限和计数算术，合法支持边界不窄化，超界明示资源拒绝。合法/非法/取消/恢复分别保真实终态，不能临时增预算或把错误变NA。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-base-mir-and-ct.md` | 新增本包已验证profile/接口/所有权与支持边界；完整AAC及M2边界不变 | 关闭Design59 §9的具体缺口 | AC-01..04 |
| SD-02 | modify | `docs/specs/auto-hir/stage-contract.md` | review通过后补本包当前事实和新Spec指针，保原历史/未交项；本轮不写current Spec | 避免目标态冒当前实现 | AC-05/06 |
| SD-03 | modify | `docs/specs/auto-ac/aac-source-frontend.md`、`docs/specs/auto-ac/aac-semantic-authority.md` | 补 1055 实际交付的语法和 Checked HIR 支持、复验与精确类型边界 | 使同仓前端增量具名可追 | AC-01..05 |
| SD-04 | modify | `docs/specs/auto-mir/architecture.md` | 登记 AAC Auto MIR 生产者、AVAD 验证与既有机器入口 | MIR 所有权和下游入口可查 | AC-03/05/06 |

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](../1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-base-mir-and-ct.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本轮语言完整性边界

1055基础LowerCt仍按原有界profile先交，完整语言执行由1050/1053关闭，不为“全语言”把基础CT依赖环重新接回。 当前CL目标取最终M2三端required及A后延增量，来源身份以阶段真实版本绑定，不以旧A有限面封顶。

## 6. 测试设计

下表保留原验收输入、入口和独立 oracle，当前实跑结果见 §9。用冻结 M2-A 语法、IR 和 API 构造，字面数值、字节、阶段及事件由独立预期给定。

| Case ID | AC / owner_stage | 输入/状态 | 入口/启用任务 | 可观察独立预期 | 边界 |
|---|---|---|---|---|---|
| V-01 | AC-01/02/03/04；work | f(x)=2*x+1，实参3；布尔恒假且RHS为trap | 本包Auto driver→1051 AV/native，T-02/04/05启用 | 两consumer各得7、false；trap计数0，Auto实际lowering进入；Rust source和HIR lowering禁止窗口内调用数0 | 实跑见 §9 |
| V-02 | AC-01/02/03/04；work | 合法MIR把一个DefRef改为越界或改参数Bool/Int映射 | Auto verifier及1051准入，T-03/05 | 受控拒绝，codegen/AV执行本次计数0；恢复原MIR仍正常，不能把字节解码当验证 | 实跑见 §9 |
| V-03 | AC-01/02/03/04；work | CT访问未授权IO或超fuel，随后执行合法返回7请求 | Auto CT入口→1051 AV，T-04/05 | 按M2-A权限/资源类别失败、无CT冻结值，句柄/子进程回收；下一请求仍7 | 实跑见 §9 |
| V-04 | AC-01/02/03/04；work | 给基础入口送尚未闭合CT或高级owned/dynamic输入 | 本包profile准入，T-02/03 | 明确非本基础profile或pending，不伪造Checked、不自动送Rustsourcecompiler；登记1050/1053对应必需行 | 实跑见 §9 |
| V-05 | AC-01/02/03/04；work | 用同一f(3)请求，在基础MIR里把常量1改为2 | 本包lowering事件及真实两consumer，T-05 | 原请求独立结果7；变更MIR实际结果8，原7的oracle必须使反控失败，证明未用fixture返回值或wrapper自报PASS | 实跑见 §9 |
| V-FRONT | AC-01/02/03；work T-01 | r4 的八个实跑输入，补合法循环的退出值、短路 RHS 的执行顺序、极值类型及对应坏类型/目标控制 | 预建 Auto 阶段 driver 的源码签发和 Atom 读回，Gen0 仅为独立参考 | 合法循环、短路和 i64 产出受检 HIR；错误引用/类型/越界及旧负控按原分类拒绝；后续 V-01..05 仍实跑 | 实跑见 §9 |

现有受影响候选组 `cargo t ace-lowering ace-av ace-native`（只读查`scripts/test-groups.toml`确认，未运行）；work核真实触面，选完整职责组与必要consumer，并实际运行本包Auto driver。普通模块排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不成为每Plan收尾门。记录实际case/执行/条件排除数，准备/发现/运行/清理/总时间及输入工具产物身份；required未跑不能PASS。

### 当前阶段交接验证

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

- [x] **AC-AC-STAGED / work**：在 `b146119a6` 上复验，正式 A 收据与完整 CL 通过实际验证器。阶段 I32 闭包不依赖 `D-1047-AC-NARROW`，实际选用的 CL-505 服务拔除时具名阻塞，恢复后原输入真实 AC 返回 7。见 §9，留独立复审复核。

## 8. 执行步骤

唯一树 `D:/autostack/.wt/lang-1055/auto-lang`，分支 `plan-1055-dev`。r4 初始基面 `196ce23f735e367500e065318c4259dbc701271c`，r5 开始前快进至 `282e6de9b3674937f3b48d19a868b9b3ef36f15e`。兄弟仓 `D:/autostack/.wt/lang-1055/auto-down` detached `45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5`，仅用于依赖解析。材料唯一 `D:/autostack/plan-artifacts/plan-1055/`，已登记 owner、用途和在途状态；SDK/生成输出/target/仓外消费均显式在 scope，不放仓内证据，不建 junction/symlink。

| Task ID / 状态 | owner_stage | 依赖 | 输入/实际工作 | 输出/验法 |
|---|---|---|---|---|
| [x] T-01 基线与前端供给 | work | G-AAC、frontmatter 正式来源、r5 用户授权均已核 | 保留 §4 基线核对；在原语法/Checked HIR 链补齐真实条件循环、基础短路及已冻结精确标量，严格复验与原合法邻例保持 | V-FRONT 实跑，见 §9 `7748ad227`，供 T-02 消费同树受检闭包；AC-01/02/03/04 |
| [x] T-02 基础lowering接口 | work | T-01 | Auto CFG/block/value/place/类型/调用构建，精确标量、短路、分支、循环、源码位置；Auto 编码真实 AVAD，不依赖1048/1050。 | `cd2b4cf47` 产物经实际 AV 执行，见 §9；原 V-01..05 联合闭环留 T-04..06 |
| [x] T-03 本地MIR准入 | work | T-02 | Auto 检查引用/归属、类型、调用映射、CFG、终结与 must-initialize，纯 profile 只发无导入和无能力请求的闭包。 | `cd2b4cf47` 的本地准入负控无产物、合法恢复；1051 独立准入对照留 T-05/06 |
| [x] T-04 CT受检调用入口 | work | T-03 | Auto 工厂只接字面参数，合成调用通过同一源码语义及 MIR 准入。Auto 原生客户端使用五轴摘要和原调用剩余期限交 1051。 | `f7c2c32e4`，V-01、V-03 实跑见 §9。1048 可接 `host/cte-entry` 与原 Auto MIR 入口。 |
| [x] T-05 真实闭环与兼容交接 | work | T-04 | 同一 Auto producer 的实际 AVAD 交 1051 CT 和 native，独立值、准入错误、资源耗尽、pending 与改常量对照均实跑。 | V-01..05 结果见 §9。1050 扩同一 `aac/mir` 实现并保最终兼容复验。 |
| [x] T-06 定向验证与交review | work | T-02..05 | Windows 职责组、全部 V 行、真实阶段驱动及 1044 来源门复验。对照基线既有红，修复新增 AGE 身份漂移。 | 冻结候选 `8483384ce`，实际终态、排除、身份、时间及审核的 Spec delta 见 §9。 |
| [x] T-07 独立复审 | review | T-06 | 同一树冻结 `8483384ce` 按实际数据流、独立反例、基础合同、存量红和规范源复验。 | AC-05 pass。merge 复核落主、归档与清理。 |
| [x] T-08 合入沉淀与清理 | merge | T-07 pass | `8483384ce` 快进合入，五处规范验证，归档入库，校验并移除双树、临时分支和材料 scope。 | AC-06 pass。验收结论与清理收据见 §9。 |

## 9. 复审记录

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r3：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r2：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14 stage=new，revision=1；规划已完成，outcome=blocked（等G-AAC和未来实施授权），next=开工门满足后work。本轮只是文档审计/计划补齐，没有实现/测试执行/独立代码review/合入；进度0/8，Design59 §9记本次发现与责任分配。

2026-11-21 `stage: work | plan_id: PLAN-1055 | plan_revision: r4 | outcome: blocked | worktree_path: D:/autostack/.wt/lang-1055/auto-lang | branch: plan-1055-dev | base_commit: 196ce23f735e367500e065318c4259dbc701271c | code_commit: none | task_ids: T-01 | next_responsible_role: 前端生产者或用户`。§4 交付祖先及供给文件已核对。来源文件与核对时 master 的 diff 为空，工作树 clean。T-01 的必需输入尚有缺口，仍为 0/8。T-02..06、独立复审和合入均待前置关闭。

在工作树 `experimental/ace` 以 `cargo run --locked --jobs 2 -p ace-aac-syntax-conformance --bin prepare-native -- D:/autostack/plan-artifacts/plan-1055/hir-baseline semantic` 独立预建 1047 已有 Auto driver。总 232.406s（Cargo dev 构建报告 74s），exit 0；driver manifest 源指纹 BLAKE3 `f55483cd147f9c3004cb50708be7f61df8ef58346eddd8e68ebe8195145fd0c9`、exe BLAKE3 `38afcf5d568b1d2ef7b01afd5dd019591dc01aa6fe00154233793b384c436b46`。预建日志 `prepare-hir.log` SHA256 `143375d3d5d4e4139c3bf39d1762443df14b0dc6e3f8c33850fd6f656bfca36a`。预建出现基线 warning，本次没有实现 diff。

在工作树运行 `pwsh -NoProfile -File D:/autostack/plan-artifacts/plan-1055/probe-hir-baseline.ps1`（首轮六个输入，2.628s）及同脚本 `-CaseFilter 'loop-*'`（补充两个输入，1.121s）。每个输入分别运行 `AAC1047_HIR=facts` 与 `verify`。八个输入、16 次 driver 调用、0 排除，两个脚本 exit 0。诊断由报告正文判定。无测试发现或编译步骤；临时材料保留在 scope，未在探针内清理。

| 输入 | 类型事实 | HIR 报告 | 双模式含启动秒数 |
|---|---|---|---:|
| `compute(3)`，`compute(x) = 2*x+1` | `aac.semantic-facts/1` | `core-i32-draft`，含有类型的乘加与直接调用 | 0.419 |
| Bool 函数直接作 if 条件 | `aac.semantic-facts/1` | `core-i32-draft` | 0.413 |
| `false && rhs()`，`rhs()` 返回 true | `aac.semantic-facts/1` | `hir.unsupported 0 50..80` | 0.458 |
| 冻结 `ace-B04-shortcircuit.at`，右侧除零 | `aac.semantic-facts/1` | `hir.unsupported 0 20..54` | 0.367 |
| 条件循环 `for counter < 3` | `source.syntax 0 35..43` | 同左 | 0.305 |
| `i64` 返回 `9223372036854775807i64` | `aac.semantic-facts/1` | `hir.unsupported 0 3..47` | 0.283 |
| 同一循环加括号 | `source.syntax 0 35..43` | 同左 | 0.305 |
| 冻结 `sc-for-loops.at` | `source.syntax 0 163..171` | 同左 | 0.411 |

脚本和日志在 §8 的材料 scope。首轮 `probe-hir.log` SHA256 `e78fc199a85a8e36e06f80ce089ed56fd2c8bff02a2bb87795c955a757698fa6`，循环补充 `probe-loop.log` SHA256 `4707af934d8beee0b0eb80749508dd07408ab0eef93a58e9fad394882cd199`。源与报告留在 `probe-hir/`。这些是前置诊断；T-02..06 的 V-01..05、V-AC-STAGED 和固定组仍未运行，未新增入库测试。

阻塞定位：1047 的 Auto `checked_core_atom` 只发射已交的 i32/Bool 表达式和终结分支等 HIR 子集，类型事实并不带 HIR 准入权限。V-01 的短路/除零与 §1 循环、宽标量无法进入当前正式生产路径。需要 1046 语法及 1047 Checked HIR 原生产者的正式增量供给，或用户授权修订本计划的实施归属。1051 已交机器服务可用，Rust 前端只能作为参考。

`stage: goal | plan_id: PLAN-1055 | plan_revision: r4 | round: 1 | from: work | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1055/auto-lang | commit: 196ce23f7 (base only) | next: user`

`stage: goal | plan_id: PLAN-1055 | plan_revision: r4 | round: 2 | from: user | to: user | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1055/auto-lang | commit: 196ce23f7 (base only) | next: user`。主干 `483fe91d3` 的相关前端与桥接路径相对基面无变更（`git diff --exit-code` 为 0），§10 未收到归属决定。复用第 1 轮阶段探针证据。

`stage: goal | plan_id: PLAN-1055 | plan_revision: r4 | round: 3 | from: user | to: user | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1055/auto-lang | commit: 196ce23f7 (base only) | next: user`。第三轮核对仍为同一前置缺口，主干 `483fe91d3` 的前端、桥接和规范路径无变化，§10 的归属决定仍待确认。保留工作树和阶段探针。

2026-11-21 `stage: new | plan_id: PLAN-1055 | plan_revision: r5 | outcome: pass | next: work`。用户授权同仓前端补齐，修订 T-01、V-FRONT 与候选 SD-03。承接 r4 基线红盘，原验收与 8 项任务的总数保留。未把旧结果计为 r5 通过。

2026-11-21 r5 T-01 在途，未勾任务。修复组合 profile 同时把两条不同的循环条件串联为必需项的问题，保原 `cond` / `body` 语法字段。用已同步的共享 grammar 再生成八个 profile 的嵌入数据，生成 SHA256 `32022bfe11ec29b2dd9702770327321227a98ed1bda579497df2c3d9e7e1e36d`。语法提交 `15f6de3ab`，`cargo nextest run --locked --jobs 2 -p ace-aac-syntax-conformance --test conformance -E 'test(=aac1055_conditional_for_consumes_one_condition) | test(=aac1046_grammar_data_identity_binding)'` exit 0，2/2（49 未选），新增条件循环样例 1.016s，身份例 0.244s，编译 25.17s、测试窗口 1.020s、整命令 27.625s。预建 Auto 语法驱动 82.297s。合法 AAC/scalar 循环与 Rust 参考 CST 和诊断一致，缺条件的对照保诊断。

基础惰性短路 HIR 提交 `8bcf7a09e`。用已预建 Auto 语义驱动与 Rust 独立 HIR verifier 核源码签发和 Atom 读回。基线 `false && (1/0 > 0)` 保留逻辑和可能 trap 的 RHS，实际运行路径及零 trap 数留 V-01；实际把 RHS 换成 i32 的负控报告 `hir.type`，实际改挂旧 core profile 的负控在修复前错误接收，修复后报告 `hir.unsupported`。首轮新例变异脚本漏写 RHS 字段，在 shape 阶段失败，补字段名后实测类型负控。定向命令同包的 `--features aac-semantics --test semantic_conformance`，选 `aac1055_admits_lazy_logic_and_rechecks_operand_types`、`emits_core_arithmetic_traps_and_rejects_an_erased_overflow_policy`、`verifies_terminating_branches_and_rejects_a_back_edge`，nextest `e0544212-fdf2-458b-8ba6-4a8a2b4a2e` 3/3（48 未选），分别 0.905/0.501/0.651s，编译 3.47s，测试窗口 1.158s，整命令 6.242s。日志于材料 scope 的 `syntax-r5-check.log` 和 `semantic-r5-profile-check.log`；两轮均无新生产 Rust warning。固定组及 V-01..05 未跑。

宽整数受检 HIR 提交 `3bde88038`。Gen0 的 `List<u64>` 与直转 `u64 → i32` 在准备时分别受既有容器和转换合同拒绝，已改为四个 16 位 limb 及原有有界中间转换。另一轮 A2R 生成的 Rust 因借用字符串传法失败，修正生产 Auto 调用。完整 numeric schema 在 Auto 泛型形状检查触发首次读回超时（nextest `56620782`，新例 8.438s 本体失败）；核该 HIR 子集共享标量 tag/field 后复用同一形状检查，保 numeric identity、位宽与引用的独立验权。源签发、Rust 独立 HIR 验证、Auto Atom 读回三处保 i64 MIN/u64 MAX、`add_i64` 和 trap 标签，超界值及错标运算宽度负控均 `hir.type`。相同包/test 定向 `aac1055_wide_integer_hir_preserves_width_and_extrema`、原 i32 边界例与短路例，nextest `f4aa4779-1cd3-43fa-b654-7010ce160465` 3/3（49 未选），分别 1.072/0.552/0.921s；编译 1.75s，测试窗口 1.474s，整命令 3.804s，最终准备 82.528s。材料 `semantic-r5-wide-subset-check.log`。循环 Checked HIR 实施中，精确浮点及后续 MIR/CT 待验证，T-01 保开放。

结构化控制流提交 `fd1c6144f`。条件循环在结构化 HIR 内保留重复条件、正常续行、退出与词法归属；普通分支只在可达路径继续。作用域初始化核对经过进入子块的实际语句，Atom 读回拒绝改指其他循环。以 `cargo nextest run --locked --jobs 2 -p ace-aac-syntax-conformance --features aac-semantics --test semantic_conformance` 运行整个语义测试二进制，nextest `52fc7e2d-821c-4aed-a2f9-876446f1eb28` exit 0，53/53、0 排除；编译 2.21s、测试窗口 25.408s、整命令 28.736s、显式 Auto 驱动准备 80.321s。新循环例 1.108s，旧短路/宽整数例 0.960/1.103s。日志 `semantic-r5-control-check.log`。循环动态结果仍须由 T-02..06 的真实 MIR/机器服务验证。

精确浮点提交 `9c01713ae`。Auto 复用目标位宽的原舍入算法，HIR 等值传 Float bits，f64 负零为 `9223372036854775808f64`，f32 半途取偶为 `1065353216f32`，超 f32 32 位载荷被拒绝，保 `add_f32` 类型。相同包/test 定向浮点 HIR 新例、宽整数 HIR、既有浮点舍入和次正规例，nextest `1a4c249e-f63a-4564-a88f-c56e8b69d38b`，4/4、50 未选，分别 1.194/1.248/0.331/0.538s，编译 1.51s、测试窗口 1.732s、整命令 3.830s、显式准备 92.112s。日志 `semantic-r5-float-check.log`。

T-01 r5 完成于 `7748ad227`。`cargo nextest run --locked --jobs 2 -p ace-aac-syntax-conformance --features aac-semantics --test semantic_conformance -E 'test(/aac1055_/)'`，nextest `3a8eda3d-fc14-4744-b991-48dc03bce81a`，exit 0，4/4，50 未选。循环、短路、浮点、宽整数分别 1.115/1.259/1.201/1.283s；编译 2.52s、测试窗口 2.543s、整命令 6.327s；显式准备 135.535s。循环和短路把 Auto 生成的同版 HIR 交独立 Rust verifier/lowerer + AV 参考执行，观察冻结循环返回 0、短路返回 42，改为 `true &&` 实际除零 trap。该 Rust 路径仅在测试参考函数中，V-01 的 Auto MIR 实际机器执行尚待 T-02..05。日志 `semantic-r5-frontend-check.log`。工作树 clean；T-02 待/继续，整体执行中 1/8。

T-02/T-03 r5 在途收据 `cd2b4cf47`。已受检的 Auto HIR 直接进入 Auto `lower_base.at`，体、slot 与 block 独立编号，短路显式走 CFG；调用依序求值再按形式槽映射，循环 break/continue 保最近词法区域。Auto `verify_base.at` 先核结构、类型、引用归属与可达初始化，再按 AVAD v1 编码执行体和源图。真实 AV 解码独立复验 Auto 产物并执行，观察 7、false、冻结循环 0 和 i64 MAX，无 Rust HIR→MIR 调用。控制分别实际变异 DefRef、异型参数槽、CFG 与同型未初始化 slot，拒绝且产物缺席，恢复合法输入有产物。第一轮编制曾因 Auto 保留字命名失败，改名重编。最终 `semantic-r5-mir-verify-prepare.log` 准备 177.563s；同包语义筛选 `test(/aac1055_/)` nextest `e1d990b4-1ee7-4e91-9918-53649bf413e1`，exit 0，6/6，50 未选，编译 3.33s、测试窗口 3.479s、整命令 8.248s。两条新 MIR 用例 1.262s（四实际执行）、1.137s（四负控加恢复），日志 `semantic-r5-mir-verify-check.log`。联合 V 行、1051 的 CT 与 native 及固定组待验。

T-05 r5 首轮原生实证（准备基面 `cd2b4cf47`，T-05 保开放）。`pwsh -NoProfile -File <scope>/probe-native-r5.ps1` exit 0，以 Auto 阶段驱动的原始 AVAD 交正式 1051 `native_objects`，用现有链接器生成并执行 Windows PE。原例 `7`（0.496s，COFF BLAKE3 `3db207773178f078…`），短路 `false`（进程返回 0，0.306s，COFF `35aab67ce5accfd0…`）。服务对实改越界 DefRef 返回 admission `mir.callee-ref`、无对象 payload（0.069s），同一合法产物紧接恢复 `7`（0.295s，COFF 内容相同）。在 Auto lowering 后将常量 `1` 改为 `2` 导出真实 variant，另产生 COFF `1cb1f3cf2afaf0ff…`，执行得 `8`（0.354s）；相同 variant 与原 `7` oracle 比较按预期断言失败，探针记录子 exit 101。Auto 输入 `fn main() int { return #{ 1 } }` 保 `aac.pending-work/1`、`PENDING 2`，无 ABC。原始日志 `probe-native-r5.log`；以上原生结果尚须跟进主干 1090 与最终冻结版本复验。

T-04/T-05 r5 冻结候选代码 `f7c2c32e4`，工作树同步到 `master@9ae4956fe`。`git range-diff 282e6de9b..cd2b4cf47 9ae4956fe..e6cb643cd` 七提交判等。`2af7f6d9d` 再生成 8 profile 与重绑 1044 grammar/MIR 来源，未动 42 条 CL。重绑前真实 1044 验证器报告 `profile.identity.grammar` 且 0/6，重绑后以相同验证器及 SHA256 运行 6/6（0.03s，`probe-frozen-contract-r5-rebound.log`），含漏 CL 行、假 NA 拒绝及恢复。分母 41 required + CL-906 具名后继。V-AC-STAGED 的特定消费来源控制继续在 T-06 执行。

CT 工厂 `factory.at` 通过现有源码判型，组装零参数调用，再走 Auto HIR/MIR。A2R Gen0 将 1051 客户端 owned 记录误生成 `Copy` 且不能安全取得列表字段（见 `semantic-r5-ct-local/rustc.log`），沿 1051 已验路径用 AC 预建单独 Auto CT 客户端；语义驱动仍用 Auto parser/binder/HIR/MIR。第一次 AC 探针把 216700 字节 grammar 读为集合触及已有资源限制（trap 22，`probe-ct-client-r5.log`）。最终准备方按实际扩展源码、grammar、SDK manifest、pass、依赖输入生成摘要，Auto 客户端将十六进制摘要解码为 AQ01 的精确 digest 载荷，不复制 grammar 到运行时集合。CT 请求及响应文件回收，测试以 Windows 进程树限时运行。

`cargo nextest run --locked --jobs 2 --manifest-path experimental/ace/Cargo.toml -p ace-aac-syntax-conformance --features aac-ct-bootstrap --test semantic_conformance -E 'test(/aac1055_/)'` nextest `6cd3466d-28ee-44f9-a756-8ea92fbfec1a` exit 0，10/10，50 未选，编译 38.27s、运行 12.800s、总 52.287s。准备分别为语义驱动 129.320s、runtime-dev 0.863s、最终 CT 客户端 9.885s、正式 bridge 29.869s。真 1051 CT 值 `I32(7)`、`Bool(false)`，读集/foreign call/存活文本为零；循环资源耗尽 `av.resource.instr-steps`，无 Value，非字面参数 `f(2)` 由 Auto 先拒绝，随后同程序合法字面量恢复 7。两 CT 单例分别 3.610/4.963s。Auto MIR 常量改动实走 AV得 8、旧预期 7 为假（0.555s）；待决 CT 资料仍为 pending，无 ABC（0.977s）。见 `selected-r5-bootstrap-check.log`。

同一冻结候选重新运行 `probe-native-r5.ps1 -DriverDirName selected-r5-semantic` exit 0（`probe-native-r5-final.log`）。真实 1051→COFF→Windows PE 返回 7/false/恢复7/变异8，各 1.255/1.170/1.926/1.984s；DefRef 变异 `mir.callee-ref` 无 payload（0.124s）；旧 oracle 子 exit 101。COFF BLAKE3 分别 `3db207773178f078…`、`35aab67ce5accfd0…`、`3db207773178f078…`、`1cb1f3cf2afaf0ff…`。T-06 继续完整固定职责组和 V-AC-STAGED 余验。

T-06 上一冻结候选（基面 `9ae4956fe`）已实际运行 `ace-syntax`、`ace-semantics`、`ace-aac-syntax`、`ace-aac-bridge` 的兼容 Cargo 包并集，以 nextest 的 `aac-ct-bootstrap` 额外开启本包 CT 用例，bridge hosted 附加腿仍保显式。`final-r5-syntax-and-semantics.log` 573/573，13 个条件排除，nextest 执行 71.288s，整命令 99.596s，exit 0。V-AC-STAGED 的基线阶段探针 `probe-ac-staged-r5.ps1` exit 0，真实 1044 checker 准入 42 CL、16 CS、11 identity，正式 M2A-08 为祖先。原 CL SHA256 `60192D3985696B6DB264CDF06BDF1D1C4163BD26D9264F6A569D1D68E6BE70A8`。实际 Auto 标量切片与 CL-906 具名后继并存；选用 CL-505 的机器入口时拔除服务路径，定位为 PLAN-1051 封闭条件且无机器命令，恢复同一已交 provider，原 ABC 实际 AC 返回 7。上述试验证据随最新基面复验后方关闭 T-06。

主干随后同步至 PLAN-105 交付 `0b754af59`。`git range-diff 9ae4956fe..f7c2c32e4 0b754af59..471c7c9fc` 九条原提交判等，但受影响共享 grammar、源码和 MIR 须重验。同步后真实生成器因 `generic-call` 被裁而在 AAC 面退出 1（`grammar-r5-after105.log`）。`3b482d5e7` 恢复 AAC 引用闭包中值实参的原规则和嵌入八套数据，保 PLAN-105 AGE 分支，重绑 1044 的三个输入。CT 客户端初次重编在共享 lowerer 拒 `.move` 整局部条件，诊断变体观察内层已是 `Move`；`639e9f000` 在原 producer `move_marked_value` 保留单层转移，原生准备 exit 0（45.357s，`after105-ct-single-move.log`）。清除了临时诊断实现。`6c91aa47a` 工作树提交五份待复审规范及其身份 pin，1044 原检查六例均通过（`probe-frozen-contract-r5-final.log`）。相关 Windows 阶段产物与消费者回归仍在运行。

同步后的语义驱动第一次重编观察到 Gen0 A2R 生成的 `JsonStrStep(next_pos, out.move)` 同时把 `out` 入记录又按本地值销毁，rustc 拒三个 E0382（`after105-semantic/rustc.log`）。`b146119a6` 恢复组合面普通调用/构造的显式 move 表达式，列表方法仍接已落主 `MethodOut` 路径。最终真实准备均 exit 0，`verified105-ct` 65.658s、`verified105-semantic` 103.360s、`verified105-syntax` 59.509s，1051 bridge 19.659s。当前代码冻结 `b146119a620010f3d8e75f07a243fd3b71cb2d4c`，基面 `0b754af59`，同树 clean。

该版本的 Windows 原生 V-01/02/04/05 与来源 V-AC-STAGED 均重跑 exit 0。`verified105-native-v01-v05.log` 真 PE 原 `7`、短路 `false`、恢复7、变异8（0.325/0.313/0.357/0.328s），越界 `mir.callee-ref` 无对象（0.071s），待决 CT 无 ABC，原 7 oracle 子 exit 101。`verified105-ac-staged.log` 真 1044 A/CL checker 得 42 CL、16 CS、11 pin；`D-1047-AC-NARROW` 和 CL-906 仍具名且 I32 独立切片成立，CL-505 拔除 1051 服务路径时无机器调用及产物，恢复相同已有正式源及同一 ABC 后实际 PE 退出 7。首轮因 D 盘空间不足在链接阶段失败，保留记录并以本计划可回收的 incremental 缓存执行 NTFS 压缩后重跑。完整职责组仍运行中。

在 `b146119a6` 执行 Windows 前端/语义/桥接并集，发现 54.459s、执行命令 65.365s（nextest 63.922s），571/573、13 条条件排除，exit 100。1055 的新增语义/CT 验收全部通过。两条现有合同回归为 `explicit_type_arguments_require_an_adjacent_angle_token` 和 `distinguishes_fallthrough_statements_from_match_expressions`。最新共享语法缺少 `id<int>(2)` 的相邻 `<` 实参调用形及 AAC 完整结果类型优先于 Scheduler `!` 的声明形，两者曾在上一个基面 573/573 实测。恢复原 `generic-call` 与 `fn-tail` 规则，保 PLAN-105 引入的 AGE profile，重新生成、重绑和准备 Windows drivers。日志 `verified105-gate-frontend-check.log`。D 盘 NTFS 压缩仅触本计划可回收 incremental 缓存，压缩前/后存储 6,847,779,608 / 3,938,896,177 B，guard clean，与原生重放通过一致记录。

`571b35644decc860b60eeef4bc833dde8e27cde0` 上的最终 Windows 前端并集复验 573/573、13 条条件排除，nextest 执行 76.940s，exit 0。原两条红例分别 2.059s 和 1.612s 通过，真实 CT 字面和短路组 1.719s、资源失败及恢复组 2.243s。显式准备 `compat105-ct` 21.700s、`compat105-semantic` 116.580s、`compat105-syntax` 65.451s、bridge 14.544s 全部 exit 0。共享 grammar SHA256 `C4FEA17E27B5D09B29A8C98526ED62911790E1D79D3B53003FF68B1FB29FE808`，嵌入八套 SHA256 `A6D8499584DE99F3F385BE4E937E197C10684909B2210C5FE6E12ADE99BD378F`。`compat105-native-v01-v05.log` 和 `compat105-ac-staged.log` 同 head 均 exit 0，仍观察原 7、false、变异 8、无越界对象、CL-505 拔除及实际恢复 7，CL 原分母哈希保持。完整下游职责组进入准备和验证。

`571b35644` 下游验证在明示排除 Linux/WSL 和正式性能用例后发现并执行 `ace-core`、`ace-lowering`、`ace-av-core`、`ace-av`、`ace-native` 的 2,825 例，47 条条件排除。Worker 准备 15.985s、runtime-dev 0.541s、Windows runtime warm 2.554s、发现 96.658s，全部 exit 0。nextest `e15cb00b-3867-49a6-a4f8-d03d204434aa` 首次 2,679 绿、146 红，运行 153.505s、整命令 155.052s、exit 100。保真实子状态。目录 `experimental/ace/target/x86_64-pc-windows-msvc/runtime-dev` 属本工作树，探针与其余材料都在登记 scope。

首轮失败中运行库准备态出现 `PREPARATION_REQUIRED`，实际 launcher 为 Cargo 包装，环境与直接预建不同。改用既有固定组的 `cargo-nextest nextest run` 启动方式，原 146 个失败的 `(binary, test ID)` 全部重放，97 绿、49 红，执行 63.321s、整命令 65.231s，exit 100。然后在同一 worktree 切到纯父提交 `0b754af59` 重跑剩余 49 个 ID，48 同红、1 绿，执行 2.173s、整命令 32.853s，exit 100，随后切回候选。对账前后集合完全相同。共同红分布 `ace-av` 5、`ace-av::conformance` 26、`ace-av-core` 1、`ace-core::conformance` 5、`ace-native::conformance` 11，主要在普通 AV #CT/演员、记录方法与 owned、动态对象及 native worker 诊断，空记录旧 oracle 单列。详见 `consumer-id-comparison-r5.log`，48 个均从本包验收结果分离保留原状态。

唯一候选新增红是 `plan981_grammar_scope::p981_age_package_grammar_id_is_frozen`，基线实际通过。单独回置不同语法规则定位到循环规则被扩到 AGE。`8483384ce` 让 AAC 组合面拥有单条件规则，其他 profile 保原规则。对 28 个声明面实际编译比较 canonical identity，与上一候选相比有 26 面逐字节身份一致，`age-package` 回到父提交的 `0d016cd528fc40adfb88969aa51be45e095abb6cdb730af3562c2859f5fd73c3`，`aac` 更新为 `16852b94f594e1869ccb109351180714294420848c0baec8df328325ab9558ab`，材料 `isolated105-grammar-id-matrix.json`。最终 AGE 组和 AAC 复验在途，余输入及机器后端实现未动。

2026-11-21 `stage: work | PLAN-1055:r5 | outcome: pass | code_commit: 8483384ce2dee99eb4a11e518d23e9421d321d7f | base_commit: 0b754af59dd3da68e90d3739093bd66c9ca0b8a8 | auto-down: 45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5 | task_ids: T-01..06 | next: review`。

- 最终 `check-frozen-r5.ps1 frontend` 为本仓 `ace-syntax`、`ace-semantics`、`ace-aac-syntax`、`ace-aac-bridge` 路由的固定 Cargo 包并集。另显式开启 `aac-ct-bootstrap` 验收 Auto 阶段。发现/编译 33.870s，执行 62.922s、命令 64.758s，exit 0，nextest `929ba29a-dee9-4980-85e0-14a1a4c859ca`，573/573，13 条条件排除。CT 两例 2.157s 和 2.221s，所有新增例低于 10s。`isolated105-gate-frontend-check.log` SHA256 `5183B960A053AF5EB0554E86A860622653167F3F12AA43813670AB176AF43439`。
- `cargo-nextest nextest run --locked --offline --jobs 2 -p ace-core --test conformance -E 'test(plan981_grammar_scope)'`，nextest `33500702-d230-45f7-8271-9745c177150d` exit 0，4/4、868 个本定向重放未选，编译 33.23s、执行 0.447s、总 40.061s。原 AGE 身份红例 0.043s。除 AAC 与回到基线的 AGE 包面，其他 26 个 grammar 编译身份与下游验证输入一致。下游原 2,825 执行加定向重放归并为 2,777 绿、48 父提交同红。48 同红核对 SHA256 `07E3957C67EC4013A1F8DD948F99F3699BA6637C2F249D583F9598A3857B42AE`。Linux 目标执行、WSL 和同用例中绑定 Linux 的不可分支腿按 Windows 裁定排除，归专用 Linux 环境。
- 最终准备 Auto 原生 CT 客户端 26.742s、Auto 语义驱动 90.307s、Auto 语法驱动 60.481s、机器桥 10.270s，均 exit 0。`probe-native-r5.ps1`（`-DriverDirName isolated105-semantic`）和 `probe-ac-staged-r5.ps1`（`-SemanticDir isolated105-semantic`）均 exit 0。V-01 真 CT 与 native 为 7/false。V-02 Auto 负控无产物，机器 DefRef 负控 `mir.callee-ref`、无对象、恢复 7。V-03 `av.resource.instr-steps`，无冻结值、交换文件清理、恢复 7。V-04 `aac.pending-work/1` 拒发布，1048/1050/1053 按原归属。V-05 原 7、真 AV/PE 变异 8、原 7 对照失败。1044 原 6 个检查 exit 0，CL 表 SHA256 保持 `60192D3985696B6DB264CDF06BDF1D1C4163BD26D9264F6A569D1D68E6BE70A8`。源、产物、论证和 scope 列见前述证据。
- 五份工作树规范对应 SD-01/02/03/04/AC-STAGED，冻结于 `6c91aa47a`，与当前 HEAD 鉴别的路径和版本见 `spec-delta-final-identities.log`。Windows debug 检查实际完成，性能套件和完整代际签章仍按原路线。

`stage: goal | plan_id: PLAN-1055 | plan_revision: r5 | round: 4 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1055/auto-lang | commit: 8483384ce | next: review`

2026-11-21 `stage: review | scope: final | PLAN-1055:r5 | outcome: pass | reviewer: 同一 Codex 会话，按冻结树独立重建验证，未启用另一个 agent | worktree: D:/autostack/.wt/lang-1055/auto-lang | branch: plan-1055-dev | reviewed_commit: 8483384ce2dee99eb4a11e518d23e9421d321d7f | base: 0b754af59dd3da68e90d3739093bd66c9ca0b8a8 | auto-down: 45b642dadcbd4fb71a8c7f2e6f09435faf7a65f5 | next: merge`。

| 验收 | 独立核验与终态 |
|---|---|
| AC-01 / T-01..05 | 按输入、定义引用、slot、CFG 和真实子进程结果核 V-01/05。另以 `review-combined-r5.ps1` 实跑循环内 `continue`/`break`、倒序命名参数和 RHS 直接调用组合。false 得 7，改 true 同一 RHS 触发实际 PE trap exit 70。整命令 2.083s、exit 0，日志 SHA256 `296DD1E8CD47B0EF70BD2BC2CF6FA5DE197AA1A94D36E3D42CD885DD4F8F67D6`。pass。 |
| AC-02 / T-03..05 | 比对 V-02 本地与机器各自拒绝路径，从 `verify_base` 跟到 `encode_base_abc`，核调用引用、参数映射、初始化对照和恢复；V-03 真资源码、空 Value、无交换文件且合法续跑；V-04 原 pending 闭包无 ABC。pass。 |
| AC-03 / T-01..05 | 源语义、HIR、MIR 和造请求主体均在 Auto 执行，Rust 仅在 Gen0 准备、机器服务与独立 oracle。绑定有效 source、grammar、pass、SDK/依赖、ABI 与执行域，未发现隐式源编译回落。基础交接保 1048/1050/1053 的原边界。pass。 |
| AC-04 / T-06 | 查看实际 Windows 573/573、AGE 4/4、原生 V 与 1044 复验 6/6。逐 ID 归并下游 2,825 原终态、97 恢复例与 48 父提交同红；AGE 独有红已回归绿，26 个其他 profile 身份不变，缓存/工具身份没有绕过。新增 11 例全在 0.203–2.221s，本轮无新增隐式构建；测试 RUSTC warnings 为已落主存量。固定组 Linux/WSL 依裁定排除。pass。 |
| AC-AC-STAGED | 原 42 行、41 required、1 具名后继未缩；用生产 1044 checker 源重跑控件 6/6。CL-103 原差额在途，I32 阶段可独立运行。CL-505 选用服务缺失具体拒绝，按实物恢复同源返回 7。pass。 |
| AC-05 / T-07 | `git diff --check master...HEAD`、触及的 Rust 格式和 Debug 残留检查、五处规范链接均过。审查遗漏/延期，保 48 条既有红及其 owner 候选 D-1055-BASE-01。Spec delta SD-01/02/03/04/AC-STAGED 与现行代码、路径和身份一致，见 §5。pass。 |

冻结规范 SHA256，按 §5 的新 Spec、语法、语义、HIR、MIR 顺序分别为 `79DF49FCE4D220C3029B1A02D64B6F4CE9E4431698CAE1C8AD1345ABC762D383`、`42E8A4A562DC9E1DA71C85D4CFFB08F61B62A24C83F9E2AB731DCD3CC1D83B0B`、`08D24E1FB4F764DE70FC09516FEE0CD5B5ED732FE635EC13B9494FF95B8D1F60`、`441B558A7CD9CA9EA5FD17950463FEA4367B9DB2AE7D43025070B15EC2C5B571`、`3D80B1F1730E086B01AF1F2E53937E895FC3F080C50D5706349B96C9D980AEB8`。merge-owned 待职责为冻结门证据复用、快进、规范核验、归档、材料与双树清理、批量到期核查。Windows 本包未安排运行桌面/发布安装。

`stage: goal | plan_id: PLAN-1055 | plan_revision: r5 | round: 5 | from: review | to: merge | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1055/auto-lang | commit: 8483384ce | next: merge`

2026-11-21 `stage: merge | PLAN-1055:r5 | worktree: D:/autostack/.wt/lang-1055/auto-lang | reviewed_commit: 8483384ce2dee99eb4a11e518d23e9421d321d7f | delivery_commit: 8483384ce2dee99eb4a11e518d23e9421d321d7f | completion_kind: delivered | next: cleanup`。

- `prepared`。冻结工作树 clean，交付候选从 `0b754af59` 线性继承，锁定 review 的五份规范和来源 SHA。合入门复用同一候选的 573/573、AGE 4/4、定向阶段机和在 §9 明示的下游原终态。最后变更仅改变 AAC 并恢复 AGE 原身份，其他 26 面与下游执行态内容一致。保留全部原断言、规模和排除条件。11 新例墙钟 0.203–2.221s。D-1055-BASE-01 交原 owner。
- `landed`。主检出 `git merge --ff-only plan-1055-dev` exit 0，实际 tip `8483384ce2dee99eb4a11e518d23e9421d321d7f`，合入前工作树 `wt-guard` clean。实际主检出 1044 checker 准入 42 CL、16 CS、11 identity，`aac` grammar identity `16852b94f594e1869ccb109351180714294420848c0baec8df328325ab9558ab`。PATH 查无已安装 bridge，交付使用本次准备的 Windows debug 产物。
- `canonical_verified`。`git diff --exit-code HEAD plan-1055-dev -- docs/specs experimental/ace` exit 0，实际主检出五份规范的全文 SHA256 与候选一致，分别 `79DF49FCE4D220C3…`、`42E8A4A562DC9E1D…`、`08D24E1FB4F764DE…`、`441B558A7CD9CA9E…`、`3D80B1F1730E086B…`。legacy ledger 未写入。
- `archived`。`git mv` 到 `docs/plans/archive/1055-aac-early-typed-mir-and-ct-bootstrap.md`，状态 `archived`，核 5 个相对链接、T-01..08 唯一及清理前 7/8。归档和 D-1055-BASE-01 书面移交入库，T-08 留实际清理。

<details>
<summary>下游父提交同红 48 个固定 ID</summary>

- `ace-av language_source_worker::compile_time_tests::ct_cache_reopens_after_failed_edit_and_uses_current_source_and_grants`
- `ace-av language_source_worker::compile_time_tests::ct_cancel_reaches_the_av_and_preserves_the_demand_span`
- `ace-av language_source_worker::compile_time_tests::ct_dispatches_once_and_leaves_no_runtime_call`
- `ace-av language_source_worker::compile_time_tests::explicit_package_target_changes_ct_selection_and_identity`
- `ace-av modules::packages::tests::ordinary_method_calls_share_one_owner_bound_instance`
- `ace-av-core layout::tests::layout_rejects_non_record_and_empty`
- `ace-av::conformance av_data_semantics::av_data_instruction_contract_round_trip_all_opcodes`
- `ace-av::conformance av_method_modes::use_after_move_of_method_arg_rejected`
- `ace-av::conformance av_method_owned::v03_flagship_value_thirty`
- `ace-av::conformance av_method_owned::v03_token_microprogram_fifty_seven`
- `ace-av::conformance av_method_owned::v10_method_field_borrow_return_zero_alloc`
- `ace-av::conformance av_method_owned::v10_second_alloc_failure_rolls_back_partially_built_output`
- `ace-av::conformance av_owned_first_case::av_owned_first_case_whole_path_and_live_zero`
- `ace-av::conformance av_owned_recipe::av_owned_recipe_fixture_face_matches_hand_audit`
- `ace-av::conformance av_test_batch::c_data_profile_keeps_its_scalar_entry_boundary`
- `ace-av::conformance component_event_sessions_1106::v02b_text_leaves_are_pure_reads`
- `ace-av::conformance dynamic_objects_964::av_object_alias_shared_write_visible_20`
- `ace-av::conformance dynamic_objects_964::av_object_cycle_runtime_defense_traps`
- `ace-av::conformance dynamic_objects_964::av_object_indirect_cycle_runtime_defense_traps`
- `ace-av::conformance dynamic_objects_964::av_object_map_bracket_write_600`
- `ace-av::conformance dynamic_objects_964::av_two_closures_share_object_cell_visible_12`
- `ace-av::conformance dynamic_objects_964::ordinary_object_growth_keeps_resource_stop_outside_script_catch`
- `ace-av::conformance plan1082_collections_places::t01_observe_face_matrix`
- `ace-av::conformance plan1088_generics::typed_filter_allocation_fault_releases_owners_and_allows_replay`
- `ace-av::conformance plan1090_tasks_actors::budget_exhaustion_after_a_park_keeps_its_own_outcome`
- `ace-av::conformance plan1090_tasks_actors::cancellation_while_awaiting_clears_the_owned_capture`
- `ace-av::conformance plan1090_tasks_actors::console_failure_while_awaiting_remains_a_host_failure`
- `ace-av::conformance plan1090_tasks_actors::full_mailbox_parks_and_wakes_the_sender_before_stop`
- `ace-av::conformance plan1090_tasks_actors::owned_capture_survives_wait_and_releases_after_delivery`
- `ace-av::conformance resource_915::resource_915_r02_l04_arena_sweep_is_complete`
- `ace-av::conformance resource_915::resource_915_r02_l04_value_sweep_is_complete`
- `ace-av::conformance resource_915::resource_915_r02_l05_v05_sweeps`
- `ace-core::conformance text_record_source::flagship_combination_compiles_and_verifies`
- `ace-core::conformance text_record_source::generated_atom_carries_the_combination_surface`
- `ace-core::conformance text_record_source::move_arguments_transfer_whole_roots_not_views`
- `ace-core::conformance text_record_source::named_ctor_source_order_is_the_written_order`
- `ace-core::conformance text_record_source::positional_ctor_and_owning_contexts_promote_literals`
- `ace-native::conformance collections_source_tier::collections_original_owned_first_cases`
- `ace-native::conformance data_owned_tier::mn_owned_mut_writethrough_wrapper_three_paths`
- `ace-native::conformance gap_closure::gap_hp01_text_record_hir_entry_runs`
- `ace-native::conformance record_method_tier::mn_method_token_microprogram`
- `ace-native::conformance shadow_tier::shadow871_text_record`
- `ace-native::conformance source_diagnostics_911::mn_sd911_worker_cold_default_shape_unchanged`
- `ace-native::conformance source_diagnostics_911::mn_sd911_worker_cold_strict_legal_echo`
- `ace-native::conformance source_diagnostics_911::mn_sd911_worker_cold_strict_rejection_shape`
- `ace-native::conformance source_diagnostics_911::mn_sd911_worker_incremental_default_keeps_legacy_covered_result`
- `ace-native::conformance source_diagnostics_911::mn_sd911_worker_incremental_strict_first_failure_stops`
- `ace-native::conformance source_diagnostics_911::mn_sd911_worker_incremental_warmup_reject_shapes`

</details>

2026-11-21 `stage: merge | PLAN-1055:r5 | outcome: pass | delivery_commit: 8483384ce2dee99eb4a11e518d23e9421d321d7f | archive_commit: 19b2f9f6f54226e825c0e12c9466ee82e24df2ae | completion_kind: delivered`。两棵 checkout 实际 `wt-guard` 均 clean，`git worktree remove` 均 exit 0；`git branch -d plan-1055-dev` exit 0，空组已清除。输出路径逐项对账为两个 checkout（AutoLang 下有正常 Windows runtime target）及唯一材料 `D:/autostack/plan-artifacts/plan-1055/`，无外部安装。材料扫描无重解析点、无本计划占用，折入证据后用 PowerShell `Remove-Item -LiteralPath ... -Recurse` exit 0，目录不存在。删除前 224,259 个文件、逻辑 22,487,931,240 B，压缩容量独立记录。临时树、分支、组和材料目录无残留。主检出只有并行的其他计划/设计簿记 WIP。

`stage: goal | plan_id: PLAN-1055 | plan_revision: r5 | round: 6 | from: merge | to: regress | status: archived | outcome: pass | worktree: removed | commit: 8483384ce | next: due batch`。1055 为上一实际覆盖提交 `b99fc0ff3` 后归档的五倍计划号，依整五规则到期；单实例在主检出等待其他重门结束。

## 10. 待澄清事项

2026-11-21 前端实施归属已由用户确认，见 §4。具体数据结构由 work 在冻结的语言、权限和预算合同内选择。
