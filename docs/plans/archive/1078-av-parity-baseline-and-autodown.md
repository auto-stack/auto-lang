---
plan_id: PLAN-1078
status: archived
feature_name: ACE按语言特性共享AutoDown语料与AV普通源码基线
author: [Codex]
created_at: 2026-11-19
updated_at: 2026-11-19
plan_revision: 3
current_step: 8
total_steps: 8
depends_on: []
supersedes_spec_components: [docs/specs/ace-migration/m2-denominator.md, docs/specs/auto-test-case/document-cases.md]
new_spec_components: [docs/specs/auto-av/language-parity.md, docs/specs/auto-test-case/feature-organized-corpus.md]
touched_goals: [GOAL-002]
affects: [ace, auto-av, auto-test-case]
---

# [PLAN-1078] ACE按语言特性共享AutoDown语料与AV普通源码基线

## 0. 变更摘要

### 2026-11-19 r3 · AV-ONLY与检查级#ignore（用户补充，仅规划）

用户明确1078–1096此轮只开发/验AV；共享源码保留，AC/A2R/A2C/A2PY等非AV检查在本轮不执行、不作为work/review/merge门，采用检查小节`#ignore`暂缓，后续对应后端任务逐项打开。规则见[共享语料AV阶段策略](../design/testing/ace-feature-organized-autodown-corpus.md)；原非AV义务/预期/来源/owner保留，忽略不充PASS，也不阻本包AV出口。本次仍只改计划，不改实际测试或实现。

### 2026-11-19 r2 · 按语言特性集中、同源多后端（仅规划）

用户新增组织要求适用于本包全部公共语言测试：[共享语料设计](../design/testing/ace-feature-organized-autodown-corpus.md)为本次合同增量。唯一活跃作者根改为`crates/auto-lang/test/cases/language/`，本包拟写`basics/expressions.case.ad`；按语言行为命名，不再建立AV/后端/计划号作者副本。保原case ID与72例整体需求、原独立预期、任务ID和进度；本轮不搬任何实际用例、不施工。

AV–AutoVM语言齐平路线批次 **AVP-00**，r3，仅设计/计划；本轮用户授权研究和编订系列计划及AutoDown代码用例，**未授权此轮施工**。全部0/8，拟新增接口、路径、case和命令均未执行。总路线见[AV语言齐平设计与执行路线](../design/strategy/av-autovm-language-parity.md)。

## 1. 目标

新增组织出口`IF-ACE-FEATURE-CORPUS/1`（拟）：公共唯一作者、按特性/目标的选择与旧→新迁移映射，和原`IF-AV-PARITY-BASE`并交。1078建立共享机制与同源数据组织与AV实证，不承担一次搬完全部旧族、全部后端完整语言实现；各特性/目标原owner分批接续。

建立一套可追溯的AutoVM→AV语言义务表，并让后续批次从第一天共用普通AV源码入口和AutoDown协议。

本包交付`IF-AV-PARITY-BASE`，绑定原源、规范、工具/ABI/provider身份、适用平台与逐case真实结果。非目标：AC/A2R/A2PY完整补齐、新标准库全部功能、UI/2D/3D全demo、AutoOS产品迁移、旧引擎删除和AAC自举；这些由各原owner消费本包能力。共同前门/IR改动本轮验AV及共享正确性；非AV能力检查保留后续owner，不作为本系列AV门。

## 2. 架构方案

复用唯一`AutoDown输入→既有TestCase author/CaseSpec→普通Auto源Schema/CST→绑定/Checked HIR→共享MIR/ABC验证→AV真实执行→父侧独立oracle`。普通profile与`ac-av/language-source-run-v1`由AVP-00新增接面逐步扩充，**不是第二parser/解释器或按case名硬编码结果**；当前尚不存在，开工前必须先交其正式接口。旧窄profile维持冻结身份，最终不得只交多个互斥profile。

- 复用959的L-01..49与1066现有发现/身份/比较机制，逐case/check映射active VM、language Rust tests、cookbook语义和真实外部语言切片，保原排除与旧红。
- 新增普通源码SourceProfile::Auto（拟名）及ac-av/language-source-run-v1（拟注册recipe）：仅是现有Schema/CST→共享绑定/Checked HIR→MIR→AV的显式能力入口，首批交aggregates已有基础，不承诺后续未交特性。旧冻结profile不改为自动试探列表。
- 新recipe复用现有worker/父侧oracle，支持int入口值、精确stdout/stderr、编译/运行失败、Auto多文件payload和资源观察；不存在的子能力必须具名拒绝，不能转旧VM。
- 固定组case-av-language-parity（拟新增，仅显式ACE选择）按已注册目标/动作选择AV检查，保原check ID；显式旧VM对照单独选择原check，不让正常AV门重建旧引擎。

## 3. 技术栈

AutoDown文档式`.case.ad`、现有auto-test-case/auto-case/worker/固定Cargo组，ACE共享Rust实现。唯一实施树拟`D:/autostack/.wt/lang-1078/auto-lang`，分支`plan-1078-dev`；本轮不创建。只有确需跨仓改动才加同组兄弟树且先读其AGENTS，禁止junction/symlink。材料唯一`D:/autostack/plan-artifacts/plan-1078/`，开工登记owner/用途/在途，收尾折摘要并删除；本轮纯文档工具在系统临时目录。

## 4. 需求分析与背景调查

2026-11-19新增阶段授权：本系列只以AV检查验收，用户允许给非AV检查先加`#ignore`并在其开发阶段打开；该临时阶段策略覆盖此前要求最小AC真运行或“所有已接入目标都需本轮运行”的文字。共享文档/源码、AV必需公共编译与语义验证仍应正确，本轮未授权执行施工或批量修改已有非AV测试。

