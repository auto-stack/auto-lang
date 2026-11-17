---
plan_id: PLAN-1037
status: executing
feature_name: ACE CT命名实参遍历与发布失败观察事实
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-17
plan_revision: 3
current_step: 2
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: [docs/specs/auto-cte/source-query-integration.md, docs/specs/auto-cte/project.md]
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1037] ACE CT命名实参遍历与发布失败观察事实

## 0. 变更摘要

2026-11-17路线状态联络：本包仍drafting r3、0/6，尚无登记的实施树；本次只更新状态与排程，不推进Task/AC。当前位置为**第1：A必闭正确性，下一实施包**；总表见[1024当前路线状态](1024-ace-m2-code-quality-and-refactoring-audit.md#quality-route-20261117)。9个同路线包已交付归档，不再照2026-11-13旧队列重复开工；实施合同/Case/独立oracle保持。

2026-11-13 r2当前：用户要求对1028以外计划逐份核是否足够定制。本包补真实输入、状态/故障设置、独立关闭断言、测试落点、任务改面和Spec具体增量；原发现/AC/Case/T-ID及6任务、drafting 0/6保留。只做规划与静态源码核对，不修改实现/测试/配置，不接管1028实施，也不宣称动态反例或产品通过。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F26/F32，优先级**P1（F26为P2）**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。真实动态影响由T-01反例界定，不把静态风险冒实际失败。

2026-11-16 r3阶段裁定：F26/F32全部A-required，先关闭原CT-01/02与对应独立oracle再供985/A；Named CT完整发现/替换/回灌与真实av_reached事实关系到A完整CTE和证据分类。原6任务/0完成/既定设计与历史保留，仅规划。

## 1. 目标

CT发现、替换、查找、callee闭包和product回灌遍历相同Arg.value子树；真实AV已到达的事实不因随后publish失败改成false。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。只修当前scalar-CT可达子树与事实；不提前实现完整M2宏/条件编译；与966相关改动同文件单写对账。

## 2. 架构方案

沿现AST/source-cte共享child枚举和CtExecutor接缝，保Pos/MovePos/Named源码顺序；call/expr观察一致，不因冻结失败重跑AV，不增加新executor。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。等价机制由work选择；公共语义/支持/跨核心合同变化先具名设计及修约，不能临时加深度/规模限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。唯一树 D:/autostack/.wt/lang-1037/auto-lang，分支 plan-1037-dev（2026-11-17 建，基面 e85096628；plan 内建议名 codex/plan-1037-dev 与仓约定不符，按 AGENTS.md 取 plan-NNN-dev）；work/review/merge 顺序使用同树，禁止链接。材料唯一 D:/autostack/plan-artifacts/plan-1037/（2026-11-17 建，owner PLAN-1037，用途实施/验证日志，in-flight，merge 折叠后删除）。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2当前取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/source-cte/src/schedule.rs](../../experimental/ace/crates/source-cte/src/schedule.rs) | 529fce63a24f580cd748e4eeb5754e075b27f07c5dd651890b92ee8e85df59c7 | 静态取材，非运行证据 |
| [experimental/ace/crates/source-cte/src/product.rs](../../experimental/ace/crates/source-cte/src/product.rs) | 86846f6a3dca37530b4dd5e255cf99edb95abf556c523df22f0f3fd3254afbce | 静态取材，非运行证据 |
| [experimental/ace/crates/source-cte/src/service.rs](../../experimental/ace/crates/source-cte/src/service.rs) | 9b6eb85d126b6f4794e5017b697345ed89e38a01b2f319413958a9986ccbefe8 | 静态取材，非运行证据 |
| [experimental/ace/crates/source-cte/src/executor.rs](../../experimental/ace/crates/source-cte/src/executor.rs) | a970487d747940caf9bcae9c0e98b94cff131da58cd7dbacecc4ced8a3855355 | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-cte/source-query-integration.md](../../docs/specs/auto-cte/source-query-integration.md) | a605853533d7f70a0f9ccce5d825d1cf461741910e434800d65323bbd312c6ae | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-cte/project.md](../../docs/specs/auto-cte/project.md) | 3e9220a203e65b08cd0192f32546609b65757e7ebb10a35b22462571e768fd0a | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1未冻结CT capability、实际公开compile_scalar_ct入口、call/expr发布失败的计数与日志差分，容易使用不带权限的语法拒绝证明错规则。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/source-cte/src/tests.rs | 5bba88afa016d1bdda86aac4d8d785873c275172e4f53f14cadcac42b914c4fb | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/source-cte/src/store.rs | 46698a45319694fe6727b827e7d61f0e61e514c59107c6e161fc51e872cb7f42 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/av/src/query_entry.rs | eab8602115ef3666e7ef4d201a66d803e2f54e3837435590dec48d1f8e1ee7c0 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。

