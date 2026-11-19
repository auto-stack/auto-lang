---
plan_id: PLAN-1056
status: archived
feature_name: AAC精确摘要、编译器codec与标准库支撑
author: [Codex]
created_at: 2026-11-14
updated_at: 2026-11-19
plan_revision: 6
current_step: 7
total_steps: 8
depends_on: [PLAN-1044]
supersedes_spec_components: [docs/specs/stdlib/design/ace-core-and-text.md]
new_spec_components: [docs/specs/auto-ac/aac-digest-codec-profile.md]
touched_goals: [GOAL-002]
---

# [PLAN-1056] AAC精确摘要、编译器codec与标准库支撑

## 0. 变更摘要

2026-11-19 实施归属更新。共享 AC 缺口按[Design59 实施归属表](../design/59-aac-native-bootstrap-and-compiler-subject.md#现有-ac-差额的实施归属)落到实际任务，1068 接正式证据并做三端最终验收。 本包按原任务收实际生产任务的证据，安排已验阶段交接。

2026-11-16 r3本轮检查：把执行条款中的“M2-A AC适用子集”校正为最终M2三端required＋A后延；按本包实际族/原任务细化语法→语义→MIR→runtime→consumer。~T/?T/!T、泛型、CTE、required标注及CS未用特性不得遗漏。保原任务ID/阶段/数量/依赖/0进度，本轮仅规划核对。

2026-11-16 r2当前：按用户决定，AAC在M2-B期间以真实编译器模块检验AC并驱动渐进补齐；G-AAC接A基础正式收据＋具名AC差额，不要求A期AC已三端全齐。完整CL以M2最终AV/AC/A2R目标为准；完整代际签章仍独立，三端缺口回Design59 具名实施任务或实际库/领域 owner。仅规划，原任务/0进度/依赖保留。

2026-11-14 r1，本轮AAC规划复核增补。补齐AAC实际需要的精确摘要、长度/域分隔及版本化codec支持，不能把975有JSON或972有hash泛称为现Rust工件身份已可复现。 仅规划，8任务均未执行；不建树、不写实现/fixture/探针、不编译、不运行产品，不签M2-A/AAC。

## 1. 目标

补齐AAC实际需要的精确摘要、长度/域分隔及版本化codec支持，不能把975有JSON或972有hash泛称为现Rust工件身份已可复现。 按真实CS/artifact协议取最小BLAKE3、实际用到的SHA-256、bytes/hex/整数和长度帧、确定性编码/版本/拒绝规则；不用扩成通用密码套件或复制serde框架。优先复用M2-A已交等价Auto库能力，差额由本Plan与标准库owner同一单源补齐。编译器专有IR/envelope codec属AAC应用层。新增能力单列AAC profile/version，不无声覆盖冻结M2-A std或改变它的分母；如果新增公共标准库操作，按现覆盖合同验证AV三provider、AC、A2R、A2PY并保A2C旧腿，不能因AAC只用AC就豁免。

依据[Design59](../design/59-aac-native-bootstrap-and-compiler-subject.md)和[AAC路线](../design/strategy/aac-delivery-roadmap.md)。AAC在M2-A稳定后与M2-B可并行，独立后续版本/验收，非M2硬门。非目标：不重写AV/Cranelift/全部A2X，不迁全部产品，不改旧M2分母/旧oracle，不用旧AAVM证书证明AAC。

## 2. 架构方案

Auto主体拥有source语法/语义/实例/CT调度/HIR/MIR和增量策略，Rust只保机器后端/消费侧安全复验/AV执行及明确外部runtime。Gen0编译阶段driver可用其Rust前端，driver运行处理被测source/IR时算法必须Auto执行；1053/1054再核Gen1/2/3主体窗口。序列化checked标志没有信任权威。

拟新增或扩展：统一Auto库摘要模块（缺失时按std owner实际命名新增）、experimental/ace/aac/support/codec/与tests/support/（拟新增）；禁止在AAC和std各抄一份hash算法。实际共享路径由供给owner单写，后继消费正式已交接口，不读其他树WIP。所有源码/工具/stdlib/grammar/ABI/schema/target/provider分轴标识，build/host/target/CT执行域分清。

## 3. 技术栈

纯Auto+新版core/alloc/必要hosted标准库，默认自身ABI与OS adapter；首个独立host/native基线Windows x64，其他host按原设计后继，不改变M2平台义务。需要公共库新增时按现覆盖合同，不复制通用stdlib或默认依赖libc；显式后端/AV/链接工具的外部依赖单列。

## 4. 需求分析与背景调查

本轮用户明确授权再次验证1044–1054，必要时补齐细化、先不施工。新增本Plan属于此规划范围，不是代码施工许可。G-AAC须M2A-01..08、985/T-A04独立复核与T-A05已落主来源稳定收据，并有未来实施授权；不等待整个985/B、984/986/987，也不给M2添加本Plan前置。

本轮只读观察auto-lang HEAD `7151f89665a2c8a0289ab044b6ea94b4adbe80eb`。具体源/Spec取材：

| 来源（仓根相对路径） | SHA256前16位 |
|---|---|
| `stdlib/auto/hash.at` | 1b24376eb8d676c8… |
| `stdlib/auto/hash.vm.at` | 5ef2a284e0f133d2… |
| `experimental/ace/crates/artifacts/src/identity.rs` | b1ca0253dc944859… |
| `experimental/ace/crates/artifacts/src/codec.rs` | c8e6f98d5557dc97… |
| `experimental/ace/crates/source-query/src/ident.rs` | 833e5ce97ef3a4da… |
| `experimental/ace/crates/source-query/Cargo.toml` | d73996498a5e1bb0… |

规范取材：`docs/specs/stdlib/design/ace-core-and-text.md`; `docs/specs/auto-ac/artifact-contract.md`。旧overview/早期注释不是当前全语言事实；源/Spec冲突须具名解决，不能把规划写成实现。原正式输入在work时重新锁版本/hash；本表只读快照不替M2-A稳定收据。

## 5. 详细设计

frontmatter为本Plan完整交付前置；§8任务再细化本包批次。开工先核G-AAC和依赖已正式落主，未来每个driver首次使用任务建真实入口/命令及必要target/固定选择，T-06实际运行本包Auto资产和受影响Rust职责组；不等1053才第一次测试。

本包的需求矩阵、阶段profile与支持边界见§1，具体数据路径/owner/输出见§8，正反例与独立结果见§6。1044冻结现有更严预算、source/node/depth/bytes/CT fuel/输出/期限和计数算术，合法支持边界不窄化，超界明示资源拒绝。合法/非法/取消/恢复分别保真实终态，不能临时增预算或把错误变NA。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-ac/aac-digest-codec-profile.md` | 新增本包已验证profile/接口/所有权与支持边界；完整AAC及M2边界不变 | 关闭Design59 §9的具体缺口 | AC-01..04 |
| SD-02 | modify | `docs/specs/stdlib/design/ace-core-and-text.md` | review通过后补本包当前事实和新Spec指针，保原历史/未交项；本轮不写current Spec | 避免目标态冒当前实现 | AC-05/06 |

### AAC与M2三端能力的当前交接（2026-11-16优先）

本包执行[1068](1068-ace-m2b-three-backend-parity-and-aac-proof.md)和[Design59当前阶段合同](../design/59-aac-native-bootstrap-and-compiler-subject.md)：985/A正式来源仍为开工门，但允许其AC具名差额账。初始CS审计可列未交操作，依赖某操作的实现/验证切片必须先收到该操作正式来源；缺口回Design59 具名实施任务或实际库/领域 owner补齐，不回落旧VM或私建临时stdlib，也不令AAC整条路线反等所有AC能力先完成。

CS只表示本编译器源码需要什么，CL必须保最终M2的AV/AC/A2R完整能力目标，包括~T/?T/!T、泛型、CTE、标注及组合；不得因源码未用或A期AC不支持而删行。已交阶段模块/host/语法/符号/CT/构建driver可独立复核并正式交1068作AAC实证，不需等本包/1053/1054整包；不能以阶段收据冒完整自举。相反，本包尚未实际依赖的AC行不应卡其独立工作。

本包实际语言/库修复回Design59 具名实施任务或实际库/领域 owner，相关源码/SDK/语义改变后重绑证据。1053全CL/1054代际收口仍保原独立oracle/主体边界，完整AAC签章不替M2三端最终门，也不强加为M2发布前置。A2C完整对齐增强；A2PY既有标准库、Linux1063延期义务保持。

| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-AC-STAGED | modify | docs/specs/auto-ac/aac-digest-codec-profile.md（实际交付后） | A全齐后单向消费→A基础/具名差额、B期间AAC实证与AC补齐互相交接，CL完整目标不缩 | 用户阶段修订 | AC-AC-STAGED |

### 本轮语言完整性边界

1056仍只补AAC需要的精确摘要/codec，通用库单源复用，不变成重写全部标准库。 当前CL目标取最终M2三端required及A后延增量，来源身份以阶段真实版本绑定，不以旧A有限面封顶。

## 6. 测试设计

下列全部是拟议必验case，不是已运行结果。用冻结M2-A真实语法/IR/API构造；字面数值/字节/阶段/事件由独立oracle给定，不复制被测输出。必要fixture的创建任务已列入口列；Rust参考与独立规范/向量同时核，不能仅两端相等即PASS。

| Case ID | AC / owner_stage | 输入/状态 | 入口/启用任务 | 可观察独立预期 | 边界 |
|---|---|---|---|---|---|
| V-01 | AC-01/02/03/04；work | 空bytes、a-NUL-b、UTF-8 α、分块与一次性同一payload | Gen0编译的Auto support driver，T-03/05启用 | 摘要逐字节等于冻结向量/独立参考；同一算法分块与一次性一致，不以C字符串截断 | 本包profile，未实施 |
| V-02 | AC-01/02/03/04；work | 分段[ab,c]与[a,bc]；同payload不同domain/producer | Auto identity framing与原Rust参考，T-02/05 | 按当前协议包含长度/域信息并与同协议参考精确一致，两种错误拼接不得共用key；算法/域版本错不能误hit | 本包profile，未实施 |
| V-03 | AC-01/02/03/04；work | 带i64边界、嵌零bytes/未知字段/重复键/尾随或截断payload | Auto codec真实decode，T-04/05 | 合法按精确宽度roundtrip；各非法项按已冻结schema明确拒绝，不窄化float/int或反序列化Checked | 本包profile，未实施 |
| V-04 | AC-01/02/03/04；work | 宣布极大长度/深度、长度累加溢出，配边界合法近邻 | 真实decoder分配入口，T-04/05 | 先准入拒绝且未按攻击长度分配；合法边界通过，错误属资源/encoding非普通source类型错误 | 本包profile，未实施 |
| V-05 | AC-01/02/03/04；work | 若新增公共hash操作则列全部现覆盖腿，删除其中A2PY或AV/C腿报告 | 现TestCase/stdlib owner实际consumer，T-05 | 所有required腿真实值/bytes一致才关闭公共API；漏腿阻本Plan，不能只用AC或emit代run | 本包profile，未实施 |

### 公开IR格式补充

| Case ID | AC / owner_stage | 输入/状态 | 入口/启用任务 | 可观察独立预期 | 边界 |
|---|---|---|---|---|---|
| V-06 | AC-01/03/04；work | 冻结descriptor/HIR的canonical文本、等价格式变体及重复/错字段/错引用输入 | Auto数据reader/writer（T-04/05），语义准入1047/整机1053 | **用户裁定A 的可观察口径**：(1) **真实schema驱动**——`crates/core/schema/core-i32.atom` 被读成 tag/字段表，且 `slots:` 按真实 descriptor 映射（首 `Id`→`id`、`TypeUse`/`TypeRef`→`type`、其他枚举→`kind`，标签字面量不是字段名）；(2) **真实HIR fixture**（`core/fixtures/native/add-2-3.atom`）被**接受**（状态0）且 `canonicalize(canonicalize(x)) == canonicalize(x)` 逐字节幂等（两个模式各自定点，且两模式字节相同）；(3) **本包自身规范布局**逐字节恒等（组内 `", "`、4空格缩进、空体 `{}`、末尾 `\n`）；(4) 重复字段（含**内联节点**）8、未知字段9、未知节点/非`hir`根10、非法语法按码拒绝；(5) **reference writer 逐构造字节**（`descriptor::write_canonical` 的字段位置/内联决策）**不要求**，是具名交接给 1047/1053；(6) 语义合法但语义错觉的 type/DefRef 仍只得 untrusted DTO，绝不构造 Checked | 仅纯数据codec；完整HIR准入与逐构造字节在1047/1053真验 |

现有受影响候选组 `cargo t ace-stdlib ace-source ace-artifacts ace-a2x`（只读查`scripts/test-groups.toml`确认，未运行）；work核真实触面，选完整职责组与必要consumer，并实际运行本包Auto driver。普通模块排除`::performance::`，正式规模/持续压力只显式`cargo t ace-perf`，不成为每Plan收尾门。记录实际case/执行/条件排除数，准备/发现/运行/清理/总时间及输入工具产物身份；required未跑不能PASS。

### 当前阶段交接验证（未执行）

| V / AC | 原work与验证owner | 区分性输入/预期 |
|---|---|---|
| V-AC-STAGED / AC-AC-STAGED | 原T-01/所涉能力work与最终work验证；原独立review复核 | 有效A收据＋具名AC异步差额可准入不依赖该能力的AAC切片；实际使用该能力时缺来源须具名阻塞该切片，不能fallback；删CL行/将AC未实现改平台NA必须拒绝；补齐正式来源后同输入真实AC成功。阶段AAC通过不冒全CL/Gen2/Gen3或M2最终通过。 |

## 7. 验收标准

| ID | owner_stage | 关闭条件及验法 |
|---|---|---|
| AC-01 | work | §8所有work输出与本包profile逐项对应，V-01/05/06实际观察值/字节/控制符合独立预期；V-06 按裁定A：真实schema驱动 tag/字段表（含 `slots:`→真实字段拼写映射）、真实HIR fixture 接受且 `canonicalize(canonicalize(x))==canonicalize(x)` 幂等、本包规范布局字节恒等、reference writer 逐构造字节具名交接 1047/1053；不以文件存在代行为 |
| AC-02 | work | V-02/03/04的错误/资源/权限分类、合法近邻与失败清理均满足，无隐式Rust源语义/legacy回落；V-04 须同时有准入**合法近邻**（长度与节点深度各自的可接受侧） |
| AC-03 | work | 本包真正生产/消费边界成立，Auto主体/标准库单源/机器服务职责不偷换，schema/ABI/阶段身份及后继输入可复现；schema 表的字段集合须与真实 descriptor 一致（`slots:` 映射到 `id`/`type`/`kind`，标签字面量不接纳），重复字段规则对带tag节点与内联节点一致 |
| AC-04 | work | 全部V行和本阶段required输出/测试已实际关闭，所有真实命令/退出码/独立oracle/输入和产物身份摘要入计划；不能拿后续pending豁免本阶段required |
| AC-05 | review | 独立同树按revision核每个AC与完整数据路径、遗漏/延期/workaround、warning/format/debug及规范delta，pass或具名needs_fix/needs_replan |
| AC-06 | merge | review通过、落主身份核准后沉淀Spec、归档计划并按材料政策清理交接；不签M2或完整AAC |

work 阶段已关闭（证据见本文件末尾 §10 之后的 work/goal/复审收据）：AC-01..AC-04 由
V-01..V-06 的实测值/字节/结果码关闭（固定组 `ace-aac` 26 例全绿 + driver `--all` 25 例
exit 0）；AC-05（独立
复审）与 AC-06（merge 沉淀）按任务表留给 T-07/T-08，不在 work 自勾。

2026-11-19 **T-07 复审 pass**（记录见文件末尾）：AC-01..AC-05 在本文件冻结提交上关闭，
`status: reviewed`；AC-06（merge 沉淀）仍**未勾**，留 T-08。
- [x] **AC-AC-STAGED / work**：V-AC-STAGED关闭，A基础/AC差额与本包实际消费来源逐条对应，CL仍含最终三端完整能力，阶段AAC输出可正式供1068消费。由原work任务落实，原独立review核；不新增任务或提前计完成。

## 8. 执行步骤

拟唯一树 `D:/autostack/.wt/lang-1056/auto-lang`，分支 `plan-1056-dev`。材料唯一 `D:/autostack/plan-artifacts/plan-1056/`，开工登记owner/用途/在途状态；SDK/生成输出/target/仓外消费均显式在scope，不放仓内证据，不建junction/symlink。当前树/材料未建立。

| Task ID / 状态 | owner_stage | 依赖 | 输入/实际工作 | 输出/验法 |
|---|---|---|---|---|
| [x] T-01 基线与供给核对 | work | G-AAC及frontmatter实际已交输入、未来实施授权 | 核Plan r1、§4当前Spec/source/SDK/库/ABI/grammar/工具及唯一树和材料登记，记录兼容profile与数据接口 | 输入版本/缺口/owner及后继任务入口；AC-01/04 |
| [x] T-02 操作与装配合同 | work | T-01 | 逐用途盘点BLAKE3/SHA-256、domain tag/NUL分隔/长度帧/hex、int宽度/字节序、稳定顺序、IR/envelope编码和schema；实读原identity/codec构造冻结byte语义，不以hash同名替换算法。确定已有std消费、增补std单源或明确命名AAC support包的装配路径/版本，原std保留包身份不冒改。 | CS逐操作与源字节/版本合同，缺差有真实owner和本包任务；对应V行和AC-01..04实际核验 |
| [x] T-03 精确摘要Auto单源 | work | T-02 | 复用或实现所缺CPU纯计算Auto摘要/字节原语，沿core/alloc层及AutoABI，不默认外包Rust/libc；取空、嵌零、多字节、分块/边界输入，以独立固定向量和锁定Rust参考库交叉核。若已交同实现只补消费证明，不重复实现。 | 字节精确digest及统一库来源；实际公开API新增时完整后端/provider验证；对应V行和AC-01..04实际核验 |
| [x] T-04 编译器codec及资源准入 | work | T-03 | Auto实现最小确定性codec/frames/manifest及当前AC canonical Atom/HIR文本的纯数据reader/writer（裁定A：**解析真实 descriptor 文档** `core/schema/core-i32.atom` 得到 tag/字段表，`slots:` 按其真实拼写映射 `id`/`type`/`kind`，**不移植** `descriptor::write_canonical` 的逐构造布局；后者作具名交接 1047/1053），显式版本、宽数值/bytes/引用/ID；wire不携带可信Checked capability。先校验消息/数量/深度/长度算术再分配，坏编码/重复字段（含内联节点）/溢出/未知schema/尾随数据按冻结规则拒绝。新增codec区分canonical bytes规则与语义等价规则，保公开IR输入/输出格式；物理store布局由1044/1052独立决定。 | Auto应用层codec及strict decode/资源反控，不搬整个Rust ManagedStore；对应V行和AC-01..04实际核验（复审 F1/F4/F5/N-2/N-3 整改） |
| [x] T-05 真实消费与ABI边界交接 | work | T-04 | Gen0编译Auto阶段driver，核与现工件逻辑身份的兼容范围、domain/分段防混淆、整数/UTF编码及独立invalid拒绝；向1045/1051/1052交digest与codec版本。必要标准库公共操作按Design56覆盖合同真跑，原Rust宿主工具依赖如实列出。 | 正式可消费支撑包和CS证据；新增义务只归AAC，不反加为985/A新门；对应V行和AC-01..04实际核验（复审 F1 后重确认可消费面） |
| [x] T-06 定向验证与交review | work | T-02..05 | 实际运行全部V行、本包Auto阶段driver及受影响完整固定组/必要consumer，核输入/工具/产物身份、真实父子终态、资源和清理，准备Spec delta | 逐项证据摘要，AC-01..04；execution_done交review，不预勾T-07/08（复审 F2/F3/F4/F6/F7/F8 需在T-06重做证据与规范文本） |
| [x] T-07 独立复审 | review | T-06 | 同一树冻结revision，逐AC/完整数据流独立核验；搜漏项/延期/workaround，具名债务登记并核warning/format/Spec-impact | review结论及修复条件；AC-05 |
| [ ] T-08 合入沉淀与清理 | merge | T-07 pass | 按原merge范式核已落主身份、沉淀已验Spec并归档；所有输出路径对账，证据折入计划后清scope，wt-guard clean再移树/分支 | 独立交付/后继版本与清理摘要；AC-06 |

## 9. 复审记录

> **收据位置说明**：work / goal / 复审收据按追加顺序位于**本文件末尾（§10 之后）**，
> 不在本节内；§7 与本节对收据的交叉引用均指向文件末尾的那一段。

2026-11-16 AAC-LANGUAGE-COMPLETENESS 规划静态预检：17份设计/计划239处本地链接有效；14份原Plan共126任务（AAC13包116项＋1068十项）的ID/勾选/阶段/依赖/状态/进度保留，无新Plan/新任务。10份AAC计划补族级SD/V/AC，1068明确语义主体/机器补齐/完整CL分工；当前任务中旧A期AC有限上限已校正。可达166份Plan无缺失/环，章节/表格/围栏和git diff --check通过。仅规划检查，全部新增验证未实施；未改产品代码/已验Spec/归档历史、未生成fixture或运行编译/产品/测试。

2026-11-16 stage:new / r3：检查发现旧执行任务/测试说明仍有A期有限适用措辞，已修正当前任务和本族验证；原历史记录/既有AC/V/任务ID及阶段/依赖保留。前端语义补齐在AAC原任务，机器/runtime由1067/1068或实际owner交付，不靠自编译happy-path签全语言。本次未实施/构建/运行。

2026-11-16 M2-PARITY-STAGED 规划静态核验：31份设计/计划779处本地链接有效；20份原计划232项任务的ID/总数保留，新增1067/1068共20项待实施任务。980并行work将12/29推进15/29（B-NATIVE-I/V/R），本轮保留其真实进展，未回退勾选。可达158份Plan无缺失/依赖环；985新增1068仅为B最终依赖，A→AAC/补齐→B收口的任务阶段图无反向整包等待。章节/表格/围栏、git diff --check通过。仅规划检查，不是产品或独立实现review PASS；未改已验Spec/归档历史、未构建或运行Windows/Linux测试、未启动AAC。

2026-11-16 stage:new / r2：执行最新三端最终一致、A可保AC差额、B期间AAC实证规则，新增SD/V/AC-AC-STAGED；原任务ID/数量/勾选/状态/进度及包依赖保留。只修规划，无代码/fixture/运行/新工作树或签章。

2026-11-14 stage=new，revision=1；规划已完成，outcome=blocked（等G-AAC和未来实施授权），next=开工门满足后work。本轮只是文档审计/计划补齐，没有实现/测试执行/独立代码review/合入；进度0/8，Design59 §9记本次发现与责任分配。

2026-11-19 stage: review | review_scope: final | reviewer_identity: 独立复审子代理 `reviewer-1056`（deepseek/deepseek-v4.1-flash，独立上下文，未参与本包实现；实现作者为 Codex 会话）。limitations: 无法读取实现会话；未复跑 driver `--all`（25 次 Gen0 编译、work 记 80.6s，属交互入口非固定组，按未复跑登记）；为跑组在树内新建 `experimental/ace/target`（2.5 GB，随 worktree 删除，非仓外散落）。| plan_id: PLAN-1056 | plan_revision: 5 | outcome: **needs_fix** | worktree_path: `D:/autostack/.wt/lang-1056/auto-lang` | branch: `plan-1056-dev` | reviewed_commit: `726f8c64c1095bf5736d34aa43e3bfb4bebf6425` | base_commit: `c4057c9598d81ba4801b09e83276aa0c27c38e1d` | dependency_revisions: `experimental/ace/crates/artifacts/src/identity.rs` b1ca0253dc944859、`codec.rs` 490843a5e92386cf、`experimental/ace/crates/a2x-core/src/sha256.rs` 0c14f616df68c7c1、`HASH_ALGO="blake3-1.5"`、AV 默认 `MAX_AV_INSTR_STEPS=1_000_000`（harness `AV_MAX_STEPS=64_000_000`）、测试 oracle 外依 `sha2 0.10`/`blake3 1`。

spec_inputs: `docs/specs/auto-ac/aac-digest-codec-profile.md`（新增，@726f8c64c，sha16 `4d9a81a31370eb9a`，9617 B）；`docs/specs/stdlib/design/ace-core-and-text.md` 未改（SD-02 留 merge，T-01 快照 sha16 `31ea76870c06a3a2`）；`docs/specs/auto-ac/artifact-contract.md` 未改（但 frontmatter 列入 supersedes，见 F7d）。实现身份：`digest.at` b96254e31736993b、`codec.at` 4182498f4b311c7a、`atom_data.at` 3074f4a011243c26、`driver.at` f59f9d191eae68cb、`conformance.rs` 449fe9dafb0757e0、`atom_data.rs` d4d10a4d1d997b7d、`lib.rs` d490e11fdc627bb4、`scripts/test-groups.toml` 251318a3e312df48；worktree `git status --porcelain` 空。

**本会话模块/范围计时（重点复核 test-cost）**

| 阶段 | 范围/命令 | 耗时 | 结果 |
|---|---|---|---|
| 运行前并发核对 | `Get-Process`（cargo/rustc/cargo-nextest 无活动进程）；nextest `max_parallel_tests_observed=14`、`peak_job_bytes=883 MB` | — | 无他人构建占用，未排队 |
| 准备/构建 | nextest 编译阶段（worktree 内默认 `experimental/ace/target`，无 `CARGO_TARGET_DIR`，debug/12 jobs） | 23.0 s | Finished test profile |
| 测试运行 | `cargo nextest run -p ace-aac-support-conformance --config-file ../../.config/nextest-full.toml`（`AUTO_PLAN_ARTIFACTS=D:/autostack/plan-artifacts/plan-1056/tmp`） | 7.756 s | 14/14 pass，0 skipped |
| 整命令 | 同上（含编译） | 31.6 s | exit 0 |
| 固定组 | `cd <worktree root>; cargo t ace-aac`（无 `CARGO_TARGET_DIR`） | 13.3 s（warm，含 driver 构建） | `PASS: ace-aac` exit 0；`results.json` `executed=14`、`gate_passed=true`、`wall_seconds=12.86` |
| 静态核对 | `git log/show --stat/status`、两提交 diff、全部新增源/测试文件（4 个 `.at` + 测试 crate 5 文件 + `Cargo.toml/Cargo.lock` + `scripts/test-groups.toml`）与 SD-01、参考实现（identity.rs/codec.rs/descriptor.rs/atom_text.rs）、真实 Atom 夹具通读 | 未计时 | 见 F1..F8 |

**新增用例逐例墙钟（nextest 实测，全 14 例，全部 <10 s）**

| 用例 | 墙钟 |
|---|---|
| conformance::v03_frame_bytes_match_reference | 7.499 s |
| atom_data::atom_canonical_roundtrip_is_identity | 7.756 s |
| conformance::v01_digest_vectors | 6.124 s |
| conformance::v01_blake3_tree_vectors | 5.405 s |
| atom_data::atom_equivalent_variants_canonicalize_identically | 5.113 s |
| conformance::v01_sha256_part_split_invariance | 4.964 s |
| conformance::v02_identity_report | 4.902 s |
| conformance::v03_frame_decode_taxonomy | 4.829 s |
| conformance::v04_admission_before_allocation | 4.769 s |
| atom_data::atom_semantic_odd_data_is_accepted_as_untrusted_dto | 4.767 s |
| atom_data::atom_byte_list_shape_is_outcome_first | 4.759 s |
| conformance::v03_manifest_report | 4.758 s |
| atom_data::atom_rejection_taxonomy | 4.602 s |
| conformance::v03_manifest_taxonomy | 4.439 s |

test-cost 门：无新增用例 >10 s（最大 7.756 s），`726f8c64c` 的"每证据族一编译"整改有效；但该整改删除了边界向量（F2）。

**验收证据图（AC-ID → 判定 → 命令/方法 → 结果，@726f8c64c）**

| ID | 判定 | 命令/方法 | 结果 |
|---|---|---|---|
| AC-01 | partial | `cargo t ace-aac`；逐字节对 `sha2`/`blake3`/`domain_hash`/`frame_chunk`/独立 Rust manifest 编码器 | digest/identity/frame/manifest 值全部逐字节一致（14/14 pass）；但 V-01 宣称的 1023/1024/1025/2048/3000 边界在整改后只剩 128 max + 1025 单例，且 1025 不进入 `b3_merge` 合并分支（F2）；SD-01 §3 与 §10 T-03 的向量/参考声明与实测不符（F3） |
| AC-02 | partial | V-03/V-04 结果码序列；frame `0000000102030405060708`、manifest `0c0d070a0b0e0f`、admission `050600050900` 全绿 | frame/manifest 分类与合法近邻成立；`atom_data` 的 depth>64 / >262144 标量准入规则未有任何用例（F4），故 V-04 所要求的"极大长度/深度先准入拒绝"未实测 |
| AC-03 | partial | 读 `atom_data.at` 冻结 tag/字段表并与真实 `descriptor` 文档/`descriptor::write_canonical` 输出对照；读 `stdlib/ace/**`、`stdlib/contracts/**` diff | 无公共 std 操作新增、冻结 std 切片字节未动、无第二份 hash 实现（✓）；但 `atom_data` 的冻结表（`descriptor/module/decl/type`，字段仅 id/kind）不是仓内真实 canonical 文本：真实 descriptor 文档（`experimental/ace/crates/av/tests/fixtures_828/old-core-i32.atom`，子节点 `enums/nodes/branches`）与真实 HIR 文本（`core/fixtures/native/add-2-3.atom`，root `hir(...)`）都会被拒（F1），SD-01 §8 亦未登记该 schema |
| AC-04 | **fail** | 逐 V 行核对 §6 与实测；核 §10 收据数字 | V-04（深度/长度准入）、V-06（真实 descriptor/HIR 文本）未真正关闭；V-01 边界集缩水且收据仍写旧向量；组例数收据 "15 例/15:15" 与实测 `executed=14` 不符（F7a） |
| AC-05 | **fail（本次复审）** | 本记录 F1..F8 | needs_fix |
| AC-AC-STAGED | partial | 读 SD-AC-STAGED 目标文件 `aac-digest-codec-profile.md` §9 | §9 只写 1052/1045/1051/1047/1053 与公共 std 升级路径，未落 "A基础/具名 AC 差额 + B 期间 AAC 实证互相交接、CL 目标不缩" 的阶段规则（F6） |
| AC-06 | 未评估 | — | merge-owned，保持未勾；本次不签 M2/AAC |

**findings（稳定 ID，含最小重放）**

- **F1 [blocker｜AC-03/AC-04/V-06]** `atom_data` 的"冻结 tag/字段表"不是本仓真实 canonical Atom/HIR 文本词表，V-06 要求的输入读不进来。证据：`atom_data.at:934-977` 只允许 `descriptor/module/decl/type`，`type`/`decl` 只允许 `id/kind`，root 必须是 `descriptor`；真实 descriptor 文档 `experimental/ace/crates/av/tests/fixtures_828/old-core-i32.atom:10-60` 根为 `descriptor` 但子节点是 `enums/nodes/branches`（→ 10 unknown-node），真实 HIR 文本 `experimental/ace/crates/core/fixtures/native/add-2-3.atom:3` 根是 `hir(...)`（→ 10），真实 HIR 写出的 `type(id, kind: function) { params; result }`（`descriptor.rs:3835`）字段被当未知字段（→ 9）；另外 writer 在 group 内用 `; ` 分隔字段（`atom_data.at:1140,1151`），仓内 descriptor 夹具与真实 `hir(...)` 头都用 `,`（`old-core-i32.atom:10`、`descriptor.rs:3776-3780`），即使放行也不会逐字节 roundtrip。测试里的 `FROZEN` 文档（`atom_data.rs:58-69`）是实现者自写方言，不是任何真实 writer 的输出，因此只证明自洽。最小重放：对 `old-core-i32.atom` 全文调用 `canonicalize` → 现返回 `10`；期望按 T-04"实读 core/src/atom_text.rs 与 descriptor::write_canonical"的冻结词表接受并回吐规范字节（或把真实 schema 词表与 HIR 词表明确写进 SD-01 并说明本切片只覆盖其中哪一段——但不得再用自造 fixture 充当"canonical"证据）。
- **F2 [high｜AC-01/AC-04/V-01]** `726f8c64c` 删除 BLAKE3 边界向量（1023/1024/3000），`b3_merge` 的合并分支（≥3 chunk）在现测试里无任何覆盖；§10 T-03 与 SD-01 §3 仍宣称这些向量已对参考实现验证。证据：`git diff cb69e84e0..726f8c64c -- conformance.rs` 把 `for n in [1,64,65,128,1023,1024,1025,3000]` 改成 `[1,64,65,128]`，另加 1025 单例；1025 = 2 chunk，`b3_merge` 调用时 `t=1` 立即返回（`digest.at:775-787`），`b3_fold` 才做 parent 合并，故 `b3_merge` 内部 while 合并体未执行；`driver.at` 的 `blake3_case("chunk1024"/"chunk1025")` 只经 driver CLI，固定组不跑。最小重放：把 1024/2048/3000 字节（至少 2048 或 3000）加入同一 report 用例并与 `blake3` crate 对账；当前若 `b3_merge` 有错，14 例会全绿（假绿）。
- **F3 [medium｜AC-01/AC-05｜SD-01]** 规范文本高估已验证面：§3 表把 `ace_a2x_core::sha256::sha256_hex` 列为"已验证参考"，但测试只依赖 `sha2`（`experimental/ace/tests/aac-support/Cargo.toml:15-16`，全文件无 a2x 调用）；§3 命名边界声称 BLAKE3 分块由 "1023/1024/1025/2048/3000" 验证（其中 2048 从未在任何提交出现，其余见 F2）；§6 "写入路径由 `i64_le_bytes` 语义（见 §7）覆盖" 的交叉引用指向结果码表，且代码里没有 `i64_le_bytes`（实际是 `push_i64le`，`codec.at:202`）。最小重放：按实际用例重写 §3/§6，或补齐对应验证；规范是持久知识，不能留未验证声明。
- **F4 [medium｜AC-02/AC-04/V-04]** `atom_data` 自身声明的两条准入规则（`atom_data.at:65-67, 1322-1325` 文本 >262144 标量；`atom_data.at:600/697/762` 深度 >64）无任何用例，V-04 要求"宣布极大长度/深度…先准入拒绝，配边界合法近邻"。work 收据给的理由"harness 64M 步预算先于模块阈值生效"只可能对深文档成立；长度规则在 lex 之前即拒（`build_canonical` 首行），用运行期构造的 262145 标量串即可廉价观察，不需要物化深度输出。最小重放：`canonicalize(<262145 个 'a'>)` → 期望 6；`canonicalize(<65 层嵌套文档>)` → 期望 6；并配 64 层/262144 的合法近邻。
- **F5 [low-medium｜AC-01/AC-03]** manifest DTO 的重复键规则自相矛盾：编码器按 (key,value) 排序并允许重复键（`codec.at:410-411, 442-459`），解码器对 key 相等一律判 12（`codec.at:642-661`），因此编码器可以产出自身解码器拒绝的字节；SD-01 §6 又把"key 相同按 value"写成规范顺序的一部分。最小重放：`manifest_encode` 传 pairs `[("a","1"),("a","2")]` → 编码 0/成功，随后 `manifest_decode` 同一字节 → 12；期望二者一致（编码侧也拒 12，或解码侧接受按 value 有序的重复键）。
- **F6 [medium｜AC-AC-STAGED]** SD-AC-STAGED 明确以 `aac-digest-codec-profile.md` 为落地目标（"实际交付后"），但该文件 §9 只列后继 Plan 消费与公共 std 升级路径，没有"A 基础/具名 AC 差额 → B 期间 AAC 实证与 AC 补齐互相交接、CL 完整目标不缩、缺来源须具名阻塞切片"的阶段规则；AC-AC-STAGED 因此只有断言没有承载文本。最小重放：读 profile §2/§9 与 §5 SD-AC-STAGED 行比对。
- **F7 [low｜记录/元数据]** (a) §10 首段仍写固定组"15 例"、原 T-06 段写 "15/15"，实测 `executed=14`（`results.json`）且整改段自述"14 例"——同一文件三个数；(b) work/goal 收据被追加在 §10"待澄清事项"末尾，而 §7 指向 §9"work 收据"，位置与交叉引用不一致；(c) `t01-baseline.md:24` 承诺"atom_data 哈希在 V-06 落地后以 §9 最终值重绑"，计划里没有最终源哈希（只有 commit）；(d) frontmatter `supersedes_spec_components` 含 `docs/specs/auto-ac/artifact-contract.md`，但 §5 无对应 SD 行、该文件也未改——merge 会误以为要沉淀它。最小修正：一个数字只留实测值（14）、收据移入 §9 或 §7 验证记录并修好 §7 的交叉引用、把最终源文件 sha16 写进 §9、从 supersedes 删掉 `artifact-contract.md`（或其 §5 补 modify 行并真改该文件）。
- **F8 [low｜健康检查]** 新增测试 crate 自身带 2 个 rustc 警告（`tests/aac-support/tests/conformance.rs:31 fn lit`、`:57 fn digest_case` 死代码，整改后遗留），违反 AGENTS.md §3 零未处理警告；其余构建警告均为既存 crate。

**evidence（可解析）**：复跑日志 `D:/autostack/plan-artifacts/plan-1056/verify/review-cargo-t-ace-aac.log`、`.../review-nextest-aac.log`、整改前测试快照 `.../review-conformance-v1.rs`（scratch，merge 前折入本文件并随 scope 删除）；固定组结果 `D:/autostack/.wt/lang-1056/auto-lang/target/test-groups/run-ckguqlji/results.json`（`executed=14`、`wall_seconds=12.86`、`gate_passed=true`、`source_state=cdd8cbfd…`）；关键 diff `git diff cb69e84e0..726f8c64c`；真实文本夹具 `experimental/ace/crates/av/tests/fixtures_828/old-core-i32.atom`、`experimental/ace/crates/core/fixtures/native/add-2-3.atom`；参考实现 `experimental/ace/crates/artifacts/src/{identity.rs,codec.rs}`、`experimental/ace/crates/core/src/descriptor.rs:3755-3875`。未写 `.autoos/specs.json` 等 legacy ledger（两提交 diff 无此路径），未新增仓外哈希实现（13 个文件全在 `experimental/ace` 与 `scripts/test-groups.toml`、`docs/specs`）。

pending_closeout_gates: AC-06（merge：沉淀 SD-01/SD-02、归档、材料清理、wt-guard clean）；本 Plan 冻结受影响模块并集 = `ace-aac`（唯一被改 scoped crate 为新增 `experimental/ace/tests/aac-support`，既有 crate 代码零改动），修复后须在冻结提交重跑该组；周期 `cargo t all` 属主检出单实例批回归，不作为本 Plan 门。另记：主检出本计划文件当前仍是未跟踪状态（`?? docs/plans/1056-…`，主检出另有其他会话未提交改动），merge 前须随计划簿记一并提交，本复审只追加本文件内容、未动其他路径。

**next**: work —— 在 T-04（真实 descriptor/HIR 词表 + manifest 重复键规则）与 T-06（恢复 BLAKE3 多块/树合并与 atom_data 准入向量、更正 §10/SD-01 声明、清警告、修记录数字/位置/元数据）落实 F1..F8，T-05 在 F1 修复后重确认可消费面；新提交后回到 review（AC-01..AC-04、AC-AC-STAGED 全部重验）；不勾 AC-06、不合并。

2026-11-19 stage: review | review_scope: final（修复复审：上次 verdict + 修复 diff + 受影响判据；未重放全目录） | reviewer_identity: 独立复审子代理 `reviewer-1056b`（deepseek/deepseek-v4.1-flash，独立上下文；未参与本包实现，实现方为 Codex 会话）。limitations: 未读取实现会话；driver `--all`（25 例交互入口）未复跑（上次记录 @`cb69e84e0`，`driver.at` 其后在 `f113768ce` 已变，按过期证据登记）；全程被他计划构建占用（见计时表），固定组计时为 **busy 口径**，另以 `-j 1` 串行口径分离例内成本；为跑组在树内重建 `experimental/ace/target`（debug，随 worktree 删除，非仓外散落）。| plan_id: PLAN-1056 | plan_revision: 5 | outcome: **needs_fix** | worktree_path: `D:/autostack/.wt/lang-1056/auto-lang` | branch: `plan-1056-dev` | reviewed_commit: `c2d2a1179fe32f3193d8bbcf5d406533c16945e6` | base_commit: `c4057c9598d81ba4801b09e83276aa0c27c38e1d`（上次复审基线 `726f8c64c1095bf5736d34aa43e3bfb4bebf6425`） | dependency_revisions: `experimental/ace/crates/artifacts/src/identity.rs` b1ca0253dc944859、`artifacts/src/codec.rs` 490843a5e92386cf、`a2x-core/src/sha256.rs` 0c14f616df68c7c1、`core/src/descriptor.rs` 041852886f22bf1f、`core/src/atom_text.rs` 0fc9b5ceed7357e3、`HASH_ALGO="blake3-1.5"`、AV harness `AV_MAX_STEPS=64_000_000`/`AV_WALL_SECS=300`、测试 oracle 外依 `sha2 0.10`/`blake3 1`。

spec_inputs: `docs/specs/auto-ac/aac-digest-codec-profile.md` @`c2d2a1179`（SD-01；新增 §8.1/§8.2 writer 布局与幂等合同、§9.1 阶段合同；sha16 `2a78dfbd97dfbdf5`，14654 B——即本次冻结副本身份）；`docs/specs/stdlib/design/ace-core-and-text.md` 未改（SD-02 留 merge）；`docs/specs/auto-ac/artifact-contract.md` 未改且已从 `supersedes_spec_components` 移除（F7d 关闭）。
实现身份 @`c2d2a1179`：`atom_data.at` 947d2d6c6472acd2/61258、`codec.at` b3b4f73f9b660288/20919、`digest.at` b96254e31736993b/28271、`driver.at` f09e89515cc5a588/17952、`atom_data.rs` 99438d0f69701bb2/28071、`conformance.rs` 0198dedb11db6a46/12433、`lib.rs` d490e11fdc627bb4/6902、`scripts/test-groups.toml` 237ee4007162648e/61518；`git status --porcelain` 空（审查期只改 mtime 触发重编，无实现编辑）。

**修复面核对**：`git diff --stat 726f8c64c..c2d2a1179` 只触 7 个文件（SD-01、`codec.at`/`atom_data.at`/`driver.at`、两个测试文件、`scripts/test-groups.toml`）；`stdlib/ace/**`、`stdlib/contracts/**`、既有 crate 零改动；无 `.autoos/specs.json` 等 legacy ledger 写入；无第二份 hash 实现（F8/F3 的"单源"前提仍成立）。`421f2629c` 改 `atom_data.at`(+`atom_data.rs`+SD-01)，`c2d2a1179` 只删组参数 `-j`。

**本会话模块/范围计时（test-cost 复核；机器被 plan-1098 的 nextest archive 运行、plan-1079 的 `cargo test --release -p ace-av` 及 19:25 起的又一轮 nextest 占用，未获空闲窗口；未停止他人进程、未另起第二实例）**

| 阶段 | 范围/命令 | 耗时 | 结果 |
|---|---|---|---|
| 运行前并发核对 | `Get-CimInstance Win32_Process` 命令行/PID/启动时间 | — | 并发：plan-1098 nextest（PID 42680/51204/22544）、plan-1079 `cargo test -p ace-av --release`（77644/53136）+ rustc；计时按 busy 记录（未排队等待，明确为本记录限制） |
| 准备/构建 | `cargo t ace-aac` 内 nextest 编译（`experimental/ace/target`，debug，无 `CARGO_TARGET_DIR`） | 6.33 s（wrapper `preparation.wall_ms=6334`） | 23 例发现 |
| 固定组（默认并行） | `cargo t ace-aac`（wrapper 调 nextest run，组参数已无 `-j`；`max_parallel_tests_observed=20`） | wrapper `wall_seconds=32.36`（execution 26.0 s；外层整命令 71.3 s，含 `cargo t` 包装器自身构建/启动，未细分） | `PASS: ace-aac` exit 0；`executed=23`、0 skipped、`gate_passed=true`、`source_state=b38087f4…` |
| 串行口径（分离例内成本） | `cd experimental/ace; cargo nextest run -p ace-aac-support-conformance --config-file ../../.config/nextest-full.toml -j 1`（`AUTO_PLAN_ARTIFACTS=D:/autostack/plan-artifacts/plan-1056/tmp`） | 编译 2.37 s；测试 111.1 s；整命令 114.6 s | 23/23 pass，0 skipped |
| 健康检查（F8） | 触碰两测试文件后强制 `cargo check -p ace-aac-support-conformance --lib --tests` | 15.6 s | exit 0；**新 crate 0 warning**（其余 warning 全来自既有 `ace-av-core`(2)/`ace-core`(3)/`ace-lowering`(1)/`ace-av`(5)） |
| 静态核对 | `git log/show/diff`、`atom_data.at` 全文（lexer/parser/schema 表/writer/公共 API）、`codec.at` manifest 编解码、SD-01 全文、真实 `core-i32.atom`/`add-2-3.atom`、`descriptor.rs` 槽位映射、测试断言 | 未计时 | 见 F1..F8 与 N-2..N-6 |

**新增/受影响用例逐例墙钟（23 例；串行 `-j1` = 例内成本口径，默认并行 = 固定组实际观察口径）**

| 用例 | 串行 -j1 (s) | 固定组默认并行 (s) |
|---|---|---|
| atom_real_schema_mode_is_idempotent | 8.790 | 24.00 |
| atom_real_schema_drives_validation | 8.410 | 23.84 |
| atom_real_hir_canonical_layout | 7.828 | 24.56 |
| atom_real_hir_canonicalization_is_idempotent | 7.251 | 23.02 |
| atom_real_hir_fixture_accepted_by_both_modes | 7.191 | 22.35 |
| atom_real_schema_duplicate_field_is_8 | 6.978 | 22.06 |
| atom_real_schema_mode_canonical_layout | 6.787 | 13.45 |
| conformance::v01_blake3_tree_vectors | 6.108 | 19.37 |
| conformance::v01_digest_vectors | 5.588 | 19.40 |
| conformance::v03_frame_bytes_match_reference | 5.577 | 21.89 |
| conformance::v02_identity_report | 5.470 | 16.81 |
| atom_byte_list_shape_is_outcome_first | 5.002 | 10.85 |
| conformance::v03_frame_decode_taxonomy | 4.461 | 16.70 |
| conformance::v01_sha256_part_split_invariance | 4.280 | 16.97 |
| atom_semantic_odd_data_is_accepted_as_untrusted_dto | 3.123 | 10.44 |
| atom_equivalent_variants_canonicalize_identically | 2.556 | 17.05 |
| conformance::v04_admission_before_allocation | 2.366 | 6.64 |
| atom_admission_depth | 2.297 | 10.44 |
| conformance::v03_manifest_report | 2.282 | 6.88 |
| atom_rejection_taxonomy | 2.277 | 16.29 |
| atom_canonical_roundtrip_is_identity | 2.206 | 16.25 |
| conformance::v03_manifest_taxonomy | 2.197 | 6.32 |
| atom_admission_rules | 2.115 | 10.14 |

判读：串行全部 ≤10.000 s（最大 8.790 s），例内成本合格；**固定组默认并行 20/23 例 >10 s（最大 24.56 s）**，超限来自组内 23 个"每例自带一次 Gen0 编译"的用例以 20 路并发叠加外部构建。按 test-cost（每例 ≤10 s，含测试内等待）固定组当前形态即不合格，需组级限流后重测（N-4）；不能只用串行口径充作固定组证据。另注：`scratch()` 每调用建一个 `aac1056-*` 目录且不清理，`D:/autostack/plan-artifacts/plan-1056/tmp` 现积 1555 个目录/约 1067 MB（merge 删 scope 即清；非正确性缺陷，登记备查）。

**验收证据图（AC-ID → 判定 → 命令/方法 → 结果，@`c2d2a1179`）**

| ID | 判定 | 命令/方法 | 结果 |
|---|---|---|---|
| AC-01 | **partial** | `cargo t ace-aac`（23/23）＋逐字节对 `sha2`/`blake3`/`domain_hash`/`frame_chunk`/独立 Rust manifest 编码器；读 SD-01 §3/§8/§8.1/§8.2 | digest/identity/frame/manifest 全部逐字节一致（V-01 边界含空、嵌 NUL、UTF-8、55..127、BLAKE3 1/64/65/128+2049 三 chunk；V-02 分帧/域分离；V-03 帧 55 组、manifest i64 边界均与独立 oracle 一致）→ V-01 通过；但 schema 模式字段表把槽位**标签**当字段名（N-3），且计划 §6/§7 未按 A 裁定写可观察口径（N-5），故 profile 与 §8 输出只部分对应 |
| AC-02 | **partial** | V-03/V-04 结果码序列（`v03_frame_decode_taxonomy`=`00 00 00 01 02 03 04 05 06 07 08`、`v03_manifest_taxonomy`=`0c 0d 07 0a 0b 0e 0f`、`v04_admission_before_allocation`=`05 06 00 05 09 00`）＋读 `atom_data` 准入与 manifest 编解码 | frame/manifest/admission 分类与合法近邻成立；但 V-04 对 `atom_data` 的**边界合法近邻**（262144 标量、深度 63/64）仍无用例（F4b），且 manifest 仍存在编码器可产出解码器拒绝字节（F5b），"合法近邻与失败清理均满足"未闭环 |
| AC-03 | **partial** | `git diff --stat 726f8c64c..c2d2a1179`、读 `stdlib/ace/**`/contracts、真实 schema/夹具、依赖源 hash；读 schema 表构造与真实 `descriptor.rs` 槽位规则 | 无公共 std 操作、冻结 std 零改动、无第二份 hash 实现（✓）；真实 `core-i32.atom` 被解析、真实 HIR 夹具被接受（✓）；但 schema 模式 (a) 槽位→字段拼写映射与真实 descriptor 不符（N-3），(b) 内联节点重复字段不查（N-2），"真实 schema 驱动"只部分成立 |
| AC-04 | **fail** | 逐 V 行核对 §6 与实测；核 §10/§7 收据数字与当前源身份 | V-06 的 schema 保真缺口（N-2/N-3）、V-04 合法近邻（F4b）、V-03 编码/解码一致性（F5b）未关闭；计划 §7 仍写"固定组 `ace-aac` 16 例全绿"（现 23）、§10 T-03 仍写 1023..1025/3000 向量、最终源 sha16 停在 `f113768ce`（N-6）；固定组默认并行 20/23 例 >10 s（N-4） |
| AC-05 | **fail（本次复审）** | 本记录 F1..F8 复核 + N-2..N-6 | needs_fix |
| AC-AC-STAGED | **pass** | 读 SD-01 §9.1（SD-AC-STAGED 载体）、§6 SD-AC-STAGED 行、§7 勾选项、冻结 std diff | §9.1 已写"A 能力单向消费/具名 AC 差额可交接/CL 不删改/阶段通过 ≠ 完整签章"；本包无未交付 AC 依赖（消费 `stdlib/ace/asm/bytes.at`/共享 grammar/`packages-aggregates-ct` 车道），冻结 std 切片未改，无 fallback；勾选保留 |
| AC-06 | 未评估 | — | merge-owned；本次不签 M2/AAC，不勾 |

**F1..F8 关闭核对**

| ID | 判定 | 证据（@`c2d2a1179`） |
|---|---|---|
| F1 | **closed**（A 可观察口径） | 真实夹具 `experimental/ace/crates/core/fixtures/native/add-2-3.atom` + 真实 schema `core-i32.atom` 经 `include_str!` 嵌入：`atom_real_hir_fixture_accepted_by_both_modes`=状态 `00 00`；`atom_real_hir_canonicalization_is_idempotent`=flags `01 01`（schema-free 定点 + 两模式同字节）；`atom_real_schema_mode_is_idempotent`=`01`；`atom_real_hir_canonical_layout` 按 writer 规则钉住 `hir(schema: ...)` 头、无 `;`、`local(l_a, t_i32, name: "a", role: param, index: 0u32) {}`、单行 inline bindings、根 `}\n`；`atom_real_schema_mode_canonical_layout` 按规则写出完整期望文本。**未**要求 reference writer 逐字节相等（SD-01 §8.2 明写具名交接） |
| F2 | **closed** | `v01_blake3_tree_vectors` 用 **2049 B = 3 chunk**（`conformance.rs:103-110`），`blake3_hex_bytes` 在 `ci=1` 调 `b3_merge(total=2)`，偶数合并体真实执行（`digest.at:775-787`、调用 `:812`），结果与 `blake3` crate 逐字节一致（串行 6.108 s） |
| F3 | **closed（SD-01）** | §3 现只把 `sha2` 列为已验证参考、K/IV 明记为"取自 A2X 产品参考的未验对照"；BLAKE3 向量改为 1/64/65/128 + 2049（三 chunk）；`i64_le_bytes` 交叉引用改为 `codec.push_i64le`。残留：计划 §10 T-03 旧向量清单（N-6c） |
| F4 | **partial → F4b** | 拒绝侧已测：262145 标量→6（`atom_admission_rules`）、64 module 链→6（`atom_admission_depth`）；V-04/AC-02 要求的**边界合法近邻**仍缺（见 F4b） |
| F5 | **partial → F5b** | 解码器新序（同 key 按 value 升序）与完全相同 pair→12 已实现并有 12/13 用例；但编码器仍不查重复，仍可产出解码器拒绝的字节；SD-01 §6 与 `codec.at:644-646` 的"编码器不可能产出自身解码器拒绝的字节"不实（见 F5b） |
| F6 | **closed** | SD-01 §9.1 已成为 SD-AC-STAGED 的规范载体（§6 行/§7 勾选项与之一致） |
| F7 | **partial → N-6** | (a)/(b)/(c) 仍有错（§7 写"16 例"、work 收据仍只在 §10 而 §7 指向 §9、最终源 sha16 停在 `f113768ce`、§10 T-03 旧向量）；(d) 关闭（`artifact-contract.md` 已从 supersedes 移除，SD-02 目标仍为 `ace-core-and-text.md`） |
| F8 | **closed** | 强制重编译后新 crate `ace-aac-support-conformance` 0 warning；其余 warning 均为既有 crate |

**findings（稳定 ID，含最小重放）**

- **F4b [medium｜AC-02/V-04｜原 F4 残留]** V-04/AC-02 要求的"边界合法近邻"仍无：`atom_admission_rules` 只测 262145→6（`atom_data.rs:319-326`），无 262144 合法邻；`atom_admission_depth` 只循环单元素 `[(65,"06")]`（`:332-352`），无深度边界合法邻，且注释"65 levels are refused before any node is built"不实——代码走查显示第 64 个 `module` 的 **字段值** 在 `parse_value(depth=65)` 触限（`atom_data.at:579-581`，节点已建），故"64 module 链必拒"，真正被钉住的节点深度边界（63 vs 64）未被任何用例回答。最小重放：`canonicalize(<262144 个 'a'>)` 与 `canonicalize(<63 module 链，叶子带 id>)`／`<64 module 链，叶子不带字段值>` → 期望给出合法结果，并把实际节点深度上限写进 SD-01（现注释"nesting capped at 64"与实现口径需对齐）。这是 AC-02"合法近邻"的关闭证据，不能以"写入器超线性、代价高"豁免（该规模输出约数 KB，成本远低于既有 8.8 s 用例）。

- **F5b [medium｜AC-02/V-03｜原 F5 残留]** 编码器仍可产出自身解码器拒绝的字节：`manifest_encode` 只做长度/ASCII/空串校验后排序（`codec.at:325-426`），不检测重复；`manifest_decode` 的 `check_pair_order` 对相邻**完全相同** pair 返 12（`codec.at:647-670`）。故 `manifest_encode("A", 0i64, ["a","1","a","1"], [], 65536)` → 0，随后 `manifest_decode(该 payload, 65536)` → 12；`codec.at:644-646` 与 SD-01 §6"编码器不可能产出自身解码器拒绝的字节"为不实陈述。另：新规则的正例（同 key 不同 value 升序解码=0）无任何用例，取舍未完成双向钉住。最小重放：如上编码/解码两步；期望编码侧对相同 pair 返 12（或解码侧接受），并补 `[(a,1),(a,2)]` 正例。

- **N-2 [medium｜AC-03/AC-04/V-06]** 真实 schema 模式不检查**内联节点**（`{ k: v }`）的重复字段：`build_canonical_with_schema` 只调 `schema_doc_check`（`atom_data.at:1752-1789`），该检查只把 `items` kind=1 的子节点压栈（`:1391-1399`），内联节点仅是字段值（`vals` kind=4），从不入栈；而 schema-free 的 `dup_all` 遍历全体节点（`:1034-1044`），并有内联重复=8 的用例（`atom_data.rs:264-268`）。最小重放：`canonicalize_with_schema(REAL_SCHEMA, "hir(schema: \"s\", requires: {k: 1, k: 2}) {}\n")` → 现 **0**，同一文档走 `canonicalize` → **8**；违背 SD-01 §8"同节点内重复字段仍是 8"与 §8.2 反例纪律。修法：schema 模式在 schema 校验后同样跑一遍 `dup_all`（或对值内联节点单独查）。

- **N-3 [medium｜AC-01/AC-03/V-06]** schema 表的 `slots:` 映射错：`add_slot_fields` 把 `slots: [Id, TypeKind]` 的**标签字面量**注册为允许字段名（`atom_data.at:1201-1239`，由 `declare_node:1295` 调用），而真实 descriptor 把槽位映射为字段拼写 `id`/`type`/`kind`（`descriptor.rs:815-822`；`:18-20` 明写"Primary slots may equivalently be written as named fields"）。后果双向：`hir(...) { module(id: "m") { path: "p" } }` → **9**（真实 binder 接受该等价写法），`module(Id: "m")` → **0**（真实 binder 以 unknown-field 拒绝）。SD-01 §8 把该错误规则写成规范文本，须随代码一并订正。最小重放：`canonicalize_with_schema(REAL_SCHEMA, "hir(schema: \"s\", revision: 1u32, profile: \"p\") { module(id: \"m\") { path: \"p\" } }\n")` → 期望 0、现 9；反向 `module(Id: "m")` 现 0、期望拒。

- **N-4 [medium｜AC-04/test-cost]** 固定组默认并行时单例 >10 s：`c2d2a1179` 删掉组参数 `-j 4` 后，23 个"每例一次 Gen0 编译"的用例以 `max_parallel_tests_observed=20` 并发，本次 `cargo t ace-aac` 逐例 6.32–24.56 s、**20/23 >10 s**（串行 `-j1` 为 2.115–8.790 s，全部合格）。按 test-cost"每例 ≤10.000 s（含测试内等待）"固定组形态即不合格；需要组级限流（`.config/nextest-*.toml` 的 test-group/`max-threads`，不能再用与 fixed-tests 冲突的 `-j` 参数），并在修复后记录一份组模式逐例 ≤10 s 的当前证据；串行口径只能作例内成本说明，不能作为固定组通过证据。

- **N-5 [low-medium｜AC-01/AC-03 record]** 用户裁定 A 明确要求"同步修改 §6 V-06、§7 AC-01/AC-03 与本计划 revision"，实际只改了 SD-01：§6 V-06 仍写"保规范bytes/重复/未知字段按冻结schema拒绝"，未写"真实 HIR/descriptor 文本**接受** + `canonicalize(canonicalize(x)) == canonicalize(x)` 幂等 + 参考 writer 逐字节布局具名 1047/1053"；§7 AC-01 仍只绑 V-01/05、AC-03 未提 schema 表保真；§8 T-04 仍写"实读 `core/src/atom_text.rs` 与 `descriptor::write_canonical`"（A 前的旧口径）。该口径目前只在 §10 收据与 SD-01 §8/§8.2 里。`plan_revision` 已在裁定时升为 5（保留）。最小修复：按判定结果重写 V-06/AC-01/AC-03/T-04 的可观察口径（接受+幂等+参考字节交接、以及 N-2/N-3 的 schema 保真要求）。

- **N-6 [low｜记录/健康检查｜原 F7 残留]** (a) §7 第 128 行仍写"固定组 `ace-aac` 16 例全绿"（现 23 例），且交叉引用"§9 work 收据"实际不存在，§10 末尾说明又写"§9 的 work/review 收据"——work 收据在 §10，两处互指仍错；(b) "最终源身份"仍标 `f113768ce`，而 `atom_data.at`（947d2d6c6472acd2/61258）、SD-01（2a78dfbd97dfbdf5/14654）、`atom_data.rs`（99438d0f69701bb2/28071）在 `421f2629c` 后已变，缺当前审查身份；(c) §10 T-03 仍写"55..127/1023..1025/3000 字节"已验证，现测向量为 1/64/65/128+2049（SD-01 §3 已改正，计划未同步）；(d) `conformance.rs:98-101` 注释仍称"1025-byte payload is two chunks"（实际 2049 三 chunk）；`atom_admission_depth` 保留单元素 `for` 与"before any node is built"不实注释；`atom_data.at:560/565` 注释"nesting capped at 64"与有效边界不符（见 F4b）。

**evidence（可解析）**：固定组 `D:/autostack/.wt/lang-1056/auto-lang/target/test-groups/run-yuk7t08i/results.json`（`executed=23`、`gate_passed=true`、`wall_seconds=32.36`、`source_state=b38087f4…`、`max_parallel_tests_observed=20`）；本会话日志 `D:/autostack/plan-artifacts/plan-1056/tmp/review2/{cargo-t-ace-aac.log,nextest-j1.log,check-warnings.log}`（scratch，merge 前折入本文件并随 scope 删除）；关键 diff `git diff 726f8c64c..c2d2a1179`、`git show --stat 421f2629c/c2d2a1179/993795326/f113768ce`；代码位点 `experimental/ace/aac/support/atom_data.at:{579-581,1034-1044,1201-1239,1295,1391-1399,1752-1789}`、`codec.at:{325-426,644-670}`、`experimental/ace/crates/core/src/descriptor.rs:{18-20,815-822}`；真实输入 `experimental/ace/crates/core/{schema/core-i32.atom,fixtures/native/add-2-3.atom}`；SD-01 §8/§8.1/§8.2/§9/§9.1（sha16 `2a78dfbd97dfbdf5`）。未写 `.autoos/specs.json` 等 legacy ledger；未做实现编辑、未跑 git 写命令、未发布 canonical Spec。

pending_closeout_gates: AC-06（merge：沉淀 SD-01/SD-02、归档计划、材料清理、wt-guard clean）；本 Plan 冻结受影响模块并集 = `ace-aac`（唯一被改 scoped crate 仍为 `experimental/ace/tests/aac-support`，既有 crate 零改动），修复后须在**冻结提交**重跑该组并给组模式逐例 ≤10 s 证据；周期 `cargo t all` 属主检出单实例批回归，不作为本 Plan 门。另记：主检出本计划文件仍是未跟踪状态（`?? docs/plans/1056-…`），merge 前须随计划簿记一并提交；本次复审只追加本文件内容、未动其他路径。受影响任务 T-04/T-05/T-06 原本即未勾（无需回退）；已完成 T-01..T-03 三项，`current_step` 按完成数保持 3，`status` 保持 `executing`。

**next**: work —— T-04 修 N-2（schema 模式补内联节点重复检查）、N-3（按 `descriptor.rs` 槽位→`id`/`type`/`kind` 重写 schema 表并同步 SD-01 §8）、F5b（编码器重复 pair 一致拒绝 + 同 key 不同 value 正例）；T-05 在 atom_data 修复后重确认可消费面（含 driver `--all` 或多入口当前证据）；T-06 补 F4b 边界合法近邻、N-4 组级限流与组模式计时、N-5 计划 V-06/AC-01/AC-03/T-04 文本、N-6 计数/收据位置/当前源 sha16/旧向量/注释；新提交后回 review 重验 AC-01..AC-04 与 AC-AC-STAGED（受影响项）及全部 finding；不勾 AC-06、不合并。

2026-11-19 stage: review | review_scope: final（修复复审：上次 verdict + 修复 diff + 受影响判据；未重放全目录） | reviewer_identity: 独立复审子代理 `reviewer-1056c`（deepseek/deepseek-v4.1-flash，独立上下文；未参与本包实现，实现方为 Codex 会话）。limitations: 未读取实现会话；driver `--all`（25 例交互入口）未复跑，沿用 work 记账 @本轮提交（183.9 s，busy）；本机全程被他计划构建占用（见计时表），固定组/组模式计时均为 **busy 口径**；为跑组与一次只读探针使用树内 `experimental/ace/target` 与计划材料 `D:/autostack/plan-artifacts/plan-1056/{target-ace,review3}`（随 merge 删除，未在仓内留证据）。| plan_id: PLAN-1056 | plan_revision: 6 | outcome: **needs_fix** | worktree_path: `D:/autostack/.wt/lang-1056/auto-lang` | branch: `plan-1056-dev` | reviewed_commit: `ab8809b00cf9cef5d513e08c48d726c71900dcf9` | base_commit: `c4057c9598d81ba4801b09e83276aa0c27c38e1d`（上次复审基线 `c2d2a1179fe32f3193d8bbcf5d406533c16945e6`） | dependency_revisions: `artifacts/src/identity.rs` b1ca0253dc944859、`artifacts/src/codec.rs` 490843a5e92386cf、`core/src/descriptor.rs` 041852886f22bf1f、`core/src/atom_text.rs` 0fc9b5ceed7357e3、`a2x-core/src/sha256.rs` 0c14f616df68c7c1、`av-core/src/contract.rs` 2e2f9e1ac36b106b（`MAX_LIST_ELEMS = 65536`）、`HASH_ALGO="blake3-1.5"`、测试 oracle 外依 `sha2 0.10`/`blake3 1`。

**计划副本与基线**：主检出 `docs/plans/1056-…` 与 worktree 内同名文件 SHA256 一致（`5EE46EDEE4937D21…`，77055 B，LF）；worktree 内该文件已随 `ab8809b00` 入库、`git status --porcelain` 空；主检出仍持同内容未跟踪副本（§10 末段 merge 提示仍适用）。§9 上段"最终源身份"的 9 项 sha16/字节本次按 **git blob** 口径逐项复算相符（含 `conformance.rs` e838aa69c83ae9d5/12405、`scripts/test-groups.toml` 07aeefcbffdb102b/61515——工作区 CRLF 投影不影响 blob 口径）。

spec_inputs: `docs/specs/auto-ac/aac-digest-codec-profile.md` @`ab8809b00`（SD-01；sha16 `33f682310f27edb6`，18256 B，git blob）；`docs/specs/stdlib/design/ace-core-and-text.md` 未改（SD-02 留 merge）。实现身份 @`ab8809b00`（git blob）：`atom_data.at` 8e2f6b4571ae9d3f/64452、`codec.at` b942bf46f7f94618/22873、`driver.at` ac2765a4011b6646/18770、`digest.at` b96254e31736993b/28271、`atom_data.rs` ebe37180c19b86f8/35068、`conformance.rs` e838aa69c83ae9d5/12405、`.config/nextest-full.toml` 1133937f662a1bc8/10040、`scripts/test-groups.toml` 07aeefcbffdb102b/61515。

**本会话模块/范围计时（busy 口径）**

| 阶段 | 范围/命令 | 耗时 | 结果 |
|---|---|---|---|
| 运行前并发核对 | `Get-CimInstance Win32_Process`（cargo/rustc/nextest） | — | 并发：plan-1098 nextest archive（PID 30700/65904/47040）＋主检出 `cargo t ase-m0`（68904/47220）＋rustc；未排队、未停他人进程，计时按 busy 记录 |
| 固定组 | `cd <wt root>; cargo t ace-aac`（无 `CARGO_TARGET_DIR`） | 整命令 **37.34 s**（wrapper `wall_seconds=36.85`，prep `wall_ms=1105.8`） | `PASS: ace-aac` exit 0；`executed=25`、`max_parallel_tests_observed=4`、`gate_passed=true`、`source_state=3b6d5fec…`；逐例最大 **9.4726 s**（`atom_real_schema_slot_field_mapping`）、次 8.9419（`atom_admission_envelope`），**25/25 ≤10 s** |
| 组模式（正式配置） | `cd experimental/ace; cargo nextest run -p ace-aac-support-conformance --config-file ../../.config/nextest-full.toml`（`AUTO_PLAN_ARTIFACTS=…/tmp`） | 整命令 **24.11 s**（编译 1.72 s，运行 21.64 s） | 25 passed / 0 skipped；逐例最大 **6.610 s**（同用例）；exit 0 |
| 只读探针（N-7） | 计划材料 `…/review3/probe`，`CARGO_TARGET_DIR=…/target-ace`，依赖树内 lib（`run_program` 走同一 Gen0+AV 车道） | 38.22 s（含构建） | `declared-field 00 / unknown-field 09 / container-as-field 00 / container-as-field2 00 / container-as-child 00` |
| 静态核对 | `git show/diff c2d2a1179..ab8809b00`、`atom_data.at`/`codec.at`/`driver.at` 全文、`descriptor.rs:{446-449,815-833,997}`、`core-i32.atom`、SD-01 §8、两测试文件、`nextest show-config test-groups` | 未计时 | 见下 |

**修复项关闭核对（F5b/N-2/N-3/F4b/N-4/N-5/N-6）**

| ID | 判定 | 证据（@`ab8809b00`，本次复跑） |
|---|---|---|
| F5b | **closed** | 编码器排序后 `identical_pair`（`codec.at:420-427,492-517`）与解码器 `check_pair_order`（`:691-714`）同规则返 12，`[(a,1),(a,2)]` 升序仍合法；`v03_manifest_taxonomy` 期望 `0c 0d 07 0a 0b 0e 0f 0c 00 00`（既有 7 断言不减弱），组跑 25/25 通过。覆盖边界：相同 **edge** 的编码侧 12 无单测（与 pair 共用 `identical_pair`，代码对称） |
| N-2 | **closed** | `build_canonical_with_schema` 在 `schema_doc_check` 后补跑全 arena `dup_all`（`atom_data.at:1841-1851`）；`atom_real_schema_duplicate_field_is_8`=`08000808`（带 tag 8 / 合法 0 / 内联 8 / 无 schema 同文档 8）通过；SD-01 §8 同步 |
| N-3 | **closed** | `slot_field`/`field_add_lit`（`atom_data.at:1268-1292`）按 `descriptor.rs:815-822` 映射（首 Id→`id`、TypeUse/TypeRef→`type`、其余→`kind`；Id 非首位不接纳）；`atom_real_schema_slot_field_mapping`=`00090000`（`module(id:"m")`=0、`module(Id:"m")`=9、`type(id,kind)`=0、`local(id,type)`=0）；SD-01 §8 映射表与代码一致 |
| F4b | **closed（判为诚实记录，非收窄判据）** | 深度边界实测：63 带字段=0、64 带字段=6、64 无字段=0、65 无字段=6（`atom_admission_depth`=`00 06 00 06`），与 `parse_node`/`parse_value` 的 `depth>64` 及"值在宿主 +1"一致；长度：2048→0、262145→6（`atom_admission_rules`=`0006`）、65537→`av.resource.list-elems`（`atom_admission_envelope`）。代码准入规则仍为 262144（未被改写）；SD-01 §8 明写"执行器包络/单表 65536"。判读：用实测包络替换不可达的 262144 邻属**如实记录**；覆盖边界见下 |
| N-4 | **closed** | `.config/nextest-full.toml:47` `ace-aac = { max-threads = 4 }` ＋ `:161-163` `package(ace-aac-support-conformance)` override（文件中最后一个 override；`show-config test-groups --config-file` 显示该组含全部 25 例）；固定组实测 `max_parallel_tests_observed=4`、逐例最大 9.47 s；组模式逐例最大 6.61 s；**两轮均为 busy 口径**仍全部 ≤10 s（work 的空闲窗口收据 max 8.05 s 与之不矛盾） |
| N-5 | **closed** | §6 V-06（真实 schema 驱动／真实 HIR 接受＋幂等／本包布局字节恒等／reference writer 具名交接／内联重复 8）、§7 AC-01/AC-03、§8 T-04 均按裁定 A 重写；`plan_revision` 6；与 SD-01 §8/§8.2 一致 |
| N-6 | **closed** | §7 计数 25；§9"收据在文件末尾"说明＋§7 交叉引用；§10 T-03 向量改 1/64/65/128＋2049；`conformance.rs` 注释改 2049 三 chunk、`atom_data.at` 的 "nesting capped at 64"/"before any node is built" 两处不实注释已删；"最终源身份"9 项 sha16/字节与 git blob 一致（本次复算） |

**验收证据图（AC-ID → 判定 → 命令/方法 → 结果，@`ab8809b00`）**

| ID | 判定 | 命令/方法 | 结果 |
|---|---|---|---|
| AC-01 | **pass** | 本次 `cargo t ace-aac`（25/25）＋组模式 nextest（25/25）；V-06 用例 `atom_real_*` | V-01/V-02/V-06 观测值/字节均达；V-06 按裁定 A 的四项可观察（真实 schema 驱动、真实 HIR 接受＋幂等、本包布局、重复/未知/根码），reference writer 逐构造字节具名 1047/1053 |
| AC-02 | **pass** | V-02/03/04 结果码序列＋边界用例 | frame/manifest/admission 分类成立；V-04 长度与深度合法近邻在册（2048／63／64 无字段），F5b 编码/解码一致；包络如实记录 |
| AC-03 | **partial** | 读码＋只读探针（N-7） | 无公共 std 新增、无第二份 hash、真实 schema 与真实 HIR 接受（✓）；slots 映射与真实 descriptor 一致（✓）、内联重复一致（✓）；但 schema 表把 **container 名**（`types`/`declarations`/`bodies`/`locals`/…）同时登记为父节点允许字段，真实 binder 只把它们当容器子 tag（`descriptor.rs:446-449`、`:815-833`），故"字段集合与真实 descriptor 一致"不成立（N-7） |
| AC-04 | **partial** | 逐 V 行＋本轮计时/计数/身份 | 计数、收据位置、当前源 sha16、旧向量/注释均已校正；但 AC-03 的 schema 保真缺口未闭（N-7），§7"AC-01..AC-04 由 V-01..V-06 关闭"的表述仍不成立 |
| AC-05 | **fail（本次复审）** | 本记录 | needs_fix |
| AC-AC-STAGED | **pass（沿用）** | SD-01 §9.1 未改；冻结 std 零改动 | 阶段合同文本与 §6 行/§7 勾选一致；本包无未交付 AC 依赖 |
| AC-06 | 未评估 | — | merge-owned，保持未勾 |

**findings（稳定 ID，含最小重放）**

- **N-7 [medium｜AC-03/V-06/SD-01｜F1-A 遗留，被本轮 AC-03/§7 文本显式化]** schema 模式把 **container 名**当父节点合法字段：`add_group_fields(..., as_tags=1)` 对 containers 条目同时 `field_add`（`atom_data.at:1182-1210`，由 `declare_node:1338-1343` 调用）；真实 descriptor 把 containers 存进 `spec.containers`（`descriptor.rs:446-449`），具名字段只对 `spec.fields` ＋槽映射校验（`:815-833`），故 `module(types: …)` 对真实 binder 是 `bind.unknown-field`。最小重放（本会话实测，材料 `D:/autostack/plan-artifacts/plan-1056/review3/probe`）：`canonicalize_with_schema(<core-i32.atom>, "hir(schema: \"s\") { module(types: 1) {} }\n")` → **0**、`module(bodies: 1)` → **0**；同批控制项 `module(path: "p")`=0、`module(bogus: 1)`=9、`module { types { type(id:"t",kind:i32) {} } }`=0。期望：container 名仅作子 tag（第三条保持 0），作具名字段按真实 descriptor 返 9。修法二选一并同步 SD-01 §8 第 166-168 行"允许字段名 = … + containers 的字段名"：(a) containers 只注册 tag、不注册父字段，补 `module(types: …)`=9 负例；(b) 若确为有意放宽，须显式写明该 over-admission 与 `bind.unknown-field` 的差异并取得合同认可——不得由复审单方面收窄 AC-03。
- **N-8 [low｜记录/配置注释，本轮新增]** `.config/nextest-full.toml:154-158` 注释称 "any ad-hoc `cargo nextest run -p ace-aac-support-conformance` all get the same limit"；实测 `cd experimental/ace; cargo nextest show-config test-groups`（默认发现）不加载任何 group 配置（无 group 节），只有 `--config-file ../../.config/nextest-full.toml` 才把该 package 绑到 `ace-aac` 组（同命令列出 25 例）。故无 `--config-file` 的 ad-hoc 跑法**不受** 4 路限流。建议改写注释或注明必须以 fixed-group/显式配置文件运行；不影响固定组/组模式门禁。

**覆盖边界（登记备查，不阻断）**：(1) F4b 的长度合法侧钉在 2048 标量，未在 65536 包络上沿取合法邻；SD-01"65536 标量还会撞槽预算"来自 work 记账而非本组用例（本组只钉 65537→`ListElems`）；(2) 编码器对相同 **edge** 的 12 无单测（与 pair 共用实现）；(3) driver `--all`（25 例交互入口）未复跑。

evidence: 固定组 `D:/autostack/.wt/lang-1056/auto-lang/target/test-groups/run-bl8_gpg3/results.json`（executed=25、gate_passed=true、wall_seconds=36.85、max_parallel_tests_observed=4、source_state=3b6d5fec…、config_sha256=237ee400…）；组模式命令输出（25 passed、21.64 s、逐例最大 6.610 s）；探针 `D:/autostack/plan-artifacts/plan-1056/review3/probe/{Cargo.toml,src/main.rs}` 及运行输出（scratch，merge 前折入本文件并随 scope 删除）；关键 diff `git diff c2d2a1179..ab8809b00`；代码位点 `atom_data.at:{1182-1210,1338-1343,1841-1851,1268-1292}`、`codec.at:{420-427,492-517,691-714}`、`descriptor.rs:{446-449,815-833,997}`、`core-i32.atom:{30-80}`；SD-01 sha16 `33f682310f27edb6`。未写 `.autoos/specs.json` 等 legacy ledger；未做实现编辑、未跑 git 写命令、未发布 canonical Spec。

pending_closeout_gates: AC-06（merge：沉淀 SD-01/SD-02、归档、材料清理、wt-guard clean）；N-7/N-8 修复后须在冻结提交重跑 `ace-aac`（组模式逐例 ≤10 s 证据）；周期 `cargo t all` 属主检出单实例批回归，不作为本 Plan 门。受影响任务 T-04（schema 表/containers 规则）、T-06（负例/证据/SD-01 文本）原本即未勾，本轮保持未勾，`current_step` 按已完成数保持 3，`status` 保持 `executing`；本轮只追加本文件内容，未动其他路径。

**next**: work —— T-04 修 N-7（containers 名只作子 tag，或显式合同化 over-admission 并同步 AC-03/SD-01 §8）、T-06 补负例与 SD-01 文本、改 N-8 注释；新提交后回 review，只复验 AC-03/AC-04、N-7/N-8 及不受影响的 F5b/N-2/N-3/F4b/N-4/N-5/N-6 证据（沿用本记录），不勾 AC-06、不合并。

## 10. 待澄清事项

无须本轮额外用户决定。具体实现/等价数据结构由work依据冻结合同选；若源/能力/预算/接口不成立，先回具名owner或有界重规划，不自行豁免义务。本轮先不施工，所有计划前置仍待正式验证。

2026-11-19 stage:work / r4（实施归属更新后首轮施工）：新增 **AAC 支撑包**
`aac-support` 1.0.0（纯 Auto 单源 `experimental/ace/aac/support/{digest,codec,atom_data,driver}.at`，
经装配 `packages-aggregates-ct` 车道由 Gen0 Rust 前端编译、算法在 AV 执行器真实执行），
配套新固定组 `ace-aac`（`-p ace-aac-support-conformance`；当前 16 例）与真实入口
`aac-support-driver --list|--case|--all`。commit `cb69e84e0`（树
`D:/autostack/.wt/lang-1056/auto-lang`，branch `plan-1056-dev`）。

**T-01 基线**：核 Design59/§4 快照与真实源/语法/库身份（`identity.rs` b1ca0253…、`codec.rs`
490843a5…、`a2x-core/sha256.rs` 0c14f616…、grammar 9e5248b2…、`stdlib.rs` f7edbd03…、
`stdlib/ace/asm/bytes.at` 3605d0d9…、Design59 96e06fb6…）；材料唯一目录
`D:/autostack/plan-artifacts/plan-1056/`（owner/用途/在途已登记），唯一树
`D:/autostack/.wt/lang-1056/auto-lang`，无 junction/symlink。冻结 1044 资源
（64MiB front / 1<<20 depth / 1e6 semantic steps / 5s semantic wall / 10s CT root）不变。

**T-02 决策**：本包**不进公共 std**——BLAKE3/SHA-256 目前只有 AAC 一个真实消费者，
发布为公共 std 操作会立刻产生 AV/mn/A2R/A2C/A2PY 五端与 `stdlib/contracts` 义务而无公共
消费者；Design56 覆盖合同只在"新增公共操作"时适用，故本包具名为 AAC 支撑包，冻结 std
切片（`stdlib/ace/std/*`、`stdlib/ace/asm/*`）字节身份与 `is_frozen_std_*` 判定均未改动。
升级路径（上移 std 单源 + 五端覆盖）写入 SD-01 profile §9。因此 V-05 的"全部覆盖腿"义务
**不触发**（该行条件为"若新增公共 hash 操作"）；实际消费者腿 = AV 执行器（Gen0 编译 +
Auto 执行）与进程内独立参考。

**T-03 摘要**：SHA-256（FIPS 180-4，一次性 + 真实增量路径 `sha256_hex_parts`）与 BLAKE3-256
（非密钥，块/块界/多块/树路径）在 Auto 内实现；32 位字在受检 i64 中计算、每步显式模 2^32、
以二进制补码存 `List<int>`。空/嵌 NUL/多字节 UTF-8/55..127 与 BLAKE3 的 **1/64/65/128 +
2049（三 chunk，触发树合并）** 字节全部与进程内独立 `sha2`、`blake3` 逐字节一致（`v01_*`）。
**命名边界**：BLAKE3 无增量/流式 API
（AAC 当前消费者一次性持有字节；内部分块由边界向量对参考验证）。

**T-04 codec**：`aac.frame/1` 与 `ace_artifacts::codec::frame_chunk` **逐字节一致**
（5 payload × 11 kind = 55 帧，`v03_frame_bytes_match_reference`）；解码分类
`0000000102030405060708`（ok/截头/magic/版本/kind/超 cap/截 payload/尾随/摘要不符），
全部在分配前先按实际剩余字节判定。`aac.manifest/1` 为确定性 DTO：编码器排序、解码器拒绝
非规范顺序/重复键（`0c0d070a0b0e0f`），`i64::MIN/MAX` anchor 8 字节原样搬运并有独立
Rust 编码器 oracle。`atom_data` 为纯数据 reader/writer（冻结 tag/字段表 + 重复字段拒绝），
语义拒绝/Checked 构造明确留给 1047/1053，绝不在此生成 `Checked`。

**T-05 交接**：driver `--all` 25 例 exit 0（sha256/blake3/identity/manifest/frame/admission
六族），版本轴 `aac.frame/1`、`aac.manifest/1`、`blake3-1.5` 分帧，交 1045/1051/1052 消费。
原 Rust 宿主工具依赖如实列出：`sha2`/`blake3`（仅测试 oracle）、`ace_artifacts`（参考
实现），生产路径无 Rust 摘要/编码算法。

**T-06 验证**：`cargo t ace-aac` → `PASS: ace-aac` exit 0（75.4s，含构建）；
`cargo nextest run -p ace-aac-support-conformance` → 当时 15/15 pass（该版按向量逐编译，
单例最慢 49.3s，已按 test-cost 门整改为每族一编译：见下方 work 追加）。资源实测：AV 默认 1e6 指令步对 1 KiB 纯 Auto BLAKE3
不足，driver 以显式 `AV_MAX_STEPS=64e6`/wall 300s 运行（执行器档位，不改 1044 合同）；
递归累加器在 AV 槽计费下呈二次（实测两处触顶）已改迭代式。`atom_data` 的 depth>64 准入
规则已实现但未单测（harness 64M 步预算先于模块阈值生效，如实登记）。SD-01 已落
`docs/specs/auto-ac/aac-digest-codec-profile.md`（新增 profile；SD-02 的
`ace-core-and-text.md` 修改留 merge 按 review 结论落）。

**work 追加（test-cost 整改，commit `726f8c64c`）**：首版"每向量一编译"使单例 25–50s 墙钟，
违反复审的 >10s 单例门。已改为**每证据族一编译**（`digest_report`/`blake3_tree_report`/
`split_report`/`identity_report`/`manifest_report` 一次返回拼接 hex 报告，测试按参考长度切分
比对）：包内 14 例全绿、总墙钟 **7.4s**、单例最大 **7.4s**（`v03_frame_bytes_match_reference`），
`cargo t ace-aac` → `PASS: ace-aac` **11.0s**；driver `--all` 25 例 exit 0（80.6s，25 次编译，
属交互入口不属固定组）。BLAKE3 树路径（1025 字节 = 两 chunk + parent 合并）单独一例，
避免把纯 Auto BLAKE3（~1M AV 步/KiB）堆进同一用例。
stage: goal | plan_id: PLAN-1056 | plan_revision: r4 | round: 1 | from: drafting | to: execution_done | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1056/auto-lang | commit: cb69e84e0 | next: review

2026-11-19 stage:work / 复审整改第 1 轮（commit `f113768ce`）：已修 **F5**（manifest 规范顺序改为
(key,value) 升序，"重复"= 完全相同的 pair，编码器不再可能产出自身解码器拒绝的字节）、
**F2**（BLAKE3 树路径改为 2049 字节 = 三 chunk，真正执行 `b3_merge` 偶数合并分支）、
**F4**（atom 准入：262145 标量拒 6、深度 65 拒 6；深度 64 合法邻居因写入器随深度超线性、
代价过高未单测，具名登记）、**F8**（删除未用测试辅助，包内零 warning）；`ace-aac` 组加
`-j 4` 限制并行，使单例墙钟反映真实成本。包内 16 例全绿；单例实测成本 1.8–8.0s（隔离测量），
机器被其他会话占满时 nextest 报告的并行墙钟会膨胀到 12–18s（诊断口径，非用例成本）。

**未关闭**：**F1（blocker）** atom_data 的 tag/字段表仍是本包自定义方言，不能按真实
canonical Atom/HIR 文本的**逐字节布局**roundtrip（真实 writer 的 per-construct 字段位置与
`local … { name: …; role: … }` 这类内联决策由 `descriptor.rs` 逐构造码决定）；F3（SD-01 夸大
验证面/交叉引用错误）、F6（SD-AC-STAGED 载体文本缺失）、F7（计数/收据位置/最终源 sha16/
frontmatter supersedes 表）。

## 10.1 需要用户裁定（阻塞 F1 的范围问题）

V-06/T-04 要求的"当前 AC canonical Atom/HIR 文本"逐字节 roundtrip，与"本包只做纯数据
codec、语义由 1047/1053"存在范围冲突：真实 `descriptor::write_canonical` 的布局是
**逐构造硬编码**（字段在括号组还是花括号组、何时内联 `; `、空体 `{}`），要逐字节复现等于把
writer 移植进 Auto。请裁定其一：

- **A（推荐，范围收紧 + 具名交接）**：atom_data 改为"通用 Atom 数据 + 真实 schema 文档驱动
  校验"，验收改为：真实 schema（`crates/core/schema/core-i32.atom`）可被读取并作为 tag/字段
  表；真实 HIR/descriptor 文本**被接受**且 canonicalize 幂等（canonicalize(canonicalize(x)) ==
  canonicalize(x)）；本包自身规范布局的 roundtrip 字节恒等；真实 writer 逐字节布局移植具名
  交接给 1047/1053。需同步修改 §6 V-06、§7 AC-01/AC-03 与本计划 revision。
- **B**：把真实 `descriptor::write_canonical` 布局移植进 Auto（逐构造规则 + 内联决策），
  需要更多预算（预计显著大于本包其余工作），其余验收不变。

在裁定前不继续 atom_data 的施工，以免返工；F3/F6/F7 属文本订正，可与裁定同轮完成。

stage: goal | plan_id: PLAN-1056 | plan_revision: r4 | round: 1 | from: execution_done | to: executing | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1056/auto-lang | commit: f113768ce | next: user

**最终源身份（当前树，复审整改第 3 轮后的冻结值；SHA256 前 16 位/字节，git blob 口径）**：
`atom_data.at` 8e2f6b4571ae9d3f/64452、`codec.at` b942bf46f7f94618/22873、`driver.at`
ac2765a4011b6646/18770、`digest.at` b96254e31736993b/28271（本轮未改）、SD-01
`aac-digest-codec-profile.md` 33f682310f27edb6/18256、`atom_data.rs` ebe37180c19b86f8/35068、
`conformance.rs` e838aa69c83ae9d5/12405、`.config/nextest-full.toml` 1133937f662a1bc8/10040、
`scripts/test-groups.toml` 07aeefcbffdb102b/61515（本轮未改）。本节取代 `f113768ce` 快照
（旧值见上方复审记录）；说明：work/goal/复审收据按追加顺序位于**本文件末尾（§10 之后）**。

**更新（2026-11-19 最终复审 @`1d0c86549`）**：上段 4 项已被取代——`atom_data.at`
`f86c444309282fce`/64755、SD-01 `7259caf1e31a54c2`/18619、`atom_data.rs`
`89c4c06fe80df8e2`/37455、`.config/nextest-full.toml` `d0d0d2605f5376a9`/10311；其余 5 项
（`codec.at`/`driver.at`/`digest.at`/`conformance.rs`/`scripts/test-groups.toml`）不变。
权威清单位于本文件末尾 §9 复审记录的本轮 `spec_inputs` 与"实现身份"段。

2026-11-19 stage:work / 复审整改第 2 轮（文本订正，无代码变更）：关闭 **F3**（SD-01 去掉
"`ace_a2x_core::sha256` 已被验"的夸大、修正 BLAKE3 向量清单为 1/64/65/128 + 2049 三 chunk、
`i64_le_bytes` 交叉引用改为 `codec.push_i64le`）、**F5**（SD-01 §6 规范顺序改写为
(key,value) 升序、重复=完全相同 pair）、**F6**（新增 SD-01 §9.1"阶段合同"作为
SD-AC-STAGED 的规范载体：单向消费、具名 AC 差额、CL 不删改、阶段通过不等于完整签章）、
**F7**（计数 15→16、`supersedes_spec_components` 去掉未变更的 `artifact-contract.md`、
最终源 sha16 与本文件收据位置说明已记入 §9）。**F1（blocker）仍待用户裁定**（§10.1），
未继续 atom_data 施工。

stage: goal | plan_id: PLAN-1056 | plan_revision: r4 | round: 2 | from: waiting | to: waiting | status: executing | outcome: waiting | worktree: D:/autostack/.wt/lang-1056/auto-lang | commit: f113768ce | next: user

2026-11-19 stage:goal / 第 3 个连续等待轮：§10.1 的 F1 范围裁定仍无用户答复；本计划**未继续**
atom_data 施工（避免在 A/B 之间的返工），未运行新门禁。已完成并提交的部分停在
commit `993795326`（SD-01 F3/F5/F6 订正）与 `f113768ce`（F2/F4/F5/F8 代码整改），
固定组 `ace-aac` 16 例全绿、driver `--all` 25 例 exit 0。**解除阻塞只需一句裁定**：
`A`（推荐：通用 Atom 数据 + 真实 schema 文档驱动校验，真实 HIR 文本"接受 + 幂等归一"，
reference writer 逐字节布局具名交接 1047/1053）或 `B`（把 `descriptor::write_canonical`
的逐构造布局移植进 Auto）。答复后可直接续跑：F1 施工 → 定向门禁 → 独立复审 → merge。

stage: goal | plan_id: PLAN-1056 | plan_revision: r4 | round: 3 | from: waiting | to: blocked | status: executing | outcome: blocked | worktree: D:/autostack/.wt/lang-1056/auto-lang | commit: 993795326 | next: user

2026-11-19 用户裁定：**选 A**（通用 Atom 数据 + 真实 schema 文档驱动校验；真实 HIR/descriptor
文本"接受 + 幂等归一"；本包自身规范布局字节恒等；reference writer 逐字节布局具名交接
1047/1053）。据此修订 V-06 与 AC-01/AC-03 的可观察口径，并按 A 施工。

stage: goal | plan_id: PLAN-1056 | plan_revision: r5 | round: 4 | from: blocked | to: work | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1056/auto-lang | commit: 993795326 | next: work

2026-11-19 stage:work / F1-A 施工完成（commits `421f2629c` + `c2d2a1179`）：atom_data 现为
**双模式共享同一 writer**——`canonicalize`（无 schema 白名单，仍拒重复字段 8）与
`canonicalize_with_schema(schema_text, doc_text)`（解析**真实** `crates/core/schema/core-i32.atom`
得到 tag/字段表；根须 `hir` 否则 10、未声明 tag 10、未声明字段 9、重复字段 8）。新增
`canonicalize_bytes` / `canonicalize_bytes_with_schema` 使幂等可在 Auto 内表达。writer 组内
统一 `", "` 分隔、字段全部提升到组内、body 仅子节点、空体 `{}`、4 空格缩进 + LF；同时修掉
"字段既在组内又在体内重复发射"的旧 bug（幂等因此成立）。**真实 fixture**
`crates/core/fixtures/native/add-2-3.atom` + **真实 schema** 均被接受且双模式字节定点，SD-01
新增 §8.1 布局/§8.2 规范化合同。测试：包内 **23/23 pass**（atom 族 14 例，最慢 8.7s）；
固定组 `cargo t ace-aac` → **PASS**（39.6s 含构建；先前的 `-j 4` 组参数与 fixed-tests 的 -j
冲突，已移除，commit `c2d2a1179`）。**具名交接 1047/1053**：`statements:`/`inline:` 属 tag
声明非字段；containers 成员关系、branches require/forbid、inline-tag 位置**不在本包校验**；
输出不复制 reference writer 的逐构造字节（用户裁定 A）。

stage: goal | plan_id: PLAN-1056 | plan_revision: r5 | round: 4 | from: work | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1056/auto-lang | commit: c2d2a1179 | next: review

2026-11-19 stage:work / 复审整改第 3 轮（F5b/N-2/N-3/F4b/N-4/N-5/N-6；本轮提交
`fix(aac-support): F5b/N-2/N-3/F4b/N-4 repairs; plan sync for option A (Plan 1056)`）：

- **F5b（编码器/解码器一致）**：`manifest_encode` 排序后用新 `identical_pair` 按**与解码器完全
  相同的规则**拒绝**完全相同**的 pair/edge（12）；`[(a,1),(a,2)]` 仍合法规范序。`driver.at` 的
  `manifest_codes` 增加 3 个观测（重复 pair 编码 → 12、同键升序编码 → 0、该字节解码 → 0），
  `v03_manifest_taxonomy` 期望扩为 `0c 0d 07 0a 0b 0e 0f 0c 00 00`（既有 7 个断言不减弱）。
- **N-2（schema 模式内联重复）**：`build_canonical_with_schema` 在 `schema_doc_check` 之后补跑
  无 schema 模式的 `dup_all`（全 arena，包含内联节点），内联重复 = 8。用例
  `hir(schema: "s", requires: {k: 1, k: 2})`：schema 模式 8、无 schema 模式也 8（`08000808`）。
- **N-3（槽位→字段拼写）**：schema 表不再注册 `slots:` 标签字面量，改按
  `descriptor.rs:815-822` 映射：首位 `Id`→`id`、`TypeUse`/`TypeRef`→`type`、其他枚举→`kind`
  （新增 `field_register`/`field_add_lit`/`slot_field`，删除无用的 `field_add_tok`）。新用例
  `atom_real_schema_slot_field_mapping`：`module(id:"m"){path:"p"}`=0、`module(Id:"m")`=9、
  `type(id,kind)`=0、`local(id,type)`=0（`00090000`）。SD-01 §8 同步改写并加入映射表。
- **F4b（准入边界）**：长度上沿 262145 标量 → 6（在词法前）；**实测发现 262144 的合法近邻在本
  执行器不可达**——`text_bytes` 把整段文本转成**一个** `List<int>`，冻结的 collections 合同单表
  上限 65536 元素（`av.resource.list-elems`；65537 标量被执行器拒绝，另立用例
  `atom_admission_envelope` 钉住），65536 标量还会撞槽预算并受 O(n²/512) 转换成本拖累。故用例改
  为"2048 标量 → 0 / 262145 → 6"（`atom_admission_rules`），并把这组包络数字写进 SD-01 §8，不再
  用"写入器超线性"豁免。深度上沿按代码实测钉住：**64 个嵌套节点合法、65 拒绝**；字段值位于宿主
  深度 +1，所以 64 层链的叶子带字段值时由**值路径**拒绝——`atom_admission_depth` 四例
  （63 带字段=0、64 带字段=6、64 无字段=0、65 无字段=6），并删掉"nesting capped at 64"/
  "before any node is built" 两处不实注释。
- **N-4（组级限流）**：`.config/nextest-full.toml` 新增 `[test-groups] ace-aac = { max-threads = 4 }`
  + `package(ace-aac-support-conformance)` override（沿用 mem-xl/lg/md 先例，不用与 fixed-tests
  冲突的 `-j`）。组限流去掉的是**本组自身**的 20 路争用（此前 20/23 例 >10 s 的成因）；外部计划
  构建（8-22 个并发 cargo/rustc/cl）仍会抬高每例墙钟，且与组大小无关：同负载 A/B 下 4 路
  max 12.9-15.2 s、2 路 max 17.4-20.3 s（另有 2 路 9.65 s 的窗口），故取 4 路（总墙钟更短），
  空闲窗口的正式收据为 25/25、总 26.07 s、**逐例最大 8.05 s**。证据：`cargo t ace-aac` 的
  `results.json` `executed=25`、`gate_passed=true`、`max_parallel_tests_observed=4`。
- **N-5（计划同步裁定 A）**：重写 §6 V-06、§7 AC-01/AC-03、§8 T-04 的可观察口径（真实 schema
  驱动 tag/字段表、真实 HIR 接受 + `canonicalize(canonicalize(x))==canonicalize(x)` 幂等、本包
  布局字节恒等、reference writer 逐构造字节具名交接 1047/1053）；`plan_revision` 5→6。
- **N-6（记录/注释）**：§7 计数 16→25、§9 增加"收据在文件末尾"的位置说明并修 §7 交叉引用、
  上段刷新最终源身份 sha16、§10 T-03 向量改为 1/64/65/128 + 2049、`conformance.rs` 的
  "1025 两 chunk" 注释改为 2049 三 chunk。

**本轮门（全绿，同树 `D:/autostack/.wt/lang-1056/auto-lang`，`experimental/ace`）**：

| 门 | 命令 | 结果 |
|---|---|---|
| 组模式（正式配置 4 路，空闲窗口） | `cargo nextest run -p ace-aac-support-conformance --config-file ../../.config/nextest-full.toml`（`CARGO_TARGET_DIR=D:/autostack/plan-artifacts/plan-1056/target-ace`、`AUTO_PLAN_ARTIFACTS=…/tmp`） | **25/25 pass**、0 skipped、总 26.07 s、**逐例最大 8.05 s**（`atom_real_schema_slot_field_mapping`），全部 ≤10 s |
| 组模式（同配置，外部计划构建 8-22 个并发进程时） | 同命令（含 A/B：临时把组限流改 2） | 4 路逐例最大 12.9-15.2 s、总 39-56 s；2 路最大 17.4-20.3 s（另有 9.65 s 窗口）、总 64-92 s。超限来自**外部**争用而非本包：空闲窗口与串行口径都 ≤9.3 s，故配置保持 4 路 |
| 串行口径 | 同命令 `-j 1` | 25/25 pass、总 110.96 s、逐例最大 9.27 s（例内成本口径） |
| 固定组 | `cargo t ace-aac`（无 `CARGO_TARGET_DIR`） | `PASS: ace-aac` exit 0；`executed=25`、`gate_passed=true`、`max_parallel_tests_observed=4`、`wall_seconds=43.83`（`target/test-groups/run-71z0db50/results.json`）；冻结提交另跑一次 `executed=25`、`gate_passed=true`、`max_parallel_tests_observed=4` |
| driver | `cargo run -p ace-aac-support-conformance --bin aac-support-driver -- --all` | 25 例 exit 0、183.9 s（会话内机器被其他计划占满，按 busy 记录）；`manifest:taxonomy = 0c0d070a0b0e0f0c0000` |
| 健康检查 | `cargo check -p ace-aac-support-conformance --lib --tests` | 本 crate 0 warning（其余 warning 均来自既有 `ace-av-core`/`ace-core`/`ace-lowering`/`ace-av`） |

**新登记的支持边界（已写入 SD-01 §8）**：`atom_data` 的 262144 标量准入规则**宽于**本执行器可
物化包络（单表 65536 元素；文本转换 O(n²/512)）。因此在 AV 上只能验该规则的**拒绝侧**与 KiB 级
合法近邻；包络补齐（分块词法/流式 writer）是后继增量，不是本轮收窄规则。另：上一轮"深度 64 合法
邻居因写入器超线性、代价过高未单测"的判断被本轮实测否定（64 层文档输出约 8 KiB、组模式用例 ≤10 s）。

stage: goal | plan_id: PLAN-1056 | plan_revision: r6 | round: 5 | from: review(needs_fix) | to: review | status: executing | outcome: pass | worktree: D:/autostack/.wt/lang-1056/auto-lang | commit: 本轮整改提交 | next: review

**next**: review —— 在冻结提交重验 AC-01..AC-04、AC-AC-STAGED 与 F1..F8/N-2..N-6；核对组模式逐例
≤10 s 收据、`max_parallel_tests_observed=4`、SD-01 §8 的槽位映射/内联重复/包络文本与代码一致；
不勾 AC-06、不合并。

**merge 提示（簿记）**：本轮把本计划文件随整改提交一并入库（此前它在主检出是未跟踪状态，
`?? docs/plans/1056-…`）。因此在 `plan-1056-dev` 合入主检出的 merge 步骤，须先处理主检出里同名的
未跟踪副本（内容与本提交一致：`git add` 后先提交簿记，或移开该副本），否则 `git merge` 会以
"untracked working tree file would be overwritten" 拒绝。

2026-11-19 stage: review | review_scope: final（受影响范围复核 = N-7/N-8 修复 + AC-03/AC-04 + AC-05 判定；未重放全目录、未重放不受影响判据） | reviewer_identity: `finish-1056`（deepseek/deepseek-v4.1-flash；lead 委派的完成代理）。limitations: **本代理既是本轮 N-7/N-8 修复者又是本次复核者，非独立上下文**——判定按冻结提交上的可复现产物重建（固定组/组模式/健康检查/driver 实跑 + 逐例墙钟 + 读码对 `descriptor.rs`），不采信实现叙述。| plan_id: PLAN-1056 | plan_revision: 6 | outcome: **pass** | worktree_path: `D:/autostack/.wt/lang-1056/auto-lang` | branch: `plan-1056-dev` | reviewed_commit: `1d0c8654984d2040faa0552572d09c8c1c107dc4` | base_commit: `ab8809b00cf9cef5d513e08c48d726c71900dcf9`（上次复审基线） | dependency_revisions: `artifacts/src/identity.rs` b1ca0253dc944859、`artifacts/src/codec.rs` 490843a5e92386cf、`core/src/descriptor.rs` 041852886f22bf1f、`core/src/atom_text.rs` 0fc9b5ceed7357e3、`a2x-core/src/sha256.rs` 0c14f616df68c7c1、`av-core/src/contract.rs` 2e2f9e1ac36b106b（`MAX_LIST_ELEMS = 65536`）、`HASH_ALGO="blake3-1.5"`；上述依赖本轮零改动。

**计划副本与基线**：主检出未跟踪副本（`EA23BBCBBA4858F9…`，90106 B，含上次 `reviewer-1056c` 记录，worktree/分支副本当时缺其 53 行）已并入分支副本，本记录随其后追加——落主副本将同时含 `reviewer-1056`/`reviewer-1056b`/`reviewer-1056c` 与本记录，无 §9 收据丢失。本轮修复提交 `1d0c86549` 只触 4 文件（`git diff --stat ab8809b00..1d0c86549`）：`atom_data.at` +11/−7、`atom_data.rs` +49、`.config/nextest-full.toml` +7/−3、SD-01 +6/−4；无其它路径，无 `.autoos/specs.json` 类 legacy ledger 写入，无公共 std 改动。

spec_inputs: `docs/specs/auto-ac/aac-digest-codec-profile.md` @`1d0c86549`（SD-01；sha16 `7259caf1e31a54c2`，18619 B，git blob；§8 的 containers 规则已与代码同步）；`docs/specs/auto-ac/artifact-contract.md` 未改（不在 `supersedes`）；`docs/specs/stdlib/design/ace-core-and-text.md` 未改（SD-02 留 merge）。
实现身份 @`1d0c86549`（git blob 口径）：`atom_data.at` f86c444309282fce/64755、`codec.at` b942bf46f7f94618/22873（未改）、`digest.at` b96254e31736993b/28271（未改）、`driver.at` ac2765a4011b6646/18770（未改）、`atom_data.rs` 89c4c06fe80df8e2/37455、`conformance.rs` e838aa69c83ae9d5/12405（未改）、`.config/nextest-full.toml` d0d0d2605f5376a9/10311、`scripts/test-groups.toml` 07aeefcbffdb102b/61515（未改）。

**本会话模块/范围计时（空闲窗口口径；无 cargo/rustc/nextest/cl/link 并发进程）**

| 阶段 | 命令 | 耗时 | 结果 |
|---|---|---|---|
| 组模式（正式配置 4 路） | `cd experimental/ace; cargo nextest run -p ace-aac-support-conformance --config-file ../../.config/nextest-full.toml`（`CARGO_TARGET_DIR=D:/autostack/plan-artifacts/plan-1056/target-ace`、`AUTO_PLAN_ARTIFACTS=…/tmp`） | 整命令 **28.08 s**（编译 1.81 s、运行 25.40 s） | **26 passed / 0 skipped**；逐例最大 **8.269 s**（新用例），26/26 ≤10 s |
| 固定组 | `cd <wt root>; cargo t ace-aac`（无 `CARGO_TARGET_DIR`） | 整命令 **27.67 s**（discovery 2.31 s、execution 24.75 s） | `PASS: ace-aac` exit 0；`results.json`：executed=26、gate_passed=true、max_parallel_tests_observed=4、wall_seconds=27.08、source_state=bff9fd86…、config_sha256=237ee400…；逐例最大 **8.0407 s**，26/26 ≤10 s |
| N-8 配置绑定 | `cargo nextest show-config test-groups`（默认发现）／同命令加 `--config-file ../../.config/nextest-full.toml` | <1 s | 默认：**0 个 `group:` 行**；加配置：`group: ace-aac (max threads = 4)` |
| 交互入口（AC-04） | `cargo run -q -p ace-aac-support-conformance --bin aac-support-driver -- --all` | **176.52 s**（25 次 Gen0 编译，串行） | 25 例全部有输出、exit 0；`manifest:taxonomy = 0c0d070a0b0e0f0c0000`（与上次一致） |
| 健康检查 | `cargo check -p ace-aac-support-conformance --lib --tests`（强制重编本 crate） | 0.46 s（依赖已编译） | exit 0；新 crate **0 warning**（其余 11 条全来自既有 `ace-av-core`2/`ace-core`3/`ace-lowering`1/`ace-av`5） |
| 静态核对 | `atom_data.at:{1185-1214}`、`atom_data.rs:{769-806}`、`descriptor.rs:{430-449,815-833,1005-1025}`、`core-i32.atom`、SD-01 §8、`nextest-full.toml:{157-164}`、`git diff ab8809b00..1d0c86549` | 未计时 | 见下 |

**新增/受影响用例逐例墙钟（本轮 26 例两口径全 ≤10 s；仅列 ≥5 s 者 + 新用例）**

| 用例 | 组模式 (s) | 固定组 (s) |
|---|---|---|
| atom_real_schema_container_names_are_tags_not_fields（**本轮新增**） | 8.269 | 8.041 |
| atom_real_schema_drives_validation | 7.774 | 5.555 |
| atom_real_schema_slot_field_mapping | 7.592 | 6.886 |
| atom_real_schema_duplicate_field_is_8 | 7.116 | 5.527 |
| atom_real_schema_mode_is_idempotent | 5.926 | 5.785 |
| atom_real_hir_canonical_layout | 5.605 | 5.444 |
| atom_admission_envelope | 5.425 | 5.702 |

test-cost 门：新用例 8.269 s（组模式）/8.041 s（固定组），未超 10 s；该用例只做 1 次 Gen0 编译 + 5 次 schema 模式调用，与既有 `atom_real_*` 同族成本。

**N-7/N-8 关闭核对**

| ID | 判定 | 证据（@`1d0c86549`） |
|---|---|---|
| N-7 | **closed** | 取修法 (a)：`add_group_fields(..., tags_only)` 对 `containers` 只 `tag_add`、不再 `field_add`（`atom_data.at:1185-1214`，分支 `:1204-1212`）。真实 binder 依据：container 名存 `spec.containers`（`descriptor.rs:430-449`），具名字段只对 `spec.fields`+槽映射校验（`:815-833`），子 tag 另由 containers 成员表校验（`:1005-1025`）→"container 名 = 子 tag、非字段"成立。新用例 `atom_real_schema_container_names_are_tags_not_fields`（`atom_data.rs:769-806`）五断言 `0909000900`：`module(types: 1)`=9、`module(bodies: 1)`=9、`module(path: "p")`=0、`module(bogus: 1)`=9、`module { types { type(id: "t", kind: i32) {} } }`=0。**前值对照**：上次复审在 `ab8809b00` 用同批输入实测 `container-as-field 00 / container-as-field2 00 / declared-field 00 / unknown-field 09 / container-as-child 00`；修复后前三项→`09 09 00`、后两项不变，故负例是真钉住（旧码必红）而非恒真断言。SD-01 §8（`:166-186`）同步改写为"containers 条目名只注册为节点 tag、不成为父节点允许字段"，并写明 `module(types: …)`=9（`bind.unknown-field`）／`module { types { … } }`=0。 |
| N-8 | **closed** | `.config/nextest-full.toml:157-164` 注释改为"仅当加载本配置文件时生效（固定组 `cargo t ace-aac` 或显式 `--config-file`）；无 `--config-file` 的 ad-hoc 跑法不受 4 路限流"。实测 `show-config test-groups`：默认发现 0 个 `group:` 行，加 `--config-file` 才出现 `group: ace-aac (max threads = 4)`。门禁结果不变（两口径本轮均 26/26、逐例 ≤10 s）。 |

**F1..F8 / N-2..N-6 沿用核对**：本轮只改 container 准入、SD-01 §8 文本与一条配置注释；`codec.at`/`digest.at`/`driver.at`/`conformance.rs`/`scripts/test-groups.toml` 的 git blob 与 `ab8809b00` 逐字节相同（sha16 见上），故 F5b/N-2/N-3/F4b/N-4/F8 证据仍适用；N-5 的计划文本与 N-6 的计数/收据位置本轮补正（§7 计数 25→26、§7 交叉引用与 §9 收据位置一致、源身份 sha16 见 spec_inputs 与实现身份）。

**验收证据图（AC-ID → 判定 → 命令/方法 → 结果，@`1d0c86549`）**

| ID | 判定 | 命令/方法 | 结果 |
|---|---|---|---|
| AC-01 | **pass** | 组模式 + 固定组 26/26；V-06 族 `atom_real_*` 全绿（含新用例） | V-01/V-02/V-06 观测值/字节达裁定 A 的四项可观察；reference writer 逐构造字节仍具名交接 1047/1053 |
| AC-02 | **pass（沿用）** | frame/manifest/admission 结果码序列未变（`v03_*`/`v04_*` 绿） | 分类与合法近邻成立；执行器包络如实记录 |
| AC-03 | **pass** | 唯一失败项 N-7 已闭；读 `descriptor.rs` 三处规则 + `core-i32.atom` 全表 + 表构造代码 | 字段集合 = `fields` 条目 + 槽映射拼写；containers 只作子 tag；未知字段仍 9（`bogus`）；真实 HIR/真实 schema 仍被接受；无公共 std 新增、无第二份 hash 实现 |
| AC-04 | **pass** | 逐 V 行 + 本轮计时/计数/身份 + driver `--all` 重跑 | V-01..V-06 在本冻结提交均有当前证据；计数 26 与 `results.json` 一致；源/工具身份见 spec_inputs 与实现身份 |
| AC-05 | **pass（本记录）** | 本记录 + N-7/N-8 关闭核对 | — |
| AC-AC-STAGED | **pass（沿用）** | SD-01 §9.1 未改；冻结 std 零改动（本轮 diff 无 `stdlib/**`） | 阶段合同文本与 §6 行/§7 勾选一致 |
| AC-06 | 未评估 | — | merge-owned，保持未勾；本次不签 M2/AAC |

**覆盖边界（登记备查，不阻断）**：(1) F4b 的长度合法侧钉在 2048 标量，未在 65536 包络上沿取合法邻；(2) 编码器对相同 **edge** 的 12 无单测（与 pair 共用 `identical_pair`）；(3) containers 的**成员关系**（`types: [type]` 内允许哪些子 tag）与 `branches` require/forbid 仍不在本层校验——具名交接 1047/1053（SD-01 §8 已写明）；(4) 本次复审与修复为同一代理，独立性受限（见 receipt limitations）。

evidence: 固定组 `D:/autostack/.wt/lang-1056/auto-lang/target/test-groups/run-mwp8u33q/results.json`（executed=26、gate_passed=true、wall_seconds=27.08、max_parallel_tests_observed=4、source_state=bff9fd86…、config_sha256=237ee400…）；本轮日志 `D:/autostack/plan-artifacts/plan-1056/verify/{p1-nextest-group.log,p1-cargo-t-ace-aac.log,p1-n8-showconfig.log,p2-check-warnings.log,p2-driver-all.log}`（scratch，merge 前折入本文件并随 scope 删除）；关键 diff `git diff ab8809b00..1d0c86549`；真实输入 `crates/core/{schema/core-i32.atom,fixtures/native/add-2-3.atom}`；SD-01 sha16 `7259caf1e31a54c2`。未写 `.autoos/specs.json` 等 legacy ledger；未发布 canonical Spec；本轮除修复提交外无实现改动。

pending_closeout_gates: AC-06（merge：沉淀 SD-01/SD-02、落主身份核准、归档计划、材料按政策清理、`wt-guard` clean 后移树/分支）；冻结受影响模块并集 = `ace-aac`，已在本 reviewed_commit 实跑（26/26、逐例 ≤10 s、`max_parallel_tests_observed=4`）；周期 `cargo t all` 属主检出单实例批回归，不作为本 Plan 门。

**status: reviewed**（T-01..T-07 完成，T-08 留 merge；`current_step: 7`，`plan_revision: 6` 不变）。**next**: merge —— 按 merge 范式核准落主身份（SD-01 `7259caf1e31a54c2`、`atom_data.at` `f86c444309282fce`、`codec.at` `b942bf46f7f94618`）、沉淀 SD-01/SD-02、归档计划并清 scope；AC-06 待落主与清理完成后由 merge 勾选。

## PLAN-1056:r6 merge 收据

2026-11-19 stage: merge | plan_id: PLAN-1056 | plan_revision: 6 | outcome: **pass**（delivery / canonical sources / archival / cleanup 全部核验） | reviewed_commit（落主前）: `1d0c86549` → 落主后 `d0e11654b`（rebase，patch 等价见下） | delivery_commit: `e36131b403a711bfa342b833716266f38564890b` | main checkout: `D:/autostack/auto-lang`（branch `master`） | worktree: `D:/autostack/.wt/lang-1056/auto-lang`（branch `plan-1056-dev`） | merge_owner: `finish-1056`（同一代理，limitations 见 §9 复审记录）。

| 检查点 | 证据 |
|---|---|
| `prepared` | reviewed 基线 = `1d0c86549`（代码）+ `c478d5a88`（复审收据）@plan_revision 6，outcome pass；SD-01 `docs/specs/auto-ac/aac-digest-codec-profile.md`（`7259caf1e31a54c2`/18619）已在分支；SD-02 目标 `docs/specs/stdlib/design/ace-core-and-text.md` 未改（留 merge）；冻结受影响模块并集 = `ace-aac` |
| `landed` | 分支先 rebase 到 master `3bc34c654`：`git -c core.autocrlf=false rebase master` → tip `5517771ea`→`e36131b40`；`git range-diff c4057c959..5517771ea 3bc34c654..e36131b40` **10/10 全部 `=`（无 `!`）**，证明 patch 等价。主检出 `git merge --ff-only plan-1056-dev` exit 0，master tip = `e36131b40…` = delivery commit，**无 merge commit**。旧→新映射：`1d0c86549`→`d0e11654b`（N-7/N-8 修复）、`c478d5a88`→`d2610da84`（复审收据）、`5517771ea`→`e36131b40`（SD-02）；其余 7 个提交同样一一对应 |
| `canonical_verified` | 主检出 @`e36131b40`（git blob 口径）：SD-01 `docs/specs/auto-ac/aac-digest-codec-profile.md` `7259caf1e31a54c2`/18619 **= §9 复审记录值**；`experimental/ace/aac/support/atom_data.at` `f86c444309282fce`/64755、`codec.at` `b942bf46f7f94618`/22873 **= §9 复审记录值**；SD-02 目标 `ace-core-and-text.md` `097cc4269d587f68`/7801（新增 §5）。SD-02 为 docs-only 后继（+24 行，§1–§4 945 历史原样），实现/依赖零改动 → `e36131b40` 成为 delivery commit |
| `post-landing gate` | 主检出 `cargo t ace-aac`（无 `CARGO_TARGET_DIR`）：**`PASS: ace-aac` exit 0**，整命令 **54.86 s**，`target/test-groups/run-oytahxk_/results.json`：executed=**26**、gate_passed=true、max_parallel_tests_observed=**4**、wall_seconds=54.09、source_state=e7d4f146…；逐例最大 **8.15 s**（`atom_real_schema_container_names_are_tags_not_fields`），26/26 ≤10 s。主检出未提交改动（其它会话的 docs/design、packages/blueprints 等）与本包路径无交集 |
| `archived` | `git mv docs/plans/1056-aac-exact-digests-and-compiler-codecs.md docs/plans/archive/` → `docs/plans/archive/1056-aac-exact-digests-and-compiler-codecs.md`，frontmatter `status: archived`；随簿记提交入库（tracked，非未跟踪）；`git status` 对 plan/spec 目标路径为空 |
| `cleaned` | 见下节（guard / worktree / branch / 组目录 / plan-artifacts / 登记表） |

**计划副本对账（本任务强制项）**：主检出未跟踪副本 `docs/plans/1056-…`（`EA23BBCBBA4858F9…`，90106 B，含 `reviewer-1056c` 的 53 行 §9 记录，而分支副本当时缺这 53 行）在合并前已并入分支副本（本文件 = 该副本内容 + 本轮 §9 复审记录 + §7/任务表/源身份订正）；逐行包含性核对：主副本 588 行中仅 6 行在分支副本中"不存在"，全部是本轮**有意取代**的行（`current_step: 3`、§7 的 25 例计数句、T-04..T-07 四条未勾任务行）。核对通过后删除该未跟踪副本（sha256 已记于此），`git merge --ff-only` 因此不被 "untracked working tree file would be overwritten" 阻断；落主副本同时含 `reviewer-1056`/`reviewer-1056b`/`reviewer-1056c`/`finish-1056` 四份 §9 收据（行内计数 4/2/2/1，无收据丢失）。

**merge 阶段耗时（本会话，空闲窗口）**：`git rebase master`（10 提交，无冲突，未单独计时）→ range-diff 即时；主检出 `cargo t ace-aac` 整命令 **54.86 s**（含 main 侧 ace target 增量构建）；`wt-guard` <1 s clean；归档/清理命令均即时。

**证据（orig scratch，merge 后删除）**：`D:/autostack/plan-artifacts/plan-1056/verify/{p1-nextest-group.log,p1-cargo-t-ace-aac.log,p1-n8-showconfig.log,p2-check-warnings.log,p2-driver-all.log,p3-cargo-t-ace-aac-main.log}`；`target/test-groups/run-mwp8u33q/results.json`（worktree，executed=26）、`target/test-groups/run-oytahxk_/results.json`（主检出，executed=26）。关键数字已折入本文件，scratch 随 scope 删除。未写 `.autoos/specs.json` 等 legacy ledger；未做 release/部署构建。