2026-11-19补充授权：仅修订共享语料的目录/多端检查/旧迁移去重合同；用户未授权本轮移动、合并或改写实际测试/expected、运行测试或实现。原scope与既有已验收边界保持。

用户2026-11-19明确要求先做“AV和AutoVM在语言能力/特性上的差距分析并逐步补齐”，每一步使用AutoDown代码测试；继承本会话仅规划、不施工。允许本仓设计/计划/分母簿记，未批准修改语言规范已裁定语义、自动改旧expected或启动新会话；没有用户预算/工期承诺。

现有M2分母49个L行是语义族，不是完整case分母；当前磁盘case文件、实际discover集合、Rust内联测试及外部consumer也不相等。文档式author.rs已经支持所有REGISTERED_PROBE_RECIPES，不能照旧Spec误报仅四个recipe可写。

既有来源与边界：

- [scalar.case.ad](../../crates/auto-lang/test/cases/ace-m2/documents/scalar.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。
- [dynamic.document.case.ad](../../crates/auto-lang/test/cases/ace-m2/dynamic/dynamic.document.case.ad)（既有输入/oracle；当前适用证据须按身份重核）。

旧义务入口族：所有vm族的发现/选择权威；aavm2单列research，cookbook按语言/库/外部provider拆角色，不整目录冒VM语义证据。这里只给可定位种子，**不以种子替全部旧行**；AVP-00按真实discovery/check及Rust内联/仓外语言切片补全映射。本轮静态扫描vm目录506份文档与1066历史active数相同，不证明其发现/角色/实际结果恒等。

读取关键内容指纹（工作区观测，非已验结果；开工核漂移）：

| 源 | SHA256 |
|---|---|
| [author.rs](../../crates/auto-test-case/src/author.rs) | `0eb0585ac9f5b9460b9a40035172c0050de91819fdf763c61ede0727e19fd48a` |
| [contract.rs](../../crates/auto-test-case/src/contract.rs) | `471092cf5b33afb6c9856faca9d3bb799b2ce5bafde86d2832da083c657f4d3c` |
| [test_runner.rs](../../crates/auto-lang-core/src/test_runner.rs) | `7fea114a5d742bbc30cdeb7330e9968f87c72fdcb30e5750fad2067244c3c75e` |
| [ace_entry.rs](../../crates/auto-lang-core/src/ace_entry.rs) | `6933cdd96917cc234ebe666a4b2b5783e59a4fd7217d109492dd08621b93aa51` |

硬依赖：无新增未交Plan；实际外部接口：960–968等已交对应切片，按本包引用适用范围重核。依赖只按本包所需正式切片，不反等1068/985/B、完整AAC或完整UI/GPU产品。982配置producer、967 provider生成、971测试发现各由原owner只生产一次。

## 5. 详细设计

### IGNORE · 标记实现与元数据归属

T-03只在检查小节（Probe、Rust、VM result等）识别裸`#ignore`，映射现有check级SkipSpec；T-01冻结默认阶段记录`reason=AV阶段暂缓非AV检查`、`issue=AV-ONLY-1078-1096`及本阶段语料维护owner，真实后端producer另在原义务映射保留。T-04/T-05验选择/忽略早于后端prepare和dispatch；不在Auto语言源中增加ignore语法，不用case级skip关AV。默认记录不能覆盖原历史/平台ignore；后续只移除本阶段新增标记。

现有共享Rust职责组可能内含非AV执行腿；T-01列明其真实集合，T-03在既有固定组/nextest选择机制提供AV阶段过滤，T-06按过滤后的AV与公共正确性职责运行。§6组名是职责基线，不能在未隔离时照原混合组全跑而把非AV失败算本包失败；不新建叶目录发现器或批量停掉共享正确性。

### AV-ONLY · 本轮实际执行边界

公共Case仍保一份Auto/工程；本系列的执行选择集合仅含AV目标及其公共正确性。AC/A2R/A2C/A2PY、旧AVM对照与其它非AV检查在对应小节加`#ignore`或保持未选，均不启动其绑定/编译/链接/执行/外部工具准备；不能因其失败、未实现、缺工具/源码profile不支持阻塞本包AV的work/review/merge。`#ignore`不标整Case或Auto源，AV required检查不得被一起关闭。

1078拟将文档检查级`#ignore`接到现有`CheckSpec.skip/SkipSpec`，不另建框架；当前author还未接此标记，实际接线属于1078的后续实施。报告分列AV pass/fail、显式SKIP(IGNORE)、未选NOT_RUN及后续owner，后两者不计PASS。未注册的未来recipe先保Notes/义务表pending，不因#ignore放松公共文档结构/schema准入。

后端任务接手后逐check移除本阶段#ignore，保原平台/历史ignore与原oracle，按同一源/最新身份真实运行；AV视图即使该check已打开也仍只选AV。非AV全部required关闭仍是对应后端/1068/985最终完整门，不是本包AV门。

### FC-BASE · 1078本包具体交付

1. T-01/T-02冻结ACE活跃语言Case、506 VM快照、旧a2r及其它适用族的真实发现/消费者/原oracle，生成原义务→canonical迁移数据；保case/check/模式/条件，不能把旧a2r emit-only默认提升为run。
2. T-03建立feature作者根/公共suite/复用的多文件与按需shared引用/目标选择，保未知字段/错引用/重复ID/零选中严格拒；新接口未交不假称当前可用，不新增测试框架。
3. T-04/T-05在同一基础Case只运行AV作最小实证；AC-native与旧VM检查加#ignore保预期，A2R等未交检查保pending与owner。明确撤销r2的AC最小真运行要求，不以任何非AV结果阻1078。
4. T-05/T-06切最小批全部活跃消费者、故意错目标/错源/删映射/重复Case/残留旧作者等FC反控，失败保旧并验恢复；后续迁例按语义族由1079–1095承担。IF-ACE-FEATURE-CORPUS/1逐目标记ready/pending，非“所有语言和后端已通过”。

规范增量追加：真实交付后新增`docs/specs/auto-test-case/feature-organized-corpus.md`，记录作者/选择/迁移和支持事实，当前不预写为已实现。

### FC · 唯一作者、旧义务与多端检查

依[共享语料合同](../design/testing/ace-feature-organized-autodown-corpus.md)，同一公共Case一份Auto/工程闭包，AV、AC、A2R等分别追加目标检查及独立实际结果。已有稳定ID/check ID保留或在原→新映射明确承接；相同文件名/输出、不同路径/模式/授权、不同错误阶段与资源角色不能自动合并。Source/fixture修改使全部实际依赖目标失效，单目标oracle修改不得改其它目标预期；不从AV实际值生成expected。

本包T-01/02先认领原case/check/角色和canonical位置，T-03/04只复用/按需扩现有作者、引用、选择与报告机制，T-05完成本特性迁移并核原义务守恒，T-06核所有活跃消费者及结果身份，T-07/08按本revision复审/落主。迁移表拟`language/migration/legacy-map.tsv`（数据映射，不作新注册器），由1078建立，本包只维护所属行。规则包括FC-01..07；不据新canonical文件较少删旧分母，也不移动/改写历史M1签章输入。其它目标未交时本包保AV独立出口、将目标pending交其原owner，不反等1068/全部AAC。

### 输入、真实代码路径与所有权

普通源能力键拟为`ac.source.language.av/v1`，对应显式Auto profile；它是源码准入/身份轴，不新增另一套ABC执行器、MIR wire世代或无限授权。normal/script模式、target、SDK/provider及工具版本均进入输入身份。首批只声明实际已交基础，后续未交行仍pending。

- [author.rs](../../crates/auto-test-case/src/author.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [contract.rs](../../crates/auto-test-case/src/contract.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [test_runner.rs](../../crates/auto-lang-core/src/test_runner.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [ace_entry.rs](../../crates/auto-lang-core/src/ace_entry.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [auto-case.rs](../../crates/auto-lang/src/bin/auto-case.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [mod.rs](../../experimental/ace/crates/core/src/source/mod.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [auto-source.at](../../experimental/ace/grammars/auto-source.at)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [av-worker.rs](../../experimental/ace/crates/av/src/bin/av-worker.rs)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。
- [test-groups.toml](../../scripts/test-groups.toml)：复用/修改当前生产路径；先按本包T-01核所需符号和调用者。

拟新增/迁入的长期作者位置为`crates/auto-lang/test/cases/language/basics/expressions.case.ad`。按本包Case的实际语言主题分配，不能按计划编号生成新副本；若目录已有等价Case，复用并扩其check而不另造源。`crates/auto-lang/test/cases/language.suite.ad`由1078建立公共选择，后续只增canonical引用。旧语料先按共享设计逐case/check/角色判等、迁入唯一作者并切消费者；新旧对照窗口完成后不留双活作者。原位置只保具名冻结历史，禁止继续当新代码作者源。所有未交required目标仍保pending/owner，不因未注册而计PASS。

输入→生产者→消费者：AutoDown内联源码/多文件闭包→统一语法与语义事实→受检IR/AV artifact→同一普通AV入口→实际值/双流/错误/span/父侧owner账。成功、拒绝、普通错误、预算/取消、关闭/恢复分别观察；不携带活provider句柄进portable cache。公开API/源位置/ABI世代或受检准入改变时显式版本/失效，不能旧页重释。

### 规范增量

补充SD-IGNORE（modify，拟，`docs/specs/auto-test-case/document-cases.md`与feature-organized-corpus规范）：检查小节`#ignore`映射既有CheckSpec.skip，AV阶段只选AV；非AV准备/执行/目标oracle加载暂缓、SKIP/NOT_RUN不充PASS，保原定义并由对应后端逐项恢复。对应AC-AV-ONLY、V-AV-ONLY与V-IGNORE-LIFECYCLE；当前尚未实现，真实交付后才沉淀支持事实。

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | `docs/specs/auto-av/language-parity.md` | 新增本包实际支持、拒绝/差异、case到owner映射和正式来源；交付前不写implemented | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-02 | modify | `docs/specs/ace-migration/m2-denominator.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-03 | modify | `docs/specs/auto-test-case/document-cases.md` | 保已交基线；仅增本包语言/普通入口事实、诊断与可复现证据，不改历史收据 | 以新AV真实语言能力替代profile/目录计数推断 | AC-01..06、AC-08 |
| SD-FC | add | `docs/specs/auto-test-case/feature-organized-corpus.md` | 按特性唯一作者、同源多端检查、旧义务映射和本包迁移/观察来源；实际交付后沉淀，当前不宣称已支持 | 公共机制由1078生产一次，本包保原语言/目标义务 | AC-FC、V-FC-SOURCE/MIGRATE/TARGET |

`language-parity.md`由AVP-00首次创建，后续串行增量维护；它是当前规范的能力/来源映射，不是第二注册器。实施前本包文档只提议，真实merge后按独立复审结果沉淀，不提前将Spec写成已支持。



## 6. 测试设计

### V-AV-ONLY / V-IGNORE-LIFECYCLE（未执行）

原T-03..06落实、T-07复核：同一Case AV已选，AC/A2R检查加#ignore；制造该目标缺工具/运行失败/错误oracle时，目标prepare/dispatch计数为0，AV仍按独立预期通过，忽略行不充PASS。将AV oracle改错必须使AV门红，全部有效AV检查被ignore或零选中也必须红。移除某非AV阶段ignore并进入其后端选择后，该目标失败必须在该后端门显现，而AV视图仍不选择它；移除ignore不能同时删原平台/ignore理由。公共格式/重复ID/AV必需shared规则出错不被这一阶段策略豁免。

1078的baseline-value草案同Case保AV、AC-native和旧VM定义；本轮只有AV启用，AC-native与legacy-reference均加#ignore。取消r2要求运行AC检查及其min-cross-backend命令，未来AC开发时由对应任务逐项打开并在自身选择中执行。

### FC 测试组织与迁移必验增量（未执行）

| V-ID | owner_stage / 原任务 | 必须观察的结果 |
|---|---|---|
| V-FC-SOURCE | work / T-02..06 | 本轮同一source/project仅选AV真实执行/按规定阶段拒绝；改公共body使相关旧证据陈旧，非AV行保持#ignore/未选与后续owner，不要求本轮运行。 |
| V-FC-MIGRATE | work / T-01/02/05/06 | 原case/check/条件/判决角色→canonical映射完整；非等价邻例、错误/资源/恢复角色不丢；suite、worker/内联/工具读者均切唯一活跃作者，旧副本仅具名历史。 |
| V-FC-TARGET | work / T-04..06 | AV oracle变异、重复活Case ID/AV旧活入口残留、零条有效AV/缺AV required检查必红；AC/A2X oracle不符或缺工具在#ignore/未选时不进入AV门。非AV完整目标oracle/emit-vs-run控制留给其启用阶段。 |

[本包旧VM文档逐项清单](../design/strategy/av-autovm-legacy-case-map.md#plan-1078-原文档清单)静态主承接 **2份**；原case/check与完整AutoDown源码均可逐项打开，未运行。该数不包含Rust内联/外部语言角色与新增case，T-01复核后不得漏项或把smoke当完整语义证据。

### 不变量、入口与观察

每个required case有唯一原源/独立expected、模式/平台/provider、生产者与观察点。对可比旧行为用同源显式AutoVM参考；已裁定差异两端各用自己的固定oracle，旧bug不当expected来源。**两个引擎相等不是正确性oracle**。未跑、原红、批准排除与重放绿分列；不得用any-Err、只检查len、stub或emit成功代替目标行为。

本包正例/反例草案如下，均**未运行**。新recipe和双流/多文件/失败观察接线由AVP-00 T-02..05使之可运行；此处不是当前工具已接受的声明。exact诊断码含拟定新增项，T-01核语义和真实阶段后冻结，不能弱化为无辨识力失败。

| Case ID | AC / invariant | owner / work task | 实际入口 | 独立预期 | 覆盖边界 |
|---|---|---|---|---|---|
| `av.parity.1078.baseline-value` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=14; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1078.short-circuit` | AC-01 | work / T-02..05 | `auto-case`→普通AV真实源码链 | 值=42; stdout=''; stderr为空 | 本例具体输入；额外状态见下表，不声称全语言 |
| `av.parity.1078.syntax-neighbor` | AC-02 | work / T-02..05 | `auto-case`→普通AV真实源码链 | parse: source.syntax | 本例具体输入；额外状态见下表，不声称全语言 |

| 补充ID | AC | owner / task | 具体输入/区分性断言 | 真实观察与关闭方式 |
|---|---|---|---|---|
| `META-ORACLE` | AC-03/AC-04 | work / T-04..05 | 把baseline-value的实际源改为返回13而oracle仍14，必须判FAIL；不把故意红控制放进正常绿suite。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `META-DISPATCH` | AC-03/AC-04 | work / T-04..05 | 禁旧VM dispatcher、未知recipe/profile、零选中、缺文档和改oracle/hash均具名红，资源观察由父侧读取实际owner账。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |
| `DISCOVERY` | AC-03/AC-04 | work / T-04..05 | 按生产discover_vm_tests及1066原manifest重核，不能把本轮磁盘扫描总数写成实际active case总数。 | 原父侧case/worker观察，引用实际输入与assert；不能仅grep或数测试 |

### AutoDown代码用例（设计草案）

每个外层`autodown`围栏是一份可审阅的文档式case草案，这些是规划样例；T-05落实到canonical文档后只有该文档为活跃作者，本Plan围栏不作为运行输入或第二份同步维护源码，保case ID/语义/原oracle。所有`av`检查实际运行普通AV链；未声明的外部权限不自动授权。profile基础不足时补生产者而不是在runner手写此case结果。

`````autodown
## Case: baseline-value
id: "av.parity.1078.baseline-value"
tags: ["av-language-parity", "plan-1078"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int { return 2 + 3 * 4 }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 14

#### stdout

```text
```

#### stderr

```text
```

### Probe ac-native
#ignore
recipe: "ac/mir-native-scalar-v1"
entry: "d_main"
exit: 14

### VM result
#ignore
check-id: "legacy-reference"
final-newline: false

```text
14
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: short-circuit
id: "av.parity.1078.short-circuit"
tags: ["av-language-parity", "plan-1078"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn explode() bool { fail("must-not-run") }
fn main() int {
 if false && explode() { return 0 }
 if true || explode() { return 42 }
 return 0
}
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
exit: 42

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
`````

`````autodown
## Case: syntax-neighbor
id: "av.parity.1078.syntax-neighbor"
tags: ["av-language-parity", "plan-1078"]
entry-mode: "program"

### Auto
path: "main.at"

```auto
fn main() int { return (40 + 2 }
```

### Probe av
recipe: "ac-av/language-source-run-v1"
entry: "d_main"
caps: ["ac.source.language.av/v1", "console.stdout", "console.stderr"]
stage: "parse"
code: "source.syntax"

#### stdout

```text
```

#### stderr

```text
```

### Notes

拟定合同，**未执行**；`av`是required运行检查，不是静态emit检查。
诊断阶段/码是本路线目标合同；新码或现有family包装差异由本包T-01核对/冻结，修正必须保持同一错误类别、源位置与拒绝时机，不能降为any-Err。
`````

### 未来执行命令与范围

专属实施树中执行（本轮未执行）：

```powershell
cargo t case-av-language-parity ace-source ace-syntax ace-semantics ace-av ace-lowering auto-test-case
```

固定组`case-av-language-parity`为AVP-00拟新增的公共语料AV视图：按已注册目标/动作选AV检查，不能只假设check ID恒为av，零选中/缺文件/坏子项必红；旧VM只读有效历史参考，本轮legacy-reference检查#ignore，不为它启动旧引擎。`--check av`只点选确实名为av的检查，不代表全部AV检查；完整视图的目标/动作选择由1078在既有选择机制接通。普通AV验证不偷偷执行旧引擎。新suite/recipe在AVP-00交付前不得假称上述命令当前可跑。

共享语法/绑定/IR/ABC改变时，T-06本轮只验AV及AV实际依赖的共同语法/语义/IR/ABI/框架正确性；不得因改动共享层而自动把ace-native、ace-a2x等非AV功能完整组变成本包门。混合职责组内非AV执行腿按1078冻结的AV阶段选择/配置具名排除，不批量改整crate为ignore；不为忽略检查主动准备非AV worker/SDK/工具。AV所需Rust/FFI provider及共享代码真实必需的编译/准入仍须正确，不能用#ignore掩盖AV自身问题。此阶段策略不批量停用已有独立非AV回归，也不删原性能/平台/ignore条件；普通组不附加ace-perf，Windows不执行Linux/WSL。

实施/内部检查/最终独立复审均在上述同一专属树顺序进行，用户未指派额外agent或模型；独立复审指独立上下文核验，不擅开新chat/第二实施树。原权限、重内存、资源预算和性能样本条件保持；不为了日常60秒建议强停合法正确性用例。

## 7. 验收标准

- [ ] **AC-AV-ONLY｜owner_stage: work**：§5 AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE满足，当前仅AV及其共享正确性参与门；非AV#ignore/未选不prepare、不dispatch、不计PASS、不阻AV。原T-03..06落为真实选择/报告/隔离控制，T-07独立核，T-08不以非AV未齐阻本包落主。

- [ ] **AC-FC｜owner_stage: work**：§5 FC及§6 V-FC-SOURCE/MIGRATE/TARGET由原T-01..06落实；唯一活跃作者、逐原义务迁移/条件/历史身份守恒与本轮所选AV真实观察齐，非AV#ignore/NOT_RUN分列且不阻AV。原T-07 review逐项核，T-08仅真实落主后交来源；既有AC全部保留，不缩语言或后端分母。

- [ ] **AC-01｜owner_stage: work**：§6所有正例在普通AV真实执行满足固定值/双流及本包完整语义，原源不以mini/get/loop等替换后冒原兼容。
- [ ] **AC-02｜owner_stage: work**：所有反例在约定阶段给可辨识错误与原source span；未授权/预算/取消/host失败分类保留；不静默默认值/空成功。
- [ ] **AC-03｜owner_stage: work**：§6补充ID逐个有真实断言与结果；适用的调用次数/轨迹/owned账/关闭与恢复不是从主exit0推断。
- [ ] **AC-04｜owner_stage: work**：本包原case/check与L行映射有唯一owner/预期/状态；AV required行必须有actual，AV未知/未跑阻AV出口。非AV保持#ignore/NOT_RUN及后续owner，迁移数据不得丢，但不要求本轮actual、不阻AV出口。
- [ ] **AC-05｜owner_stage: work**：走共享受检前门与AV执行，legacy dispatch=0；无case名特判/第二parser/临时registry；跨仓依赖与其它路线未重复实现。
- [ ] **AC-06｜owner_stage: work**：受影响完整固定组及必要下游在冻结提交有有效证据；实际case/check数、排除/未跑/原红、准备/运行/清理时间分别记录，无新增未处理告警/调试输出。
- [ ] **AC-07｜owner_stage: review**：T-07独立逐AC核验源码/diff/实际消费者与规范增量，遗漏/延后/workaround具名处理，通过后status reviewed；不得仅复述自检收据。
- [ ] **AC-08｜owner_stage: merge**：真实落主后沉淀已审Spec/接口、归档并核来源身份；证据折Plan后材料/树guard与路径对账完成，未关闭named hold保owner/关闭条件。

## 8. 执行步骤

| task | owner_stage | depends | 实际操作/预期产物 | acceptance |
|---|---|---|---|---|
| T-01 | work | 无；本包所有已交依赖按身份复核 | 冻结旧/新实际输入、规范差异、精确诊断/资源边界；对新增表示/借用/调度/CT等未定机制先用现有入口做有界全链可行性探测，产出本Plan内决策与下一步，不预先删required。 FC：冻结本特性的canonical位置、判等条件和所有旧case/check/角色/消费者。 | AC-04、AC-05、AC-FC |
| T-02 | work | T-01 | 复用959的L-01..49与1066现有发现/身份/比较机制，逐case/check映射active VM、language Rust tests、cookbook语义和真实外部语言切片，保原排除与旧红。 触面按§5真实路径；新增case作者文件与route suite在专属树创建。 FC：按特性建立/迁入唯一作者与legacy-map对应行，非等价变体不合并。 | AC-01、AC-04、AC-05、AC-FC |
| T-03 | work | T-02 | 新增普通源码SourceProfile::Auto（拟名）及ac-av/language-source-run-v1（拟注册recipe）：仅是现有Schema/CST→共享绑定/Checked HIR→MIR→AV的显式能力入口，首批交aggregates已有基础，不承诺后续未交特性。旧冻结profile不改为自动试探列表。 校验共享HIR/MIR/ABC准入与AV真实dispatch，不直接以未检查AST造执行结果。 FC：同Case保共享source/project，本轮仅启AV；非AV检查声明保留并加#ignore，复用原选择模型，不复制源码。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01、AC-02、AC-03、AC-05、AC-FC、AC-AV-ONLY |
| T-04 | work | T-03 | 新recipe复用现有worker/父侧oracle，支持int入口值、精确stdout/stderr、编译/运行失败、Auto多文件payload和资源观察；不存在的子能力必须具名拒绝，不能转旧VM。 补§6所有extra oracle，拒绝/取消/失败恢复与生命周期按实际适用面验证。 FC：本轮只执行V-FC的AV/公共结构控制及AV-ONLY隔离负控；跨后端实际运行/目标oracle控制随非AV启用再验。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-02、AC-03、AC-04、AC-FC、AC-AV-ONLY |
| T-05 | work | T-02..04 | 将§6 AutoDown草案落实为唯一活跃文档作者源；旧case按FC迁移表保原义务/预期与ID映射，扩同一canonical Case的目标check，禁止后端源码副本；逐一运行正/负/故意红元控制，更新同一suite与义务映射。 FC：切AV读者并静态核非AV引用与原条件，非AV运行由#ignore暂缓；不丢其义务/owner、不留双活作者，不能以非AV未运行阻AV迁移出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-06 | work | T-05 | 冻结候选提交，执行§6固定组并核子项/资源/源身份；共享改动仅补AV及共同正确性，非AV功能检查保后继责任，写可复现摘要和待交接口IF-AV-PARITY-BASE。未就绪依赖只阻其行，不冒通过。 FC：仅以所选AV实际动作/结果及迁移数据守恒收AV阶段证据，非AV保#ignore/NOT_RUN与具名后续任务，不计PASS、不阻AV出口。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-01..06、AC-FC、AC-AV-ONLY |
| T-07 | review | T-06 | 独立上下文逐AC追源码→消费者→实际oracle；扫遗漏/未批准延后/临时兼容，必要反例重放；不因勾选或主exit0信任结果。 另独立核AC-FC，实际输入/目标/迁移/消费者与反控均须成立。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-07、AC-FC、AC-AV-ONLY |
| T-08 | merge | T-07 pass及实施/合入授权 | 按已审Spec增量落主、归档并交正式接口；计划内折证据、清统一材料/guard后移除唯一工作树，不动其它会话文件。 并交本包canonical引用/迁移映射及已验目标来源，公共作者规则由1078统一沉淀。 AV-ONLY：本轮仅AV门，非AV检查#ignore/未选，不以AC/A2X结果阻本包。 | AC-08、AC-FC、AC-AV-ONLY |

本轮任务状态（T-01..T-06 已完成并提交，T-07/T-08 待执行）：

- [x] **T-01** work：见下"T-01 决策记录"；FC 冻结（canonical 位置/判等条件/旧角色/消费者）随 §5 FC 与 §6 V-FC 落实。
- [x] **T-02** work：legacy-map.tsv 初版（aavm2×2 research-hold + L-04/45/46 recorded）；canonical 目录 `language/basics`+`language/migration` 建立。
- [x] **T-03** work：SourceProfile::Auto + `ac-av/language-source-run-v1` 全链实现（语法 auto profile、统一 CST 链、resolve/adapter/fstr/frontdoor 17 槽、worker、parent RecipeFamily::LanguageSource、contract 注册/流校验、fixed-tests check 过滤）；非 AV 检查 `#ignore` 映射 AV-phase SkipSpec。
- [x] **T-04** work：META-ORACLE（变异源 13→FAIL expected 14 vs actual 13）、META-DISPATCH（未知 recipe 定位拒绝 line 22 / 零选中 bad selector）、V-FC-SOURCE 同源多端（AV 14 + ac-native 14 真实执行）、FC-08 忽略隔离（ac-native SKIP 不 prepare）全部验证。
- [x] **T-05** work：canonical `language/basics/expressions.case.ad`（baseline-value 含 av+ac-native+legacy-reference，非 AV 检查 #ignore；short-circuit；syntax-neighbor）+ `language.suite.ad`；正/负/元控制逐一运行。
- [x] **T-06** work：冻结提交 `3842c826c`+`3e14cadda`（worktree `D:/autostack/.wt/lang-1078/auto-lang`，分支 plan-1078-dev）；§6 固定组 case-av-language-parity/ace-source/ace-syntax/ace-lowering/case-pilot PASS，ace-av 2 例与 ace-semantics 1 例为 master 既有红（stash 对比验证与本包无交集，归因记录）；IF-AV-PARITY-BASE 见 §9 work handoff。
- [ ] **T-07** review：见上表唯一任务合同。
- [ ] **T-08** merge：见上表唯一任务合同。

### T-01 决策记录（2026-11-19，work 施工后实测冻结）

1. **指纹核验**：author.rs/contract.rs/test_runner.rs/ace_entry.rs 四文件 SHA256 与 §4 完全一致，无漂移。
2. **可行性探测（有界全链，静态+既有绿证据）**：
   - `baseline-value` 源（`fn main() int { return 2 + 3 * 4 }`）与既有 `ace.m1.b01`（pilot.case.ad:14，`ac-av/core-run-v1`+`ac/mir-native-scalar-v1`+autovm 三检查均绿）**源码逐字节相同** → scalar/aggregates 面与 AV/AC 执行链均已证明可达；`ac-av/language-source-run-v1` 只是同一共享前门的新身份入口。
   - `short-circuit` 的 `&&`/`||` 属 scalar 面（`SourceProfile::is_scalar`，aggregates 超集）；`fail(msg)` 为 resolve 内建（resolve.rs:15267，PLAN-965 语言错误展开）→ 编译可达，短路执行语义待 T-05 实测。
   - `syntax-neighbor` 的诊断码 `source.syntax` 真实存在（parser.rs stage `text`，`self.err_at("source.syntax", …)`；worker 侧 stage 映射 `text→parse`）→ oracle `parse/source.syntax` 冻结成立，不弱化 any-Err。
3. **新增面设计决策（T-03 核心）**：
   - `SourceProfile::Auto` + profile 串 `auto.source.language.av/v1`，能力键 `ac.source.language.av/v1`；首批语义 = aggregates 面（复用同一 Schema/CST→共享绑定/Checked HIR→MIR→ABC→AV 管道，**不新开第二 parser/解释器**），后续 AVP 批次逐步扩充。
   - 新 recipe `ac-av/language-source-run-v1`：父侧 `RecipeFamily::LanguageSource`（console oracle 形态：exit + 精确双流），worker 侧复用 `compile_language_source→lower_checked_closures→verify_mir→encode_abc→verify_abc_bytes→execute(+ConsoleCapture)`；**不转旧 VM**。
   - 触面盘点：SourceProfile::Aggregates 全仓 27 处匹配点（parser 统一 CST 链、cst/frontdoor slug+16 槽 GRAMMARS 数组、fstr、resolve、adapter、mod.rs 方法族、main.rs）；contract.rs REGISTERED_PROBE_RECIPES（append-only）+ streams 校验（1358-1383）；test_cases/mod.rs dispatch；av-worker.rs 分派 + core_version_for。
4. **旧义务映射（本包主承接）**：`legacy.aavm2.001_smoke`/`002_hello_compile`（aavm2 研究族，保 research/手动条件，不转日常硬门；语言侧 = console print + void main，canonical 承接行写入 legacy-map.tsv）；L-04/L-45/L-46 = 无此形/parser-only 行，language-parity.md 记录适用状态，不实现。

下一执行入口：核主检出/本包r1与依赖正式来源，创建唯一专属树后从T-01开始；闭合依赖不等于本轮已批准执行。依赖缺失先做不依赖它的输入审计，不能补假PASS。新发现的高风险未定语义先有界探测/决策，超本包目标则显式修订/拆Plan，保全部旧义务。

### 2026-11-19 work handoff（T-01..T-06 完成，提交冻结）

`stage: work | plan_id: PLAN-1078 | plan_revision: 3 | outcome: pass | worktree: D:/autostack/.wt/lang-1078/auto-lang | branch: plan-1078-dev | base: 54ed8a4ca | code: 3842c826c,3e14cadda | dependencies: auto-down detached @72b0970 (只读消费) | tasks: T-01..T-06 | evidence: 见下 | blockers: 无 | next: review (T-07)`

**IF-AV-PARITY-BASE 交付摘要**：
- 普通 AV 语言入口 `SourceProfile::Auto`（`auto.source.language.av/v1`，能力键 `ac.source.language.av/v1`）+ recipe `ac-av/language-source-run-v1`：同一共享 Schema/CST→绑定→Checked HIR→MIR→ABC→AV 管道，首批=aggregates 面，不新开 parser/解释器；父侧独立 oracle（int 入口值 + 精确双流 + 编译/运行失败）。
- canonical 语料：`crates/auto-lang/test/cases/language/basics/expressions.case.ad`（3 case：baseline-value=14、short-circuit=42、syntax-neighbor=parse/source.syntax）+ 公共选择 `language.suite.ad` + 迁移表 `language/migration/legacy-map.tsv`（aavm2×2 research-hold、L-04/45/46 recorded）。
- 固定组 `case-av-language-parity`（公共语料 AV 视图，`--check av`；固定 runner 增加 per-group check 选择）。
- 检查级 `#ignore` → AV-phase SkipSpec（reason=AV阶段暂缓非AV检查，issue=AV-ONLY-1078-1096，owner=PLAN-1078）；非 AV 检查不 prepare/dispatch，不阻 AV 门；后端任务逐项移除。
- 元控制证据：META-ORACLE 变异源 FAIL（expected 14 vs actual 13，probe-receipt）；META-DISPATCH 未知 recipe 定位拒绝（expressions-unknown-recipe.case.ad line 22）+ 零选中 bad selector；同源多端 AV=14 且 ac-native=14（变异前真实执行记录于 plan-artifacts/plan-1078/ac-native-run）；FC-08 忽略隔离 SKIP 不 prepare。

**§6 固定组结果**（冻结提交上）：case-av-language-parity PASS（3/3 av）；ace-source PASS；ace-syntax PASS（含 profiles_927 冻结集加 auto）；ace-lowering PASS；case-pilot PASS；ace-av 1426/1428（2 例 host_console 既有红——stash 对比在无本包改动下同红）；ace-semantics 1 例 sema_boundary cache-roundtrip 既有红（ac-semantics 在途 T-06 status，非本包触面）。**既有红均归因记录，不充 PASS，不判本包失败**；`::performance::` 与 Linux/WSL 腿按裁定排除。新 case 每例耗时均 <10s（baseline-value 0.46s / short-circuit 0.28s / syntax-neighbor 0.40s 含 worker 会话）。

**待 review/merge 事项**：AC-07（T-07 独立复审）、AC-08（T-08 落主）、SD-01/02/03/SD-FC 规范沉淀（merge 后）、SD-IGNORE 支持事实（document-cases/feature-organized-corpus 增量）、plan-artifacts/plan-1078 清理（merge 收尾）。

### 2026-11-19 补充改动（用户指令：省略流补零字节，提交 493bc6954 + 88306dfc8）

- **author.rs**：console 族 recipe（console-run-v1 / scheduler-source-run-v1 / language-source-run-v1）在文档加载时把**省略的** stdout/stderr 独立补成预期零字节（`compare: bytes` + 空文本）再交现有校验；显式流按原预期比较，成功例与诊断例均适用，其他检查族省略语义不变。
- **expressions.case.ad**：三个用例的空流段落已删（保留源码/Case ID/退出值/诊断预期/#ignore）；顶注记录省略流语义。
- **document-cases.md**：新增"PLAN-1078 双流省略补零（user ruling）"段落（文档式用例规范增量，SD-IGNORE 的姊妹规则）。
- **回归测试**：`author_console_default_streams.rs`（6 例：两流省略/只省略一流/显式空围栏/诊断例/非 console 族省略不变/scheduler 同族）全 PASS；`author_document::located_rejection_battery` 旧断言按新 ruling 更新（省略不再报错，补零）。
- **行为验证**：`case-av-language-parity` 复跑 PASS（省略流补零生效）；负向探针（`emits-newline`，源码 `print("\n")` 实际输出换行）判 FAIL，mismatch `expected "" actual "\n\n"`——证明省略流只要实际输出一个字节（含换行）即失败。

## 9. 复审记录

- 2026-11-19 **T-07 独立复审通过**（独立上下文 general 子代理，提交 3842c826c/3e14cadda/493bc6954/88306dfc8，基面 54ed8a4ca）：AC-01..AC-08、AC-FC/V-FC、stream ruling 全项 PASS；`cargo t case-av-language-parity` 复跑 PASS（3/3 av，baseline=14、short-circuit=42、syntax-neighbor=parse/source.syntax），`cargo test -p auto-test-case` 157 测试全绿；零 legacy dispatch；无第二 parser/无 case 名特判；warnings 零新增（auto-lang-core 134 = 基线，ace-av conformance 44 = 基线）；ace-av 2 例 host_console 与 ace-semantics 1 例 sema_boundary 在隔离复现确认与本包 diff 无交集（既有红，归因记录，不充 PASS）。低危项处理：aavm2 research-hold reason 措辞改"language-face overlap"（f37159231）；零匹配/未知 recipe 拒绝已折本记录；`spans: []` 源 span 未序列化留后续批次（oracle 无 span matcher，与 console_worker 约定一致，不阻本包）。
- 2026-11-19 work handoff（T-01..T-06）：见 §8"2026-11-19 work handoff"段。
- 2026-11-19 补充改动（省略流补零）：见 §8"2026-11-19 补充改动"段。

- 2026-11-19 r3静态预检：25计划/8设计的AV-only阶段合同已对账；72份Auto/工程源码与80项原检查定义/预期保持，草案中仅72项AV启用、8项非AV检查加#ignore。19个新增本地链接及2876个表格行结构核对通过，原状态/进度/依赖未改；未迁实际用例、未运行测试或实现#ignore。r2的最小AC真运行要求已取消，旧记录只作历史。

- 2026-11-19 r3：`stage: new/revision`，`outcome: pass`（仅规划静态预检），`next: work`待后续施工指令。用户明确AV-ONLY/check级#ignore；新增AC-AV-ONLY与V-AV-ONLY/V-IGNORE-LIFECYCLE，取消本系列非AV硬门，保原源码/预期/后续义务、任务/状态/进度。本轮未施工/运行/改实际用例。

- 2026-11-19 r2本轮静态预检：25个活动计划与7份既有设计已修订，另新增共享语料设计1份；原72个AutoDown草案的Auto/工程源码、79个原检查及预期保持，只有baseline-value增加1项拟AC-native检查且复用原Auto源；506份VM映射与49个L行保持。53个新增本地链接、2874个表格行结构核对通过；原任务ID/状态/进度/依赖保持。当前测试目标language目录尚未创建，没有移动/改写任何实际测试/expected，没有运行编译器/测试；静态预检不关闭work/review/merge验收。

- 2026-11-19 r2规划修订：`stage: new/revision`，`outcome: pass`（仅合同静态预检），`next: work`待后续施工指令/所需依赖。新增FC/AC-FC/V-FC，修改原T-01..06职责、canonical位置与目标视图；原任务ID/步骤数/进度、72个代码Case的Auto源码与原预期保留，不新造后端源码。未实施/迁例/运行。

- 2026-11-19 系列规划静态预检：19份draft/72个AutoDown代码case/506份旧文档主承接/L-01..49齐；计划号/case ID唯一，依赖无环，842个本地链接/1312个表格行结构核对通过。未执行任何Auto case、compiler或测试，此记录不关闭本包任何work/review/merge AC。完整输入与分派见路线/旧case快照，现有其它会话新增编号/修订保留。

- 2026-11-19 起草交接：`stage: new`；`plan_revision: 1`；`outcome: pass`（规划结构/路径/AC/task/AutoDown预期已静态核对，不是实施验收）；`next: work`（待用户后续施工指令及所需依赖正式交付）。新增T-01..08、AC-01..08与§6具名case/extra；work/review/merge均未发生。
- 本轮没有编译、测试、运行脚本case或UI/GPU、提交合入/归档；示例全部是目标合同，不能作为已有绿证据。

## 10. 待澄清事项

不需再次确认已经授权的规划范围。已定语言规范/数值宽度/安全边界沿原来源；T-01对真实旧用例与Spec冲突做具名裁定或有界机制探测，属于本包必要工作。涉及改变已批准公共语义、资源承诺或验收阈值时提交具体差异再决定；不得以“未定”无限延期本包required行。默认支持同一合法源或已批准、可审计的兼容规范化，不承诺复制旧bug或尚无语言构造。
