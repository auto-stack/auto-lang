---
plan_id: PLAN-1031
status: archived
feature_name: ACE QueryHost提交负依赖重验
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-17
plan_revision: 3
current_step: 5
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-semantics/source-query-session.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1031] ACE QueryHost提交负依赖重验

## 0. 变更摘要

2026-11-13 r2当前：用户要求对1028以外计划逐份核是否足够定制。本包补真实输入、状态/故障设置、独立关闭断言、测试落点、任务改面和Spec具体增量；原发现/AC/Case/T-ID及6任务、drafting 0/6保留。只做规划与静态源码核对，不修改实现/测试/配置，不接管1028实施，也不宣称动态反例或产品通过。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F22，优先级**P1**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

2026-11-16 r3阶段裁定：F22全部A-required，先关闭原NEG-01/02与对应独立oracle再供985/A；A增量提交的正负依赖同属真实性，新增声明/歧义后不能提交过期target。原6任务/0完成/既定设计与历史保留，仅规划。

## 1. 目标

候选提交前所读正负依赖均须仍新鲜；负候选集变化即拒绝安装，旧current/target保持。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。不扩大solver语言域；先核真实产品/测试调用面，风险不表述为已执行错误target。

## 2. 架构方案

复用现page_deps_fresh/候选集版本机制，candidate带真实读集；不以whole-module hash替代局部依赖，不恢复持久页的trust。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。等价机制由work选择；公共语义/支持/跨核心合同变化先具名设计及修约，不能临时加深度/规模限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。后续唯一树D:/autostack/.wt/lang-1031/auto-lang，建议分支codex/plan-1031-dev，尚未创建；work/review/merge顺序使用同树，禁止链接。材料唯一D:/autostack/plan-artifacts/plan-1031/，创建即登记owner/用途/状态，当前无scope。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2当前取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/source-query/src/host.rs](../../experimental/ace/crates/source-query/src/host.rs) | 81e930720f18b45d6b44de2991a774c6e44b3754efac77144c1d16c4a90ed9bd | 静态取材，非运行证据 |
| [experimental/ace/crates/source-query/src/session.rs](../../experimental/ace/crates/source-query/src/session.rs) | d32b10eb0d6cecf983054640cbac15c85a932bc031570bfa58f344e85a2e2d8a | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-semantics/source-query-session.md](../../docs/specs/auto-semantics/source-query-session.md) | 1a5375476e35f39015136c89f9e1a868e73c8b9448789c87d8675d441e90e8b9 | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1的“无关变化仍允许”过于宽泛；现负依赖用整个fn候选集指纹，新增无关函数名也会失效，不能借修复暗中改成更精细依赖。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/source-query/src/tests.rs | 6d23b5201799fb3677fa22569a548d52bcca68cf793dbc226274ff543406283d | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

## 5. 详细设计

唯一范围=F22。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：候选提交前所读正负依赖均须仍新鲜；负候选集变化即拒绝安装，旧current/target保持。 复用现page_deps_fresh/候选集版本机制，candidate带真实读集；不以whole-module hash替代局部依赖，不恢复持久页的trust。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r2执行合同预检（当前优先）

**输入到消费路径。** 沿sq06/sq26/sq27/sq28。先在同一QueryHost安装old函数的合法事实，绑定TargetHandle(77)为测试身份；再prepare缺helper的main候选，不给这个带诊断候选伪造可执行目标。保持host revision不变，只改变当前Program候选集再commit。MemorySink的len必须在过期拒绝前后不增。

**状态、故障与责任。** 候选携现事实的negatives和fn_candidates_fp或等价只读版本，commit复用相同freshness规则；不能强行whole-module hash，也不借此增加逐名负依赖精度。比较current的revision/key/artifact/facts/target完整快照、sink len和旧值；拒绝可用既有DepChanged或明确新负依赖错误，但须在T-01冻结稳定类别，不只查Err。

**先决决定边界。** 上述固定输入与原支持域可从T-01直接核证；T-01仅选择等价局部机制，不把所有设计留到实施。若真实入口与假设不符，必须给可重放反证和合法邻例后修约，不能仅写unknown或删除原Case。

**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。任何新增limit或公共支持变化按本节具名决定，不推定已批准；等价内部实现不需重复许可。

### A-EXIT-20261116：本包在重冻985/A中的阶段（当前优先）

