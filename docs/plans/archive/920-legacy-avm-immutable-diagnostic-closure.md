---
plan_id: PLAN-920
status: archived
feature_name: 旧AVM不可变赋值诊断与M1 B18收口
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-03
plan_revision: 1
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-gap-audit.md, docs/specs/auto-test-case/ace-m1-applicability.md, docs/specs/auto-lang/vm/overview.md]
new_spec_components: [docs/specs/auto-test-case/legacy-immutable-diagnostic.md]
touched_goals: [GOAL-002]
affects: [auto-lang/vm, auto-test-case]
depends_on: [PLAN-908, PLAN-911, PLAN-916]
current_step: 7
total_steps: 8
---

# [PLAN-920] 旧AVM不可变赋值诊断与M1 B18收口

## 0. 变更摘要

只关闭corpus B18 avm一格：旧codegen已拒绝let重赋值，但报AutoError::Msg，父侧正确将其分类infra，不能满足语言负例。改真实生产错误事实、保持父侧严格准入；覆盖同类普通/复合/语句赋值与合法近邻，不用字符串特判把任意Msg变成语言错误。drafting r1、0/8，本轮只规划。

依据[Design43 §20](../design/43-ac-av-typed-mir-and-abc.md#20-915917在途时的下一波2026-11-02target-state)。旧实现有限修复仅服务原M1证据；M2仍须替换并disable，不能据此延长双引擎生命周期。

## 1. 目标

G1：生产codegen发出已有typed NameError::ImmutableAssignment事实，stage/code/source/span保真。G2：原B18通过语义原因判面，NB18合法可写变量真运行得2，普通与复合赋值及作用域遮蔽都区分。G3：panic/IO/未知Msg/运行陷阱/坏证据仍不能通过该语言负例。

非目标：全VM错误体系重构、改变let/var语义、重写typechecker/VM、为旧VM新增增量cache、改原source/expected/NA/accepted、重开911/916或签全M1。

## 2. 架构方案

原源码→旧生产parser/codegen实际不允许赋值的分支→AutoError::Name(NameError::ImmutableAssignment)→run_with_capture_and_path_staged→classify_staged→avm adapter→独立父判。复用auto-diagnostics已有NameError及miette稳定码auto_name_E0203，不创建第二诊断器/新字符串词表。

不得在adapter通过case id/expected/message substring补stage/code；不得把所有AutoError::Msg/MsgWithSource无条件升级为B18语义。原source/作用域/目标是否immutable由codegen事实判，不靠wrapper重分析源码。

## 3. 技术栈

Rust，auto-lang-core旧vm/codegen、auto-diagnostics既有错误、staged API与auto-test-case；既有文件化负例/合法近邻和固定职责组。

## 4. 需求分析与背景调查

授权：2026-11-02用户要求提前规划下一波，本轮仅文档/取号；未授权施工/测试/启动agent或跨仓写入。main=5a0ef7609，913/914/916已交付，915/917在途不代改。
2026-11-03执行授权：`/goal 完成计划920`（auto-plan-goal 单授权链）——授权本计划全链条 work/review/有界repair/merge（建树、施工、测试、SD沉淀、archive、树/scope清理）；不改目标/验收/阈值/兼容承诺/仓范围/预算。树 D:/autostack/.wt/lang-920/auto-lang（分支 plan-920-dev，基面 master@b823471d3），scope D:/autostack/plan-artifacts/plan-920/ 已登记开工。

| 实读来源 | 当前事实 |
|---|---|
| crates/auto-lang-core/src/vm/codegen/expr_ops.rs | SHA256 26e0a392eb2d931b9be973bdefa9443b8bb58a3af48c51532c3f953bf6f28483；普通/复合赋值immutability分支仍返回Msg |
| crates/auto-lang-core/src/vm/codegen/stmt_emit.rs | 第三处immutable赋值拒绝同族，必须核并覆盖，不只修B18命中一处 |
| crates/auto-diagnostics/src/error.rs | NameError::ImmutableAssignment{name,span}已有，miette code auto_name_E0203；无需扩公用错误enum |
| crates/auto-lang-core/src/test_cases/mod.rs::classify_staged | Name映Type；Msg无typed trap为Infrastructure，原严格边界正确 |
| crates/auto-lang-core/src/test_cases/avm.rs | SHA256 138f4931f1565b46728f08005f25a53b2b142761d60e04d1c6f4e62253f728f3；catch_unwind/infra不可满足语言负例 |
| crates/auto-lang/test/cases/ace-m1/src/B18.at | 原let x int=1; x=2; return x；不得改为另一种易诊断输入 |
| docs/specs/auto-test-case/ace-m1-applicability.md | B18明确不在R-ACCEPTED，infra观察仍阻核心；NB18合法邻居结果2 |

903/908/911/916交付证明其他路径已处理，不证明此旧VM一格已关闭；不开旧归档。T-01核当前生产路径span来源、三拒绝分支、staged/非staged全部公开consumer，差异具名入计划。

## 5. 详细设计

### 5.1 生产事实与兼容

使用实际lhs源码范围/变量名构造已有typed错误；若语句节点缺精确span，T-01选现有合法源码上下文范围并明确精度，禁止猜一个指向其他变量的span或用全文解析重建诊断。源码路径/输入身份沿既有API；staged与普通运行都传播相同错误类型，不仅测试入口特供。错误展示允许按已有NameError标准文本，冻结语义expected不改。

核普通赋值、+=等复合赋值及stmt_emit对应写路；合法var、同名内层var、不同域let不误判。已支持捕获的赋值分支按原支持规则，未实现捕获复合赋值不能为此改成immutable，保原unsupported/infra分类。不把此包扩为全部assign相关错误修复。

### 5.2 父侧判面与文件化

父侧用真实typed Name→Type、code auto_name_E0203及原因判定；确需将原AutoDown义务接到typed raw code时加独立观察/补充case，保原B18 source/outcome/expected及各他方格，不修改原oracle来适应错误结果。原B18 avm格只有在完整正式父证据通过后回填。

新增case族拟crates/auto-lang/test/cases/ace-m1/immutable-avm-920/，以原B18与NB18为基点，补普通/复合/遮蔽/异族。unknown Msg、含完全相同B18文本的IO/Msg、panic、错误type原因、缺/旧typed事实都非绿；受管源/操作/事实绑定沿原TestCase身份，不给请求端带“应报原因”。

### 5.3 重复/失败修复与并行

本包不新建旧VM函数级生产系统：fresh/重复执行/进程重启/失败负例→合法body修复→再负例，均真实走生产入口并计量实际工作，不把旧VM整源重编写成未改P0。ACE函数级增量不由此获得新能力；公共诊断/测试改动须验证已有ACE消费者无回退。普通VM小修不套30样本性能矩阵；源4MiB/每流1MiB/原Job/截止/资源条件保持，panic和超时仍infra。

P/S=旧codegen expr_ops/stmt_emit、诊断回归与avm adapter；与917的trans/rust主体独立，与915/919 AV主体独立。auto-diagnostics只复用既有类型，若新增公用形状需先重核所有construct/match，不能顺手修改。H=test_cases/mod.rs/avm.rs/auto-test-case/固定组/binding/source-manifest与917/919/921逐文件短窗单写。无需D915/D917整包前置；915/917运行采样不争机器。每窗登记owner/revision/SHA/schema/全部consumer与释放。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/auto-test-case/legacy-immutable-diagnostic.md | plain Msg infra→真实typed不可变赋值诊断 | 因果与父信任 | AC-01/02/03 |
| SD-02 | modify | docs/specs/auto-lang/vm/overview.md | 补生产诊断/原语义/覆盖入口 | 不变语言范围 | AC-01/04 |
| SD-03 | modify | docs/specs/auto-test-case/ace-m1-conformance.md | B18 avm真实父证据回填 | 不签总M1 | AC-02 |
| SD-04 | modify | docs/specs/auto-test-case/ace-m1-gap-audit.md | 单格owner关闭与历史保留 | 不改批准/NA | AC-02/05 |
| SD-04b | modify | docs/specs/auto-test-case/ace-m1-applicability.md | B18 avm 两处例外句从"infra阻核心"刷新为"PLAN-920 typed关闭ok:P920，非R-ACCEPTED"（review轮补录实际触面；计数/规则零改动） | 登记与事实一致 | AC-02/05 |

## 6. 测试设计

既有固定组vm、auto-test-case、case-ace-m1-base、case-ace-m1-nb-diags-908、ace-adapters、infra-ace-m1；若实际修改公用diagnostics实现，补其真实消费者完整组，不仅跑parser。新拟case-ace-m1-immutable-avm-920由T-04注册。所有命令/用例均为未来实施门，本轮未执行。

| Case ID | AC | 输入/入口/使能 | 独立结果与区分性反控 | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/02 | 原B18，真实staged及非staged，T-02/04 | Type+auto_name_E0203/immutable真实原因；发同文本Msg时父判必须红 | 原source不改 |
| V-02 | AC-01 | 普通/复合/语句三真实拒绝分支，T-02 | 同族typed错误/有效范围，实际写入0 | 原不允许let写语义 |
| V-03 | AC-02/04 | NB18＋内层var/外层let/合法相邻，T-04 | NB18真实2、合法均可执行；一律拒赋值反控红 | 无新捕获语义 |
| V-04 | AC-03 | IO/Msg含B18原文本、panic、超时、run trap/异族typed，T-03 | infra/错误族不能匹配B18；不substring分类 | 父严格边界 |
| V-05 | AC-02/03 | 坏源/操作/旧或缺事实、原完整基套件，T-04 | 坏身份拒绝；恰单格改善、其他差异保持 | 313旧义务守恒 |
| V-06 | AC-04 | fresh/重复/restart、负→合法修复→负，T-05 | 类型/原因不串、合法重验、真实工作分账，不报伪P0 | 无新旧VM增量承诺 |
| V-07 | AC-04/05 | 完整vm/必要adapter组，T-05 | 原执行/API/错误分类回归；无警告/格式/调试输出 | 独立review另核 |

实现者自查后交同树独立上下文final review，不指定/创建另一agent/chat。review冻结SHA暂停写；merge据真实diff重核冻结职责并集，身份不变证据可复用，不叠全仓轮次；cargo t all仅周期due主检出单实例。

## 7. 验收标准

| AC | owner_stage | 关闭条件 |
|---|---|---|
| AC-01 | work | 三生产拒绝路径typed Name事实/原因/来源正确，普通与staged一致，V-01/02通过 |
| AC-02 | work | 原B18＋合法近邻/遮蔽/单格登记V-01/03/05通过，原source/expected/分母不改 |
| AC-03 | work | V-04/05真实异族/infra/坏事实控制非绿，不放宽通用Msg分类 |
| AC-04 | work | V-03/06/07原VM/API/ACE adapter兼容，失败修复重验，无虚构函数增量声明 |
| AC-05 | review | 独立冻结review核事实生产→消费全链、补例/SD，无漏项或未批延期 |
| AC-06 | merge | 主线落地/SD读回/archive及全路径材料/树/scope guard清理收据 |

## 8. 执行步骤

| T | owner_stage | 前置/写面 | 结果与验证 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 当前main/917 H窗 | 核三拒绝生产点/span/staged公开consumer，冻结最小修复与用例 | AC-01/04 | [x] 已完成（7d9119215 前置调查：SHA零漂移；三拒绝位=expr_ops:92复合/expr_ops:226裸赋值(B18命中)/stmt_emit:841 Store防御位；防御位生产不可达实证=仅Field过`!is_new_declaration`且Field语句位走TypeDecl；D-01 span裁定=Plan199行号×Plan057源文本行级范围+`fn_span(None)`退化约定；D-03 H窗=917已归档释放、919 AV侧无重叠；设计冻结于plan-artifacts/plan-920/t01-frozen-design.md） |
| T-02 | work | T-01，vm/codegen | 复用已有NameError正确传播，V-01/02；cargo t vm | AC-01 | [x] 已完成（7d9119215：三拒绝位→`immutable_assign_error`辅助（Name/E0203+行级span），staged与非staged同型；新`vm/immutable_assign_920.rs`7例绿——B18非staged/staged判Language·type·E0203、复合同族、防御位AST证人、近邻不误拒、顶层span(14,5)与fn体(34,9)精度、伪造Msg反控；cargo t vm 912/912） |
| T-03 | work | T-02，adapter H窗 | 保/补严格typed事实消费与V-04/05反控，不靠文本分类；cargo t auto-test-case ace-adapters | AC-03 | [x] 已完成（f93000923：adapter本身零改动——classify_staged/avm run_check已type驱动（实读核）；补3单元反控=异族typed E0201各归其码、typed run-trap判Run不串Type；cargo t auto-test-case 111/111、ace-adapters 48/48、定向9/9） |
| T-04 | work | T-02/03，登记 H窗 | AutoDown原B18/近邻/补例、新组及单格回填；V-01..05完整相关case组、infra-ace-m1 | AC-02/03 | [x] 已完成（a871bbebf：新族immutable-avm-920（B18原源/B18C复合/TOP顶层三负例+NB18/NB18S同名遮蔽近邻）5/5绿；固定组case-ace-m1-immutable-avm-920注册；binding.tsv B18行avm格`diff:pending→ok:P920`（其余列/行不动）；manifest suites_cases 76→81+plan_920_update+binding sha256；applicability.md两处例外句刷新；V-05守恒=base 97/97、nb-diags-908 9/9、infra-ace-m1 23/23含未变异对照真verifier绿） |
| T-05 | work | T-01..04 | V-06/07和明确完整职责组、SD提案/自查，不跑无关性能矩阵 | AC-04 | [x] 已完成（e0fe3ba64：V-06负→合法修复得2→再负同typed事实循环入单元锁（10例）；SD-01新legacy-immutable-diagnostic.md、SD-02 vm/overview bullet、SD-03 conformance PLAN-920节+例外句、SD-04 gap-audit F-INFRA-3关闭节（历史行保留）；冻结职责并集7组一次全PASS=vm 915/915、auto-test-case 111、ace-adapters 48、base 97、nb-diags 9、immutable-avm-920 5、infra-ace-m1 23；健康=my区域零新warning/零fmt diff/无调试输出） |
| T-06 | review | work完成，冻结SHA | 独立上下文审每AC与反控、Spec delta；pass后才reviewed | AC-05 | [x] 已完成（2026-11-03 final review pass @e0fe3ba64：同会话独立复审（自工件重建——全 diff 重读、绑定/清单 hash 独立核算、冻结树上复放 10/10 单元+5/5 套件、独立探针 3/3 嵌套块复合/while体裸赋值/var 对照经真实 avm adapter 全 typed E0203 或成功；实施会话内复审限制如实声明）；SD 表补录 SD-04b applicability.md 实际触面；详见 §9 review 收据） |
| T-07 | merge | review pass | 漂移重验/冻结职责并集、实际合入/SD-01..04读回/archive | AC-06 | [x] 已完成（rebase onto master(9c81ee6d7)+range-diff 四提交全等（e0fe3ba64→06a6f8c72 映射：7d9119215→03f1b8787、f93000923→12be4cd47、a871bbebf→96882d2ce、e0fe3ba64→06a6f8c72）；复验冻结职责并集于 06a6f8c72＝7组全PASS同计数（vm 915/111/48/97/9/5/23，run-5z5qf0ss，gate_passed=true）；wt-guard clean 后 master `--ff-only` 落地，tip=06a6f8c72 线性无merge提交；SD-01..04b 五路径落 landed-identity 全OK+marker 全在；archive 本提交） |
| T-08 | merge | T-07 | 全实际输出对账/摘要折计划/guard后清树分支组和整个scope、刷新登记 | AC-06 | [ ] |

拟唯一树D:/autostack/.wt/lang-920/auto-lang，分支plan-920-dev，本轮未建；材料唯一D:/autostack/plan-artifacts/plan-920/开工登记，不散落/不放attachments。merge整删，只有具名未解决hold可留，禁止链接及删除他方材料。

## 9. 复审记录

2026-11-02 | stage:new | PLAN-920 r1 | outcome:pass（规划静态准入，非功能/最终review） | 0..10节、T8/AC6/V7/SD4定义引用/路径锚点/固定组/单格互斥全核；next:work T-01，可独立核旧codegen，H与917实际owner短窗。全部运行门未执行，未建920树/scope；原批准/规则/源码/expected未改，不从915待review推定共享H已释放。

2026-11-02 | stage:new | PLAN-920 r1 | 起草：实读三处codegen Msg与既有NameError/严格父分类，按单格有限修复立约；不重复旧VM工程重构或912增量。本轮只规划/正式取号，无实施/测试/current Spec或原登记变更；静态预检另记，所有运行V/AC待work。

2026-11-03 | stage: work | PLAN-920 r1 | outcome: pass | worktree: D:/autostack/.wt/lang-920/auto-lang | branch: plan-920-dev | base_commit: b823471d3 | code_commit: e0fe3ba64（7d9119215→f93000923→a871bbebf→e0fe3ba64） | dependency_revisions: auto-down 组内 detached 7c4d5cb（本包测试未消费） | task_ids: T-01..T-05 | evidence: B18 avm单格关闭=avm/run对src/B18.at干净typed拒绝（Language/type/auto_name_E0203，staged与非staged同型）；三生产拒绝位全typed化（含复合与不可达防御位AST证人）；单元10例全绿；套件immutable-avm-920 5/5；冻结职责并集一次7组PASS（vm 915/915、auto-test-case 111/111、ace-adapters 48/48、case-ace-m1-base 97/97、nb-diags-908 9/9、immutable-avm-920 5/5、infra-ace-m1 23/23，run-wsyc9ieq）；V-05守恒=131行/12列/524格不动、仅B18行avm一格翻ok:P920、base套件零改动；反控=伪造B18文本Msg/异族typed E0201/run-trap通道分离/IO·panic infra既有+新增单元锁；健康=零新warning、my区域零fmt diff、无调试输出 | blockers: 无合同内未决（D-01/02/03已闭见§10）；SD-01..04已按§5表格落盘worktree待review核与merge沉淀 | next_responsible_role: review（T-06 独立冻结复审）

stage: goal | plan_id: PLAN-920 | plan_revision: r1 | round: 1 | from: work | to: review | status: execution_done | outcome: pass | worktree: D:/autostack/.wt/lang-920/auto-lang | commit: e0fe3ba64 | next: review

2026-11-03 | stage: review | review_scope: final | reviewer: 同会话独立复审（自工件重建：全 diff 逐 hunk 重读、registry hash 独立核算、冻结树复放、独立探针；未采信执行摘要。限制如实声明：与实施同会话/同上下文，独立上下文不可用——计划明文不指定/不创建另一 agent/chat） | plan_id: PLAN-920 | plan_revision: r1 | outcome: pass | worktree: D:/autostack/.wt/lang-920/auto-lang | branch: plan-920-dev | reviewed_commit: e0fe3ba64 | base_commit: b823471d3 | dependency_revisions: auto-down 组内 detached 7c4d5cb 未消费 | spec_inputs: SD-01 全文 vs 代码行为（三拒绝位/行级span/classify边界/已知扁平mutability边界全对码核）；SD-02/03/04/04b diff 逐行；frontmatter supersedes/new 与实际触面一致（review 补录 SD-04b applicability.md）；规范链接解析全通 | acceptance_results: AC-01 pass（三生产拒绝位 typed 化逐 hunk 核；单元 10/10 复放含 staged/非staged 同型+span (34,9)/(14,5)；AC-02 pass（B18 原源零改动 diff 证、套件 5/5 复放、binding 单格 ok:P920 且 131行×12列、base 97/97、manifest 81 文件 hash 零漂移+binding.sha256 核对）；AC-03 pass（adapter/classify_staged 零改动 diff 证；伪造 B18 文本 Msg/异族 E0201/run-trap 三反控+既有 IO/panic 控制绿；infra-ace-m1 23/23 含真 verifier 未变异对照）；AC-04 pass（vm 915/915、auto-test-case 111/111、ace-adapters 48/48 冻结基线一次并集 run-wsyc9ieq；V-06 负→修复→负循环绿；无函数级增量声明）；AC-05 本 review 关闭。独立探针（评审者自选输入，真实 avm adapter，plan-artifacts/plan-920/probe）：嵌套块复合/while 体裸赋值→typed E0203、var 对照→2，3/3 | pending_closeout_gates: merge-owned——冻结职责并集在合入基面复验（identity 不变可复用 run-wsyc9ieq）、852 三 scope 复放、SD-01..04b 读回、archive、guard 清树/分支/组/scope | findings: 无阻断。F-920-O1（P4 簿记，review 已修）SD 表漏 applicability.md 触面→补 SD-04b；F-920-O2（P4 信息）master 漂移 2 个 919 簿记提交（仅 docs/plans/919 文件，本分支未触该文件，合并不冲突）；F-920-O3（P4 信息）全仓既有 fmt 漂移 181 hunks 均在本计划 hunk 之外（expr_ops:322+ 等），非本包债务 | evidence: 本记录内嵌命令/结果即持久证据；探针输入/报告在 plan-artifacts/plan-920/probe（merge 折叠后整删） | next: merge

stage: goal | plan_id: PLAN-920 | plan_revision: r1 | round: 2 | from: review | to: merge | status: reviewed | outcome: pass | worktree: D:/autostack/.wt/lang-920/auto-lang | commit: e0fe3ba64 | next: merge

## 10. 待澄清事项

| D | owner / 有界关闭 |
|---|---|
| D-01 span来源 | **已闭（T-01）**：旧AST无节点级span（Name=AutoStr）；裁定=Plan199行号×Plan057源文本的当前语句行字节范围（行粒度，fn体实证(34,9)、顶层(14,5)），无行号/源文本退化(0,1)同`fn_span(None)`约定；不猜变量位、不全文重建 |
| D-02 第三写路覆盖 | **已闭（T-01/02）**：stmt_emit防御位生产不可达（仅Field过守卫且Field语句位走TypeDecl）；同族typed对齐+AST级证人锁行为，不伪造源级用例；未支持捕获复合赋值保持原Msg/infra族未动 |
| D-03 H交接 | **已闭（T-01）**：917已归档（9ae8df983合入）其窗释放；919在途但AV侧（av.rs/driver）与本包写面（vm/codegen、test-groups.toml、binding.tsv、manifest）无文件重叠，实际单写者 |
