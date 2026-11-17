---
plan_id: PLAN-1038
status: archived
feature_name: ACE复用候选、成本与模块边界有界研究
author: [Codex]
created_at: 2026-11-13
updated_at: 2026-11-17
plan_revision: 3
current_step: 6
total_steps: 6
depends_on: [PLAN-1024]
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [ACE]
---

# [PLAN-1038] ACE复用候选、成本与模块边界有界研究

## 0. 变更摘要

2026-11-17路线状态联络：本包仍drafting r3、0/6，尚无登记的实施树；本次只更新状态与排程，不推进Task/AC。当前位置为**第3：B有界研究（P2/F15 P3）**；总表见[1024当前路线状态](1024-ace-m2-code-quality-and-refactoring-audit.md#quality-route-20261117)。9个同路线包已交付归档，不再照2026-11-13旧队列重复开工；实施合同/Case/独立oracle保持。

2026-11-13 r2当前：用户要求对1028以外计划逐份核是否足够定制。本包补真实输入、状态/故障设置、独立关闭断言、测试落点、任务改面和Spec具体增量；原发现/AC/Case/T-ID及6任务、drafting 0/6保留。只做规划与静态源码核对，不修改实现/测试/配置，不接管1028实施，也不宣称动态反例或产品通过。

承接[1024](1024-ace-m2-code-quality-and-refactoring-audit.md)的F04/F07/F09/F11/F13/F15，优先级**P2（F15为P3）**。本轮仅立规划合同，drafting、0/6；未建树、未写实现/探针/fixture、未构建或运行ACE。用户当前仅授权复审与规划，不授权本包代码实施。本包交付六项有界研究决定与后继合同，不实施重构。

2026-11-16 r3阶段裁定：本包归B，不作为985/A整包前置；原6任务/0完成、Case和独立关闭条件保持，当前仅规划。

2026-11-17 r3执行（/goal 完成计划1038，work）：T-01/T-02/T-03 完成，3/6。六项决定按当时源重核后出齐：D04 go（首个有界切片合同见§9 T-03）、D13 go（单一私有模块首片，次序列）、D15 go（纯文档更正）；D07/D09/D11 retain/no-go 并具名理由与重开前置。全部为只读研究与 Plan 簿记，未改实现/测试/配置/canonical Spec、未建树、未运行 ACE 或性能门，无产品 PASS。

2026-11-17 r3归档（/goal 完成计划1038，merge）：T-04 独立复审 pass@r3、T-05/T-06 合并/归档/清理完成，6/6；本文件已移入 `docs/plans/archive/` 并置 `status: archived`，SD-00 无规范增量的 approved no-impact 结论与关闭收据见 §9 T-05/T-06。

## 1. 目标

复用收益必须有真实成本/合同依据；共享机制不合并独立oracle、不丢目标差异/凭证门/ABI/取消/预算。

非目标：扩大语言/ABI/平台支持、默认切换、删除oracle、降低样本/规模/原条件、重写全ACE或接管其他Plan。1002试点不阻只读研究；涉及作者/执行接线的实际重构须1002接口稳定。1017同路径成本基线先冻结；本包不擅跑ace-perf或修改实现。

## 2. 架构方案

本包只交付六项go/no-go/保留决定和首个可实施切片合同，不实施跨模块重构。F04可先定52行算术机制；F07测parse/resolve/emit/bind/reanchor；F09核正常依赖/宿主；F11核no_std ABI/hooks；F13仅私有模块边界；F15据现Spec修注释的方案。若需L2改造先形成正式设计及独立实施Plan。

沿1024真实producer→consumer分层，恢复不授信，source/typed、guest错误/Resource/Cancelled/Infra分账。不因共用文件扩大范围。等价机制由work选择；公共语义/支持/跨核心合同变化先具名设计及修约，不能临时加深度/规模限制。

## 3. 技术栈

Rust、现固定组/原TestCase及Markdown。仅语言仓。后续唯一树D:/autostack/.wt/lang-1038/auto-lang，建议分支codex/plan-1038-dev，尚未创建；work/review/merge顺序使用同树，禁止链接。材料唯一D:/autostack/plan-artifacts/plan-1038/，创建即登记owner/用途/状态，当前无scope。

## 4. 需求分析与背景调查

授权：2026-11-13用户要求先复审1024，再逐发现分配到已有/新有界计划，明确优先级与1002关系，暂不改实现。无指定预算或自动施工授权。r1固定源Git=81d8fc03f10f4a7d179b0bc3a917602406ed7b91（历史）；r2当前取材Git=c2760324da25795594ab0cc01b581e3a306d2375，具体文件身份以本节SHA为准；1024 r2审查包与源码分开绑定，开工重核配置/输入/来源漂移。旧反例是静态证据，不是运行日志。

2026-11-17 work 开工重核：本节 r2 表与差额行保留为历史快照，**当前身份与漂移以§9 T-01表为准**（冻结时源 HEAD `decfaf4e…`；T-01 的19项身份中13项相对 r2 已变、4项同、2项为本轮新增冻结）。工作树：本包只读研究+Plan 簿记，**未建实施树**（§3 的 `lang-1038` 树只属于未来有界实施项）；主检出唯一代码路径未提交改动是他人 `a2x-worker/tests/cases/sched_m2_968.rs`（+69行 probe），非本包产物、未触碰。

已读docs/specs/overview.md作导航；当前规则以module Specs为准，原始证据及测试入口如下：

| 已核源路径 | 字节SHA256 | 状态 |
|---|---|---|
| [experimental/ace/crates/native/src/collections_lower.rs](../../../experimental/ace/crates/native/src/collections_lower.rs) | 844b233e0b04f2a3566a5ee748bc8dce6c3e8dfacefabf1dd12dc08c5182b5aa | 静态取材，非运行证据 |
| [experimental/ace/crates/native/src/closures_lower.rs](../../../experimental/ace/crates/native/src/closures_lower.rs) | 6b5345401301b1dc64c5e6dfc12cbd4736bf6e5df337c431778d924b6585e438 | 静态取材，非运行证据 |
| [experimental/ace/crates/core/src/source/adapter.rs](../../../experimental/ace/crates/core/src/source/adapter.rs) | 64fb2234a0c39875e4100fb873f6b1ef696f3d180d585072c2060931553d4300 | 静态取材，非运行证据 |
| [experimental/ace/crates/core/src/link.rs](../../../experimental/ace/crates/core/src/link.rs) | 7e19a7d49894d30a4d20297c36690559e5dc3dff70f15922f54fa1a361cda0d0 | 静态取材，非运行证据 |
| [experimental/ace/crates/data-runtime/src/lib.rs](../../../experimental/ace/crates/data-runtime/src/lib.rs) | 4c2327b22406e09b24f46edadcf1b1deb1f5fa95db9de980c283470d948d5226 | 静态取材，非运行证据 |
| [experimental/ace/crates/text-runtime/src/lib.rs](../../../experimental/ace/crates/text-runtime/src/lib.rs) | 5ec6d366ef69f223a73b6603de6559f04440cf2e80a56b9f535682fff4222b5e | 静态取材，非运行证据 |
| [experimental/ace/crates/core/src/source/resolve.rs](../../../experimental/ace/crates/core/src/source/resolve.rs) | 80eb12467732dd657008918c90bf36158d52d6bbea63bc87e7c511a698263c10 | 静态取材，非运行证据 |
| [experimental/ace/crates/core/src/verify.rs](../../../experimental/ace/crates/core/src/verify.rs) | cba93ead10ab8ab01ddbdd91e615453870d2fae7b851666c110fc5348baf3632 | 静态取材，非运行证据 |
| [experimental/ace/crates/av-core/src/mir_verify.rs](../../../experimental/ace/crates/av-core/src/mir_verify.rs) | 9f1427164bd6a54f9d7b6d5893b7ebd649af1975a4e12dfc191d91a12f69d1d3 | 静态取材，非运行证据 |

| Spec来源 | SHA256 | 适用边界 |
|---|---|---|
| [docs/specs/auto-ac/native-backend-and-retirement.md](../../specs/auto-ac/native-backend-and-retirement.md) | 77ba8a3c2fa19128b7bbc958d21c5d928de788157daa39b6978b9d8fb0731a78 | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-a2x/architecture.md](../../specs/auto-a2x/architecture.md) | 9984e8854b3bd7e9fbd00a195154a0c62e78861c2d46a3121b64b188c350cfdd | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-test-case/architecture.md](../../specs/auto-test-case/architecture.md) | 2e37d29c0df993f95809bf262af6a53b4b53acebf353453dcfc3e1626bc89fe4 | 当前规则保持；开工重读核漂移 |
| [docs/specs/auto-av/architecture.md](../../specs/auto-av/architecture.md) | 05bd373cfff3d9901c5a790786ad5152842d2360ff2f2723aa23fb81322a0282 | 当前规则保持；开工重读核漂移 |

