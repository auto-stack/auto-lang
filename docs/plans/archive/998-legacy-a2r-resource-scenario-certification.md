---
plan_id: PLAN-998
status: archived
feature_name: 旧A2R六资源场景观测与独立认证闭环
author: [Codex]
created_at: 2026-11-08
updated_at: 2026-11-08
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-scenario-oracles.md, docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-test-case/resource-lifecycle-observations.md, docs/specs/auto-lang/trans/architecture.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [legacy-a2r, auto-test-case]
depends_on: [PLAN-991, PLAN-993, PLAN-994, PLAN-996, PLAN-997]
current_step: 12
total_steps: 12
---

# [PLAN-998] 旧A2R六资源场景观测与独立认证闭环

## 0. 变更摘要

按用户H-07裁定保六个scenario:a2r required，建设旧A2R真实产物的资源/错误观察与独立场景认证，纠正来源注记并逐场景关闭；不新增NA，不从只emit/返回值推资源语义。

本轮只规划，所有实现/认证/复审未执行；当前drafting r1/0-of-12。原991待裁定项由本轮具名选择解决，实施策略由T-01/02有界证明，不预填运行PASS。

## 1. 目标

本包原6个scenario:a2r格逐项真实关闭，source/语义/身份/完整产物/真实执行/独立oracle/strict认证/日常准入及余账可追溯。原source/outcome/expected与producer义务保留，不能以新HIR/NA/accepted凑绿。

非目标：全语言新特性、M2产品替换或旧引擎退役、改变旧默认int/所有公开API、放宽预算/原样本、减少313 required/改旧批准、修939/941或其余不相关代码、把证据哈希当恶意本机历史签名证明。

## 2. 架构方案

旧source→现绑定/准入→相同RustTrans（显式有限配置如本包需要）→目标自身表示/私有helper→完整实际产物→原受控工具→独立typed/value/events oracle→域专用strict reader→原required父判。新配置/新证书域须版本化且所有producer/consumer同时适配，不在运行时绕回旧VM/AV/新HIR业务实现。

## 3. 技术栈