2026-11-17 实施授权（goal 会话「完成计划1037」）：用户以该 goal 目标授权本包全链——建树、work 实施与有界修复、独立 review、merge（规范沉淀/归档/清理）；不改变目标、AC、范围、预算与仓库集合，不跳过独立复审，不以未绑定 revision 的 pass 合入。

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

## 5. 详细设计

唯一范围=F26/F32。关闭行必须连接“发现→现代码不变量→反例→真实consumer→永久断言→修复提交”，承接登记不算关闭。T-01使§6全部case在原入口可运行。

数据/错误/责任：CT发现、替换、查找、callee闭包和product回灌遍历相同Arg.value子树；真实AV已到达的事实不因随后publish失败改成false。 沿现AST/source-cte共享child枚举和CtExecutor接缝，保Pos/MovePos/Named源码顺序；call/expr观察一致，不因冻结失败重跑AV，不增加新executor。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r2执行合同预检（当前优先）

**输入到消费路径。** CT-01调用ace_source_cte::compile_scalar_ct或真实AV query_entry产品入口，capabilities含ac.source.cte.scalar/v1；不带cap的同源是独立拒绝负控。CtProduct.program必须已替换HashExpr，checked经shared lower/MIR admission→真实AV，不能只搜AST或比较atom文本。Named/MovePos/Pos按实际Arg字段遍历，发现/闭包/meta/substitute/find/product回灌一致。

**状态、故障与责任。** 冻结前后CtCounters与exec_log长度/entry.key_hash/attempt，从真实边界核字段，不能只assert error code。FailPublishSink通过原CtEnvelopeSink trait即可实现，无须新executor或产品failpoint；失败后第二次请求应重执行而非命中不存在页，清故障后发布并hit才完成后续使用闭环。

**先决决定边界。** 上述固定输入与原支持域可从T-01直接核证；T-01仅选择等价局部机制，不把所有设计留到实施。若真实入口与假设不符，必须给可重放反证和合法邻例后修约，不能仅写unknown或删除原Case。

**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。任何新增limit或公共支持变化按本节具名决定，不推定已批准；等价内部实现不需重复许可。

### A-EXIT-20261116：本包在重冻985/A中的阶段（当前优先）

本包所列原发现/Case全部为A出口必须关闭的正确性/可信证据输入，**不改变已冻结语义、预算或测试规模**。A取原T-01→T-02/03实现/反证与永久例→T-04独立复审→T-05正式合入/Spec来源；T-06按原规约收尾归档，不能把仅分支绿当正式D。可用已审落主切片准备消费，但985/A最终必须绑定真实代码/输入/oracle/目标/工具/原条件及结果。当前草案/0完成保持；本轮没有证明任何发现已修复。

实现若发现主线已有修复，须保原反例/合法邻例、真实入口/事实和充分反证，经原review确认后关闭，不能为了“做修复”重复改代码，也不能用未复现/组总绿当反证。文档/G1002只在实际改写作者格式或相应adapter接线时消费；原入口/Rust回归不等1066全量迁移。UI三波只约束图形新增支持，已有非图形AV/AC/A2R及原C/Py支持腿/错误/资源回归不取消。
**本包绑定**：F26/F32 → 原CT-01/02 → T-01/02/03 → T-04/05正式来源 → 985 T-A03/A04。原因：Named CT完整发现/替换/回灌与真实av_reached事实关系到A完整CTE和证据分类。关闭谓词：Named/Pos/MovePos同源CT独立值3、无未服务hash且能力拒绝；真实执行后publish Err保持av_reached=true/execute增/publish不增，下一次重执行、修复发布/hit；前门拒绝仍false。