源码/Spec是实际规则依据。1024 M/S/D/L/A/N/U为保留账，不代Rust reader/nextest发现；同文件在途修改串行单写，正式合入来源重核，不用他Plan WIP当已交付依赖。


### r2来源差额与执行入口预检

当前取材c2760324da25795594ab0cc01b581e3a306d2375；r1关键文件无差额的证据可保留，实际有差额者已回读并按本轮合同修正。预检发现：r1六个候选共用少数抽象验证行，缺少每项明确的样本、决定产物、go/no-go标准及成本unknown处理，容易变成无界模块拆分。

| 现有测试/补充实现入口 | SHA256 | 本轮用途 |
|---|---|---|
| experimental/ace/crates/native/tests/cases/closures_native_tier.rs | b13c15acdff363589738ee73fcfd90aefe181ebf8697ddc417d7f7d922603d12 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/av/tests/cases/av_collections_session.rs | bd116666026a7a4cfb6bbb2a8fe33e4c4ec3bf407b3bb5a0a0845020c31fa2b2 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/data-runtime/src/lib.rs | 4c2327b22406e09b24f46edadcf1b1deb1f5fa95db9de980c283470d948d5226 | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/text-runtime/src/lib.rs | 5ec6d366ef69f223a73b6603de6559f04440cf2e80a56b9f535682fff4222b5e | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/artifacts/src/subproc.rs | a5a1d17a62da34f9556b0632ead5a3482a93b3cb6f99cfc24dab6c6a179f09af | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |
| experimental/ace/crates/core/src/lib.rs | 906442f4d5f2ab0f585c37e7ce44965914bf521e706e7a62fc0ca648482ea48d | 已存在；T-02在原模块扩展或接入，不冒已创建测试 |

1028已由用户启动实施，本轮未写其计划或工作树；不要仅因其主检出frontmatter尚未更新就判断未开工。与它共享的计量结果在后续验证时绑定实际已合入SHA，本包不新增1028整包硬依赖。963/947等新源与本批修复共用位置时按正式合入基线串行重核，主检出其他文档WIP保留。

本轮授权与来源（2026-11-16）：用户只要求重冻985/A验收合同、核1024及审计后继和1066的A/B边界，明确任务/关闭条件并更新A顺序；不启动实施，不新增Plan。读取主线107b23d0f69c6ad11d036dc37bed0b688493db30、当前Spec导航/相关模块合同、1024 F01..35/S01..13、1027/1028归档收据、1029..1038和1066当前合同及实际源码消费者。静态风险不是已经复现的bug；A行须真实修复后验证或可复核充分反证，未执行保持pending，不能以本轮分类当PASS。原资源/平台/性能/ignore与UI三波边界保持。

## 5. 详细设计

唯一范围=F04/F07/F09/F11/F13/F15。决定行必须连接“候选→现代码/合同→真实consumer→证据/unknown→go/no-go/保留→有界后继合同”，承接登记不算研究完成。T-01使§6六项决定输入与原入口可复核，不要求实现尚未批准的抽取。

数据/错误/责任：复用收益必须有真实成本/合同依据；共享机制不合并独立oracle、不丢目标差异/凭证门/ABI/取消/预算。 本包只交付六项go/no-go/保留决定和首个可实施切片合同，不实施跨模块重构。F04可先定52行算术机制；F07测parse/resolve/emit/bind/reanchor；F09核正常依赖/宿主；F11核no_std ABI/hooks；F13仅私有模块边界；F15据现Spec修注释的方案。若需L2改造先形成正式设计及独立实施Plan。 confirmed/candidate/rejected/unknown沿1024分开；充分反证需保源码/合同依据，不能删分母。当前测试与新增反例分别保完整身份、源hash、expected、条件；共享机制不共享被测oracle。


### r2执行合同预检（当前优先）

**输入到消费路径。** 本包仍只研究/形成有界后继合同，不修改实现或测试。不要求为六项全部跑runtime/压力；能从当前源、Spec和既有断言判断的先直接做决定。成本取样若确有必要，未来在专属树与统一scope用既有固定输入/命令短取样，1017 r5同路径基线单独对账，不能从其Formal总时长猜本项收益。

**状态、故障与责任。** 全部六项都必须有决定，不能只复述清点；不适合抽取的no-go或保留也是有效交付。go至少要求等价边界/独立oracle可验和一个有限改面；性能收益需真实同输入证据，否则明确unknown。给首个go项完整修复/重构Plan合同；若涉及公共IR/ABI/跨核心架构先正式设计，不能在研究中实现。D15文档更正也由后继有界任务执行，不在本包顺手改实现注释。

**先决决定边界。** 上述固定输入与原支持域可从T-01直接核证；T-01仅选择等价局部机制，不把所有设计留到实施。若真实入口与假设不符，必须给可重放反证和合法邻例后修约，不能仅写unknown或删除原Case。

**范围与执行门。** 当前仍只规划；T-01/02/03分别按本节和§6执行，T-04逐Case核证，不把正确性留给merge。G1002只约束改写新作者格式/相关接线，不约束原入口反例。任何新增limit或公共支持变化按本节具名决定，不推定已批准；等价内部实现不需重复许可。

### A/B裁定：复用/拆分研究在B，约束核验仍在A

本包F04/F07/F09/F11/F13/F15六个原go/no-go/保留决定全部B，可与产品路线按资源并行，不是985/A前置；本包仍只研究，不在研究中实现重构。T-01/02/03取得真实源/consumer/成本或unknown、独立oracle保留与有限改面，T-04独立核六决定、T-05/06交付；no-go/保留是有效关闭，不强行创造重构工程。没有真实收益证据不能宣称性能改进。

1024 F03/F05/F08及F12在A由985 T-A03核已交969/970/978的结构诊断/profile/会话/取消/ABI与既定执行器约束；这不要求A先完成抽取/并库/拆crate。若后续确需结构改造，其边界/保留约束记录在本包原REUSE-02/03、D09/D13的研究材料里，不能以归档969/970为“未来会修”的执行owner；新代码只在正式有界合同授权后实施。本包研究不是这些约束已通过的证据；实际A约束若红，985拒签，不能用B研究登记抵消。


### 路线排程与当前进入条件（2026-11-17）

本包交六项go/no-go/保留决定而非实现，默认在1037正确性与1030历史修复之后交研究结论；纯只读调查可提前并行，不新增1030/1037整包hard dependency、不抢实现写窗。T-01必须以1017正式成本合同及已交1028/1029/1034/1035/1036等当前源重核重复/收益假设；旧计数/拆分建议不当新收益证据。no-go/保留可按原oracle关闭；go须后继有界实施合同，禁止在本包顺手抽模块/改注释/ABI或擅跑ace-perf。

本轮只做计划簿记；原T-01..06、AC-01..06、Case及现阶段分类不变，plan_revision仍r3。正式实施从当时master建立本包唯一树并核源漂移/可用consumer，不凭本状态快照跳过T-01。Windows不运行Linux/WSL腿且不记PASS，Linux义务保留给专用环境；常规正确性与显式ace-perf分开。

