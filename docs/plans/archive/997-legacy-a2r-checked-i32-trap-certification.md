---
plan_id: PLAN-997
status: archived
feature_name: 旧A2R受约束i32 checked发射与四格trap独立认证
author: [Codex]
created_at: 2026-11-08
updated_at: 2026-11-08
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-test-case/ace-m1-scenario-oracles.md, docs/specs/auto-lang/trans/architecture.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [legacy-a2r, auto-test-case]
depends_on: [PLAN-991, PLAN-994]
current_step: 8
total_steps: 8
---

# [PLAN-997] 旧A2R受约束i32 checked发射与四格trap独立认证

## 0. 变更摘要

按用户H-06裁定保原四源checked/trap期望，不把旧i64差异改accepted，不全局改变旧int；建立明确适用域的旧RustTrans checked发射与独立trap认证/日常准入，关闭四格。

**已交付（2026-11-08，merge pass）**：显式`legacy-a2r.checked-i32/1` profile（运算时i32域检查+受控typed trap通道）+独立trap认证域（`ace.m1.a2r-trap/1`、`ace-certify --trap`、声明式trap-oracle、独立reference、strict双reader）；四格双侧认证关闭，cert-gap 10→6（与996并集记账），snapshot日常41→45，值域38证书身份不变；交付`9be317d9c`（master线性），SD-01..04沉淀（含ADR-12），独立review pass（r1@修复后复审）。详见§7.3/§9。余6格=H-07场景（998域）；`can_claim_m1=false`。

## 1. 目标

本包原4个corpus:a2r格逐项真实关闭，source/语义/身份/完整产物/真实执行/独立oracle/strict认证/日常准入及余账可追溯。原source/outcome/expected与producer义务保留，不能以新HIR/NA/accepted凑绿。

非目标：全语言新特性、M2产品替换或旧引擎退役、改变旧默认int/所有公开API、放宽预算/原样本、减少313 required/改旧批准、修939/941或其余不相关代码、把证据哈希当恶意本机历史签名证明。

## 2. 架构方案

旧source→现绑定/准入→相同RustTrans（显式有限配置如本包需要）→目标自身表示/私有helper→完整实际产物→原受控工具→独立typed/value/events oracle→域专用strict reader→原required父判。新配置/新证书域须版本化且所有producer/consumer同时适配，不在运行时绕回旧VM/AV/新HIR业务实现。

## 3. 技术栈