### 路线排程与当前进入条件（2026-11-17）

本包F26/F32与CT-01/02是本路线最后一份尚未交付的A必闭整包，优先于B的1030/1038；1024 T-05可独立收尾。1002试点已落主（70a680dba），不新增等待测试框架全迁移的门。T-01先核当前source-cte/AST/持久层及既有文档case入口、受控publish故障与av_reached/counters；主线已有等效修复时须真实反证+合法邻例+永久回归交原review，不能为“修复”重复改代码。与985/1066或实际共写文件的在途计划排写窗，不能以本排序占有其WIP。

本轮只做计划簿记；原T-01..06、AC-01..06、Case及现阶段分类不变，plan_revision仍r3。正式实施从当时master建立本包唯一树并核源漂移/可用consumer，不凭本状态快照跳过T-01。Windows不运行Linux/WSL腿且不记PASS，Linux义务保留给专用环境；常规正确性与显式ace-perf分开。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify（拟议） | docs/specs/auto-cte/source-query-integration.md | 现规则/支持域保持；补充以下修复后的明确合同：发现/替换/回灌遍历完整现Arg.value面，当前cap与source order不变，未服务hash不能进运行。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |
| SD-02 | modify（拟议） | docs/specs/auto-cte/project.md | 现规则/支持域保持；补充以下修复后的明确合同：执行成功后发布失败仍观察av_reached=true；尝试/发布/hit/语言结果分开且不重跑AV。 | 对应本Plan具体Case及真实consumer；不是候选已实现声明 | AC-02/03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