### 与1002及其他计划的依赖

- 本包调查、原入口回归与核心修复/研究不以1002为硬前置。只依赖1024审计交接；不等待1002全语料迁移或985终验。
- G1002=1002 T-03四例七检查及负控通过，T-04独立复审且读取/比较接口已正式合入并绑定SHA。只有新作者格式迁移或相关document/IR/adapter接线重构须等待G1002；Rust/旧case回归可先交付。
- 1002不反向等待本包全完。若本包真实反例影响其七行基线，只对受影响行具名先修/同源重跑，不抹真实红。
- 1017同路径计量/执行基线先协调冻结输入/构建身份，不等待整包、不擅跑ace-perf。能力计划只消费已审已合入修复，其M2义务不自动通过。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-00 | 无（研究） | 无 | 不改canonical Specs；go项后继实施另声明增量，跨核心变更先正式设计并注册 | 六项决定不冒交付行为 | AC-03/04 |

本轮不改canonical Specs。frontmatter是拟议影响，review按实际交付收敛；touched_goals空，工程质量不擅扩产品目标。

## 6. 测试设计

本包只研究：git diff --check及六项决定/路径/hash守恒。若成本取证需探针，先在专属树与统一scope明确有界命令；不默认跑ace-perf。T-01核原测试入口与断言，六条发现各形成可复核的决定输入；本包不要求创建产品fixture或运行修复回归。原样本/规模/平台/ignore/正式性能轴保持，后继实施保留这些义务，不追加全仓/ace-perf。

| Case ID | AC / work owner | 区分性输入/设置 | 实际入口 | 独立关闭oracle | 当前状态 |
|---|---|---|---|---|---|
| REUSE-01 | AC-02 / T-02 | 两份binary完整体与原算术/cleanup用例 | collections_lower/closures_lower真实consumer及独立oracle | 只共享相同机制；Div/Rem/MIN/-1/溢出、closure/frame差异有保留表 | 决定已出（§9 T-02）；未运行，无产品PASS |
| REUSE-02 | AC-02 / T-02 | 同固定输入的阶段分配/时间、真实Cargo依赖；no_std两库符号/hooks/错误码 | adapter→descriptor bind；core link/现executor；data/text ABI consumer | 收益unknown如实保留；每项给go/no-go与理由，不用文件长/同源当性能证明 | 决定已出（§9 T-02）；未运行，无产品PASS |
| REUSE-03 | AC-02 / T-02 | 六项候选逐个保留/拆分决策，陈旧注释与Spec对照 | 本Plan六行决定表/后继合同 | 六项均有源hash、consumer、保留oracle、成本证据或明确unknown、实施边界；无未分配候选 | 决定已出（§9 T-02）；未运行，无产品PASS |

代表性路径/失败状态不冒全笛卡尔覆盖。发现/执行/原条件排除、准备/冷编译/发现/运行/清理、首次红/环境归因/重放绿分别记；父exit0不代子终态。当前未运行，无产品PASS。

角色：后续获授权work实施；review独立核证（无指定模型/另起agent授权）；merge收尾。使用同一实施树，本轮规划自检不冒实施review。


### r2具体输入与状态断言（当前，全部待实施）

固定六个决定D04/D07/D09/D11/D13/D15，每项一行：源hash→真实consumer→同/异合同→保留oracle→成本证据或unknown→go/no-go/保留→首个可实施切片。数据持久形态直接在本Plan§9，不生成第二registry/reports。

| 原Case ID / 子面 | 实际设置/适用状态 | 独立关闭断言 |
|---|---|---|
| REUSE-01 / D04 | collections_lower/closures_lower的完整binary体、Div/Rem/compare/trap及frame/capture调用上下文 | 确认仅共享同机制；独立算术/cleanup oracle及目标ABI保持，首片不合lowerer |
| REUSE-02 / D07 | source adapter parse/resolve/emit/bind/reanchor的cold/同源hit/单body三输入 | 如需测以同一源/构建分配和时间归因；无可归因收益则保留或仅设计，不承诺typed builder加速 |
| REUSE-02 / D09 | Cargo正常依赖、core link与artifacts/subproc现委托、公开fixture/bench消费者 | 区分compile dependency与最终链接；先看已有executor接缝，不新建反向依赖/隐藏正式测试 |
| REUSE-02 / D11 | data/text各extern ABI、错误码/allocator hooks、metadata/取消与no_std符号 | 同源不等于同合同；没有符号/预算/取消保留方案就no-go，不靠libc并库 |
| REUSE-03 / D13 | resolve/verify/execute/project/adapter/mir_verify六热点的规则family和public凭证边界 | 先选择单一crate私有模块边界，原API/授信owner不变；仅行数无收益证据不得推广拆crate |
| REUSE-03 / D15 | core/native/av-core头注释对照Cargo与当前Spec | 只决定需要更正的事实和位置，保旧Plan/wire身份；不因旧注释判依赖违规 |

六项决定直接落本Plan§9，不生成fixture/新测试函数；REUSE行的oracle是逐源码/合同/证据可复核的决定，不要求运行未实施的抽取代码。首个go的后继合同才拥有实现/永久回归。

## 7. 验收标准

- [x] AC-01 / work：T-01冻结六项真实源、支持/资源边界、成本取证方法和原消费者/测试入口，所有§6决定输入可复核，重大未决具名，不私改expected。（证据：§9 T-01 19项当前身份/漂移表，冻结窗口 HEAD `decfaf4e…`；无 expected 改动）
- [x] AC-02 / work：r2具体输入/每项状态断言及§6实际测试落点均落实；§6每行独立oracle已取得；六项决定各有源/consumer/保留表，明确成本及unknown，无实现改动，原义务不减。（证据：§9 T-02 D04/07/09/11/13/15 六行决定 + D04 保留表；i32 陷阱 oracle 缺口具名）
- [x] AC-03 / work：六项决定逐一列明消费者、原测试、恢复/失败责任及1002依赖；首个go项有可执行有界后继合同，no-go/保留项有理由；没有把研究或成本unknown冒实现/收益通过。（证据：§9 T-03 D04 合同；D07/D09/D11 重开前置；D13/D15 边界与序列）
- [x] AC-04 / review：逐项核r2独立预期、后续使用闭环与具名决定，不能仅信组绿；逐AC/Case/代码/Spec独立核验、遗漏/延后/workaround及健康/来源检查通过，结论绑定revision/冻结提交。（证据：§9 T-04 独立复审，19项身份逐项重核、六项决定逐条重建、R-1038-1..3 非阻塞发现；结论 pass @ r3）
- [x] AC-05 / merge：review后冻结并集门/正式合入、规范沉淀及归档真实完成。（证据：§9 T-05/T-06 合并收据——SD-00 无规范增量的 approved no-impact 结论、归档路径与 status:archived 已入库；本包无代码变更故无固定组并集门，按 docs-only 内容检查）
- [x] AC-06 / merge：所有输出路径对账，摘要折Plan，scope删除或具名持有，guard clean后清树/分支。（证据：无实施树/分支、无 plan-artifacts scope 可清；registry 无 1038 持有；主检出他会话 WIP 未纳入）

## 8. 执行步骤

