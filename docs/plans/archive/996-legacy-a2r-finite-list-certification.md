---
plan_id: PLAN-996
status: archived
feature_name: 旧A2R有限集合六格lowering与认证闭环
author: [Codex]
created_at: 2026-11-08
updated_at: 2026-11-08
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/a2r-golden-certification.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-lang/trans/architecture.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [legacy-a2r, auto-test-case]
depends_on: [PLAN-991, PLAN-994]
current_step: 8
total_steps: 8
---

# [PLAN-996] 旧A2R有限集合六格lowering与认证闭环

## 0. 变更摘要

按用户H-04裁定保旧producer，补原有限List<int>/List<Item{Str,int}>构造及push/get/insert/remove/clear/set/len调用，关闭六格原值认证，并为998场景提供真实旧A2R集合产物。

本轮只规划，所有实现/认证/复审未执行；当前drafting r1/0-of-8。原991待裁定项由本轮具名选择解决，实施策略由T-01/02有界证明，不预填运行PASS。

## 1. 目标

本包原6个corpus:a2r格逐项真实关闭，source/语义/身份/完整产物/真实执行/独立oracle/strict认证/日常准入及余账可追溯。原source/outcome/expected与producer义务保留，不能以新HIR/NA/accepted凑绿。

非目标：全语言新特性、M2产品替换或旧引擎退役、改变旧默认int/所有公开API、放宽预算/原样本、减少313 required/改旧批准、修939/941或其余不相关代码、把证据哈希当恶意本机历史签名证明。

## 2. 架构方案

旧source→现绑定/准入→相同RustTrans（显式有限配置如本包需要）→目标自身表示/私有helper→完整实际产物→原受控工具→独立typed/value/events oracle→域专用strict reader→原required父判。新配置/新证书域须版本化且所有producer/consumer同时适配，不在运行时绕回旧VM/AV/新HIR业务实现。

## 3. 技术栈