拟执行完整受影响组：cargo t ace-comptime ace-semantics ace-av ace-native ace-a2x。T-01按实际调用/修改冻结必要消费者并集，不能以单反例代完整职责组。 shared CT回灌补native真实consumer；若变更C消费的CheckedModule/资源事实，再补ace-a2c-abi合法/拒绝近邻。目标测试名/新fixture在T-01核原目录后使其可运行，当前均未创建。原样本/规模/平台/ignore/正式性能轴保持；merge复用同身份有效证据，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| CT-01 | AC-02 / T-02 | fn id(x int) int{return x}; main调用id(x:#{1+2})与Pos对照 | compile_scalar_source_product→schedule→service→product回灌 | 独立值3；不残留unserved hash；命名倒序/嵌套/meta各保次序和能力拒绝 | 拟实施，未运行 |
| CT-02 | AC-02 / T-02 | 真实call/expr成功后sink.publish Err；执行前拒绝/trap/cancel邻例 | CtService两入口与现执行日志/计数 | 仍cte.freeze/publish错误且av_reached=true，ct_execute已增；未到达者false，不能用错误事实隐藏成本 | 拟实施，未运行 |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r2具体输入与状态断言（当前，全部待实施）

```auto
fn id(x int) int { return x }
fn main() int { return id(x: #{ 1 + 2 }) }
```
CT-01独立值3；位置实参id(#{1+2})同值。补参数反序命名且类型不同、Named内nested hash、Hash内部调用与原meta能力近邻；每类断言原求值顺序/冻结命中，不新开完整M2宏域。

CT-02实现测试FailPublishSink：probe正常委托MemoryEnvelopeSink或返回None；publish先记录key/env及次数再返回现有EnvelopeError::Decode("injected-publish".into())（与真实store错误使用同类载体，不杜撰Store变体）。使用真实call id(3)与expr 1+2执行后拒绝发布，不用假executor伪造av_reached。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| CT-01 | Named/Pos/MovePos子树，发现→替换/查找/闭包→product | 值3且无残留hash；原SourceOrder与当前budget/能力门保持；强制去cap仍受控拒绝 |
| CT-02 | 真实call成功→publish Err、真实expr成功→publish Err | Err code=cte.freeze/publish，av_reached=true；ct_execute +1，freeze_publishes +0，rejections +1；真实ok日志一条，sink无可读冻结页 |
| CT-02 | 执行前forbid/missing cap/坏绑定，执行trap/cancel/resource，正常发布与hit | 前门未到AV false；到达后失败按原事实true；hit不增execute；修观察不得重跑AV |

| 原Case ID | 拟新增永久测试名（当前不存在） | 既有源码/目标落点 | 使其可运行的owner |
|---|---|---|---|
| CT-01 | audit1037_ct_01 | [experimental/ace/crates/source-cte/src/tests.rs](../../experimental/ace/crates/source-cte/src/tests.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |
| CT-02 | audit1037_ct_02 | [experimental/ace/crates/source-cte/src/tests.rs](../../experimental/ace/crates/source-cte/src/tests.rs) | T-01固定设置，T-02新增函数/接线，T-03核真实发现/执行 |

一个Case可需多个同族函数，以上名字是最小锚，不代全部状态。优先扩已有test根/私有cfg(test)模块；若新建test文件，须在其真实conformance根或Cargo target接线，不能只创建文件然后把rg属性数当发现。T-03必须在实际固定组结果中核各锚点被发现/执行；平台/工具条件排除具名列出，不计PASS。

本轮A阶段交接：CT-01/02的全部原合法/拒绝/失败后恢复与资源断言是A必需；原AC-01..06范围不减，独立复审/正式来源和当前A/B分类由985接收控制核，不另建Plan状态或测试调度器。当前均未执行。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结真实源、支持/资源边界与前置，所有§6反例及邻例可在实际入口运行，重大未决先定，不私改expected。（2026-11-17：caps=ac.source.cte.scalar/v1 与 compile_scalar_ct/serve_program/CtService 两入口冻结；两例在真实入口首红→绿，见§9 work 收据。）
- [ ] AC-02 / work：r2具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；缺口修复或有充分反证，修前/修后及同族边界证据齐，原义务不减。（2026-11-17 work 证据在案；同轮 review 复开：**F-1037-R1** 命名实参的值/标签未入执行键，两站点共享冻结页给出错误值 33≠34、重标签给 5≠−5——CT-01 关闭谓词「Named/Pos/MovePos 同源 CT 独立值」未满足，见§9 review 收据与 r1-probe.log。）
- [x] AC-03 / work：消费者、原测试、恢复/重试/失败责任保持，完成适用门、Spec准备及1002依赖分流，未闭不冒PASS。（2026-11-17：冻结并集 ace-comptime/ace-av/ace-native/ace-a2x 全绿，ace-semantics 1 具名既有红不判本包红、不充 PASS；SD-01/02 在工作树就位待 merge 沉淀；1002 无硬前置。）
- [ ] AC-04 / review：逐项核r2独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。
- [ ] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。
- [ ] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、实施授权及当前源 | 固定cap/公开入口、完整Named源、参数同族与FailPublishSink（probe/publish/读取）并明确真实日志/counters差分。 AC-01  A-EXIT-20261116：本行按A-required原Case闭合。 **证据**：caps/入口于 e85096628 冻结；首红 CT-01=`source.cte.bind "1 expression node(s) have no type fact"`（命名实参未遍历，F26）、CT-02=`av_reached=false`（F32）。 |
| T-02 / work | [ ] T-01决定与可区分输入 | 补相同Arg.value子树面及call/expr publish错误事实，保持真实AV单执行、依赖/root budget和权限；新增CT-01/02永久例。 AC-02  A-EXIT-20261116：本行按A-required原Case闭合。 **证据（首轮）**：`arg_value`/`arg_value_mut`/`substitute_arg` 统一发现/闭包/meta/替换/查点，product 回灌同名子树；`audit1037_ct_01` 0.41s 绿（Named=Pos=3、嵌套 2/3、meta 42、倒序异型 7、去 cap 受控拒）。 **review 复开（F-1037-R1）**：执行键 `expr_canonical` 丢弃 Named 标签与值 → 命名实参需求间误命中；须补键编码+双站点/跨编译永久例后重交审。 |
| T-03 / work | [x] T-02 | 沿真实产品回灌/AV值与service两入口失效→重试→发布→hit验证，覆盖前置拒绝/到达后错误/计数；不提前实现966完整宏。 AC-02/03  A-EXIT-20261116：本行按A-required原Case闭合。 **证据**：`audit1037_ct_02` 0.45s 绿（publish Err→av_reached=true、execute+1、freeze_publishes+0、rejections+1、ok 边界日志一条同键、无页、重执行、修复发布→hit、forbid_av 前门 false）；并集 4 组全绿 + ace-semantics 具名既有红，见§9。 |
| T-04 / review | [ ] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-05 / merge | [ ] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |
| T-06 / merge | [ ] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06  A-EXIT-20261116：本行按A-required原Case闭合；当前分类不是完成证据。 |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1未冻结CT capability、实际公开compile_scalar_ct入口、call/expr发布失败的计数与日志差分，容易使用不带权限的语法拒绝证明错规则。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

本Plan r1已钉源/Spec文件无字节差额；额外读取的现测试/设施见§4。

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原2个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。T-01真实源/输入/预期及下一步已明确；T-02仍须原T-01反例/决定完成。

2026-11-16 stage:new/revision | PLAN-1037 r3 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | F26/F32/原CT-01/02明确A；6任务/勾选/0完成与既定设计保持；源/结果不得以旧审计自报替代 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。


2026-11-17 stage:planning-status-sync | scope:1024→1027..1038修复优化路线 | source_HEAD:e85096628c468376c0c3632d832420a2e7f6d439 | 方法：逐Plan元数据/末review-merge清理收据、九个正式交付提交祖先及worktree登记只读核对 | 结论：9包交付归档、1030/1037/1038仍drafting0/6；当前建议1037→1030→1038，1024 T-05文档收尾独立；1038只读研究可提前，无新增整包硬依赖。1034头部3/6与终态收据差额按收据解释，不重开归档。原Task/AC/Case/接受范围及进度保持；未建树、修改实现/测试/配置/canonical Spec、运行ACE/构建/性能或提交/合入/清理。本轮仅状态/导航簿记，不签新的实现/独立终审PASS。


2026-11-17本轮状态同步静态核对通过：九个交付提交祖先、17份来源Plan（本轮四份计划变更前冻结；其余13份保持字节不变）、12包状态9已交/3未开工、原35项发现分配、三草案18任务/18AC/原Case、章节/围栏/新增链接与quality-route锚点一致。只修改五份计划/路线文档；未改归档/实现、未运行ACE或构建/性能；此检查不是新产品验收。

2026-11-17 stage:work | plan_id:PLAN-1037 | plan_revision:3 | outcome:pass | worktree_path:D:/autostack/.wt/lang-1037/auto-lang | branch:plan-1037-dev | base_commit:e85096628c468376c0c3632d832420a2e7f6d439 | code_commit:e6973193c21498a03035d50731da482d8a3c067d | dependency_revisions:ace-core/ace-av-core/ace-lowering/ace-source-query@e85096628（同树，无外部在途依赖） | task_ids:T-01,T-02,T-03 | evidence:下方两表+四份本地日志哈希 | blockers:none | next_responsible_role:review

**本次实际执行的测试（冻结提交 e6973193c）**

| 测试模块/实际范围 | 结果与执行/排除数 | 准备/构建 | 发现 | 测试 | 清理 | 总墙钟 |
|---|---|---|---|---|---|---|
| `cargo t ace-comptime`（-p ace-comptime-contract/-core/ace-comptime/ace-source-cte，nextest-full，排除 `::performance::`；08:59 先期身份，格式前） | PASS 135/135 执行，0 失败；2 个显式 ignored 成本探针未选 | 与测试同次构建（warm 后 1.6s 增量） | 含于总数 | 135 例 | 含于总数 | 23.07s |
| 冻结并集 `cargo t ace-comptime ace-semantics ace-av ace-native ace-a2x`（09:03–09:22，e6973193c） | ace-comptime 135/135 PASS；ace-semantics 294/295+10 skipped（1 具名既有红）；ace-av 1341/1341 PASS；ace-native 299/299 PASS（3 个 `::performance::` 探针按组规则未选）；ace-a2x 308/308 PASS（2 个 inventory 探针未选） | 并集内按组构建（含工具准备，未另计） | 未单独计 | 2377 例通过 + 1 红 | 未单独计 | 并集 1089.02s |
| 定向反例复放 `cargo nextest run -p ace-source-cte -E "test(audit1037)"` | 首红 2/2 失败 → 修后 2/2 通过（0.050s） | 增量 1.0–2.4s | 未单独计 | 2 例 | 未单独计 | 约 3s/次 |

| 新增用例 | 行为与独立 oracle | profile | 实测秒 | 结果/≤10s |
|---|---|---|---|---|
| `tests::audit1037_ct_01_named_argument_subtree_is_served_and_backfilled` | 命名与位置实参同值 3（产品回灌后无残留 hash；测试侧独立 AST 收集器核站点跨度=served 键）；嵌套命名需求 2/3；命名内 meta 42；倒序异型命名 7；去 cap 受控拒；AV 端观察 3/42/7 | debug/test（nextest-full） | 0.4096（并集）/0.049（定向） | PASS |
| `tests::audit1037_ct_02_publish_failure_keeps_the_av_reached_fact` | 真实 call/expr 成功后 publish Err：`cte.freeze/publish`+av_reached=true、ct_execute+1、freeze_publishes+0、rejections+1、ok 边界日志一条且键=被拒页、sink 无可读页、二次请求重执行、修复发布→hit、forbid_av 前门 false | 同上 | 0.4469（并集）/0.028（定向） | PASS |

**首红→修→绿（同一真实入口，非自证）**

- 首红（08:59，日志 `t01-first-red.log`）：CT-02 在 `the AV really ran before the publish failed` 处证得 `av_reached=false`（F32 复现）；CT-01 先因测试侧控制过度指定（用 `checked_module_of` 走到 capability 拒）而修正控制，随后在 `id(x: #{ 1 + 2 })` 的产品编译处红：`source.cte.bind: 1 expression node(s) have no type fact`（命名实参未被遍历→站点未服务，F26 复现）。
- 修复（e6973193c）：schedule.rs 增 `arg_value`/`arg_value_mut`/`substitute_arg`，发现/闭包/meta/替换/查点五条遍历统一按 `Arg.value` 子树（Pos/MovePos/Named）；product.rs 回灌同一子树；service.rs 两入口发布失败保 `av_reached=true`（executor.rs 文档同步）。绿：定向 2/2（0.050s），并集 4 组全绿。
- 具名既有红（不判本包红、不充 PASS）：`ace-semantics-store::conformance sema_boundary_audit::sema_boundary_case_files_executed_status_honest`（0.029s）。其输入 `crates/auto-lang/test/cases/ac-semantics/**`（含 PLAN-1062 在 c8ed52191 引入、aeeecc8f3 恢复的 `*.document.case.ad`，不带 `status: executed`）不在本 diff；`git diff --name-only e85096628 e6973193c` 仅 7 文件（source-cte 源/测试 + 2 份 CT Spec），不含该语料与 semantics-store，故在基面同输入同谓词下必同样红。归属：PLAN-1062/1066 文档语料与 944 系审计谓词的交互；review 阶段登记债并具名交接。
- 机器占用：并集前检查为空闲，运行期间观察到其他会话的 `cargo check/test -p auto-web`、`a2r_std`、`lang-984` 的 ace-abi release 构建并发（无源交集）；本会话未停止他人进程。

**并发编辑对账**：08:59:25 另一 planning-status-sync 会话在本文件追加 §0 路线状态、§5「路线排程与当前进入条件（2026-11-17）」及两条 §9 收据（其“尚无登记的实施树/0完成”写于本包 08:47 建树之后，未见本轮事实）。本节保留其全部内容、不改写其收据，以本收据与本轮事实为准（§9 只追加）。

**证据（plan-artifacts，merge 折叠后删除）**：t01-first-red.log 6ec202eeb1417483a8c1f57bb23de8b60f723e4de674813d4aa60e4ee2c624ec；t02-first-green.log 07f29c5521e308445ec6bb55bcecf55b249927ba0d7820cfd78e6d6e90ed706f；t03-group-ace-comptime.log 454daac782e0ff5ea96b811f7a2fb59af6c172b0fe370527f1cc49a2ee0486a2；t03-union.log 6bd113f6d292d39c39186b37490a207b10e09f8a590fe779f4d7e6b4cdde4af1；并集 results.json＝target/test-groups/run-hagiipq4（随树删除）。

2026-11-17 stage:goal | plan_id:PLAN-1037 | plan_revision:r3 | round:1 | from:work | to:review | status:execution_done | outcome:pass | worktree:D:/autostack/.wt/lang-1037/auto-lang | commit:e6973193c21498a03035d50731da482d8a3c067d | next:review

2026-11-17 stage:review | review_scope:final | reviewer_identity:实现同会话（Plan §6 明示无另起 agent/独立模型授权；结论一律由产物重构，不采信执行者摘要） | plan_id:PLAN-1037 | plan_revision:3 | outcome:needs_fix | worktree_path:D:/autostack/.wt/lang-1037/auto-lang | branch:plan-1037-dev | reviewed_commit:e6973193c21498a03035d50731da482d8a3c067d | base_commit:e85096628c468376c0c3632d832420a2e7f6d439 | dependency_revisions:同树 e6973193c（无外部在途依赖） | spec_inputs:docs/specs/auto-cte/source-query-integration.md@e6973193c（+19 行：§3 发布失败事实、§5 实参子树）、docs/specs/auto-cte/project.md@e6973193c（+9 行：观察事实分账） | acceptance_results:AC-01 pass（入口/caps/首红→绿可复现）；AC-02 **fail**（F-1037-R1）；AC-03 pass（并集 4 组全绿，ace-semantics 具名既有红）；AC-04 由本条承担；AC-05/06 merge | pending_closeout_gates:repair 后重跑受影响并集；SD-01 修文；merge-owned 冻结门 | findings:F-1037-R1（in-contract，高） | evidence:r1-probe.log（独立探针，仓外）+本节摘要 | next:work（有界 repair）

**F-1037-R1（in-contract，高）命名实参未入执行键 → 跨需求误命中冻结页（错误编译期值）**

- 违反的既有规则：本 Plan CT-01 关闭谓词「Named/Pos/MovePos 同源 CT 独立值3、无未服务 hash」；A-EXIT 关闭谓词同句；SD-01「named 需求的键独立」。
- 已确立根因（代码+复现，非推测）：`identity.rs::expr_canonical` 的 `ExprKind::Call` 分支把 `Arg::Named { .. }` 编码为单个 `b'?'`（标签与值都不进 canonical）；`exec_key_of_expr` 以该 canonical 作为 `callee_sig_version`/`callee_body_version` 轴 → 仅命名实参内容不同的两个表达式产生**同一执行键** → 第二次需求 `probe` 命中第一次的冻结页，返回前者值。
- 复现输入/期望/实测（独立探针 `D:/autostack/plan-artifacts/plan-1037/probe`，仓外、不复用被测测试与 oracle；src/main.rs SHA256 f4821ec9128799bb844b8be73802e24fbbac08e19046fb9b18270394173738fa；warm 复跑 0.15s，exit=1 为反例所致）：
  - A 站点内双需求：`fn id(x int) int { return x }  main { let a = #{ id(x: 3) }; let b = #{ id(x: 4) }; return a*10+b }` → 期望 34，实测 **33**（execute=1, hits=1, served=2；第二站点命中第一站点页）。
  - B 跨编译重标签（同一 CtService）：`#{ sub(a: 9, b: 4) }` → 5（正确）；随后 `#{ sub(b: 9, a: 4) }` → 期望 −5，实测 **5**（hits=1）。
  - 合法对照（同探针）：位置实参双需求 → 34（execute=2, hits=0，命名专属，非形状问题）；`id(x: #{1+2})` → 3（F26 遍历修复有效）；F32 事实链 → code=cte.freeze/publish、av_reached=true、失败后 execute=1/freeze_publishes=0、修复后 publish=1 且 hit（**SD-02 表述准确**）。
- 受影响面：所有「`#{ }` 内直接调用带命名实参」的 CT 需求（含 a2x-worker daily-emit 同源消费）；Pos/MovePos 不受影响；`product`/`schedule` 的遍历修复本身成立。
- 修正方向（work 选等价实现）：`expr_canonical` 对 Named 编码标签+值（如 `named:<label>=` + 值 canonical），Pos/MovePos 现状不变；既有命名页因此重键（旧页本为共享/错页，重键是安全方向）；新增永久用例「双站点同标签异值」「跨编译重标签/换序」「位置对照」；修正 SD-01 中 “keys by its expression like its positional twin” 一句使之与实际一致。
- 覆盖边界：本 finding 只裁「hash 内调用带命名实参」这一标量可达形状。非标量面形状（Method/CallValue 等）在 `expr_canonical` 落 `other:{:?}`（含 span）→ 保守 miss，不产生误命中，列为非阻塞观察，不在本包扩面。
- 同时记录的既有红（非本包）：`ace-semantics-store::conformance sema_boundary_audit::sema_boundary_case_files_executed_status_honest` —— 输入 `crates/auto-lang/test/cases/ac-semantics/**.document.case.ad` 在基面 e85096628 已存在（PLAN-1062 c8ed52191 引入），本 diff 不含该语料/该 crate；登记为 D-1037-01（owner 1062/1066），不判本包红、不充 PASS。

**本轮实际执行（review 侧，未重跑无关全量；work 并集证据在同身份下复用）**

| 范围 | 结果 | 说明/计时 |
|---|---|---|
| 独立探针 `probe1037`（5 case，仓外） | 3 pass / 2 fail（预期反例） | 首次构建时长未单独观测（与并发会话共用机器）；warm 复跑 0.15s |
| 代码审计：source-cte 全部 `Arg::Pos/MovePos/Named` 落点 | 8 处，除 `walk_meta` 选择器提取（与 resolve `meta_query` 一致要求位置式）与 `direct_call_args`（命名→表达式入口，值正确）外均为完整匹配 | 只读 |
| 既有红复核 | `sema_boundary_case_files_executed_status_honest` 在基面同输入同谓词下必红（corpus 于 e85096628 已存在） | 只读 + Git 树核对 |
| Spec delta 复核 | SD-02 准确；SD-01 含一处过度声明（键独立）→ 随 F-1037-R1 修文 | 只读 |
| 健康检查 | 无新增告警（唯一告警 `mod performance { use super::*; }` 为基面既有）；无 dbg/println/TODO 入 diff；fmt 通过 | 只读 + `git show` |
| 复用证据 | work 阶段冻结并集（e6973193c：ace-comptime 135、ace-av 1341、ace-native 299、ace-a2x 308 全绿）在实现/依赖/配置未变下仍有效；repair 后须重跑受影响组 | 不重复执行 |

2026-11-17 stage:goal | plan_id:PLAN-1037 | plan_revision:r3 | round:2 | from:review | to:work | status:executing | outcome:needs_fix | worktree:D:/autostack/.wt/lang-1037/auto-lang | commit:e6973193c21498a03035d50731da482d8a3c067d | next:work

## 10. 待澄清事项

只修当前scalar-CT可达子树与事实；不提前实现完整M2宏/条件编译；与966相关改动同文件单写对账。 T-01使真实反例/资源或IO故障可运行，未取得前不声称动态后果已证。等价实现不需重复审批；新支持/预算/公共合同变化先最小决定，不能用隐式门限或延期替原义务。不等待“新测试框架全部完成”。

本轮T-01入口已具体化，当前不需要用户再次确认等价内部实现。未取得真实动态反例的部分仍由本节与§6指定入口/邻例核证，不把预期当已观察结果。