本包所列原发现/Case全部为A出口必须关闭的正确性/可信证据输入，**不改变已冻结语义、预算或测试规模**。A取原T-01→T-02/03实现/反证与永久例→T-04独立复审→T-05正式合入/Spec来源；T-06按原规约收尾归档，不能把仅分支绿当正式D。可用已审落主切片准备消费，但985/A最终必须绑定真实代码/输入/oracle/目标/工具/原条件及结果。当前草案/0完成保持；本轮没有证明任何发现已修复。

实现若发现主线已有修复，须保原反例/合法邻例、真实入口/事实和充分反证，经原review确认后关闭，不能为了“做修复”重复改代码，也不能用未复现/组总绿当反证。文档/G1002只在实际改写作者格式或相应adapter接线时消费；原入口/Rust回归不等1066全量迁移。UI三波只约束图形新增支持，已有非图形AV/AC/A2R及原C/Py支持腿/错误/资源回归不取消。
**本包绑定**：F22 → 原NEG-01/02 → T-01/02/03 → T-04/05正式来源 → 985 T-A03/A04。原因：A增量提交的正负依赖同属真实性，新增声明/歧义后不能提交过期target。关闭谓词：候选集或正签名变更拒绝，旧revision/current/target不变；body-only合法控制可提交，重新prepare同源后成功。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（拟议） | docs/specs/auto-semantics/source-query-session.md | 现规则/支持域保持；补充以下修复后的明确合同：提交前重验正依赖与fn负候选集，按当前粗粒度freshness；失败不发布或换current，body-only稳定保复用。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-semantics。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| NEG-01 | AC-02 / T-02 | sq06缺helper准备负事实→新增helper/产生歧义后commit | QueryHost::prepare/attach_target/commit | 拒绝过期候选；旧revision/artifact/target不变；重新prepare后可提交 | 拟实施，未运行 |
| NEG-02 | AC-02 / T-02 | 已有无关函数body或同callee body变更；新增声明按负候选集规则拒；原sq27正签名变化 | 同入口与缓存freshness | 候选集/正签名未变才允许；正负变化继续拒绝；cache-hit规则与commit一致 | 拟实施，未运行 |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r2具体输入与状态断言（当前，全部待实施）

```auto
fn old() int { return 42 }
fn main() int { return helper(1) }
```
P1如上；P2只在末尾追加fn helper(v int) int { return v }，main body/签名不变。prepare(P1,main)后commit(P2)必须拒绝，旧current仍为old/TargetHandle(77)；重新prepare(P2)后才可安装新main事实。此测试不声称负事实可作为机器代码执行。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| NEG-01 | 缺helper→新增helper；另加无关fn名称/候选集变化 | 按现page_deps_fresh粗粒度候选集规则拒绝；不写sink、不增revision、不换target |
| NEG-02 | 既有无关函数仅body变、已有callee仅body变；正签名变化/取消/late/sink Err | 候选集/正签名不变者可提交；签名变化等原拒绝保留；不把“新增声明”当合法不失效控制 |