| task / owner_stage | 依赖 / 状态 | 操作、交付与验证 / AC |
|---|---|---|
| T-01 / work | [x] 1024交接、未来研究授权及当前源 | 冻结六决定的指定输入/来源/consumer/原断言和是否需成本测量；公开事实可直接核，成本若无有效同源证据记unknown。 AC-01（2026-11-17完成：§9 T-01 表；19项身份/漂移；成本测量方法=1017同路径归因，本包不跑 ace-perf） |
| T-02 / work | [x] T-01决定与可区分输入 | 逐项完成D04/07/09/11/13/15 go/no-go/保留，交同异合同与独立oracle/源hash，不创建模块或第二基础层。 AC-02（六项决定已落§9，D04 保留表含 i32 与 typed 分界；oracle 缺口具名；无模块/实现创建） |
| T-03 / work | [x] T-02 | 交首个go项完整有界后继合同，或六项无go的具体理由；对照1002/1017及本批正确性修复新源核失效，做纯文档验证。 AC-02/03（D04 合同见§9 T-03：切片/不变量/oracle/门禁/退出条件；D07/D09/D11 重开前置已列） |
| T-04 / review | [x] T-01..03完成、冻结提交 | 独立核AC-01..03、§6/SD/漂移、遗漏延后绕行及债务；失败回同树work不缩scope；AC-04（2026-11-17 完成：§9 T-04 复审 pass@r3，19项身份重核一致、六项决定逐条重建、R-1038-1..3 非阻塞且不改决定） |
| T-05 / merge | [x] T-04 pass | 核冻结并集且复用同身份证据，正式合入、规范/归档；正确性不留merge；AC-05（2026-11-17：docs-only 交付，无固定组并集门；SD-00 无规范增量；`docs/plans/archive/1038-…md` status:archived 已入库） |
| T-06 / merge | [x] T-05 | 所有实际输出对账、摘要/删除/持有登记；wt-guard clean后清树/分支；AC-06（无树/分支/scope 可清；registry 无 1038 持有；他会话 WIP 未纳入） |

同文件串行；跨计划调度见1024分配表。草案stage:new pass不授权开T-01构建/探针。

## 9. 复审记录

2026-11-13 stage:new / r1 / outcome:pass（可审阅规划，非产品/独立实施review） / next:获实施授权后work。new-plan.sh中央顺序取号，源/Spec路径与组存在，Task/AC/Case/SD明确；drafting 0/6。未建树/材料、改实现/测试/配置、构建/运行、提交/合入/tag/归档。


2026-11-13 stage:new / r2 / outcome:pass（执行合同预检，可从T-01开始；非代码/独立实施review） / next:后续实施授权和当时基线重核后work。r1六个候选共用少数抽象验证行，缺少每项明确的样本、决定产物、go/no-go标准及成本unknown处理，容易变成无界模块拆分。 已补§4真实差额、§5边界/故障设置、§6完整输入/独立oracle/目标落点、§8前三项定制操作、SD具体规则；原AC/Case/T-ID及6任务/drafting0进度保留。

r2所读源Git=c2760324da25795594ab0cc01b581e3a306d2375；r1已存在的源码差额：

| 路径 | r1字节SHA256 | r2字节SHA256 |
|---|---|---|
| experimental/ace/crates/core/src/source/resolve.rs | 24441ead81f35f8f3928183edc3e371dd478162b7ba6ee531dffb782d7e3d0df | 80eb12467732dd657008918c90bf36158d52d6bbea63bc87e7c511a698263c10 |

本次只写本Plan，无实现/测试/配置/Spec/执行改动；未建树/材料/运行/取号/提交/合入。1028不在修改集合，同机其他计划WIP保留。


2026-11-13 r2目标文档终检：本批11份计划原66个任务/AC编号和29个Case ID保留；本Plan原3个Case不删减。共核114条当前源/Spec/测试入口hash，本批26个拟新增测试锚点只作未来T-02落点，未创建/未执行。各Plan 0–10章节、围栏、本地链接、实际组名、r2具体输入/断言、原任务顺序/owner_stage及drafting0进度通过。1028计划字节hash与本轮入场一致，本会话未写其实施树；并发其他计划WIP保留。结束HEAD=9a7674009725913f06f44b8176ae20c8aad2c455，c2760324→结束HEAD的本轮源/Spec/组范围diff为空，无相关源码WIP。git diff --check与新文档空白检查通过；这些均为静态规划检查，不是ACE功能/资源/性能PASS。只具备开始六项研究决定的合同；不是批准实施模块抽取，后继go项仍须有界实施合同。

2026-11-16 stage:new/revision | PLAN-1038 r3 | outcome:pass（规划分类/静态合同，非产品或A出口PASS） | B承接及A边界核验明确；6原任务/勾选/0完成保留，不取消原case/角色，不提前签修复/研究完成 | next:原owner未来获实施指令后按现合同执行；当前仅规划，不建树/写代码/运行测试/合入/清理/恢复目标。

本轮规划静态预检（非产品PASS）：18份规划/路线/债务文档；13份Plan共92个原任务，任务ID/勾选/状态/完成数/整包依赖保持。985本轮162父行分类索引唯一、F01..35唯一且全覆盖（20条A正确性、6条A约束/证据、2条已交复用、7条B）；新增本地链接/锚点、章节、表格、阶段/任务/原Case映射与行尾核对通过，阶段图34节点36边无环，1066的A终验不再依赖B-only格式/项目/250编译任务。仅验收合同冻结；真实矩阵/adapter/新控制及全部新产品结果仍待原work任务，未运行/签A。未改源码/测试资产/canonical Specs/取号计数器，未实施/建树/构建/窗口/合入/归档/清理或恢复目标。


2026-11-17 stage:planning-status-sync | scope:1024→1027..1038修复优化路线 | source_HEAD:e85096628c468376c0c3632d832420a2e7f6d439 | 方法：逐Plan元数据/末review-merge清理收据、九个正式交付提交祖先及worktree登记只读核对 | 结论：9包交付归档、1030/1037/1038仍drafting0/6；当前建议1037→1030→1038，1024 T-05文档收尾独立；1038只读研究可提前，无新增整包硬依赖。1034头部3/6与终态收据差额按收据解释，不重开归档。原Task/AC/Case/接受范围及进度保持；未建树、修改实现/测试/配置/canonical Spec、运行ACE/构建/性能或提交/合入/清理。本轮仅状态/导航簿记，不签新的实现/独立终审PASS。


2026-11-17本轮状态同步静态核对通过：九个交付提交祖先、17份来源Plan（本轮四份计划变更前冻结；其余13份保持字节不变）、12包状态9已交/3未开工、原35项发现分配、三草案18任务/18AC/原Case、章节/围栏/新增链接与quality-route锚点一致。只修改五份计划/路线文档；未改归档/实现、未运行ACE或构建/性能；此检查不是新产品验收。

### T-01 决定输入重核与冻结（work，2026-11-17）

冻结窗口：开工读数 Git `bd18c1363`，分析期间主检出被其它会话推进（`ab3e6ee7c` → `decfaf4ea`，均为他计划提交，本包相关源文件字节不变）；本表身份取自 `decfaf4eab682aa08f8b0261986bef51a0e1f136`。§4 的 r2 表保留为历史快照，不再充当当前身份。