- `crates/auto-lang-core/src/trans/rust/ty.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang-core/src/trans/rust/call_emit/call_value_ctors.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang-core/src/trans/rust/call_emit/call_dot_methods.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang-core/src/trans/rust/expr_emit.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- `crates/auto-lang/src/tests/a2r_tests.rs`（现存，实际符号T-01重核；拟新接口在§5标注）。
- 现入口：`crates/auto-lang-core/src/test_cases/a2r.rs`、`ace_certify.rs`，`crates/auto-lang/src/bin/ace-certify.rs`、`scripts/verify-ace-m1-852.ps1`、`scripts/tests/test_ace_m1_acceptance.py`及`scripts/test-groups.toml`。

## 4. 需求分析与背景调查

### 用户裁定、范围与真实生产路径

2026-11-08用户已逐项明确选择：H-04“补齐旧A2R有限集合”；H-06“保留原checked/trap义务，不全局改旧int语义”；H-07“保留required并补通道”。三项均不是待答复裁定，不再提出NA/accepted/新HIR迁移作为默认方案；原16格、required313、原source/outcome/expected、NA/32 accepted/45 future及零cert-gap守卫保留。此授权落实为本组三份**规划合同**，不等实施/认证/合入授权，不批准任意新golden字节。

**实施授权（2026-11-08）**：`/goal 完成计划996` 为后续实施指令（同994先例）——一次授权覆盖本计划work（含建树）、review、有界返修与merge全链；材料唯一scope `D:/autostack/plan-artifacts/plan-996/`（开工已登记.retention）。本段即该授权的§4记录。

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
| `corpus:L01:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L01.at`及独立`reference/L01.rs` |
| `corpus:L02:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L02.at`及独立`reference/L02.rs` |
| `corpus:L03:a2r` | `emit-ok-run-unverified` | `value / 0` | `src/L03.at`及独立`reference/L03.rs` |
| `corpus:L04:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L04.at`及独立`reference/L04.rs` |
| `corpus:L05:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L05.at`及独立`reference/L05.rs` |
| `corpus:L07:a2r` | `emit-ok-run-unverified` | `value / 42` | `src/L07.at`及独立`reference/L07.rs` |
### 有限实现边界与先证路径

不能把991的“List<T>伪Rust”概括成类型映射全无：当前`ty.rs::rust_type_name`已有Type::List→Vec；`call_value_ctors.rs`已有若干List/Array.new形状（含带参数/GenName/Dot/Bina分类），原六源却从另一调用形状漏出。T-01以L01、L04/L05各一个真实生产全链锁定实际AST/类型/构造分发与方法路径，不按字符串替换`List<int>`；若根因在原parser节点投影，先给最小共同路径/必要parser职责，不能复制parser或加第二类型分析。

目标仅六原源及区分性近邻，int保旧carrier类型，未批准全局i32变更；L01=42、L02=42、L03=0、L04=42、L05=42、L07=42。构造类型参数/局部推导必须真实流入Vec元素表示，不能把List<int>动态变成JSON桶或丢名义record字段。get在合法读域内不隐式转移拥有权；remove返回元素拥有者；set按原语义替换旧元素；Str/record move、临时push、clear及最后owner释放符合现有资源规则，目标端不重做ownership。

T-02覆盖空List/非空、跨两个容器不混类型、owned Item/Str移入与remove后继续使用、调用源序与单求值；借用逃逸/用后move等由现有准入权威处理。L06越界原trap不在六值认证范围，作为行为边界须保原具名义务，不偷偷转value/NA；本包不借L06扩trap cert或提前关997。

原value cert/1与std-only足以表达本组六值（独立reference已在reference/L*.rs）；经G04新增六golden与六证书，a2r-only扩展`legacy-a2r-list-996.{case,suite}.ad`/拟固定组`case-legacy-a2r-list-996`从T-03接入显式日常扩展，不扩原base的旧VM/AC/AV路径。目标值cert32→38、日常35→41、gap16→10，旧32合法strict复用。新增资源观察仅为区分性补例，本包不冒R01..06场景认证；后者仅由998闭合。

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
| SD-03 | modify | `docs/specs/auto-lang/trans/architecture.md` | 本包实际有限能力/独立域与已关闭格、保留边界及source/profile/reader约束；历史分诊只作来源，不再用过时总数 | 只在最终review后沉淀真实current；本轮不改Spec | AC-01..04 |

## 6. 测试设计

所有V均为未执行拟议用例；具体新入口在对应任务使能。

| V | AC / invariant | 输入/实际入口与owner | 区分性oracle/反控 | 覆盖边界 |
|---|---|---|---|---|
| V-01 | AC-01/02/03 有限集合真实表示 | T-01/02，六原src与各原reference，L01/L04首链 | baseline伪Rust/编译红保留；修后六值42/42/0/42/42/42，两侧各对oracle，实际Vec/名义元素正确 | 不扩Map/Set/任意泛型/新HIR |
| V-02 | AC-01/02/03 拥有/源序与方法选择 | T-02，空表/两个表/owned元素push-get-remove-set-clear、非集合同名new/len/get | 结果/移入移出/旧元素释放符合原语义，实参一次；非集合既有行为不受名称替换 | 仅原合法域，不重做资源分析；L06仍trap义务 |
| V-03 | AC-01/02/03 新六证书与旧32 | T-03/04，六golden经G04、原certifier/三既有日常加本包扩展 | 六strict PASS，删一个Case/cert或改golden/expected即红；值域38/日常41、gap10 | 不核销scenario:R03，即使其依赖集合 |
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

### 7.3 work阶段验证记录（2026-11-08，树`D:/autostack/.wt/lang-996/auto-lang`，base`70539171f`，code`78bf8414d`→`cc994cfc1`→`d9d133ce3`）

- **入口冻结**：`cargo build -p auto-lang --bin auto-case --bin ace-certify` exit 0；rustc 1.98.0 (88d9e12ae 2026-08-18) PATH身份与38证书`default_identity_at_certify`一致。
- **首次红**（scope `t01-freeze/`，六actual SHA256在FREEZE.md）：wrap编译六例全`comparison operators cannot be chained` exit1（0参泛型构造逐字`List<T>.new()`为链式比较伪语法，下游get Option算术/set错映射insert被其掩盖，见文本记录）。
- **修复**：`generic_list_ctor_elem_ty`单源helper（int/str族/标量+已知struct；GenName("List<int>")/Bina两投影；嵌套泛型与非有限域→None）+ty.rs 0参推断`Type::List(elem)`+ctor strip 0参→`Vec::new()`+stmt_emit声明注解用实型；**具名收窄**：1参GenName strip曾扩即破`25_lifecycle/002`冻结金样→收回0参限定（字节稳定义务履行，见§9）。
- **六值全链**：actual wrap编译运行`i32:42/42/0/42/42/42`全对oracle；reference独立同值（scope `t02-verify/`双侧exit/out留档）；发射=冻结金样逐字节一致（t03-goldens再生成核对）。
- **永久用例**：`plan996_*`10例（六源形状+两容器分型/Str元素remove后复用/调用实参单求值源序/裸-1参域守恒【994字符len共存】）。
- **认证**：`ace-certify --only l01,l02,l03,l04,l05,l07` → 6 certified exit 0；全量复跑 **38 reused/0 failed** exit 0（旧32 strict复用）。
- **四scope**：snapshot=**41** a2r全绿（26 base+5 ext-992+4 ext-994+6 ext-996）+38证书strict+认证面spawn=0；receipts（in-PowerShell身份——F-994-R1-01操作注记适用）38有效；acceptance cert-gap **16→10**/can_claim_m1 **false**/285 required绿/45 future/32 accepted+6新；contract=131行+manifest全验PASS。
- **固定组并集**：`a2r`(415)/`infra-ace-m1`/`auto-test-case`/`ace-adapters`/`case-ace-m1-base`/`case-legacy-a2r-cert-992`/`case-legacy-a2r-text-994`/`case-legacy-a2r-list-996`/`case-ace-m1-a2r-emit-917`——cc994cfc1全绿后fmt+manifest冻结提交`d9d133ce3`处重跑（gate_passed=True，wall≈865s/次）。
- **反控**（scope副本`c1..c5`，树未触）：C1金样1B漂移（37→38）→snapshot FAIL恰1例`ace.m1.a2rt.l04:a2r-snapshot`；C2证书未知字段→REJECTED（`--force`同拒）；C3登记expected 42→43→证书REJECTED desync无半证书；C3b删`m1_l03.cert.json`→重认证恢复certified；C4扩展套件删l07检查→snapshot FAIL `40 < 41`；C5伪BuildIdentity→STALE。首跑一次撞锁归因：union构建与C4快照并发占用auto-case.exe（os error 5）→串行重跑即绿，非代码回归。
- **941只读交接**（原owner按真实身份接纳，本包不写941）：

| 格位 | source SHA16 | golden SHA16 | cert SHA16 | 终态 |
|---|---|---|---|---|
| corpus:L01:a2r | `8247b7e7688e1931` | `cae1ca88014809c8` | `f227785e6497f6ab` | certified |
| corpus:L02:a2r | `8276953bd3fe700f` | `61cf65c61b518c92` | `14e6d684ad48e2b9` | certified |
| corpus:L03:a2r | `299034a89f81f97a` | `c076319d294ffe0b` | `310072ff7b28c9de` | certified |
| corpus:L04:a2r | `a79b03e53e6e82bd` | `b5ec91ac4dee611b` | `81883e046ed9e32b` | certified |
| corpus:L05:a2r | `3c55fc65f45724c0` | `88f24d5f8a077268` | `e328dc7c447203e9` | certified |
| corpus:L07:a2r | `8aa9b61d53b3f0ce` | `67d445576543f89f` | `fe7379138ca61e99` | certified |

- **剩余10格**（acceptance冻结，本包不预填/不改守卫）：H-06 B13/B14/B15/B20（trap/宽度，997）+ H-07 scenario R01-R06（场景面，998）。全M1仍不可冒绿：`can_claim_m1=false`（cert-gap 10>0 且 future 45 在账）。
- **健康**：新增代码零调试打印；触面核心文件rustfmt净（ty.rs一处wrap已format化并复跑单测10/10）；a2r_tests.rs既有漂移为master存量不属本包；冻结提交`d9d133ce3`树clean。
- **master前移注记**：work期间master `70539171f`→`7b7a88c15`（PLAN-939整包合入归档）；触面重叠仅`scripts/test-groups.toml`（双方各自追加[[group]]块），rebase合并时保留双方；本包全部证据绑定自身树提交，合并阶段按994先例rebase+range-diff+复用理由。
- **材料根**：唯一scope `D:/autostack/plan-artifacts/plan-996/`（t01-freeze/t02-verify/t03-goldens/t04-certify/t04-scopes/t04-controls），merge折叠后整删；树内常规target正常，无第二输出根。

## 8. 执行步骤

未来唯一树`D:/autostack/.wt/lang-996/auto-lang`、分支`plan-996-dev`；本轮未建。后续实施指令后确认唯一仓/树/分支/base/HEAD和实际D；材料只在`D:/autostack/plan-artifacts/plan-996/`，开工登记owner/用途/在途。无junction、跨树可写target、仓内临时探针/log或第二输出根；常规target专树，需外置仅本scope。merge证据摘要折Plan后整scope删，具名持有须owner/理由/关闭条件；归档后不重开。

| T | owner_stage | 前置/写面 | 具体结果与验证/AC | 状态 |
|---|---|---|---|---|
| T-01 | work | 后续实施指令；D991/D994 | 冻结六格/原值参考与构造AST/type/分发根因；L01+L04/05有界baseline与合法近邻，先闭合std-only全链；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：六格首红现场冻结（scope t01-freeze/：actual六份SHA+wrap编译全`comparison operators cannot be chained` exit1）；根因链冻结=AST为`Dot(GenName("List<int>"),"new")`（非Bina），三处同形漏接——ty.rs infer仅1参、ctor strip仅1参、stmt_emit PLAN-010仅裸形；下游Unknown接收者致get落Vec::get Option形/set错映射insert、A8正则仅补字段链；FREEZE.md含修复策略/入口冻结/字节稳定义务 |
| T-02 | work | T-01 | 修复有限List构造/方法/拥有合法化，补永久原六源及区分性兄弟；现类型/准入单源，不修新HIR；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：generic_list_ctor_elem_ty单源helper（标量+已知struct，Bina/GenName两投影）+0参构造→Vec::new()+声明注解用实型（Vec<i64>/Vec<Item>）；1参GenName曾试扩strip即破25_lifecycle/002冻结金样→收窄回0参限定（字节稳定义务遵守）；六格wrap+reference双侧编译运行42/42/0/42/42/42全对；plan996_*永久单测10例；`cargo t a2r` 415全绿（含rustc实编门）；commit 78bf8414d |
| T-03 | work | T-02 | 逐源G04审阅完整actual，六新golden/六格token/manifest、a2r-only扩展与完整日常接线；拟固定组注册；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：逐源G04（源序/单求值/拥有转移/remove语句位弃值/set替换语义/L04字段位字节remap面ASCII恒等/两独立读clone）结论记§9；oracle/m1_l01..l07.rs六金样冻结（发射=验证字节逐位一致）；binding.tsv六行f[8]→ok-emit+PLAN-996注（131/313/批准不变）；legacy-a2r-list-996.case.ad+.suite.ad（ace.m1.a2rt.l01..l07）；固定组case-legacy-a2r-list-996注册实跑PASS；ps1 snapshot四套件插桩+`-lt 41`并集守卫；manifest：binding sha/golden 33→39/certs 32→38/suites 85→87+plan_996_update块 |
| T-04 | work | T-03 | 六值正式认证、旧32strict复用、日常41/零spawn/坏轴与漏格反控，实际gap16→10；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：`ace-certify --only l01..l07`→6 certified exit0；全量复跑38 reused/0 failed（旧32 strict复用）；snapshot=**41** a2r全绿（26+5+4+6）+38证书strict+认证面spawn=0；receipts（in-PowerShell身份）38有效；acceptance cert-gap **16→10**/can_claim_m1 false/313/45/38；contract 131行+manifest全验PASS；反控C1-C5全红/恢复（金样1B漂移→snapshot FAIL/证书未知字段--force拒/expected desync→REJECTED无半证书/删证重认证恢复/伪身份STALE），详见§9 |
| T-05 | work | T-04 | 完整a2r/治理/Case职责与保留负例、SD候选/余10和998接口/941只读交接；冻结交review；V-01..V-04按对应owner，AC-01..04 | [x] 2026-11-08：冻结9组并集在`d9d133ce3` gate_passed=True；返修`947b9b34f`（F-996-R1-01六证书`git add -f`入库树内38、F-996-R1-02 rustfmt净、F-996-R1-03残留头删）后plan996单测10/10+contract/receipts 38/snapshot 41+38+0spawn/cargo t a2r全复绿（a2r 415/infra-ace-m1/auto-test-case/ace-adapters/base/992ext/994ext/996ext/917）；SD-01..03候选维持§5表（review验证、merge沉淀）；余10清单+941交接SHA16表冻结于§7.3；健康零调试打印/触面fmt净；commit `d9d133ce3` |
| T-06 | review | 全部work完成、冻结后停实现写入 | V-05独立上下文重建全链、全部SD/reader/原负控/健康/省略审查，AC-05 | [x] 2026-11-08：独立subagent上下文两轮——round1于d9d133ce3全量终审**needs_fix**（F-996-R1-01 blocker六证书被.gitignore *.json吞未入库/F-996-R1-02 fmt漂移/F-996-R1-03残留头；AC-01..03独立重推导全pass含自建l07金样1B负控）；work返修947b9b34f后round2 delta复核**pass**（三发现全闭、delta恰7文件零token变更、38证书disk==git==manifest逐字节、receipts自跑38 strict、plan996 10/10；d9d133ce3证据按fmt-only+cert-additive复用理由成立）；F-996-R1-04留merge；两轮收据在§9 |
| T-07 | merge | T-06 pass与后续合入指令 | 冻结职责/触码漂移复验、SD沉淀读回/来源交付/归档，逐格只读交接941但不写它的在途文件，AC-06 | [x] 2026-11-08：rebase两轮零冲突（master间941合入7b7a88c15→93290d8e0持续前移），range-diff 5/5全`=`安全重写证明（ef37eaa71/109249da8/3fe3e20ed/4fe361107/6ecd6c190）；冻结9组并集在rebased代码tip复跑gate_passed=True（docs-only delta不触inputs，显式复用理由）；SD-01..03以docs-only后代`6ecd6c190`沉淀，master `--ff-only`线性合入tip=`6ecd6c190`；canonical三Spec读回hash见§9（a2r-golden 65c80b68…/gap-audit 53155852…/architecture 018e4bb5…）+master receipts冒烟**38 strict PASS**（in-PowerShell身份）+contract 131行全验；941未写（交接材料在§7.3） |
| T-08 | merge | 已核实landed/归档/规范来源 | 全部实际输出根对账、摘要折Plan、scope整删/持有登记，wt-guard clean后移除唯一树/分支/空组，AC-06 | [x] 2026-11-08：输出根对账——本计划命令仅写`D:/autostack/plan-artifacts/plan-996/`（t01-freeze/t02-verify/t03-goldens/t04-certify/t04-scopes/t04-controls/t06-review）与专属树target；scope整删（无具名持有），两处`.retention/policy.json`刷新；wt-guard clean→树/分支`plan-996-dev`/空组目录移除；归档`docs/plans/archive/`，T 8/8 |

## 9. 复审记录

2026-11-08 stage:new / PLAN-996 r1 / outcome:pass（仅具名裁定后的规划静态合同，非产品/独立实施review通过） / next:后续实施指令与任务实际D就绪后T-01。0/8，全部V与T未执行，未建树/scope/会话，未改production/语料/证书/current Specs或939/941。本计划的裁定记录以本轮用户三项明确回复为准，不重开991归档。静态复核收据在此续录。

2026-11-08 规划静态复核：0..10章节，T8/AC6/V5/SD3唯一连续/引用完整，全部任务未执行；本包6格与当前binding的ns/id/token/outcome/expected逐字对账，三包共16格无重复且恰覆盖全部剩余required cert-gap。原脚本/Spec/实际代码入口已读，新接口标拟新增；依赖来源存在、三包DAG无环、链接/表格/空白/编码/现有固定组名称与拟新组标记通过。中央取号996/997/998唯一，.next-id=999。三项用户裁定已明记，不继续列为待答复；T-01/02未知实现策略有具名全链可行性门。规划前后939/941、binding/applicability内容hash相同，未写current Specs/代码/原语料/证书、未建树/scope/会话、未执行产品/认证/性能/独立实施review；仅写本组三份Plan及.next-id。本检查不关闭任何格、不签M1。

2026-11-08 T-03 G04逐源审阅（树`D:/autostack/.wt/lang-996/auto-lang`，发射=冻结字节，工作材料scope t01-freeze/t02-verify/t03-goldens）：
- **L01** push20/22→get0+get1：源序读、每实参字面量一次、索引读左先右后、i64 clone=值拷贝无别名；42=20+22 ✓reference同值。
- **L02** [10,30]+insert(1,20)=[10,20,30]、remove(1)=[10,30]（语句位弃值忠实）、10+30+2=42；insert实参usize字面量一次。
- **L03** 空表len单求值判0、push42、clear、return 0；条件/体求值序忠实。
- **L04** owned Item构造（name独占String）→push(x.move)=纯move转移（x后不再用）；两次get(0)各自clone独立读（无共享借用问题）；字段位len保持字节remap面（994冻结边界；"hello" ASCII值恒等5）；5+37=42。
- **L05** 内联构造push、remove(0)移出owner至声明Item绑定、后续x.name/x.code+x.len合法（拥有已转移）；5+37+0=42。
- **L07** set(1,22)=索引赋值**替换**语义（PLAN-019，非insert右移）；20+22=42恰与insert同值属巧合，金样以替换形锚定区分。
- 边界：L06 trap不在值域；std-only（六actual零a2r_std/零外部依赖）；求值序/单求值/拥有合法性均与独立reference双侧同输出。

2026-11-08 stage: work | plan_id: PLAN-996 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-996/auto-lang | branch: plan-996-dev | base_commit: 70539171f | code_commit: d9d133ce3（78bf8414d修复+cc994cfc1语料接线+d9d133ce3冻结prep） | dependency_revisions: D991/D994归档核验（master祖先链）；work期间master前移至7b7a88c15（PLAN-939合入），触面重叠仅test-groups.toml，rebase合并时解决；rustc 1.98.0 (88d9e12ae 2026-08-18) PATH身份=38证书default_identity_at_certify | task_ids: T-01..T-05（5/5） | evidence: §7.3（首红六例冻结/三处根因修复含1参GenName收窄/六值双侧42-42-0-42-42-42/6 certified+38 reused/snapshot 41+0 spawn/receipts 38/acceptance gap16→10 claim false 313/contract PASS/冻结9组并集d9d133ce3 gate_passed=True/反控C1-C5全红/plan996单测10例/941交接SHA16表/余10清单） | blockers: 无 | 未执行: 全仓统一回归（owner=merge按冻结并集；周期cargo t all按due由原owner）；SD-01..03沉淀（owner=merge） | next_responsible_role: review（T-06，同树独立上下文，冻结commit d9d133ce3 + 本Plan文件）

stage: goal | plan_id: PLAN-996 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-996/auto-lang | commit: d9d133ce3 | next: review（T-06）

2026-11-08 stage: review | review_scope: final | reviewer_identity: 独立 subagent 上下文（general-purpose，未参与work），同一冻结树只读，材料仅plan-996/t06-review/ | plan_id: PLAN-996 | plan_revision: r1 | outcome: **needs_fix** | worktree_path: D:/autostack/.wt/lang-996/auto-lang | branch: plan-996-dev | reviewed_commit: d9d133ce3f9f7332752342fa7fccff49fe8f1962（树clean） | base_commit: 70539171f | dependency_revisions: D991/D994归档（master祖先）；rustc 1.98.0 PATH身份=38证书default_identity_at_certify（receipts in-PowerShell复核） | spec_inputs: plan§4指纹复核全符（base ps1=2da7cfcc…；a2r-golden-certification/ace-m1-scenario-oracles/ace-m1-applicability主检出未漂移）；SD-01..03目标docs存在且before/after规则符实 | acceptance_results: AC-01 pass（三站点diff独立重推导+plan996单测10/10+certify复用6/6+L04金样/reference手编双侧i32:42）；AC-02 pass（六值42/42/0/42/42/42重推导，set替换/拥有/单求值由10例实质单测锁定，L06未触）；AC-03 pass in-tree（snapshot 41+38strict+0spawn/receipts 38/acceptance gap10 claim False/contract 131+manifest全验/自建l07金样1B漂移负控snapshot exit1恰l07/case-legacy-a2r-list-996 PASS）；AC-04 pass in-worktree但**交付面fail**（六新证书未入库）；AC-05=本审 | pending_closeout_gates: F-996-R1-01返修+新冻结提交+contract/receipts/snapshot复绿；SD沉淀（merge）；merge期rebase（test-groups.toml双方保留）；输出根对账/scope整删/清树（T-07/T-08） | findings: **F-996-R1-01 blocker**——六新证书`oracle/certs/m1_l0*.cert.json`仅磁盘未跟踪（`.gitignore:162` `*.json`吞没；d9d133ce3树内恰32证书、磁盘38；manifest已钉38——合并清树后fresh checkout契约`32≠38`、snapshot/receipts `certificate MISSING`必红；证书含created_at再生即破SHA，重生成非修复；PLAN-852先例为`git add -f`）；F-996-R1-02 minor（a2r_tests.rs新增plan996块内5处rustfmt漂移，另1处为master存量）；F-996-R1-03 info（T-01临时探针注释头残留无代码体）；F-996-R1-04 info（master前移7b7a88c15仅test-groups.toml重叠，merge处理） | evidence: plan-996/t06-review/{snapshot-run.log,snapshot/,acceptance-run.log,acceptance/,contract-run.log,receipts-run.log,cert-reuse/,hand-compile-l04/,c-golden-mut/} | next: work（返修F-996-R1-01..03纯增量修复→新冻结commit delta复核）→ merge（T-07）

2026-11-08 stage: work | plan_id: PLAN-996 | plan_revision: r1 | outcome: pass（返修复核版） | worktree_path: D:/autostack/.wt/lang-996/auto-lang | branch: plan-996-dev | base_commit: 70539171f | code_commit: 947b9b34f（=d9d133ce3+返修） | task_ids: T-05返修（F-996-R1-01..03） | evidence: 六证书\`git add -f\`入库（\`git ls-tree HEAD\`证书数38=manifest钉值；d9d133ce3树内32系\`.gitignore:162\` \*.json\`吞没——评审F-996-R1-01根因确认）；rustfmt a2r_tests.rs全净（含顺带修复master存量行5漂移，fmt-exit=0）；T-01探针残留头删除；复跑：plan996单测10/10、contract PASS、receipts 38 strict PASS、snapshot 41a2r+38strict+0spawn PASS、\`cargo t a2r\` PASS（415） | blockers: 无 | next_responsible_role: review（delta复审核\`947b9b34f\`，同评审上下文）

stage: goal | plan_id: PLAN-996 | plan_revision: r1 | round: 2 | from: review | to: review | status: execution_done | outcome: needs_fix→返修pass | worktree: D:/autostack/.wt/lang-996/auto-lang | commit: 947b9b34f | next: review（delta复核）

2026-11-08 stage: review | review_scope: delta（round 2，返修复核947b9b34f；round 1全量复核d9d133ce3见前收据） | reviewer_identity: 独立 subagent 上下文（general-purpose，未参与work），同一冻结树只读，材料仅plan-996/t06-review/ | plan_id: PLAN-996 | plan_revision: r1 | outcome: **pass** | worktree_path: D:/autostack/.wt/lang-996/auto-lang | branch: plan-996-dev | reviewed_commit: 947b9b34f68c48b1f6fd5fa81e8d37342615991a（树clean） | base_commit: 70539171f | dependency_revisions: D991/D994归档（master祖先）；rustc 1.98.0 PATH身份=38证书default_identity_at_certify（本轮自m1_l03证书in-PowerShell复核） | spec_inputs: 同round 1（plan§4指纹+SD目标docs已核，delta未触） | acceptance_results: delta面——d9d133ce3..947b9b34f恰7文件（6证书全量add+a2r_tests.rs fmt-only/头删除，零token变更）；ls-tree证书38=manifest钉值；六证书disk==git==manifest SHA逐字节一致；rustfmt a2r_tests.rs exit 0；plan996单测10/10；receipts（自跑）38 strict PASS exit 0；复用d9d133ce3有效证据AC-01..03（fmt-only+cert-additive delta不失效）；work侧947b9b34f复跑snapshot/cargo t a2r 415绿 | pending_closeout_gates: SD-01..03沉淀读回（merge）；merge期rebase（master 7b7a88c15，test-groups.toml双方保留，F-996-R1-04）+冻结并集触面复验；输出根对账/scope整删/wt-guard清树（T-07/T-08） | findings: round1 F-996-R1-01 blocker已闭、F-996-R1-02已闭、F-996-R1-03已闭；F-996-R1-04留merge；无新发现 | evidence: plan-996/t06-review/{receipts-repair-run.log,receipts-repair/}+round1全部在档材料 | next: merge（T-07）

stage: goal | plan_id: PLAN-996 | plan_revision: r1 | round: 3 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-996/auto-lang | commit: 947b9b34f | next: merge（T-07/T-08）

2026-11-08 stage: merge | PLAN-996:r1 | checkpoints: prepared→landed→canonical_verified→archived→cleaned | delivery_commit: 6ecd6c190（`--ff-only`线性；两轮rebase零冲突，range-diff 5/5全`=`等价——安全重写证明；映射 ef37eaa71/109249da8/3fe3e20ed/4fe361107/6ecd6c190） | canonical: SD-01 `docs/specs/auto-test-case/a2r-golden-certification.md` `65c80b6835f4a95f…`、SD-02 `docs/specs/auto-test-case/ace-m1-gap-audit.md` `5315585282591265…`、SD-03 `docs/specs/auto-lang/trans/architecture.md` `018e4bb51a79e1e6…` 均在master读回一致 | master冒烟: receipts **38证书strict PASS**（in-PowerShell BuildIdentity）+contract 131行/manifest全验PASS | 冻结职责并集复用: 9组在rebased代码tip（4fe361107等价位）gate_passed=True，SD docs-only后代零触面inputs不变（显式复用理由） | F-996-R1-01教训落SD-01（证书入库纪律） | landing_is_not_deployment: 本包交付=测试语料/治理脚本/规范文档+transpile期emitter行为（编译期），无后端二进制/daemon/web bundle消费者；auto-case/ace-certify为按需重建dev-harness（Find-Prebuilt），无重启项 | batch_regression_due: 见收据尾注（996%5≠0，receipt<48h） | outcome: **pass**（delivered+archived+cleaned） | next: done



## 10. 待澄清事项

1. H-04/H-06/H-07三项用户已裁定保旧域/checked期望/六场景required，不能继续以“等用户选NA/新HIR”停工或扩大授权；本轮仍仅规划，未来实施依后续指令。
2. 新旧配置的实现、观察位置/schema最小形状由T-01/02有界证明；证明失败或需改变原语义/预算/范围，先needs_replan，不用预填PASS/新NA解决。
3. 跨包/跨域证据适用性由真实身份核对；若941原owner尚未接收，分列本域已关闭与最终组合待接纳。其它required/已登记缺陷不因本包通道绿自动消失。
4. 新域证书/适用字段读者尚不存在者已标拟新增，work须列完整producer→strict reader→daily/恢复/父判路径；未适配默认拒，不依赖general ok前缀跳过认证。