- `crates/auto-lang-core/src/trans/rust.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang-core/src/trans/rust/expr_emit.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang-core/src/trans/rust/stmt_emit.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang-core/src/test_cases/a2r.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `docs/specs/auto-test-case/ace-m1-scenario-oracles.md`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `docs/specs/auto-test-case/resource-lifecycle-observations.md`（现存，实际符号T-01重核；拟新接口在§5标注）。
- 现入口：`crates/auto-lang-core/src/test_cases/a2r.rs`、`ace_certify.rs`，`crates/auto-lang/src/bin/ace-certify.rs`、`scripts/verify-ace-m1-852.ps1`、`scripts/tests/test_ace_m1_acceptance.py`及`scripts/test-groups.toml`。

## 4. 需求分析与背景调查

### 用户裁定、范围与真实生产路径

2026-11-08用户已逐项明确选择：H-04“补齐旧A2R有限集合”；H-06“保留原checked/trap义务，不全局改旧int语义”；H-07“保留required并补通道”。三项均不是待答复裁定，不再提出NA/accepted/新HIR迁移作为默认方案；原16格、required313、原source/outcome/expected、NA/32 accepted/45 future及零cert-gap守卫保留。此授权落实为本组三份**规划合同**，不等实施/认证/合入授权，不批准任意新golden字节。

**实施授权（2026-11-08）**：用户以`/goal 完成计划998`（auto-plan-goal单授予）授权全链：在`D:/autostack/.wt/lang-998/auto-lang`（分支`plan-998-dev`，基面master=a95d6dc41）内实施、独立复审、有界修复与合入；不改变目标/验收/范围/预算。开工登记：plan-artifacts scope `D:/autostack/plan-artifacts/plan-998/`（owner=PLAN-998，在途）。

读时master=`70539171f`，D992/D993/D994均归档，32份原值证书已落主线，剩16格；941分支旧组合可能还在记21/38，不在本包改其账或拿旧读数倒退主线。原991分诊是基线，不是本包正式执行证据；其scope已清，后续重测以当前源码为准。

只有旧production `RustTrans`真实生成并执行的完整产物可核销原a2r格；不跑AV/native/新HIR取值代替，不按case ID/路径/字面量特判，不用测试host拼预期事件/把reference改名actual。旧语法→绑定/语义准入→旧RustTrans子模块→目标私有表示/helper→完整实际文件集→受控编译运行→独立oracle/reference→strict父reader，每段有实际身份与来源。


### 只读输入指纹

| 路径 | SHA256 |
|---|---|
| `docs/plans/archive/991-ace-m1-legacy-a2r-cert-gap-triage.md` | `37fbf69aff2616d8ce03bb537f5592f1f1bab2c6b84735c4f1a313e230a64632` |
| `docs/specs/auto-test-case/a2r-golden-certification.md` | `4431314d56aff970aae7fa96d4e4a688101ef30e90ff17c89dba514edd7080b7` |
| `docs/specs/auto-test-case/ace-m1-scenario-oracles.md` | `6179f473f6ef7f5568b7acdf1fcc2dc3367ace36bfe6a35b0fffe7aa908ea24d` |
| `docs/specs/auto-test-case/ace-m1-applicability.md` | `787e74b4432c1a061cc8b0ab2b2506649962bfbde952fddcfbdbd5ad35fa1bfa` |
| `scripts/verify-ace-m1-852.ps1` | `2da7cfcc78f242e16afb32241f23516646d62b221048616ec814f2dfa4e38075` |
| `crates\auto-lang\test\cases\ace-m1\binding.tsv` | `35ecebeb3ff55db3c96c7d1ac50ee81032f5f2e4d195ad0775775f56015c8a44` |
| `crates\auto-lang\test\cases\ace-m1\applicability.json` | `74c9109bbe7ec8596e6fa1bbc2ab9f4f5640725a748d0e63e53eb5e07ee275c0` |

## 5. 详细设计

### 原格位与冻结期望（本包全部待执行）

| 格位 | 当前a2r token | 原outcome / expected | 实际来源定位 |
|---|---|---|---|
| `scenario:R01:a2r` | `emit-ok-run-unverified` | `events / lifo` | `ace-m1-scenario-oracles.md`的R01原输入/不变量；T-01冻结实际source/辅助fixture |
| `scenario:R02:a2r` | `snapshot-only-cert-gap` | `events / -` | `ace-m1-scenario-oracles.md`的R02原输入/不变量；T-01冻结实际source/辅助fixture |
| `scenario:R03:a2r` | `emit-ok-run-unverified` | `events / -` | `ace-m1-scenario-oracles.md`的R03原输入/不变量；T-01冻结实际source/辅助fixture |
| `scenario:R04:a2r` | `emit-ok-run-unverified` | `events / once` | `ace-m1-scenario-oracles.md`的R04原输入/不变量；T-01冻结实际source/辅助fixture |
| `scenario:R05:a2r` | `snapshot-only-cert-gap` | `events / 42` | `ace-m1-scenario-oracles.md`的R05原输入/不变量；T-01冻结实际source/辅助fixture |
| `scenario:R06:a2r` | `snapshot-only-cert-gap` | `events / -` | `ace-m1-scenario-oracles.md`的R06原输入/不变量；T-01冻结实际source/辅助fixture |
### 六个原场景的source与可观察关闭合同

T-01逐场景冻结当前原source/typed helper/独立oracle的真实映射，先纠正991已证R05引用不存在m1_k02.rs、R02仅V05就冒全域、R06缺源锚等来源注记。不能拿corpus:R01..R06值证书抵scenario同名格；不能改原resource oracle为“结果正确”。原schema无法表示的观察是T-02通道责任，不可动态减required。

拟沿旧A2R显式观察配置（如`legacy-a2r.resources-observe/1`，T-02冻结）在**生成实际资源操作路径**的拥有/借用/分配/提交/销毁/取消边界插桩；默认生产观察关闭，插桩不改业务结果。可复用既有资源计划/旧escape事实与经审定的目标私有helper，但不得以wrapper里手写一份业务算法产生事件，或把AV/native资源事件借给旧A2R。新增helper/表示/观察选项全部入actual身份。观测实现策略先用R01/R02/R05各一条小全链比较可行路径；无法观察真实操作就needs_replan而非交空通道。

| 场景 | 必需真实输入/consumer | 区分性观察与关闭条件 |
|---|---|---|
| R01 LIFO | 原T03/R04/R05与正常return、branch、break/continue/临时各实际旧产物 | 每owner create/transfer/release唯一责任、逐帧LIFO；转移不是释放。错序/漏/双释放即红，最终总数相等不够 |
| R02 fault | 原L04/L05/V05/LSTR/K02创建/clone/扩容路径（对应现规范完整域） | 每真实分配/准备域fault1..N/N+1及清fault同进程retry；失败旧值/源owner保持，半树全清，不能仅一个fault或人为常量attempt。费用/资源归属与实际运行相关 |
| R03 stale | 996真实List buffer借用/代际操作与合法fresh/无变异近邻 | 过期place/view在危险读前受检拒，buffer_reads只含合法读；若源前门静态拒与typed runtime probe分别登记，typed仅验证原明确允许的helper面，不冒未通过source |
| R04 variant | 原V06 Err(owned old)→Ok(42) | old活臂恰一次释放、禁止访问inactive payload、结果42；copy生成多个析构权/晚到函数退出才释放的错误实现可被抓 |
| R05 closure | 原K02创建者退出→返回closure→调用→最后drop，原owned Str/env必须在途存活 | 原语义42及env/backing最后owner释放，反复建销不漏。K02旧parser/返回fn发射缺能力须在本包有界补源入口，不能换成K01；corpus:K02既有accepted token仍按批准保留，不自动变绿 |
| R06 exits | 原深owned树/collections/closures实际运行到budget/cancel/trap/close/return出口 | 每owner必要cleanup、实际截止/最后pin收敛及typed失败；合作取消点与父Job kill分态，kill不能充资源释放。无剩余预算时cleanup仍完成，不能靠catch任意panic吞原错误 |

R02的原固定fault样本/规模表与R06深链/预算/平台条件T-01从现oracle引用冻结，原915/919/923/924路径仅为规范/独立对照，不据包名复制固定attempt数字来伪造a2r实际分配数；新的旧A2R表示有真实attempt清单与逐点覆盖，对原必须域逐项映射，原输入/规模不删。若旧A2R内存机制需要目标私有可失败helper/owner包装，T-02先证明语义、触面和资源预算，再依现单源准入消费，不复制infer/ownership分析或另起通用runtime工程。K02源支持补齐仅服务原R05义务，超出有限场景需求的大语言工程须先修订分包，六格原终验责任不可消失。

### 独立场景认证与证据原子性

拟schema `ace.m1.a2r-resource-scenario/1`、独立recipe/CLI `legacy-a2r-resource-certify`、`oracle/scenario-certs/`与`legacy-a2r-resources-998.{case,suite}.ad`、固定组`case-legacy-a2r-resources-998`均为新路径/接口，T-02/08使能；复用997已审的typed退出/身份/strict策略但不得把trap schema当scene或要求四端FFI同能力。

证书对每R行含实际source/辅助输入/编译配置/runtime观察版本/完整actual/header/helper/工具/flags/依赖、独立预冻结事件不变量与参考程序、逐check/故障域/退出原因/真实raw运行/终态、owner与字节/危险读等。独立reference由文字oracle手写，与actual分别编译运行/检查；它可有不同分配数量，按各自真实allocation域扫fault并分别验证规范，不要求两实现事件逐字相同。手工不变量/事件拓扑/cleanup时点为oracle，不由当前actual录下轨迹自动变expected。

只编实际产物/事件schema合法/返回42不足以认证；各源运行必须真实覆盖原状态，删一个fault/错转移/伪trace/合法ctx全拒/父超时/晚失败/collector未收齐均不PASS。持久证据只在完整成功检查后原子安装；失败保旧有效证据或明确STALE，不能覆坏件。daily沿已审完整资源actual fileset现场emit+bytes与新域strict证书身份核查，零compile/guest，显式certify单列。父acceptance新增六场景域真实证书要求，伪改六token为ok且删证书必须红；required313与批准件不改。

正式依赖D996的集合语义与D997通道基础、D993/D994旧closure/text兼容基面；不复用旧未编码32值schema承载resource字段。全部六场景成功后才从剩6gap到0；值域38、trap4、scene6分别报告。旧K02 corpus accepted/其它NA依原裁定保持，不用场景成功顺带改变其历史规则。最终证据交941按原门独立接纳，0gap不跳过其余真实门。

### 金样、证书、日常与准入守卫

原source、outcome/expected、独立参考/场景文字oracle不改；新增参考必须先从原规范独立推导并审阅，不能actual回填。golden来自当前生产actual但需逐源G04审阅：源语义/求值序/拥有借用/错误相位与完整Rust字节对应后才冻结；审阅表+SHA为工作材料，最终独立review重核，不以恰同一个整数替语义证明。不得认证时剪main/branch/runtime/deps；原值cert `ace.m1.cert/1`、wrapper、flags与std-only保持原含义，新trap/scene证据独立域，不给旧schema塞可选字段冒兼容。

原`Get-CellState`以任意ok前缀分类为green/cert=none，新增trap/scenario token不能借此绕过实证。T-01冻结新域专用token/recipe/schema与全reader清单；T-03/证书任务必须实现**独立于token表面绿**的逐域required证书/执行完整性验证：删除证书或伪改ok、错域、截断、未知字段、错profile/source/target/tool/runtime/oracle/nonce、零selected、无实际终态均拒/STALE/不可签。32旧值域严格读回，trap/scene分别计数，不把证书文件总数当覆盖。

只有本包目标格有全部真实认证证据才改token与说明；同步source-manifest中实际变化的binding/source/golden/cert/suite/规范身份，不宽泛改旧批准hash。原313 required集合不变，缺口按格而非按token机械重算；零gap守卫恒保留。capability未实现/infra/资源拒/未观测都不能填PASS。三个Plan各自交付与全M1签章分开，941原owner最终独立接收证据；本轮不改939/941，后续也不借其在途代码、不开会话、不发消息。

### 资源与验证纪律

复用原contained工具执行器、Job全树回收、独立有界排水、120s compile/30s run/每流1MiB及原场景/recipe更严预算（source/meta4MiB等），不新造runner/DB/meter、不降低原fault样本/压力/ignore/平台条件。内部fault/预算与父Job硬截止分态；OS杀进程不等逐owner释放。清理路径不得因预算耗尽或再次取消跳过；断言必须真实观察旧值/资源余额/危险读次数。

日常原base+992/994扩展及本包显式已审扩展，emit/bytes/strict身份检查不启动rustc/cargo/被认证目标；显式certify才编译运行。工具准备单列，旧32证书仅各自身份不变才strict reused，改变的profile/runtime/工具/产物必须如实STALE后重验。正式认证/资源采样与939/941错峰，不能利用同机竞争放宽门。

最终work完成必须全部本包原义务与补例闭合，不能用系统正确报red当功能交付。实现→内部自查→最终独立上下文review同一唯一树，最终review前停实现写入，不自动spawn；范围/预算/语义无法闭合先needs_replan，原required不删。已通过且输入身份未变证据可说明复用，最终冻结职责并集只按真实触面跑一次；周期cargo t all仍主检出单实例due门。

### 三包实施顺序与交接

[996](996-legacy-a2r-finite-list-certification.md)与[997](997-legacy-a2r-checked-i32-trap-certification.md)的私有探查/独立oracle/通道设计可并行；两者旧emitter、注册、cert-reader、script、manifest、Spec全文件按稳定SHA单写，推荐996先交付，再997。998可先做场景输入/原runtime/反例调查；其正式实现待D996（集合真实消费者）与D997（已审typed退出/strict新域接缝）后集成，不占窗空等。它不要求939/941整包D才调查，也不让两者接走漏项。

推荐实际关闭链：16→10（996六集合）→6（997四trap）→0（998六场景）。若996/997交换交付顺序，中间为16→12→6；必须按真实闭集对账，不能为了数字先改token。全部真实交付后，值域32→38，另有4份trap与6份scene域证据（不同域不混数）；原base/扩展的完整Case/check及余账由入口真实枚举。0gap只是M1必要条件，941其它required未全绿仍不可签。

996/997/998只交原格位→source/profile/旧producer/helper/runtime/tool→完整actual/reference→认证/raw/strict结果→原oracle的材料及已知边界。若941需要新reader/版本适配，明确字段/消费条件交原owner后续安排，不在本包更改其Plan/在途代码，不自动发送消息。原991归档不重开，历史“待裁定”由本组三份批准记录承接。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-test-case/ace-m1-scenario-oracles.md` | 六R旧A2R源/观察/typed终态与独立认证实际接线，纠正原snapshot注记，required保持 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |
| SD-02 | modify | `docs/specs/auto-test-case/a2r-golden-certification.md` | 本包实际有限能力/独立域与已关闭格、保留边界及source/profile/reader约束；历史分诊只作来源，不再用过时总数 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |
| SD-03 | modify | `docs/specs/auto-test-case/ace-m1-gap-audit.md` | 本包实际有限能力/独立域与已关闭格、保留边界及source/profile/reader约束；历史分诊只作来源，不再用过时总数 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |
| SD-04 | modify | `docs/specs/auto-test-case/resource-lifecycle-observations.md` | 本包实际有限能力/独立域与已关闭格、保留边界及source/profile/reader约束；历史分诊只作来源，不再用过时总数 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |
| SD-05 | modify | `docs/specs/auto-lang/trans/architecture.md` | 本包实际有限能力/独立域与已关闭格、保留边界及source/profile/reader约束；历史分诊只作来源，不再用过时总数 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |

## 6. 测试设计

所有V均为未执行拟议用例；具体新入口在对应任务使能。

| V | AC / invariant | 输入/实际入口与owner | 区分性oracle/反控 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/02/03 源与身份不互冒 | T-01/02，六场景真实冻结源/语义参考，特别K02、LSTR及非同名corpus输入 | 每R行有真实producer/consumer；不存在m1_k02等引用先补真实来源，不能以32/38值cert顶六场景 | 原accepted与source hash不变，支撑能力缺口须实际修复 |
| V-02 | AC-01/02/03 所有权时点/变体 | T-03，R01+R04实际旧产物及错序/transfer当free/早释放/多owner负控 | LIFO/old臂一次释放/不读非活臂成立，真实事件计数与owner存活相互核对 | 只总数或函数退出后统一free不算原生命周期保证 |
| V-03 | AC-01/02/03 fault/stale/返回closure | T-04/05/06，R02全部allocation域、R03合法+stale、R05原K02 | 每域fault/N+1/retry全覆盖，危险读0，creator退出后有效与最后env释放；合法近邻绿 | AV/native事件不代旧A2R；typed helper不代源R05 |
| V-04 | AC-01/02/03 错误/取消/深链cleanup | T-07，R06实际预算/取消点/trap及原深链/小栈规模 | typed原因与真实cleanup/peak/最后pin收敛，零余量仍清理；父kill/无运行不能绿 | 原样本/平台/预算不减，unknown不填0 |
| V-05 | AC-01/02/03 六证书/父准入闭包 | T-08/09，新域证书/原始运行、缺fault或整R行/错schema/伪ok/旧runtime | 每R全部必需check完成才安装/授信；新旧域互拒、日常零spawn，缺证据即M1不可签 | 旧value与trap保持独立域，0gap不等全M1pass |
| V-06 | AC-04 分母/域/交接 | 最终work任务原binding/applicability/source-manifest与本包全部格 | 只本包真实认证可核销，313集合hash/批准NA/accepted/future守恒，伪ok但无证书不签；计划通过非全M1pass | 939/941不写，不自动发消息/新会话 |
| V-07 | AC-05 独立复审 | T-10同树独立上下文重建source→实际runtime→observer→oracle→strict→父门 | 语义/原始执行/正反控/所有reader/SD/未观测逐项核，未知必需面不得默认绿 | 内部自查不替最终review，不自动spawn |