| 输入（源/测试/合同） | 行数 | 当前 SHA256 | 相对 r2 |
|---|---|---|---|
| native/src/collections_lower.rs | 1880 | a65d909593170318e5763dcdd5ff19b2dd0701936c056742075bd8179d607dc8 | 变（r2 844b233e0b04…） |
| native/src/closures_lower.rs | 1006 | edd1820dc3ebcff91982e929ffa59742e4b49521b55429feb92f74445ea91fe6 | 变（r2 6b5345401301…） |
| core/src/source/adapter.rs | 5547 | 1ac3cb79fd69feeb933113bdf8dc39a34a2662f97e4553e0b29290dde880445d | 变（r2 64fb2234a0c3…） |
| core/src/link.rs | 1963 | a74f557c299f8f6605ac1a041f52326eab37dbc238310489905e568a823d4e2a | 变（r2 7e19a7d49894…） |
| data-runtime/src/lib.rs | 1274 | 4c2327b22406e09b24f46edadcf1b1deb1f5fa95db9de980c283470d948d5226 | 同 |
| text-runtime/src/lib.rs | 1292 | 63029b2912b40dae133821a8e84d2a3564f8e700381fdc56ff8af2c0b1863bde | 变（r2 5ec6d366ef69…） |
| core/src/source/resolve.rs | 15541 | b3f9f3d8c1aa25d1eb08d82a564048bf569f36e760fbb484473c5444305657a0 | 变（r2 80eb12467732…） |
| core/src/verify.rs | 8257 | cb169cf4aea617a3b101ec636e4774ad9c02aa24320408893368e7cdd72c4595 | 变（r2 cba93ead10ab…） |
| av-core/src/mir_verify.rs | 5463 | c2628e251f772961d953e17698701534702f6858893eee36d7414118080a43af | 变（r2 9f1427164bd6…） |
| native/tests/cases/closures_native_tier.rs | 688 | b13c15acdff363589738ee73fcfd90aefe181ebf8697ddc417d7f7d922603d12 | 同 |
| av/tests/cases/av_collections_session.rs | 127 | c1e4baad3dd66f0f6fde392774fdbb23393abb78584aa0cf31ef365653a827c8 | 变（r2 bd116666026a…） |
| artifacts/src/subproc.rs | 115 | a5a1d17a62da34f9556b0632ead5a3482a93b3cb6f99cfc24dab6c6a179f09af | 同 |
| core/src/lib.rs | 576 | 925d9503a0f47d665885c4f3bafb1d27c8ad2c23ac330ac567a9809e718e819d | 变（r2 906442f4d5f2…） |
| av-core/src/lib.rs | 76 | 9f73f8abe09882ee304481bb7dffcc3d7da4f25162e423c47cc0bf9eb65b59fa | 新增冻结（D15） |
| native/src/lib.rs | 115 | 1e3306a8eb31f82d9c6e36398aa8667bfa2d37da6a064bf2c2da45f76c9401b3 | 新增冻结（D15） |
| docs/specs/auto-ac/native-backend-and-retirement.md | 120 | 3547178050b20180ac5c89afd76ecaf30219d8a84aa94300cd37db8218ea6ab7 | 变（r2 77ba8a3c2fa1…） |
| docs/specs/auto-a2x/architecture.md | 140 | eddf621a821a7368f00d692cfe62673883fdb6fa0523e8a83b3f194cf0d4958e | 变（r2 9984e8854b3b…） |
| docs/specs/auto-test-case/architecture.md | 509 | d86aecf49b41e06d00524d1cbe9936f5980729f66a9bb227c27386b99afa2772 | 变（r2 2e37d29c0df9…） |
| docs/specs/auto-av/architecture.md | 15 | 05bd373cfff3d9901c5a790786ad5152842d2360ff2f2723aa23fb81322a0282 | 同 |

重核结论（均为当前源静态事实，非运行证据）：

- **D04 重复仍成立且口径修正**：collections_lower.rs:1420–1469 与 closures_lower.rs:955–1004 **50 行逐字相同**（行集合比较无差，body SHA256=4be783eee99beb7a26c4578954e44d46dfd21f1e33c6361674a92a4d9c687f17）。1024 的“52 行”是 r1 冻结源含签名的完整花括号体（hash 058843c6…）；当前两 `fn binary` 仅差“collections 先分派 `BinOp::Num`→`binary_num` 并返回 `Result` / closures 无 typed 分派且返回 `()`”。细节机制同一。
- **i32 陷阱 oracle 现状（缺口具名）**：i64 typed 车道有 `numeric_codegen_tier` 的 MIN/-1、MAX+1 真跑 70；scalar 车道的 B14（10/0）、B15（MIN % -1）由 `shadow_tier::shadow871_scalar` 真跑 70——**都不经过这两个 lowerer 的 i32 车道**。collections 正常除法面由 `collections_driver_tier` 的 V02（`Ok(a / b)`→42）覆盖；两个 typed tier 的 hand-built MIR 只用 `AddI32`。故 i32 DivI32/RemI32/MIN%-1/溢出的 native PE oracle **当前缺失**，属 T-03 合同必须补的永久反例（延伸既有入口，不建新框架）。
- **成本测量方法**：本包不跑 ace-perf；若后续需要同一源/构建的分配与时间归因，按 1017 同路径成本基线（同输入、同构建身份、准备/执行/清理分列）执行，不从 Formal 总时长推断单项收益。

### T-02 六项决定（D04/D07/D09/D11/D13/D15，work）

**D04 / F04 — i32 标量算术发射重复：go（首个切片）**

- 候选：把两份 `fn binary` 的 i32 比较/四则（含除零、MIN/-1、i64 中间值、i32 上下界与 -30/-31 状态）抽成 `ace-native` crate 私有发射函数。
- 源/位置：collections_lower.rs:1412–1471（body 1420–1469，typed 分派 1417–1419）、closures_lower.rs:954–1005（body 955–1004）；两 `Lower` 结构（collections 464–477 / closures 472–489）与布局/捕获准入不同。
- 真实 consumer：collections 侧 `Lower::instruction`→`emit_admitted`→`emit_unit_coff`←`cache.rs:45/72`、`collections_incremental.rs`、`collections_driver.rs`、`mn-collections-worker`；closures 侧 `closures_incremental.rs:158/473`、`closures_driver.rs`、`mn-worker`。测试消费者：`ace-native` 组（collections_driver/source/typed tier、closures_native/typed tier）、`ace-av`（v10_layers_972 用 WrapperKnobs）。
- 同/异合同（保留表）：

| 机制 | collections | closures | 首片处理 |
|---|---|---|---|
| 6 种 i32 比较 → icmp+uextend | 同 | 同 | 共享 |
| Add/Sub/Mul：sextend I64→运算→i32 上下界→trap -30 | 同 | 同 | 共享 |
| Div/Rem：先除零 guard(-31)，再 MIN/-1 guard(-30) | 同 | 同 | 共享 |
| `BinOp::Num` typed 车道（`binary_num`，I64/I128/F32/F64/窄化拒绝） | 有 | **无**（拒绝面） | 不共享、不得借抽取 widen |
| `word_to` 64 位整词 store | 有（typed 结果） | 无 | 不共享 |
| frame/捕获/layout/env、`discard_guard`、cleanup 顺序 | 异 | 异 | 永不共享 |

- 现有独立 oracle：V02 正常除法 42（collections product driver）；closures_native_tier `assert_clean_after_trap`/frame 事件；collections seam/resource tier 的 trap 清理；`ace-av` 的 AV 语义并行面。i32 陷阱面缺口见 T-01。
- 成本/收益：收益=校验算术与 guard 单一 owner（消 50 行×2 维护重复）；风险=两 family 的 `codegen_build()` 用 `include_bytes!` 哈希 lowerer 源（collections_layout.rs:16–28 / closures_layout.rs:27–39），抽取即改 codegen 构建身份（属正常变更，须在合同中显式加新文件进哈希并记录）；运行期无行为变化。
- 决定：**go**。首个可实施切片合同见 T-03；未在本包实现。

**D07 / F07 — source adapter 的 Atom 往返 / typed builder：retain（本轮 no-go）**

- 候选：内部 typed HIR builder，省掉 `render`→`descriptor::bind`→`reanchor` 的 Atom 文本往返。
- 事实：闭包增量入口每次都整源 `parse_program_with`→`resolve_closures_cached`→`typecheck_with_skip`→`Emitter::plan_module/render`→`descriptor::bind`→`verify_closures`（adapter.rs:319–342）；只有 owner 级 typecheck 被跳过，“reused”不等于免往返。常规入口同样 render+bind+reanchor（adapter.rs:1099–1140）；packages 面在 1241–1263。既有 scoped 接缝：`emit_function_body`(556)、`FnCandidate`(710–876)、`emit_resolved_parts`(1154) 消费已 resolve 的程序、不重复 parse。
- 合同约束：Atom 是外部入口格式且 `bind`/`verify`/`reanchor` 是准入/授信面（CheckedModule/CheckedHIR minting），typed builder 不得绕开；作者格式/IR 接线受 1002 G1002 约束。
- 证据/unknown：Atom emit+bind+reanchor 的耗时占比 **unknown**（无同源归因；本包不跑 ace-perf）。
- 决定：**retain**。重开前置：(1) 1017 口径下同源 cold / 同源 hit / 单 body 编辑三输入的归因显示该段是实质占比；(2) 证明 typed 构造保持 descriptor 校验、span 重锚与凭证 minting；(3) 涉及作者格式时 G1002 已稳定。未满足前保留 Atom 入口与 scoped emit 接缝，不承诺加速。