- `crates/auto-lang-core/src/trans/rust/expr_emit.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang-core/src/trans/rust.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang-core/src/test_cases/a2r.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang-core/src/test_cases/ace_certify.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang/src/bin/ace-certify.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- 现入口：`crates/auto-lang-core/src/test_cases/a2r.rs`、`ace_certify.rs`，`crates/auto-lang/src/bin/ace-certify.rs`、`scripts/verify-ace-m1-852.ps1`、`scripts/tests/test_ace_m1_acceptance.py`及`scripts/test-groups.toml`。

## 4. 需求分析与背景调查

### 用户裁定、范围与真实生产路径

2026-11-08用户已逐项明确选择：H-04“补齐旧A2R有限集合”；H-06“保留原checked/trap义务，不全局改旧int语义”；H-07“保留required并补通道”。三项均不是待答复裁定，不再提出NA/accepted/新HIR迁移作为默认方案；原16格、required313、原source/outcome/expected、NA/32 accepted/45 future及零cert-gap守卫保留。此授权落实为本组三份**规划合同**，不等实施/认证/合入授权，不批准任意新golden字节。

**执行授权（2026-11-08）**：`/goal 计划997` 经 /auto-plan-goal 授权整链执行——在唯一树`D:/autostack/.wt/lang-997/auto-lang`（分支`plan-997-dev`，base=master `7b7a88c15`）内实现T-01..T-05、复审、有界修复与merge；scope=`D:/autostack/plan-artifacts/plan-997/`已登记。授权不改变目标/验收/预算/兼容承诺/仓范围。

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
| `corpus:B13:a2r` | `cert-diff` | `trap / overflow` | `src/B13.at`；原trap文字oracle，T-03新增独立参考（现无value参考） |
| `corpus:B14:a2r` | `cert-diff` | `trap / div-by-zero` | `src/B14.at`；原trap文字oracle，T-03新增独立参考（现无value参考） |
| `corpus:B15:a2r` | `cert-diff` | `trap / rem-overflow` | `src/B15.at`；原trap文字oracle，T-03新增独立参考（现无value参考） |
| `corpus:B20:a2r` | `cert-diff` | `trap / div-by-zero` | `src/B20.at`；原trap文字oracle，T-03新增独立参考（现无value参考） |
### 语义裁定与profile隔离（不能改默认旧int）

B13=2147483647+1、B15=MIN%-1保原overflow；B14=10/0、B20=true&&10/0保原divide-zero类（精确命名以原binding/参考为准）。旧i64 carrier的回绕/正常返回或rustc deny-lint是baseline，不是目标trap。默认旧产品/公开Rust库/整数宽度与数值API不在本包全局改义。

T-01冻结一个**显式且版本化的旧RustTrans源语义配置**（拟`legacy-a2r.checked-i32/1`，新配置不是新HIR后端）：调用者显式选择、原Auto源原样进入相同production front门/语义准入/发射器；其编译配置/产物/工具/依赖/运行身份进新证书和缓存键。未选择配置的默认输出与32旧证书原字节保持。配置应适用于声明的整个有限scalar源码域而非这4个ID/字面量：类型为源int的相关加/减/乘/除/余/neg/转换与短路需要明确规则，非int/外部Rust调用不受全局名称改写。原四trap格在binding仍是旧a2r，确切profile明记于扩展证据，不能冒无配置默认旧i64已改变；现32值证书不能证明该新配置的全部值域，应补独立合法邻例。

T-01用B13/B14+一个短路合法例串真实source→actual→运行，比较利用现发射器的typed checked helper/载体检查方式，冻结不越界的实现。i64可保ABI carrier但源int操作必须在运算时检查i32域；不能最后返回as i32才补救。两操作数源序各一次、短路右支只在需要时执行；MIN/-1与MIN%-1、负数、零除、边界邻居分开，不能依赖debug Rust默认panic或让字面常量在rustc编译阶段拒绝。若现前门无法提供可靠int分类/有限配置隔离，T-01 needs_replan，不能偷偷全局窄化int或用case-specific替代。

### 独立trap生产/认证通道

拟新schema `ace.m1.a2r-trap/1`、recipe/producer-profile与strict reader独立于value cert/1与917 panic观察，T-03冻结全部字段/旧读者互拒。actual必须完整编译成功并实际到达受控checked运行点，发出具名typed原因/位置或稳定原操作身份及终态，随后受控结束；不能捕获任意panic/parse stderr文本就判Auto trap，不能以compiler error、父超时/kill或外host直接合成原因过门。诊断通道与程序stdout、trace分离并受原流帽；默认用户源可打印相似文本不能伪造正式checked事件。

新CLI/SDK/父adapter能力与recipe在T-03沿现contained执行器使能（拟受管`legacy-a2r-trap-certify`子入口与`case-legacy-a2r-trap-997`固定组、`legacy-a2r-trap-997.{case,suite}.ad`、`oracle/trap-certs/`新独立路径，均当前不存在）。同一schema中含source/actual/helper/config/target/tool/flags/deps/profile/oracle/reference/wrapper/run/证据hash、真实命令/终态及检查全集；新旧receipt错域互拒。

四源尚无合适value reference，应从原规范手写独立trap参考并G04审阅，不能复制actual checked helper作为oracle。额外参考要通过各自真实编译运行验证原typed结果；同时造假相同trap/错阶段/省略执行的负控必须红。日常在新profile现场emit+已审golden+strict新域证书/身份核查零compile/guest；显式认证才actual/reference实际运行。旧value域32/或D996后的38不变，四trap单独计4，不能改为ok-emit让旧value certifier误收。

T-04必须扩旧acceptance的**新域required验证**，既读新证书又确认实际原case执行闭包；token虽可按既有ok词族标记交付，删除新证书或伪造ok仍不可签。source-manifest只追加实际新材料字段的明确版本/全consumer准入，不偷改原字段含义。关闭四格后剩余数量取决D996：16→12或10→6；剩余H07必保，原分母/expected完全不变。

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
| SD-01 | modify | `docs/specs/auto-test-case/a2r-golden-certification.md` | 本包实际有限能力/独立域与已关闭格、保留边界及source/profile/reader约束；历史分诊只作来源，不再用过时总数 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |
| SD-02 | modify | `docs/specs/auto-test-case/ace-m1-gap-audit.md` | 本包实际有限能力/独立域与已关闭格、保留边界及source/profile/reader约束；历史分诊只作来源，不再用过时总数 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |
| SD-03 | modify | `docs/specs/auto-test-case/ace-m1-scenario-oracles.md` | 本包实际有限能力/独立域与已关闭格、保留边界及source/profile/reader约束；历史分诊只作来源，不再用过时总数 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |
| SD-04 | modify | `docs/specs/auto-lang/trans/architecture.md` | 本包实际有限能力/独立域与已关闭格、保留边界及source/profile/reader约束；历史分诊只作来源，不再用过时总数 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |

## 6. 测试设计

所有V均为未执行拟议用例；具体新入口在对应任务使能。

| V | AC / invariant | 输入/实际入口与owner | 区分性oracle/反控 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/02/03 配置隔离与四原trap | T-01/02，四原源显式checked配置、默认旧配置与独立参考 | baseline正常i64/编译红可复现；checked产物实际编译成功并运行触发原overflow/divide-zero；默认现32值金样不变 | 不全局改int，不按case ID选择helper，不用Rust panic/deny-lint当trap |
| V-02 | AC-01/02/03 运算相位/短路/单求值 | T-02，MAX/MIN邻居、MIN/-1与MIN%-1、非零除、false&&零除、带副作用operand | 合法42等值正确；不执行短路支，operand源序各一次，溢出在操作点，不在最终cast | 有限scalar合法域；f64/i64显式类型/FFI不误窄化 |
| V-03 | AC-01/02/03 typed证书反伪与所有reader | T-03/04，四新域完整actual/ref、假trap文本/同错原因/编译失败/未知nonce/错profile | 只实际checked来源被接受；新旧receipt互拒，缺原始执行/证书/选择为空均红 | value/1不扩字段，917 panic不冒新trap证据 |
| V-04 | AC-04 分母/域/交接 | 最终work任务原binding/applicability/source-manifest与本包全部格 | 只本包真实认证可核销，313集合hash/批准NA/accepted/future守恒，伪ok但无证书不签；计划通过非全M1pass | 939/941不写，不自动发消息/新会话 |
| V-05 | AC-05 独立复审 | T-06同树独立上下文重建source→实际runtime→observer→oracle→strict→父门 | 语义/原始执行/正反控/所有reader/SD/未观测逐项核，未知必需面不得默认绿 | 内部自查不替最终review，不自动spawn |

未来显式验证使用预构建正式入口，T-01冻结实际help/CLI及Case集合；原`ace-certify --corpus <tree>/crates/auto-lang/test/cases/ace-m1 --only <value-ids> --scratch <scope>/cert-scratch`仅限原value域，不能拿它选trap/scene冒已支持。新域入口由本包任务使能，当前不存在，需沿原CLI/父adapter/受控执行器实现；不得为取help偷偷构建或执行实现。本轮仅静态检查。

通用负控：原golden变1B、合法证书删一份/改一轴、token伪改ok、独立参考审核缺失或同一artifact冒充reference、缺原命令/执行终态、空选择、未知field/旧schema、超流/晚失败/cleanup未回收；必须严格拒或诚实非绿，合法同来源非零owner/ctx应接受。真正producer错误的负控优先使用baseline复现与scope隔离副本，不破坏他会话文件。声明新scope/skip不代执行数；记录selected/executed/条件排除/未执行/首次红/重放绿及准备/运行/清理时间。

现有职责基础`cargo t a2r infra-ace-m1 auto-test-case ace-adapters case-ace-m1-base case-legacy-a2r-cert-992 case-legacy-a2r-text-994`（T-01核实际触面冻结必要并集）；若修改共享语法/ownership/类型准入，须补真实对应parser/infer/ownership消费者完整组，不能只跑a2r；若未触则记录免跑理由，不为Cargo依赖自动全仓。拟新组见§5，正式注册后须实际发现/执行完整Case/check。性能/深链/故障注入原串行/资源/平台条件保留，新字节/诊断/typed协议reader需端到端反控，不只脚本合成包自证。

## 7. 验收标准

| AC | owner_stage | 可观察关闭条件 |
|---|---|---|
| AC-01 | work | 本包全部原格位的真实旧producer/source/profile/依赖/完整actual来源正确，必要有限支持与独立合法近邻成立，非新HIR/host业务替代 |
| AC-02 | work | §5每个原值/trap/资源不变量及V补例在真实编译运行/观察上成立，错误相位/拥有cleanup/源序等不能只靠表面返回值或伪计数 |
| AC-03 | work | 经G04审核的独立完整新证据、strict全reader/日常零spawn、错域/坏件/缺Case/伪ok等反控与原职责通过；原证书身份不变才复用 |
| AC-04 | work | 原313/批准/源oracle/余账守恒，实际缺口按本包逐格闭合，全部SD候选和941只读交接可核，无未经批准删项/绕守卫 |
| AC-05 | review | 同树独立上下文逐AC/V/SD/全部reader/资源边界/遗漏/健康审查，绑定r1与冻结commit/工具配置；未知必需面不PASS |
| AC-06 | merge | review pass后冻结职责/漂移核验、SD沉淀读回与交付归档、实际输出根对账、scope摘要后整删/guard清树，真实结论与剩余项诚实记录 |

### 7.3 work阶段验证记录（2026-11-08，树`D:/autostack/.wt/lang-997/auto-lang`，base`7b7a88c15`，code`1e207539f`+`4b032b3d6`）

- **入口冻结**：`cargo build -p auto-lang --bin auto-case --bin ace-certify` exit 0；PATH rustc 1.98.0 (88d9e12ae 2026-08-18)=32值证书`default_identity_at_certify`一致。
- **基线（默认i64载体）**：B13→`i32:-2147483648`回绕、B15→`i32:0`、B14/B20→rustc E0080 `deny(unconditional_panic)`**编译期拒绝**（常量`10/0`折叠）——四格全不可核销的原状冻结（scope t01/base-run）。
- **checked链实测**：四源`legacy-a2r.checked-i32/1`下wrap编译运行全部 exit **70**+stderr `auto.trap:overflow:add`/`div-by-zero:div`/`rem-overflow:rem`/`div-by-zero:div`（逐格=原binding expected）+stdout干净；合法邻域 nb20legal=42（短路右支未执行）、nb13ok=2147483647、ndiv=5、nrem=1；边界 nmindiv=overflow:div、nnegmin=overflow:neg；副作用probe bump×2各一次且先于trap点（t01/checked-run、nside-run）。
- **默认隔离**：探针10/10字节逐位不变；repo snapshot全语料 35→39检查期间值域金样全程byte-compare绿。
- **认证**：`ace-certify --trap --only b13,b14,b15,b20`→**4 certified**（双侧同观察）；严格reuse 4 reused/0 failed；全量复验32值reused+4 trap reused。
- **token/账**：binding四格`cert-diff`→`ok-emit-trap`+注记；acceptance cert-gap **16→12**、can_claim_m1=false、313/45/32与批准集不变；`requiredSetHash`随四格转绿自然演化（无行数/层变化）。
- **三scope+contract（flip后）**：snapshot=**39** a2r全绿+32值+**4 trap**证书strict+认证面spawn=0；receipts 32+4 strict；acceptance 4 trap receipts无条件strict验证+12 gap诚实；contract 131行全验PASS；certify scope（值+trap+治理4/4）PASS。
- **反控全红**（scope副本+hash验证还原）：C1删trap证书→FAIL；C2未知字段→REJECTED；C6伪造exit=0→REJECTED（Rust与PS双reader均拒）；C3假token（T08→ok-emit-trap无证）→manifest drift先拦；C3b配manifest重冻→trap validator自拒"no oracle cell"；C4删检查→`37 < 39`计数守卫红；C5金样1B+manifest重冻→suite字节比对红（t04/controls-*）。
- **修复中自发现的bug**：首版trap validator以无换行oracle行比较带换行的raw观察→自证reuse红（forged控制即捕获）——修正为newline-terminated比较，cert格式保持raw观察。
- **固定组**：9组并集全绿（数字见T-05行）；`cargo t a2r` 414含plan997单元9例。
- **941只读交接**（原owner按真实身份接纳，本包不写941）：

| 格位 | source SHA16 | trap golden SHA16 | trap cert SHA16 | 终态 |
|---|---|---|---|---|
| corpus:B13:a2r | `54943ad037b05733` | `ca98a3bb95a219dd` | `929ab82fae4702a7` | certified |
| corpus:B14:a2r | `1330a237463208d9` | `b2b2933e1b0a5646` | `495f63a3514dacbc` | certified |
| corpus:B15:a2r | `99a24fc1af02865c` | `209d12cb13131806` | `be050b05a45a83c9` | certified |
| corpus:B20:a2r | `72df0623d74f6aeb` | `77bbcaaa69a83fb9` | `deb6e7b393a01340` | certified |

- **余账（acceptance冻结）**：H-04 L01/L02/L03/L04/L05/L07（996六格）+H-07 R01-R06（998六格）=cert-gap 12；`can_claim_m1=false`（12>0且future 45在账）。
- **健康**：触面文件rustfmt净（a2r_tests.rs等既有master漂移不属本包）、零调试打印；材料唯一scope `D:/autostack/plan-artifacts/plan-997/`（t01/t03/t04/t05），树clean。**修订（T-06后）**：首轮冻结`4b032b3d6`存在2条新可见性警告（TrapAxes/TrapOracleCell比`validate_trap_receipt`私有，F-997-R1-01）——初版"零新警告"表述不实；修复提交`02013fa77`（函数转模块私有+registry cell进reuse交叉核验，F-997-R1-02同步关闭：自洽伪造expected_trap现被Rust reader结构性拒绝，不再仅靠PS reader拦截）后0新警告、plan997 9/9、严格reuse 4 reused/证书字节不变（certifier二进制不是证书身份轴）。

## 8. 执行步骤

未来唯一树`D:/autostack/.wt/lang-997/auto-lang`、分支`plan-997-dev`；本轮未建。后续实施指令后确认唯一仓/树/分支/base/HEAD和实际D；材料只在`D:/autostack/plan-artifacts/plan-997/`，开工登记owner/用途/在途。无junction、跨树可写target、仓内临时探针/log或第二输出根；常规target专树，需外置仅本scope。merge证据摘要折Plan后整scope删，具名持有须owner/理由/关闭条件；归档后不重开。

| T | owner_stage | 前置/写面 | 具体结果与验证/AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 后续实施指令；D991/D994 | 冻结裁定域、四原反例/合法短路、显式配置与source类型/actual出口的有界整链；原budget及全reader清单先决；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：裁定域冻结（scope `t01/FREEZE.md`）：显式版本化配置`legacy-a2r.checked-i32/1`（精确id准入）；私有typed helper发射策略胜出（vs 内联checked/原生checked_add/末尾cast三案比较——helper是唯一同时满足单求值源序/原生短路组合/逃过rustc常量折叠E0080/受控exit70的表达）；基线冻结：B13/B15回绕`i32:-2147483648`/`i32:0`，B14/B20默认profile连编译都被deny(unconditional_panic)拒绝（t01/base-run）；四源+短路合法例+邻域真实source→actual→wrap编译运行：四格exit70+typed stderr逐格=原binding expected，nb20legal=42，nmindiv/nnegmin=overflow:div/neg，ndiv/nrem=5/1（t01/checked-run）；默认profile探针10/10字节逐位不变+repo snapshot 35checks/32certs/0spawn全绿隔离证明；全reader清单冻结（Get-CellState+py mirror词族/snapshot计数与套件/receipts值域过滤/manifest枚举/spawn watch m1%*/acceptance硬编码账） |
| T-02 | work | T-01 | 实现受约束checked int发射/typed运行通道与默认profile隔离；永久四源/边界/单求值负控及合法邻例；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：`IntProfile`（rust.rs）+`transpile_rust_full_with_profile`/`transpile_rust_with_source_dir_profile`+RustTrans字段+懒helper块；expr_emit四处钩子（Add数值臂/catch-all Sub·Mul·Div·Mod/一元neg含`-Int(k≠MIN)`与`-I64(2^31)`字面量折叠/int局部复合赋值脱糖）；结构化分类`expr_is_source_int`（Int字面量/int局部参量/neg/int-only算术；i64显式字面量/char/str/调用/字段/cast不入域）；contract `Options.a2r_int_profile`精确id准入+suitedefaults拒绝+project模式拒绝；a2r adapter接线；永久plan997单元9例（a2r_tests.rs）；副作用probe：bump×2各一次先于trap点（stdout可见a=10 b=10后exit70 overflow:mul）；`cargo t a2r`414 PASS/`auto-test-case` PASS |
| T-03 | work | T-02 | 实现独立trap schema/受控显式入口/父SDK/Case与新域strict reader，手写独立参考并G04审阅、独立golden；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：新域材料：`oracle/trap/m1_b1{3,4,5}.rs`+`m1_b20.rs`金样（checked actual冻结）、声明式`oracle/trap/trap-oracle.json`（`ace.m1.a2r-trap-oracle/1`，kind=_binding expected强一致）、独立参考`reference/trap/B1{3,4,5}.rs`+`B20.rs`（std `checked_*`i32载体，与actual零共享代码——G04逐例审阅：B13 checked_add/B14 checked_div/B15 checked_rem含MIN%-1区分/B20原生&&短路+checked_div RHS）；`ace.m1.a2r-trap/1`证书schema（exact-keys全层级：source/golden/actual/wrapper/reference/trap-oracle六轴+profile+expected_trap{kind,op,line,exit}+observed双侧{exit,stdout,stderr}+toolchain+deps std-only）；`ace-certify --trap`子入口（ace_trap_certify.rs，复用contained执行器/Job/120s/30s/1MiB帽；cert-diff行须--only点名、ok-emit-trap行全集）；`legacy-a2r-trap-997.{case,suite}.ad`+固定组`case-legacy-a2r-trap-997`；PS1：snapshot第4套件+计数35→39+`Invoke-TrapReceiptValidation`（token门控严格验证：缺证/未知字段/伪造终态/轴漂移全红）+manifest `trap_assets_*`递归精确枚举+certify scope trap通道（门控于token存在）；manifest：suites_cases 85→87、trap三组5/4/4 |
| T-04 | work | T-03 | 四trap实际双侧认证与daily零spawn/所有坏轴/伪ok-缺证书反控，acceptance原域追加严格适用验证，精确关4格；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：`ace-certify --trap --only b13,b14,b15,b20`→**4 certified**（actual-wrapper与独立reference双侧各exit70+typed stderr+stdout干净）；严格reuse 4 reused/0 failed；binding四格a2r→`ok-emit-trap`+PLAN-997注记（131行/12列/313/批准集不变，avm diff行不触，applicability无a2r path claim无desync）；manifest binding sha重冻；三scope+contract全绿：snapshot=**39** a2r（26+5+4+4）+32值证书+**4 trap证书strict**+spawn 0；receipts 32+4 strict（in-PS身份）；acceptance cert-gap **16→12**、4 trap receipts无条件strict验证、can_claim_m1=false（余12+future45诚实）；反控全红：C1删trap证→FAIL、C2未知字段→REJECTED、C6伪造exit=0→REJECTED（Rust与PS双实现均拒）、C3假token（T08→ok-emit-trap无证）→FAIL（manifest drift先拦）、C3b配manifest重冻→trap validator自拒（no oracle cell）、C4删检查→38<39、C5金样1B+manifest重冻→suite字节比对红（t04/controls-*脚本与输出） |
| T-05 | work | T-04 | 完整a2r/旧base/治理/SDK职责及默认32或38合法receipt重验、SD候选/余账和998 typed通道交接，交review；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：冻结职责并集9组全绿（a2r 414/trap-997 4/cert-992 5/text-994 4/base 97/emit-917 11/semantics-916 17/infra-ace-m1 23/auto-test-case 113/ace-adapters 84）；certify scope PASS（值域32 reused+trap通道+治理4/4）；值域32 receipt严格重验（snapshot+receipts双入口）；gitignore对trap域JSON加豁免（F-908-R1-01同类防复发）；触面文件rustfmt净+plan997单元9/9+零新警告；SD候选维持§5表（merge沉淀）；余账=cert-gap 12（H-04六格996+H-07六格998，分母313/批准集不变）；998 typed通道交接：`IntProfile` API/`int_profile` case选项/trap-oracle声明式注册表/`ace.m1.a2r-trap/1` reader模式可直接沿用到scene域；941只读交接表见§7.3；commit `4b032b3d6`（+T-01/02 `1e207539f`） |
| T-06 | review | 全部work完成、冻结后停实现写入 | V-05独立上下文重建全链、全部SD/reader/原负控/健康/省略审查，AC-05 | [x] 2026-11-08：独立subagent上下文终审首轮**needs_fix**（F-997-R1-01 minor：2条新可见性警告违背健康门并证伪§7.3"零新警告"表述；F-997-R1-02 info：Rust reuse不交叉核验registry cell，自洽伪造可过Rust reader但被PS reader拦；R1-03 info：reader微小差异无组合绕过）；修复`02013fa77`（仅ace_trap_certify.rs +16/−2：模块私有化+TrapOracleCell进reuse交叉核验）后受影响复审**pass**（E2自洽伪造现被Rust reader结构性拒、未篡改证书仍reused、9/9、0可见性警告、树clean）。初审全绿面：A基线（diff恰为声明触面29文件，无docs/specs/语料/值域资产意外改动）；B四格独立语义重推导（checked语义/金样helper逐行/独立参考checked_*机制/registry==binding）；C默认隔离（四钩子全profile门控+现场32值金样byte-compare绿）；D活门（snapshot 39+32+4+0spawn/acceptance gap12 claim false 313/receipts 32+4 in-PS身份）；E独立防伪（tampered拒/clean reused/force-fresh 4 certified双侧同观察、cert SHA16=941交接表）；F Rust↔PS reader交叉读（差异全info无组合绕过）；G记账（131/12、恰4行ok-emit-trap、值域资产零触碰、manifest诚实）；I SD延迟到merge正确；J省略扫描全对齐（m1%覆盖trap exe、值certifier不可误收、余12恰=996六+998六） |
| T-07 | merge | T-06 pass与后续合入指令 | 冻结职责/触码漂移复验、SD沉淀读回/来源交付/归档，逐格只读交接941但不写它的在途文件，AC-06 | [x] 2026-11-08：**rebase共存**（master被996六格+941簿记推进两次）：全链rebase上master（旧→新映射 1e207539f→376f3885d→(末轮)·、4b032b3d6→4a0868ec5·、02013fa77→bff6cb8fc·，末轮再+941 docs提交平移，tip=`9be317d9c`）；共享文件共存解：a2r_tests.rs两块并存（plan996 10+plan997 9）、PS1五套件并集45（996 list=ext3、997 trap=ext4）、manifest合并（goldens 39/certs 38/suites 89/trap三组5-4-4/binding=双flip合并sha）、toml恰141+1组；共存修复两枚：toml重复尾块78组（冲突解误取旧基尾段，fixed-tests拒"invalid/duplicate"即抓获）、scenario_oracles sha随SD-03重冻（infra-ace-m1控制副本抓获）；受影响证据全部重验：contract/snapshot **45**+38+4+0spawn/receipts 38+4（in-PS）/acceptance **gap 10→6**/plan997 9+plan996 10/冻结11组并集全PASS（a2r 414/trap4/list6/992-5/994-4/base97/917-11/916-17/infra23含py23/auto-test-case/adapters）；SD-01..04以docs-only后代沉淀（ADR-12+三规范节）；`--ff-only`线性合入master tip=`9be317d9c`，master四Spec hash读回逐位一致，master快照冒烟45+38+4+0spawn PASS；941只读交接=§7.3表（SHA16）未写其在途文件 |
| T-08 | merge | 已核实landed/归档/规范来源 | 全部实际输出根对账、摘要折Plan、scope整删/持有登记，wt-guard clean后移除唯一树/分支/空组，AC-06 | [x] 2026-11-08：输出根对账——本计划命令仅写`D:/autostack/plan-artifacts/plan-997/`（t01/t03/t04/t05/t06-review/t06-repair/t07）与两树各自target及`%TEMP%/ace-m1-*-certify-*`认证scratch（certifier自删）；无第二输出根/跨盘散落；scope整删（无具名持有），`.retention/policy.json`（仓根）登记刷新为deleted-at-merge；wt-guard clean→树/分支`plan-997-dev`/空组目录`D:/autostack/.wt/lang-997`移除；归档`docs/plans/archive/`，T 8/8 |

## 9. 复审记录

2026-11-08 stage: merge | PLAN-997:r1 | checkpoints: prepared→landed→canonical_verified→archived→cleaned | delivery_commit: 9be317d9c（`--ff-only`线性；rebase映射：1e207539f/4b032b3d6/02013fa77→376f3885d系→bff6cb8fc系→末轮平移，range-diff `!`/`!`/`=`+docs-only后代与两枚共存修复共6+2提交，共存重验全绿） | canonical: SD-01 `docs/specs/auto-test-case/a2r-golden-certification.md` `9ea40727019c9e88…`、SD-02 `docs/specs/auto-test-case/ace-m1-gap-audit.md` `3c6ea0c705e381c4…`、SD-03 `docs/specs/auto-test-case/ace-m1-scenario-oracles.md` `698a05e782ab381e…`、SD-04 `docs/specs/auto-lang/trans/architecture.md` `0a1fbf1c3fdc3c77…`（master读回=树逐位一致） | master冒烟: snapshot **45** a2r（26+5+4+6+4）+38值+**4 trap**证书strict+0spawn PASS | 冻结11组并集: 全PASS于交付tip（共存后含996 list组与infra py23） | KNOWN-DEBT: 无新增具名债 | archive: `docs/plans/archive/997-legacy-a2r-checked-i32-trap-certification.md` status archived T 8/8 | cleaned: wt-guard clean→worktree/`plan-997-dev`/空组`D:/autostack/.wt/lang-997`移除；scope `D:/autostack/plan-artifacts/plan-997/`整删无具名持有，`.retention/policy.json`刷新 | landing_is_not_deployment: 本包交付=测试语料/治理脚本/规范文档+transpile期emitter行为与显式认证工具（编译期/dev-harness），无后端二进制/daemon/web bundle消费者；auto-case/ace-certify按需重建（Find-Prebuilt），无重启项 | batch_regression_due: 否（997%5≠0；last_covered=996@2026-11-08当天<48h） | outcome: **pass**（delivered+archived+cleaned） | next: done

stage: goal | plan_id: PLAN-997 | plan_revision: r1 | round: 3 | from: merge | to: done | status: archived | outcome: pass | worktree: removed (D:/autostack/.wt/lang-997 absent) | commit: 9be317d9c (delivery) | next: done | final: H-06四trap格独立域认证关闭（值38+trap4分计、snapshot 41→45、cert-gap 10→6）；余6格=H-07场景（998域）冻结于规范与§7.3；can_claim_m1=false；SD-01..04沉淀读回；scope/树/分支已清 | pending merge-owned gates: 无

2026-11-08 stage: review | review_scope: final | reviewer_identity/limitations: 独立subagent上下文（general-purpose，未参与work），同一冻结树只读，材料仅plan-997/t06-review/；未重跑冻结9组并集（merge持有）、周期cargo t all、917/909套件 | plan_id: PLAN-997 | plan_revision: r1 | outcome: **pass**（首轮needs_fix：F-997-R1-01 minor健康门2新可见性警告；F-997-R1-02 info Rust reuse缺registry交叉核验；修复`02013fa77`后受影响复审通过，R1-03留info无组合绕过） | worktree_path: D:/autostack/.wt/lang-997/auto-lang | branch: plan-997-dev | reviewed_commit: 02013fa77ae5b0305d08bcef2ffceab34794d2e0（树clean，前后核验） | base_commit: 7b7a88c15 | dependency_revisions: D991/D994归档；D996在途未交付（自适应记账，不依赖） | spec_inputs: SD-01..04目标文件在主检出且当前文本待取代；diff零docs/specs改动（merge沉淀） | acceptance_results: AC-01 pass（显式profile/结构化分类/合法邻域）；AC-02 pass（运算时检查/单求值源序/原生短路/MIN%-1与零除分kind，双侧实跑验证）；AC-03 pass（独立重推导金样与参考/双reader strict/0spawn/防伪probe红/32值证书身份不变复用）；AC-04 pass（313/32/45守恒/gap 16→12恰关四格/manifest诚实/941交接SHA16核对）；AC-05 pass（本审+修复闭环） | pending_closeout_gates: merge冻结9组并集在`02013fa77`复验、周期cargo t all due门、SD-01..04沉淀读回、merge后漂移复验、scope整删+wt-guard清树 | findings: F-997-R1-01（minor，已关）2新可见性警告+§7.3表述不实→修复+文修订；F-997-R1-02（info，已关）自洽伪造expected_trap可过Rust reuse→registry cell进reuse交叉核验（E2伪造现REJECTED）；F-997-R1-03（info，留档）reader差异（PS double created_at/Stale-vs-Fail轴/PS无oracle键直检——oracle文件hash为证书轴兜底） | evidence: 本节+plan-997/t06-review/（A..H分节证据+re-review/，merge整删前已折本节） | next: merge（T-07）

stage: goal | plan_id: PLAN-997 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-997/auto-lang | commit: 02013fa77 | next: merge（T-07/T-08）

2026-11-08 stage: work | plan_id: PLAN-997 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-997/auto-lang | branch: plan-997-dev | base_commit: 7b7a88c15 | code_commit: 4b032b3d6（+1e207539f T-01/02） | dependency_revisions: D991/D994归档核验；D996未交付（另一goal会话在途0/8）——本包按996先/后两种顺序的自适应记账交付（gap 16→12与"16→10→6"链一致，不依赖D996） | task_ids: T-01..T-05（5/5） | evidence: §7.3（基线E0080编译拒/checked双侧四trap certified/reuse strict/snapshot 39+32+4+0spawn/receipts 32+4/acceptance gap12 claim false 313不变/反控C1-C6+C3b全红/9组并集绿/certify scope PASS/默认隔离字节证明/941交接4格SHA16） | blockers: 无 | 未执行: 全仓统一回归（owner=merge按冻结并集；周期cargo t all按due主检出单实例） | next_responsible_role: review（T-06，同树独立上下文，冻结commit 4b032b3d6 + 本Plan文件）

stage: goal | plan_id: PLAN-997 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-997/auto-lang | commit: 4b032b3d6 | next: review（T-06）

2026-11-08 stage:new / PLAN-997 r1 / outcome:pass（仅具名裁定后的规划静态合同，非产品/独立实施review通过） / next:后续实施指令与任务实际D就绪后T-01。0/8，全部V与T未执行，未建树/scope/会话，未改production/语料/证书/current Specs或939/941。本计划的裁定记录以本轮用户三项明确回复为准，不重开991归档。静态复核收据在此续录。

2026-11-08 规划静态复核：0..10章节，T8/AC6/V5/SD4唯一连续/引用完整，全部任务未执行；本包4格与当前binding的ns/id/token/outcome/expected逐字对账，三包共16格无重复且恰覆盖全部剩余required cert-gap。原脚本/Spec/实际代码入口已读，新接口标拟新增；依赖来源存在、三包DAG无环、链接/表格/空白/编码/现有固定组名称与拟新组标记通过。中央取号996/997/998唯一，.next-id=999。三项用户裁定已明记，不继续列为待答复；T-01/02未知实现策略有具名全链可行性门。规划前后939/941、binding/applicability内容hash相同，未写current Specs/代码/原语料/证书、未建树/scope/会话、未执行产品/认证/性能/独立实施review；仅写本组三份Plan及.next-id。本检查不关闭任何格、不签M1。

## 10. 待澄清事项

1. H-04/H-06/H-07三项用户已裁定保旧域/checked期望/六场景required，不能继续以“等用户选NA/新HIR”停工或扩大授权；本轮仍仅规划，未来实施依后续指令。
2. 新旧配置的实现、观察位置/schema最小形状由T-01/02有界证明；证明失败或需改变原语义/预算/范围，先needs_replan，不用预填PASS/新NA解决。
3. 跨包/跨域证据适用性由真实身份核对；若941原owner尚未接收，分列本域已关闭与最终组合待接纳。其它required/已登记缺陷不因本包通道绿自动消失。
4. 新域证书/适用字段读者尚不存在者已标拟新增，work须列完整producer→strict reader→daily/恢复/父判路径；未适配默认拒，不依赖general ok前缀跳过认证。