未来显式验证使用预构建正式入口，T-01冻结实际help/CLI及Case集合；原`ace-certify --corpus <tree>/crates/auto-lang/test/cases/ace-m1 --only <value-ids> --scratch <scope>/cert-scratch`仅限原value域，不能拿它选trap/scene冒已支持。新域入口由本包任务使能，当前不存在，需沿原CLI/父adapter/受控执行器实现；不得为取help偷偷构建或执行实现。本轮仅静态检查。

通用负控：原golden变1B、合法证书删一份/改一轴、token伪改ok、独立参考审核缺失或同一artifact冒充reference、缺原命令/执行终态、空选择、未知field/旧schema、超流/晚失败/cleanup未回收；必须严格拒或诚实非绿，合法同来源非零owner/ctx应接受。真正producer错误的负控优先使用baseline复现与scope隔离副本，不破坏他会话文件。声明新scope/skip不代执行数；记录selected/executed/条件排除/未执行/首次红/重放绿及准备/运行/清理时间。

现有职责基础`cargo t a2r infra-ace-m1 auto-test-case ace-adapters case-ace-m1-base case-legacy-a2r-cert-992 case-legacy-a2r-text-994 case-ace-m1-a2r-emit-917`（T-01核实际触面冻结必要并集）；若修改共享语法/ownership/类型准入，须补真实对应parser/infer/ownership消费者完整组，不能只跑a2r；若未触则记录免跑理由，不为Cargo依赖自动全仓。拟新组见§5，正式注册后须实际发现/执行完整Case/check。性能/深链/故障注入原串行/资源/平台条件保留，新字节/诊断/typed协议reader需端到端反控，不只脚本合成包自证。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | 本包全部原格位的真实旧producer/source/profile/依赖/完整actual来源正确，必要有限支持与独立合法近邻成立，非新HIR/host业务替代 |
| AC-02 | work | §5每个原值/trap/资源不变量及V补例在真实编译运行/观察上成立，错误相位/拥有cleanup/源序等不能只靠表面返回值或伪计数 |
| AC-03 | work | 经G04审核的独立完整新证据、strict全reader/日常零spawn、错域/坏件/缺Case/伪ok等反控与原职责通过；原证书身份不变才复用 |
| AC-04 | work | 原313/批准/源oracle/余账守恒，实际缺口按本包逐格闭合，全部SD候选和941只读交接可核，无未经批准删项/绕守卫 |
| AC-05 | review | 同树独立上下文逐AC/V/SD/全部reader/资源边界/遗漏/健康审查，绑定r1与冻结commit/工具配置；未知必需面不PASS |
| AC-06 | merge | review pass后冻结职责/漂移核验、SD沉淀读回与交付归档、实际输出根对账、scope摘要后整删/guard清树，真实结论与剩余项诚实记录 |