**D09 / F09 — 依赖层与测试宿主混合：retain（no-go）**

- 候选：(a) 按调用图拆 crate（av 的 bench/formal/fixture、native 的 typed_fixture/workload 等 test support 宿主）；(b) 把 core link 的期限执行器搬进 artifacts/subproc 或改用 auto-tool-execution。
- 事实：`artifacts/src/subproc.rs:1–28` 已是 `auto-tool-execution` 的薄兼容壳（`run_command_capture`），消费方只有 `syntax-artifacts` 的测试（syntax_budget/syntax_cache）；core `link.rs` 自己保留完整宿主（152–520：suspend-start + Job Object 约束、reader 线程、`CaptureCaps`、`SubprocError` 分类 93–150、`run_deadlined_captured(_observed)` 389–397），消费方是 `abi`（ctool/bridge/rust_provider）、`a2x-worker`（crun/pyrun/tsrun）与 link 自身；`artifacts → core` 是正常依赖方向，**core→artifacts 会反向成环**。native 的消费者为 abi/av/case-worker/comptime/m2-build。auto-tool-execution 现有 BoundedPolicy/CapExceeded 与取消/stdin 面（lib.rs:513–520、1433），但接口面与 core 的 caps/observed/typed 捕获不同。
- 证据/unknown：拆 crate 的编译时间与产物链接占用 **unknown**（本包不测）。测试宿主模块并非纯 test-only：formal_campaign/formal_worker 是正式 producer，不能 cfg(test) 藏起。
- 决定：**retain / no-go**。保留 subproc 兼容壳、core 的自有执行器与现有依赖图；不按名字搬、不以 feature 隐藏正式测试。重开前置：caps/observed/失败注入/stdin 取消/containment/typed 字节的逐项等价表 + 实测编译与身份收益 + 有界实施 Plan（届时才建专属树）。

**D11 / F11 — data-runtime / text-runtime 同源：no-go（保留双库）**

- 候选：合并两 no_std runtime，或先抽共享内部原语（ledger / try_alloc / owned_free / UTF-8）。
- 事实：data-runtime 39 个 `#[no_mangle]` 导出、状态码 -1..-11、无 allocator/free/poll hooks；text-runtime 32 个导出、同 -1..-11 **加 ERR_CANCELLED=-12**、`ac_rt_install_allocator` 三 hooks（lib.rs:60–122）、每次负返回的 `rt_fail` stderr trace、`PollHook` 驱动的取消面。两库都自供 memcpy/memmove/memset/memcmp 符号、都 no_std + panic-abort + 仅 kernel32。共同机制（ledger_insert/ledger_take/try_alloc/owned_free/utf8_check）在两库各自实现。
- 同源 ≠ 同合同：合并会丢 text 的取消/元数据/hooks 或把 data 的探针面 widen；另起第三 no_std crate 则要两 PE 各带一份新符号并重证预算/trace/取消合同，收益（约 200 行原语去重）不足以抵消 ABI 风险。
- 决定：**no-go**（本轮也不抽共享原语）。重开前置：导出/符号差分表、预算/trace/取消等价 oracle、双静态库构建变体、双 free/ledger/UTF-8 原 oracle；禁止以引入 libc 解决复用。

**D13 / F13 — 六大模块的私有边界：go（单一首片，次序列）**

- 事实：行数 resolve 15541 / adapter 5547 / verify 8257 / mir_verify 5463 / link 1963；近 400 提交触及数 resolve 87、adapter 52、verify 39、mir_verify 31、link 9、collections_lower 16、closures_lower 5（churn 是维护热点证据，行数本身不是）。授信边界：`CheckedMir` 只在 mir_verify 内构造（85–131），`CheckedModule` 在 verify；凭证构造私有。
- 决定：**go，仅一个 crate 私有模块边界**，排在 D04 之后：把 `av-core/src/mir_verify.rs` 的 init-analysis family 移入新私有子模块 `mir_verify/init.rs`——`check_init`(4410–4645)、`init_analysis_bound`(4885–4917)、位集 helpers（`set_bit`/`bit_is_set`/`words_for`/`union`/`union_into`/`intersect`，4918–4969）；父 `mir_verify/mod.rs` 保留凭证、`run()`、类型/借用/CFG 规则；`slot_ty_of` 等跨族 helper 留父级 `pub(super)`；父级 4155 的预分配准入改调 `init::init_analysis_bound`（`pub(super)`）。`union`/`intersect` 的使用点全部落在 init 区（4490/4559/4618 等），不跨族。public API、`CheckedMir` minting、`MirVerifyError` 不变，无运行期语义变化。
- oracle：av-core conformance 的 `mir_admission_*` init 组（未初始化读、分支合流、循环 init、dense analysis 预拒）+ 整组 `cargo t ace-av`（Windows 腿；Linux 腿按 2026-11-14 裁定排除、另环境执行）。
- 其余五个热点（resolve/adapter/verify/link/AV execute/project）本轮**不改**；单一 crate 拆分仍需实测收益才可提出。

**D15 / F15 — 头注释事实漂移：go（纯文档更正）**

- 事实（逐处，对照当前 Cargo/Spec）：
  1. `core/src/lib.rs:4–6` 仍写 “Atom text → … → native lowering (Cranelift) → COFF object → platform link”；Cargo 注释（core/Cargo.toml:10–11）与 [auto-ac/native-backend-and-retirement.md](../../specs/auto-ac/native-backend-and-retirement.md) §1 已定 lowering 移出 core（`native.rs` 删除、不再依赖 Cranelift）。
  2. `core/src/lib.rs:8–9` “must not depend on any auto-lang / auto-atom / auto-val product crates” 与当前图不符：core 正常依赖 ace-syntax-grammar，而 syntax-grammar 正常依赖 auto-atom/auto-val（syntax-grammar/Cargo.toml）——应改述为“不依赖产品运行时/UI，允许共享 syntax 叶子”。
  3. `av-core/src/lib.rs:16` “This crate has zero external dependencies” 不成立：Cargo 正常依赖 ace-foreign-contract + serde_json。
  4. `native/src/lib.rs:1–12` 仍以 “PLAN-818 probe / parity loop against … the old HIR-native reference” 描述，而旧 HIR-native 参照已随 PLAN-871 退役（Spec §7 G-DELETE）；且 Spec §1 已把 `ac-native` 定为 native 构建面，Cargo 消费者为 abi/av/case-worker/comptime/m2-build——“Nothing here feeds product paths” 的措辞需按当前 Spec 角色收敛（是否属产品路径以 Spec 为准，不以旧注释判定）。
  5. `core/Cargo.toml:6` description 仍称 “Windows x64 native AOT”（Spec §1b 已双 admitted 目标、构建面在 ac-native）；与注释同批改述，不影响构建身份。
- 决定：**go（纯文档）**，由后继有界任务一次提交修 5 处（含 `core/Cargo.toml:6` description）；只改注释，不改依赖/代码/wire/Plan 身份（三文件均不在 `codegen_build()` 的 `include_bytes!` 集合内，编辑不改代码生成身份；实施时复核）；不删依赖迎合注释；不新增测试 case（核验=读/diff + 目标 Spec 对照）。

### T-03 首个 go 项（D04）有界后继合同