| 原Case ID | 拟新增永久测试名（当前不存在） | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| NEG-01 | audit1031_neg_01 | [experimental/ace/crates/source-query/src/tests.rs](../../experimental/ace/crates/source-query/src/tests.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| NEG-02 | audit1031_neg_02 | [experimental/ace/crates/source-query/src/tests.rs](../../experimental/ace/crates/source-query/src/tests.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。

本轮A阶段交接：NEG-01/02的全部原合法/拒绝/失败后恢复与资源断言是A必需；原AC-01..06范围不减，独立复审/正式来源和当前A/B分类由985接收控制核，不另建Plan状态或测试调度器。当前均未执行。

## 7. 验收标准

- [ ] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，所有§6反例及邻例可在实际入口运行，重大未决先定，不私改expected。
- [ ] AC-02 / work：r2具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。
- [ ] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。
- [ ] AC-04 / review：逐项核r2独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [ ] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [ ] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、未来实施授权及当前源 | 把上述P1/P2与三类合法/非法邻例接入sq族，冻结coarse候选指纹合同和完整current快照；证明当前bug到commit而非prepare先拒。 AC-01  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-02 / work | [x] T-01决定与可区分输入 | 在Candidate/prepare/commit补齐负读集重验，尽量复用session freshness，测试过期拒绝发生在sink.put前。 AC-02  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-03 / work | [x] T-02 | 重跑sq06及sq26..28、body-only正控和取消/late/sink失败；核缓存命中与安装准入一致，完成semantics职责组。 AC-02/03  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-04 / review | [x] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-05 / merge | [x] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-06 / merge | [ ] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1的“无关变化仍允许”过于宽泛；现负依赖用整个fn候选集指纹，新增无关函数名也会失效，不能借修复暗中改成更精细依赖。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

本Plan r1已钉源/Spec文件无字节差额；额外读取的现测试/设施见§4。

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原2个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。

2026-11-16 stage:new/revision | PLAN-1031 r3 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | F22/原NEG-01/02明确A；6任务/勾选/0完成与既定设计保持；源/结果不得以旧审计自报替代 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。

### work 阶段执行记录（2026-11-17，/auto-plan-goal 全链授权）

授权：用户 `/goal 完成计划1031`（2026-11-17）= 全链授权（work 建树/实施、review、有界修复、merge/归档/清理），各阶段继承，不再重复询问。开工基线 master=24b4ba4a7（非规划时 c2760324）；§4 四个钉源文件（host.rs/session.rs/tests.rs/source-query-session.md）字节 SHA256 与 §4 完全一致（零漂移），同期 sibling 漂移（apply/catalog/ident/instances/modules/packages*/produce/restore/walk，PLAN-1064/969 落线）不在 F22 commit 路径上，具名记录不改合同。worktree=`D:/autostack/.wt/lang-1031/auto-lang`，branch=`plan-1031-dev`，base=24b4ba4a7，code commit=`212075c91`（3 文件 +222/−5）。

**T-01（反例冻结）**：P1/P2=§6 r2 输入原样落地（`P1_1031`/`P2_1031` 常量）。预修复探针 `audit1031_probe_bug_present`（已随修复删除、不入最终代码）在修复前 **PASS**：`cargo test -p ace-source-query --lib audit1031_probe_bug_present` → 1 passed, 0.02s（冷编译 27.77s）——证明 prepare 正常返回带 helper 负依赖的候选，而 commit 在 P2 候选集变化后**错误安装陈旧候选**（revision 1→2、sink 0→1→2、current 换成带负依赖的 main 事实）：bug 在 commit 而非 prepare。三类邻例（body-only 合法×2、签名变化 DepChanged、sink Err）全部进入 `audit1031_neg_02`。

**T-02（修复）**：`Candidate` 增加 `negatives: Vec<NegativeDep>` 与 `fn_candidates_fp: String`（与会话 `BodyPage` 同一负读集）；`prepare` 记录 `out.facts.negatives` + `catalog.candidates_fp()`；`commit` 在正依赖环后、`sink.put` 前按 `page_deps_fresh` 同一粗粒度规则重验：候选含 fn 域负依赖且整集指纹变化 → 新稳定错误类 `CommitError::NegativeFlipped`（不发布、不增 revision、不换 current/target）。未改粗粒度合同（无关新增 fn 名同样失效）；正签名变化仍 `DepChanged`；instance_host commit 走当前源整体重推导+集合指纹（更强机制），F22 形状不适用，未改。

**T-03（验证）**：`cargo test -p ace-source-query --lib` → **104 passed / 0 failed, 0.74s**（含 sq06/sq26/sq27/sq28 原入口 + 新 `audit1031_neg_01`/`audit1031_neg_02`）。固定组 `cargo t ace-semantics`（机器占用预检 0 cargo/rustc 进程）→ 294 selected：**293 pass / 1 fail / 10 skipped（原条件）**，wall 29.2s。唯一 fail=`ace-semantics-store::conformance sema_boundary_audit::sema_boundary_case_files_executed_status_honest`（0.031s）为**基线预存红**：同一测试在 pristine master 24b4ba4a7 主检出原样复现（panicked: cache-roundtrip.document.case.ad must carry executed status (T-06 landed)），属 1066/1062 迁移线债务，与本包改动零共享路径，不判本包红、不代 PASS，具名归因。新用例耗时：neg_01+neg_02 合计 0.02s（≤10s/例门，达标）。健康：本包改动区域 rustfmt clean；预存 `sink2_key` unused_mut（基线已有）与 2 处预存 fmt 差异（非本包行）未动。SD-01 spec 增量已按 §5 规范增量表写入 worktree `docs/specs/auto-semantics/source-query-session.md`（宿主安装 step 3 补负候选集重验合同），随 merge 正式落地。

本节测试会话报表（work 实测）：

| 测试模块/实际范围 | 结果（执行/排除） | 准备 | 执行 | 总墙钟 |
|---|---|---|---|---|
| ace-source-query --lib（104 例含 2 新） | 104/0/0 pass | 27.77s 冷编译 | 0.74s | <1s（复用增量） |
| cargo t ace-semantics（固定组并集） | 293 pass/1 预存红/10 原条件排除 | 复用已热 | 29.2s | 29.2s |

| 新用例 | 检验的行为与 oracle | 实测耗时 | 结果 |
|---|---|---|---|
| audit1031_neg_01 | 候选集变化拒装（NegativeFlipped）、旧 revision/artifact/target 不变、sink 不增、粗粒度无关名同拒、重 prepare 后可装 | ~0.01s | pass ≤10s |
| audit1031_neg_02 | body-only 正控可提交、cache-hit 与 commit 准入一致、DepChanged/sink Err 拒绝保留 | ~0.01s | pass ≤10s |

stage: work | plan_id: PLAN-1031 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1031/auto-lang | branch: plan-1031-dev | base_commit: 24b4ba4a7 | code_commit: 212075c91 | dependency_revisions: 无跨仓依赖 | task_ids: T-01,T-02,T-03 | evidence: 见本节（探针红/绿证据、104 例 lib、ace-semantics 293+1 预存红复现） | blockers: 无 | next_responsible_role: review（T-04）

stage: goal | plan_id: PLAN-1031 | plan_revision: r3 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1031/auto-lang | commit: 212075c91 | next: review


stage: review | review_scope: final | reviewer_identity/limitations: 同会话独立上下文复审（未指定独立复审者；结论从工件重构：提交 diff、组 results.json、基线提交探针，非执行方摘要） | plan_id: PLAN-1031 | plan_revision: r3 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-1031/auto-lang | branch: plan-1031-dev | reviewed_commit: 212075c912dcf618c1870c64af5a4255916e3229 | base_commit: 24b4ba4a7 | dependency_revisions: 无 | spec_inputs: docs/specs/auto-semantics/source-query-session.md（基线 1a537547…，worktree 内 SD-01 delta 已随 diff 复核） | acceptance_results: AC-01 pass / AC-02 partial（F-R1）/ AC-03 pass；探针红证据 + 基线无 NegativeFlipped 共同证明 bug 在 commit；audit1031_neg_01/02 在固定组实际执行（0.046s/0.048s，ok，与 sq06/sq27 同组）；lib 104/104；instance_host 重推导车道 F22 不适用 | pending_closeout_gates: 冻结提交上的 cargo t ace-semantics 并集（merge 所有；现有 293/294 + 1 预存红已以 pristine master 复现归因） | findings: F-R1 | evidence: 本节工件 + results.json | next: work（修复 F-R1）

**F-R1（needs_fix，minor，影响 AC-02/T-02）**：`audit1031_neg_01` 拒绝后的旧记录断言只比较 revision/facts.name/artifact/target/sink-len，而 §5 r2 冻结合同枚举"比较current的revision/**key**/artifact/**facts**/target完整快照、sink len和旧值"。违反规则：§5 完整快照枚举。输入/入口：`audit1031_neg_01` 拒绝后断言块（提交 212075c91）。预期：`cur.key == old_rec.key`、`cur.facts == old_rec.facts` 同时成立（两类型均 derive PartialEq，无需改产品代码）；实际：缺失。成因：实施侧断言遗漏（产品路径本身正确——拒绝路径不触碰 current）。同族路径：NEG-02 各断言与其自身 oracle 行一致，无同形缺口。修复与关闭 oracle：补上述两断言，重放 `cargo test -p ace-source-query --lib audit1031`；其余证据保持有效（纯测试断言补充、产品语义零变化）。

复审会话报表（复审自身验证，无新产品用例）：worktree/branch/commit 身份复核（git log/status）<1s；results.json 工件核验（audit1031×2 + sq06/sq27 执行记录）<1s；git show 基线探针（sink2_key 预存、NegativeFlipped 基线不存在）<1s；未重跑测试（代码自 work 证据后零变化，复用 work 的 104/104 与组 293+1 归因红，复用已声明）。

stage: goal | plan_id: PLAN-1031 | plan_revision: r3 | round: 2 | from: review | to: work | status: executing | outcome: needs_fix | worktree: D:/autostack/.wt/lang-1031/auto-lang | commit: 212075c91 | next: work


### work 修复记录（2026-11-17，F-R1，第1次修复轮）

F-R1 修复：`audit1031_neg_01` 拒绝后断言块补 `assert_eq!(cur.key, old_rec.key);` 与 `assert_eq!(cur.facts, old_rec.facts);`（后者涵盖原 name-only 检查），注释改为引用 §5 完整快照枚举。纯测试断言补充，产品代码零变化（diff 1 文件 +4/−2）。修复提交 `472450af9`（父 `212075c91`）。

finding→检查映射（修复轮范围）：F-R1 → audit1031 oracle 完整性 → 重放 `cargo test -p ace-source-query --lib audit1031` → **2 passed / 0 failed, 0.02s**（neg_01 含新增两断言）。产品语义未变：work 轮 104/104 lib 与 `cargo t ace-semantics` 293+1 预存红证据保持有效（复用，未重跑）；冻结组并集仍为 merge 所有 gate。本包改动区域 fmt clean（tests.rs 现存 6 处 fmt 差异全在预存区域：17/42/201/396/1146/1249，均非本包行）。无新用例（修复只加断言）；占用预检 0 cargo/rustc 进程。

stage: work | plan_id: PLAN-1031 | plan_revision: r3 | outcome: pass（F-R1 已闭） | worktree_path: D:/autostack/.wt/lang-1031/auto-lang | branch: plan-1031-dev | base_commit: 24b4ba4a7 | code_commit: 472450af9（impl 链：212075c91→472450af9） | dependency_revisions: 无 | task_ids: T-02（修复） | evidence: 本节 | blockers: 无 | next_responsible_role: review（重新 final review，范围=F-R1 及受影响复验）

stage: goal | plan_id: PLAN-1031 | plan_revision: r3 | round: 3 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-1031/auto-lang | commit: 472450af9 | next: review


stage: review | review_scope: final（重新复审，F-R1 范围） | reviewer_identity/limitations: 同会话独立上下文复审（结论从工件重构：修复 diff、复审者自有重放，非执行方摘要） | plan_id: PLAN-1031 | plan_revision: r3 | outcome: pass | worktree_path: D:/autostack/.wt/lang-1031/auto-lang | branch: plan-1031-dev | reviewed_commit: 472450af9（impl 链 212075c91→472450af9） | base_commit: 24b4ba4a7 | dependency_revisions: 无 | spec_inputs: docs/specs/auto-semantics/source-query-session.md（SD-01 delta 已在前轮复审通过，本轮未变） | acceptance_results: AC-01 pass / AC-02 pass（F-R1 关闭）/ AC-03 pass / AC-04 本行闭合；全部 6 任务中 T-01..T-04 完成 | pending_closeout_gates: 冻结提交上 cargo t ace-semantics 并集（merge 所有；212075c91 上的 293+1 预存红证据可按"输入未变"复用规则由 merge 裁定复用或重跑） | findings: 无新发现 | evidence: 见下 | next: merge（T-05/T-06）

**F-R1 关闭核验**：修复 diff（git show 472450af9）= 仅 tests.rs +4/−2：`assert_eq!(cur.key, old_rec.key);` + `assert_eq!(cur.facts, old_rec.facts);`（完整 facts 等值涵盖原 name-only），注释引用 §5 完整快照枚举——§5 六维（revision/key/artifact/facts/target/sink len）全部落地。复审者自有重放：`cargo test -p ace-source-query --lib audit1031` → **2 passed / 0 failed, 0.03s**（neg_01/neg_02，含新断言）。产品代码/输入零变化 → 前轮全部证据（104/104 lib、组 293+1 预存红归因、fmt/告警健康、instance_host 消费者面、SD-01 delta 复核）按"相关实现/依赖/配置/输入未变"复用，未重跑。修复轮计数 1/3（上限内）。检查→不变量映射：audit1031 过滤器 = F-R1 oracle 完整性 + 两个负依赖 Case 的全部断言。

复审会话报表（重新复审轮，无新用例）：git show 修复 diff 核验 <1s；复审者重放 audit1031 过滤器 0.03s（增量编译复用）；其余复用已声明。

stage: goal | plan_id: PLAN-1031 | plan_revision: r3 | round: 4 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-1031/auto-lang | commit: 472450af9 | next: merge


### merge 落地与归档记录（2026-11-17，PLAN-1031:r3）

| Checkpoint | 证据 |
|---|---|
| prepared | reviewed r3 @472450af9（修复链 212075c91→472450af9）；SD-01 delta 冻结于 worktree spec；master 前移至 b764ba9e3（1029 docs-only），rebase 后 delivery=4e6d28522 |
| landed | worktree rebase master：映射 212075c91→58264ba35、472450af9→4e6d28522，`git range-diff` 两提交全等（`=`）；主检出 `git merge --ff-only plan-1031-dev`，master tip=4e6d28522=dev tip（无 merge commit）；wt-guard clean（"wt-guard: clean — …无任何 reparse point"）后落地 |
| canonical_verified | docs/specs/auto-semantics/source-query-session.md @4e6d28522 = SHA256 15a56be161e3a13c9a0da78833b13b82ebd32eb0dff6b12966137c98bd552611（基线 b764ba9e3 = 1a5375476e35… 与 §4 钉值一致，零无关漂移；差异仅为 SD-01 增量） |
| 冻结组门 | `cargo t ace-semantics` @4e6d28522（merge 所有）：294 run = **293 pass / 1 fail / 10 skipped（原条件）**，1.76s（复用热编译）。唯一 fail=`ace-semantics-store::conformance sema_boundary_audit::sema_boundary_case_files_executed_status_honest`（0.033s）为**基线预存红**：同测试在 pristine master b764ba9e3 主检出原样复现（落地基线绑定），属 1066/1062 迁移线债务，与本包零共享路径；具名保留不充 PASS |
| archived | docs/plans/archive/1031-ace-query-negative-dependency-commit.md，status: archived，T-01..05 [x]、T-06 留待清理后闭合；本 commit 连同计划文档一并入库（未跟踪文档≠已归档） |
| cleaned | 待清理完成后追加（worktree/分支/组目录、plan-artifacts/plan-1031 删除、登记表刷新） |

生产制品新鲜度观察：本包触面为 experimental ACE source-query（host 安装协议 + 测试），无 release 二进制/web bundle 等生产制品在授权交付范围内——记录为"无受影响生产制品"，未触发任何 release/重启。

merge 会话测试报表：rebase+range-diff <2s；冻结组门 1.76s（294 例，热编译复用，占用预检 0 进程）；预存红 pristine 复现 0.01s；无新用例（交付提交仅含 F-R1 断言强化，review 轮已计 0.03s）。

stage: merge | plan_id: PLAN-1031 | plan_revision: r3 | outcome: pass（landed+canonical_verified+archived；cleaned 待 T-06） | delivery_commit: 4e6d28522（rebased from 472450af9，range-diff 全等） | canonical_spec: docs/specs/auto-semantics/source-query-session.md@15a56be1… | archive_path: docs/plans/archive/1031-ace-query-negative-dependency-commit.md | cleanup: pending | next: T-06 清理

stage: goal | plan_id: PLAN-1031 | plan_revision: r3 | round: 5 | from: merge | to: merge(T-06) | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-1031/auto-lang | commit: 4e6d28522 | next: cleanup

## 10. 待澄清事项

不扩大solver语言域；先核真实产品/测试调用面，风险不表述为已执行错误target。 T-01使真实反例/资源或IO故障可运行，未取得前不声称动态后果已证。等价实现不需重复审批；新支持/预算/公共合同变化先最小决定，不能用隐式门限或延期替原义务。不等待“新测试框架全部完成”。

本轮T-01入口已具体化，当前不需要用户再次确认等价内部实现。未取得真实动态反例的部分仍由本节与§6指定入口/邻例核证，不把预期当已观察结果。

**外部状态记录（2026-11-17 work 阶段，非本包义务）**：`D:/autostack/plan-artifacts/.retention/policy.json` 在本会话开始时可解析（98 plans），至登记时点已损坏（UTF-8 BOM + "Expecting ',' delimiter: line 314 column 5"，PLAN-080 条目后出现拼接的 `]}` 与后续 ag-examples/PLAN-017 等条目；mtime 2026-11-17 01:08:06，非本会话写入——本会话对该文件零写入）。按 fail-closed 不擅自修复他人登记表；`plan-artifacts/plan-1031/` scope 登记挂起，由 merge 收尾的登记表刷新一并处理；若届时仍损坏，merge 按其恢复规则上报而非静默覆盖。