## 8. 执行步骤

未来唯一树`D:/autostack/.wt/lang-998/auto-lang`、分支`plan-998-dev`；本轮未建。后续实施指令后确认唯一仓/树/分支/base/HEAD和实际D；材料只在`D:/autostack/plan-artifacts/plan-998/`，开工登记owner/用途/在途。无junction、跨树可写target、仓内临时探针/log或第二输出根；常规target专树，需外置仅本scope。merge证据摘要折Plan后整scope删，具名持有须owner/理由/关闭条件；归档后不重开。

| T | owner_stage | 前置/写面 | 具体结果与验证/AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 后续实施指令；私有调查无需等所有D | 冻结六R原源/完整fault与规模/独立oracle/实际runtime/type/全部读者；纠正来源注记，D996/D997未齐不借WIP；V-01..V-06按对应owner，AC-01..04 | [x] T-01✅(2026-11-08): probe matrix frozen in plan-artifacts/plan-998/t01-freeze.md; 12 fixtures landed legacy-a2r-resources-998/src (byte-copies + R01 composite value 1048); provenance corrections verified (m1_k02.rs absent, R02≠V05-only, R06 anchors frozen; NEW boundaries recorded: move-Str &str mislowering fixed in-package, R03-static compile-time rejection) |
| T-02 | work | T-01；正式集成需D996/D997/D993/D994 | 以R01/R02/R05最小source→旧产物→真实观察→strict候选闭环证明通道可行，冻结typed错误/owner接口/配置与新schema；不成立needs_replan；V-01..V-06按对应owner，AC-01..04 | [x] T-02✅: legacy-a2r.resources-observe/1 profile + frozen helper block (guards/xfer/chk counting allocator/mut_call/replace_go/view/envgo/budget/cancel) + oracle/resource-wrapper-v1.rs; minimal chains proven (R01 LIFO stream, R02 sweep k=1..N exact + N+1 + retry, R05 env chain) |
| T-03 | work | T-02 | 实现原旧产物R01/R04观察/生命周期与variant活臂保证，永久wrong-order/漏/双释放/transfer等反控；V-01..V-06按对应owner，AC-01..04 | [x] T-03✅: R01/R04 observation legs dual-side certified (R04 r#1 released exactly once at switch, r#2 at exit) |
| T-04 | work | T-03 | R02真实allocation/准备域全fault sweep、N+1/同进程retry与失败原子，清理零余量不失效；V-01..V-06按对应owner，AC-01..04 | [x] T-04✅: five-leg fault sweep both sides (actual N per clean run; every k typed exit-71 + complete unwind cleanup; N+1 control clean; same-process retry leg) |
| T-05 | work | T-03/04 | R03原List借用/代际合法+stale真实路径，危险读前拒与fresh正控，明确source静态与typed边界；V-01..V-06按对应owner，AC-01..04 | [x] T-05✅: R03 legal (130, reads=2 only-legal) + compile-boundary leg recorded honestly + typed stale face on the reference (stale before any read, exit 71) |
| T-06 | work | T-03/04 | 补R05原K02必要source入口/返回closure支持及creator退出后调用/drop/重复建销观察，不改corpus accepted规则；V-01..V-06按对应owner，AC-01..04 | [x] T-06✅: parser Fn-token return lookahead + Box<dyn Fn> carrier + Box::new return + move-Str param String lowering; K02 parses/emits/runs 42 |
| T-07 | work | T-03..06 | R06原预算/取消/trap/close/深链逐owner cleanup、父Job与合作终态分账/收敛证据；V-01..V-06按对应owner，AC-01..04 | [x] T-07✅: R06 clean 2359360 + budget + cancel + combined-profile typed trap exit 70, complete LIFO cleanup observed on every typed exit |
| T-08 | work | T-03..07 | 六场景新域显式双侧认证、父/SDK/manifest/Case/日常严格读取与伪ok缺证书/删check反控，六行才闭6→0；V-01..V-06按对应owner，AC-01..04 | [x] T-08✅: ace.m1.a2r-resource-scenario/1 certs (6) + ace.m1.a2r-resource-oracle/1 registry + 12 scene goldens + 12 references + suite case-legacy-a2r-resources-998 (12 checks) + binding flips ok-emit-scene + readers (script/python/manifest) |
| T-09 | work | T-08 | 冻结完整职责/原场景全覆盖与健康、SD候选/实际16格最终闭合表/941交接，原其它门不代；V-01..V-06按对应owner，AC-01..04 | [x] T-09✅: responsibilities frozen (scoped groups run; unit tests plan998 x8; warnings delta 0); 16-cell closure table: 996 six list + 997 four trap + 998 six scene = 16/16, cert-gap 6→0; 941 handoff read-only via binding notes |
| T-10 | review | 全部work完成、冻结后停实现写入 | V-07独立上下文重建全链、全部SD/reader/原负控/健康/省略审查，AC-05 | [x] T-10✅(2026-11-08) |
| T-11 | merge | T-10 pass与后续合入指令 | 冻结职责/触码漂移复验、SD沉淀读回/来源交付/归档，逐格只读交接941但不写它的在途文件，AC-06 | [x] T-11✅(2026-11-08) |
| T-12 | merge | 已核实landed/归档/规范来源 | 全部实际输出根对账、摘要折Plan、scope整删/持有登记，wt-guard clean后移除唯一树/分支/空组，AC-06 | [x] T-12✅(2026-11-08) |

## 9. 复审记录

2026-11-08 stage:new / PLAN-998 r1 / outcome:pass（仅具名裁定后的规划静态合同，非产品/独立实施review通过） / next:后续实施指令与任务实际D就绪后T-01。0/12，全部V与T未执行，未建树/scope/会话，未改production/语料/证书/current Specs或939/941。本计划的裁定记录以本轮用户三项明确回复为准，不重开991归档。静态复核收据在此续录。

2026-11-08 规划静态复核：0..10章节，T12/AC6/V7/SD5唯一连续/引用完整，全部任务未执行；本包6格与当前binding的ns/id/token/outcome/expected逐字对账，三包共16格无重复且恰覆盖全部剩余required cert-gap。原脚本/Spec/实际代码入口已读，新接口标拟新增；依赖来源存在、三包DAG无环、链接/表格/空白/编码/现有固定组名称与拟新组标记通过。中央取号996/997/998唯一，.next-id=999。三项用户裁定已明记，不继续列为待答复；T-01/02未知实现策略有具名全链可行性门。规划前后939/941、binding/applicability内容hash相同，未写current Specs/代码/原语料/证书、未建树/scope/会话、未执行产品/认证/性能/独立实施review；仅写本组三份Plan及.next-id。本检查不关闭任何格、不签M1。

## 9b. 执行证据（work 阶段，2026-11-08）

- 树：`D:/autostack/.wt/lang-998/auto-lang` @ `plan-998-dev`（base a95d6dc41）；材料 scope `D:/autostack/plan-artifacts/plan-998/`（probe/t01-freeze/cert-scratch，收尾时按裁定折叠+整删）。
- 六格全部真实关闭（6→0）：`ace-certify --resource --only R01..R06` = **6 certified, 0 failed**；复跑 = **6 reused**（strict receipt 验证，零编译）。证书 `oracle/scenario-certs/m1_r0{1..6}.scenario-cert.json`（schema `ace.m1.a2r-resource-scenario/1`）。
- 每证书：12 冻结 fixture（`legacy-a2r-resources-998/src/`）+ 12 场景金样（`oracle/scenario/`）+ 12 手写独立 reference（`reference/scenario/`，自有观测机制/原子 alloc 探针，与 emission 零共享）+ `oracle/scenario/scenario-oracle.json`（`ace.m1.a2r-resource-oracle/1`，hash 为证书身份轴）+ `oracle/resource-wrapper-v1.rs`（hash 轴）。
- 逐 leg 运行矩阵（双侧一致）：clean（值 1048/42×6/53/130/2359360）｜fault sweep k=1..N（每 k typed exit 71 + unwind 完整 LIFO 清理）｜N+1 对照（clean）｜同进程 retry（fault→clear→retry→值）｜budget｜cancel（合作取消，非父 kill）｜trap（组合 profile，exit 70 + 清理）｜compile-boundary（R03-static：如实记录 rustc 拒绝，非 typed 负例）｜typed-probe（R03-stale：reference 驱动 mint→合法读→结构变异→stale 使用，typed 拒绝先于任何 buffer 读）。
- 不变量检查器（certifier 内，双侧共用）：per-INSTANCE unique_terminal / transfer≠release / per-frame LIFO / no-leak / R04 switch-releases-old-once / R05 env-survives-chain / R03 stale-before-read。
- 通道（生产 emitter，`legacy-a2r.resources-observe/1`，默认字节恒等）：guard/xfer/chk 静默计数分配器（计数+fault 分离：分配器只计数，fault 在安全 chk 点）/mut_call（参数先求值的结构变异包装）/replace_go（代际 payload 观测）/view-use（R03 typed 面）/envgo（闭包 env 转移）/step（预算/取消）。T-06：parser Fn 返回位 lookahead + `Box<dyn Fn>` 载体 + move-Str 参数 String 传值（K02 42）。
- 日常面：suite `legacy-a2r-resources-998`（12 checks，emit+bytes，零 spawn）+ 固定组 `case-legacy-a2r-resources-998` PASS；binding 六格 `ok-emit-scene`；`Get-CellState` `^ok`→green/cert=none（cert-gap 6→0），scenario 证书由独立 strict reader 强制（script/python/manifest 已接）。
- 单测：`plan998_*` ×8 全绿（默认字节恒等、profile 准入、guard/xfer/代际发射、K02 形状、不变量正反控、receipt 伪造拒绝 Invalid/Stale）。警告 delta=0（132==master，新文件零警告）；改动文件已 rustfmt。
- 16 格总账：996 六集合 + 997 四 trap + 998 六场景 = 16/16 闭合，M1 cert-gap 0（值域 38 + trap 4 + scene 6 分域计数）。required 313 集合未动。
- 已知边界（如实记录，非缺口）：R03-static 编译期拒绝（rustc E0308，非语言 typed 负例）；v1 heap 域只计 fresh alloc()（realloc 增长不计数，双侧对称）；表达式位结构变异/return 位 clone/FStr 强转未插桩（fixtures 不含）；941 交接按 binding note 只读，未写其在途文件。

### K04 冻结负例通道迁移（T-06 必然后果，复审重点）

T-06 为 K02 补 fn-type 返回位源入口（用户裁定 H-04/H-07 授权）后，K04（`fn bad() fn(int) int` + 非 move 闭包 return）不再可能维持旧的 parse 级拒绝（auto_syntax_E0001）——该冻结通道是"整个构造不可解析"的观测冻结，其 case 头自记"designed type-level rule differs"。998 落地设计通道：
- a2r 面：`type/auto_type_E0101`（borrow-capturing 闭包不得逃逸创建帧的 typed 拒绝，emit 期 TypeError::Mismatch）；
- avm 面：`run/RuntimeError`（VM 编译如今合法的源后在 typed runtime 通道失败）。
`legacy-negative.case.ad` 双 check 与 binding K04 note 已同步；source-manifest 相应 re-anchor。**负例本身保留**（源仍被双路径拒绝）；通道迁移是 K02 能力的必要结果，非放宽——独立复审须专项核对此条。

## 10. 待澄清事项

1. H-04/H-06/H-07三项用户已裁定保旧域/checked期望/六场景required，不能继续以“等用户选NA/新HIR”停工或扩大授权；本轮仍仅规划，未来实施依后续指令。
2. 新旧配置的实现、观察位置/schema最小形状由T-01/02有界证明；证明失败或需改变原语义/预算/范围，先needs_replan，不用预填PASS/新NA解决。
3. 跨包/跨域证据适用性由真实身份核对；若941原owner尚未接收，分列本域已关闭与最终组合待接纳。其它required/已登记缺陷不因本包通道绿自动消失。
4. 新域证书/适用字段读者尚不存在者已标拟新增，work须列完整producer→strict reader→daily/恢复/父判路径；未适配默认拒，不依赖general ok前缀跳过认证。

2026-11-08 stage: work | plan_id: PLAN-998 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-998/auto-lang | branch: plan-998-dev | base: a95d6dc41 | code_commit: dcd65e65122dcdb92ebfd4bfbbebd084b508ed3d | dependency_revisions: D991/993/994/996/997=master a95d6dc41 | task_ids: T-01..T-09 | evidence: 六格 `ace-certify --resource` 6 certified + 6 reused（strict receipt，零编译）；证书 oracle/scenario-certs/m1_r0{1..6}.scenario-cert.json；逐 leg 双侧矩阵见 §9b；单测 plan998 ×8 绿；scoped groups 10/10 绿（a2r/parser/auto-test-case/ace-adapters/infra-ace-m1/case-ace-m1-base/992/994/996/997/998-group）；python 24/24 OK；verify script snapshot/receipts/acceptance 三 scope 过（含 SCENARIO 6/6 轴、cert-gap 0、can_claim_m1 True）；警告 delta 0（132==master，新文件零警告）；K04 通道迁移已具名记录 | blockers: 无 | next_responsible_role: review（T-10 独立上下文复审；K04 通道迁移与 R03-static 编译期边界为专项复核点）

stage: goal | plan_id: PLAN-998 | plan_revision: r1 | round: 1 | from: work | to: review | status: archived | outcome: pass | worktree: D:/autostack/.wt/lang-998/auto-lang | commit: dcd65e65122dcdb92ebfd4bfbbebd084b508ed3d | next: review

2026-11-08 stage: review | review_scope: final | reviewer: independent-context subagent (T-10/V-07 executor; non-implementation context; read-only + verification commands; limitation noted in its report) | plan_id: PLAN-998 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-998/auto-lang | branch: plan-998-dev | reviewed_commit: dcd65e65122dcdb92ebfd4bfbbebd084b508ed3d | base: a95d6dc41 | dependency_revisions: D991/993/994/996/997=master a95d6dc41 | spec_inputs: docs/specs/auto-test-case/{ace-m1-scenario-oracles,a2r-golden-certification,ace-m1-gap-audit,resource-lifecycle-observations}.md + docs/specs/auto-lang/trans/architecture.md (SD-01..05 pending merge deposit, delta rows validated) | acceptance_results: AC-01 pass (provenance byte-diffs; R06-trap bounded re-statement recorded as finding F-998-RV-01) / AC-02 pass (real-path observation verified; zero case-ID special-casing) / AC-03 pass (dual-side matrices re-run --force 6 certified byte-identical, 6 reused strict; negative control: cert removed → receipts FAIL; unit 8/8) / AC-04 pass (313 conserved; 38 value + 4 trap + 6 scene separate; evidence-first flips same-commit) / AC-05 pass (this review) | pending_closeout_gates: merge-owned frozen affected-group union already green in review (10/10 scoped groups); periodic cargo t all remains due-triggered on main | findings: F-998-RV-01..04 全部 minor（记录/注释准确性，merge 时修正：R06-trap 非 byte-copy 之注记、R01 注释 1046→1048 说明、realloc 双侧措辞、K04 token 字面）| evidence: 复审报告全文 D:/autostack/plan-artifacts/plan-998/review-scratch/（收尾整删前为准）；关键命令与结果已摘录于本节 | next: merge

2026-11-08 stage: merge | PLAN-998:r1 | prepared: reviewed dcd65e651 + docs-only descendants（SD-01..05 沉淀、binding R06 注记修正 F-998-RV-01、manifest re-anchor） | landed: rebase dcd65e651→c30dfeb36（range-diff `=` 全等，安全重写证明），delivery fff9eedf0 ff-only 落 master（master tip==delivery==plan-998-dev） | canonical_verified: docs/specs/auto-test-case/{ace-m1-scenario-oracles,a2r-golden-certification,ace-m1-gap-audit,resource-lifecycle-observations}.md + docs/specs/auto-lang/trans/architecture.md 五份 SD 沉淀随 delivery 落主线；主检出 case-legacy-a2r-resources-998 PASS + contract scope exit0 | 复审计正（merge 折叠）：F-998-RV-01 binding R06 注记已改；F-998-RV-02 R01 fixture 注释 1046 为陈旧注释（冻结值以注册表/证书 1048 为权威，此处记录不改 fixture 字节）；F-998-RV-03 realloc 措辞已在 SD-04 沉淀中按方法级对称表述；F-998-RV-04 K04 token 字面保持 `ok-parse-reject`（`^ok` 前缀语义不变，binding note 为权威） | next: archive+cleanup

2026-11-08 merge closeout（cleaned）：
- worktree 移除：wt-guard clean → `git worktree remove D:/autostack/.wt/lang-998/auto-lang` + `git branch -d plan-998-dev`（was fff9eedf0）+ 空组目录删除。
- plan-artifacts/plan-998 整删（记录已折叠本文件；无具名持有）。删除时一个早前诊断探针的挂死进程（alloctest a.exe，stdout 管道阻塞）持有空目录句柄，taskkill 后整删成功；登记表 .retention/policy.json 已刷新（PLAN-998 deleted-at-merge @fff9eedf0）。
- 输出根对账：probe/cert-scratch/review-scratch/groups logs/snapshot-out 等全部在 plan-998 scope 内随删；worktree target 树随树删；无第二输出根、无跨盘散落（certifier --scratch 显式指向 scope；review scratch 同 scope）。
- landing artifacts：本仓为语言/测试基建面，无 backend 二进制/daemon/web bundle 消费方在运行；主检出 case-legacy-a2r-resources-998 与 verify contract 已在落地后新跑为绿。
- 批量回归 due 检查：.last-batch-regression.json last_covered_plan_id=995（今日 03:22，<48h）；998%5=3 且 995 后无 5 倍数计划落地 → 不到期；周期 cargo t all 仍按主检出单实例 due 门。
- 终态核对：master tip=fff9eedf0（delivery）+ 9b7a9432f（archive bookkeeping）；plan 位于 docs/plans/archive/ status=archived；T 12/12；required 313 守恒；cert-gap 0（38 value + 4 trap + 6 scene 分域）。

stage: goal | plan_id: PLAN-998 | plan_revision: r1 | round: 1 | from: merge | to: done | status: archived | outcome: pass | worktree: (removed) | commit: fff9eedf0 (delivery) / 9b7a9432f (archive) | next: done

2026-11-08 closeout repair（gitignore 缺口，T-12）：
- 发现：仓库根 .gitignore 的 `*.json` 全局规则在 worktree 提交时静默跳过 `oracle/scenario/scenario-oracle.json` 与 6 张 `oracle/scenario-certs/*.json`（金样/代码/引用/wrapper 均正常落线；worktree 磁盘上文件存在故认证与复审全绿，git status 不显示 ignored 文件）。主检出 python 验收首跑抓住（FileNotFound）。
- 修复：按编辑史逐字重建注册表（内容与复审所见一致）+ `git add -f`；主检出 `--resource` 全矩阵确定性再生成 6 证书（6 certified 0 failed）+ force-add。修复后主检出：python 24/24 OK、contract/acceptance 双 scope exit 0（SCENARIO 6/6、cert-gap 0）、case-legacy-a2r-resources-998 + case-ace-m1-base PASS。
- 教训（并入收尾对账义务）：`git add -A` 对 ignored 交付文件不设防；凡 *.json 类交付必须显式 force-add 并以 fresh 主检出（非 worktree 磁盘）验收。

2026-11-08 final: PLAN-998 delivered. Plan r1 | delivery fff9eedf0（reviewed dcd65e651 → rebase-equal c30dfeb36 + docs descendants）| archive docs/plans/archive/998-legacy-a2r-resource-scenario-certification.md（@3d8195ac6 含 closeout repair 记录）| Specs: docs/specs/auto-test-case/{ace-m1-scenario-oracles,a2r-golden-certification,ace-m1-gap-audit,resource-lifecycle-observations}.md + docs/specs/auto-lang/trans/architecture.md | cleanup: worktree/branch/group removed（wt-guard clean）, plan-artifacts/plan-998 deleted, registry refreshed, zero holds | 主检出终验: python 24/24, contract+acceptance exit0（green=313, SCENARIO 6/6, cert-gap 0, can_claim_m1 True）, case groups PASS | 批量回归: 不到期（995 已覆盖于今日, 998%5≠0）。