- 目标切片：`ace-native` 增加 crate 私有 `src/lower_arith.rs`，唯一入口
  `pub(crate) fn emit_i32_binary(fb: &mut FunctionBuilder<'_>, trap: Block, op: BinOp, lhs: Value, rhs: Value) -> Option<Value>`
  （`None` = 非本机制 op，调用方保持原拒绝/分派）。`collections_lower::Lower::binary` 先做 `BinOp::Num`→`binary_num` 分派，再委托；`closures_lower::Lower::binary` 直接委托。两 `Lower` 的 `guard`/`i32` 保持原样，helper 内联其等价块序列（同一顺序创建 next/bad 块）。
- 不变量：发射顺序与块序不变；trap 码 -30（溢出、MIN/-1）与 -31（除零）；I64 中间值 + i32 上下界；`scalar_to` 的 uextend/store 宽度；closures 的 typed-numeric 拒绝面不 widen；frame/capture/cleanup（D08 的预检/hold/release 顺序）不动；`ABI`/`CODEGEN`/`PROFILE`/`RECIPE` 串与 manifest 身份不变。
- 身份变更：`collections_layout::codegen_build`、`closures_layout::codegen_build` 的 `include_bytes!` 列表各加 `lower_arith.rs`（源文件已变，身份本就更新；加新文件避免只改 helper 时身份不失效）；记录变更前后的 codegen_build 值与失效影响。
- oracle（全部绑既有入口，真实 PE）：
  1. 发射等价：固定代表单元（collections V02 + 一份 hand-built MIR；closures K01/K02 + 一份 hand-built typed 单元）抽取重构前后 `emit_unit_coff` 的 COFF 字节，要求 **逐字节相同**（仅 `codegen_build` 身份值可不同）。
  2. 永久反例（补当前缺口，落既有文件）：`collections_typed_tier` 增 `DivI32` 正常值 / 除零 / MIN%-1 / 溢出四例，`closures_typed_tier` 增同形四例，断言真实 PE 退出 70 与 `MN-COLL-TRAP code=-31/-30`、正常值返回；每例必须有可区分的错误实现（如把 -30 写 -31、去掉范围检查）会红。
  3. 保留反例：`closures_native_tier` 的 `assert_clean_after_trap`/frame 事件、collections seam/resource tier、`numeric_codegen_tier` 的 i64 typed 面不回归。
- 门禁：`cargo t ace-native`（Windows；Linux/WSL 腿按 2026-11-14 裁定排除，不记 PASS），涉及 wrapper/AV 并行面时补 `cargo t ace-av`；不跑 ace-perf；新增 case 每例 ≤10 秒。合并门由该后继 Plan 的 review/merge 持有。
- 退出条件：4+4 个 i32 陷阱/正常反例在真实 PE 绿；固定输入 COFF 逐字节相同；codegen 身份变更已记录；closure 拒绝面/cleanup 面无 diff。
- 实施规则：由 `scripts/new-plan.sh <slug>` 取新号，在专属 `D:/autostack/.wt/lang-<新号>/auto-lang` 树实施；本包不再改实现。
- 范围禁止：不合并 layout/cache/wrapper/emit；不合并两 family 的 frame/capture；不借机改 ABI/公共接口；不以“无行为变化”省略 oracle。

### work 收据（2026-11-17）

`stage: work | plan_id: PLAN-1038 | plan_revision: r3 | outcome: pass | worktree: none（只读研究+Plan簿记，未建树） | branch: none | base_commit: decfaf4eab682aa08f8b0261986bef51a0e1f136 | code_commit: none | dependency_revisions: 见 T-01 表 | task_ids: T-01,T-02,T-03 | evidence: §9 T-01 身份表 + T-02 六行决定 + T-03 D04 合同 + 本收据 | blockers: none | next_responsible_role: review`

- 本会话未新增/未运行任何测试（无产品 PASS）：仅静态读源/Spec/Cargo/测试入口、行集合比较（50 行 identical，body hash 4be783ee…）、SHA256 身份重冻结、churn 统计（近 400 提交）。未改实现/测试/配置/canonical Spec，未建 worktree/plan-artifacts scope（无原始证据需保留，摘要已折入本节），未提交/合入。
- 主检出预检：唯一代码路径 WIP = `experimental/ace/crates/a2x-worker/tests/cases/sched_m2_968.rs`（+69 行，`run_a2x_sched_rust_probe`，属他会话/他 Plan，非本包产物）——未触碰、未纳入本次交付；其余 WIP 为 docs/plans 等簿记文件。
- 与 1002/1017/1028 边界：D07 的 typed builder 面受 G1002 约束（仅当涉及作者格式/接线）；成本归因归 1017 同路径口径；1028 已由用户启动、未读改其树。

`stage: goal | plan_id: PLAN-1038 | plan_revision: r3 | round: 1 | from: drafting | to: review | status: execution_done | outcome: pass | worktree: none | commit: none（Plan 文件暂未提交，merge 时随归档提交） | next: review`

### T-04 独立复审（review，scope: final，2026-11-17）

`stage: review | review_scope: final | reviewer_identity: 本会话就地独立复核（无独立会话/模型/另起 agent 授权；未采信 work 摘要，逐项从源码/Cargo/Spec 重建证据；限制=与 work 同一 checkout、无实施树） | plan_id: PLAN-1038 | plan_revision: r3 | outcome: pass | worktree: none | branch: none | reviewed_commit: none（研究+计划交付，无代码提交；Plan 文件在 merge 随归档提交） | base_commit: decfaf4eab682aa08f8b0261986bef51a0e1f136（复核时主检出 HEAD 已推进到 1171c9cc，本包相关源文件字节未变） | dependency_revisions: §9 T-01 的19项逐项重算一致 | spec_inputs: auto-ac/native-backend-and-retirement.md=3547178050b20180ac5c89afd76ecaf30219d8a84aa94300cd37db8218ea6ab7、auto-a2x/architecture.md=eddf621a821a7368f00d692cfe62673883fdb6fa0523e8a83b3f194cf0d4958e、auto-av/architecture.md=05bd373cfff3d9901c5a790786ad5152842d2360ff2f2723aa23fb81322a0282（三者与 HEAD 逐字节相同）；auto-test-case/architecture.md 工作树=d86aecf49b41e06d00524d1cbe9936f5980729f66a9bb227c27386b99afa2772（含他会话未提交改动；HEAD=21129c884ce1178cb13ce246c4013309acc83b7526afe81691beefb4ec08b9f6，见 R-1038-1） | acceptance_results: AC-01 pass / AC-02 pass / AC-03 pass / AC-04 pass（下表） | pending_closeout_gates: merge 提交 Plan + 归档、SD-00 无规范增量确认、输出/scope 对账（无 scope）、无实施树可清 | findings: R-1038-1/2（非阻塞记录）、R-1038-3（已核正） | evidence: 本节复核表 + T-01/T-02/T-03 | next: merge`

复核方法（逐项，不采信 work 摘要）与结果：

| 面 | 复核动作 | 结果 |
|---|---|---|
| 身份 | T-01 19 行逐项重算 SHA256/行数（15 项 ACE 路径按 `experimental/ace/` 解析） | 19/19 一致 |
| D04 | 重抽两体行集合与 hash；fn 边界实测；consumer 实测；`BinOp::Num` 仅 collections；i32 陷阱缺口复核 | body 1420–1469 / 955–1004 仍逐字相同、hash `4be783ee…`；`collections_lower.rs` 1412/1476、`closures_lower.rs` 954；`cache.rs:45/135`、`closures_incremental.rs:158/372/473`；native 测试 BinOp 仅 `AddI32`，B14/B15 为 `Prof::Scalar`（shadow_tier），i64 陷阱在 numeric_codegen_tier → 缺口成立 |
| D07 | adapter 路径/接缝行号实测读核 | 319–342（warm 仍整源 parse+render+bind+reanchor）、556/710/865/1044/1099–1140/1154/1241–1263 均吻合；成本归因仍为 unknown |
| D09 | 依赖方向/消费者/执行器能力复核 | `artifacts→core` 为正常边、`subproc` 仅 `syntax-artifacts` 测试消费、core 执行器被 abi/a2x-worker 使用、`formal_worker` 由 `av-worker::session_serve` 正式使用（R-1038-2） |
| D11 | 导出数/错误码/hooks/自供符号复核 | data 39 导出、-1..-11、无 hooks；text 32 导出、含 `ERR_CANCELLED=-12`（lib.rs:58）、hooks（60–122）、`rt_fail`（531）；两库各 4 个自供 mem 符号 → 保留双库结论成立 |
| D13 | init family 行号与使用点复核 | `check_init` 4410、`init_analysis_bound` 4885（父 4155 调用）、helpers 4918–4969；`union`/`intersect`/`union_into` 使用点 4490/4559/4618 全在 init 区；跨族 `slot_ty_of` 留父级 → 边界成立 |
| D15 | 5 处文本逐处读核 + 身份哈希输入复核 | `core/lib.rs:4–6/8–9`、`av-core/lib.rs:16`、`native/lib.rs:1–12`、`core/Cargo.toml:6` 与 10–11 均如记录；`include_bytes!` 仅两 layout 各 6 项，三文件不在其中 → 不改代码生成身份成立 |
| 交付/健康 | `git diff --check` 面、围栏、尾随空白、本地链接、本会话写面、scope/树存在性 | 围栏 0、尾随空白 0、15 个本地链接可解析；本会话未改实现/测试/配置/canonical Spec；`plan-artifacts/plan-1038` 与 `lang-1038` 树均不存在；主检出代码 WIP 仅他人 `sched_m2_968.rs`（未触碰） |
| 测试/计时 | 本包无新增 case、无产品测试执行（研究+文档交付） | 无 timing 表可记；最终统一回归不适用（无代码变更）；merge 门=Plan 归档提交 + SD-00 无增量确认 |

发现（均非阻塞，不改变六项决定）：

- **R-1038-1（证据口径）**：T-01 的 `auto-test-case/architecture.md` 行记录的是工作树身份 `d86aec…`，该文件含他会话未提交改动（HEAD `21129c88…`）；其余 3 份 Spec 与 HEAD 逐字节相同。该 Spec 未被六项决定的任何事实引用，不构成决定证据；口径已在此记录，不改 P1/P2 结论。
- **R-1038-2（措辞精度）**：D09 “formal_campaign/formal_worker 是正式 producer” 中，`formal_worker` 确由 `av-worker` 正式实现；`formal_campaign` 由正式验证脚本（`verify-av-*`）与 `function_protocol` 测试消费。结论（不 cfg(test) 隐藏正式 producer、不拆 crate）不变。
- **R-1038-3（文档计数）**：D15 原文写“修 4 处”，实际列 5 处（含 `core/Cargo.toml:6` description）；已按核正为 5 处。

AC 复核表：AC-01 pass（19 项身份/漂移可复核、无 expected 改动）；AC-02 pass（六项决定各有源/consumer/保留表，i32 缺口具名，无实现改动）；AC-03 pass（首个 go 合同可执行且未越界，retain/no-go 有前置）；AC-04 pass（本表独立重建，未见遗漏/延后/workaround 或未分配候选）。

复审结论：**pass（final）**。本包交付为研究决定 + 有界后继合同，无代码/测试改动；merge 可执行归档，须在 closeout 确认 SD-00 无规范增量、Plan 提交与归档、无 scope/树需清理。

`stage: goal | plan_id: PLAN-1038 | plan_revision: r3 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: none | commit: none | next: merge`

### T-05/T-06 合并、归档与清理（merge，2026-11-17）

`stage: merge | plan_id: PLAN-1038 | plan_revision: r3 | outcome: pass | completion_kind: delivered | review: final pass @ r3（§9 T-04） | delivery_commit: 本归档提交（SHA 由收据提交 2 绑定，见下方 merge-receipt 行） | canonical: SD-00 无规范增量（approved no-impact） | archive: docs/plans/archive/1038-ace-reuse-candidates-and-cost-decisions.md（status: archived） | cleanup: 无实施树/分支、无 plan-artifacts scope、registry 无持有 | next: done`

| checkpoint | evidence |
|---|---|
| prepared | reviewed r3 + final pass；T-01 19项身份；本包无代码变更；SD-00 无影响；归档目标 `docs/plans/archive/1038-…md` |
| landed | docs-only 交付直接在 master（无 dev 分支/树、无 merge commit）；本提交只含 Plan 归档与 `KNOWN-DEBT-AND-RISKS.md` 六行导航更新，他会话 WIP 未纳入；`git log -1 --stat` 可验 |
| canonical_verified | 交付提交内 `docs/specs/` 零改动（`git show --stat` 核）；approved no-impact：六项决定仅研究，go 项的实施/文档更正及其规范增量归各自后继有界 Plan（D04 合同 §9 T-03） |
| archived | `docs/plans/archive/1038-…md` 已入库且 `status: archived`；6 处 `KNOWN-DEBT-AND-RISKS.md` 导航指向归档路径（`1038`→`archive/1038`）；归档位置 15 个本地链接全部可解析 |
| cleaned | `git worktree list` 无 1038 树、无 `plan-1038-dev` 分支；`D:/autostack/plan-artifacts/plan-1038` 不存在（本包从未创建 scope）；registry 无 1038 scope/持有；wt-guard 无树可扫（无树可删） |

门禁与观察：

- **测试/构建**：本包无新增 case、未执行任何产品测试（研究+文档交付），无代码变更 → 固定组冻结并集门不适用；按 docs-only 走内容检查：frontmatter（plan_id/status/current_step=6）/章节/任务与 AC 计数/16 个本地链接/围栏 0/尾随空白 0。
- **`python scripts/spec-lint.py`**：退出码 1，4 errors + 78 warnings，均为既有仓库态（errors=`ace-migration`/`ase`/`design`/`history` 缺 `project.md`；warnings 为历史 ledger/plans.md 的既有断链），无 1038 相关项；本交付未改 `docs/specs/`。
- **部署新鲜度**：无生产产物/生成物受影响 → 无需 rebuild/restart（记录为“无需刷新”的观察，不声称已刷新）。
- **批量回归到期**：`docs/plans/.last-batch-regression.json` = last_covered 1064 @2026-11-16T10:04:09Z；1038%5=3 且距今约 24h < 48h → 未到期，不触发 `cargo t all`。
- **他会话 WIP**：主检出 `a2x-worker/tests/cases/sched_m2_968.rs` 与 docs/plans 等未提交改动保留原状，未纳入本交付。

归档后状态：任务 6/6、AC-01..06 全部关闭；D04 首个有界切片合同、D13 单一私有边界、D15 文档更正分别由后继有界 Plan 承接，D07/D09/D11 保持 retain/no-go 并带重开前置。


## 10. 待澄清事项

1002试点不阻只读研究；涉及作者/执行接线的实际重构须1002接口稳定。1017同路径成本基线先冻结；本包不擅跑ace-perf或修改实现。 T-01使真实反例/资源或IO故障可运行，未取得前不声称动态后果已证。等价实现不需重复审批；新支持/预算/公共合同变化先最小决定，不能用隐式门限或延期替原义务。不等待“新测试框架全部完成”。

本轮T-01入口已具体化，当前不需要用户再次确认等价内部实现。未取得真实动态反例的部分仍由本节与§6指定入口/邻例核证，不把预期当已观察结果。

2026-11-17 work 后：六项决定已出且无阻塞问题（§9 T-02）。D04/D13/D15 的实施/文档更正**不在本包**执行，须由各自有界后继 Plan 承接（D04 合同已冻结于 §9 T-03）；D07/D09/D11 的重开前置已具名，未满足前保持 retain/no-go。本包没有待用户裁决事项；未运行任何 ACE/性能门，故不存在“已观察动态后果”的声明。
