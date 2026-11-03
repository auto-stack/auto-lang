---
plan_id: PLAN-918
status: archived
feature_name: 跨仓交付验证与依赖边界治理收口
author: [Codex]
created_at: 2026-11-02
updated_at: 2026-11-03
plan_revision: 1
current_step: 9
total_steps: 9
supersedes_spec_components: [docs/specs/repository-boundaries/web-four-repository-delivery.md, docs/specs/design/repository-layout.md]
new_spec_components: [docs/specs/repository-boundaries/delivery-admission-and-boundary-policy.md]
touched_goals: []
affects: [test-governance-web-delivery, four-repository-fixed-controls, modularity-audit, module-boundaries-policy, extraction-roadmap]
depends_on: [PLAN-896, PLAN-906, PLAN-910]
---

# [PLAN-918] 跨仓交付验证与依赖边界治理收口

## 0. 变更摘要

有限修复语言仓两项具名治理差额：D-910-1的Web发行收据实际执行ID准入，以及D-906-3的owned执行设施合法依赖与模块审计基线锚。保原用例/收据/来源与禁向规则的判定能力，正确拒绝伪PASS；同时刷新路线交付记录及各仓后继归属。

**2026-11-03 实施完成**：用户「完成计划918」经 /auto-plan-goal 授权整链；唯一组 `D:/autostack/.wt/lang-918/{auto-lang,auto-ui,auto-web,auto-man}`（外仓 detached 于正式 HEAD），语言分支 plan-918-dev。T-02 恢复并加固 `execution_ids`（web-delivery-controls 13/13 PASS，原 12/13+NameError）；T-03 新增 `[[rules.facility_admission]]` 逐边 std-only 设施准入（self-test 16/16，真实仓设施边具名 admitted）；T-04 干净树重冻可解析新锚（strict 全绿，metadata 66/66、双 observe 字节一致、陈旧/脏树/跨仓脏态负控全部实证）；T-05/06 债务台账闭合 D-910-1/D-906-3（实证），D-906-1/D-910-2 保 owner，schema 级 ID 对账差额入合同。execution_done 6/9，review 待 T-07。

## 1. 目标

- G1：收据校验路径不再因未定义函数中止；actual ID/summary/hash/source/config及原条件均真实验证，新跑与合法复用路径同等严格。
- G2：基于PLAN-896已交付的std-only owned设施合同，区分合法设施消费与真正上层实现依赖；规则精确表达前者，后者仍稳定判红，不回退到裸Command或删除owned寿命保护。
- G3：基线锚可解析、可复现且绑定已审查冻结树；全部新增/退出边和owner路径变化有据，未知违规不因重采样转为合法。
- G4：路线记录接受已交付890/898/900/901/906/910、UI014/018/007，跟踪031及UI033/OS053；准确保留Web/Man未闭义务，不将本包包装成全四仓/全产品已绿。

非目标：重跑完整890发行/所有四仓腿、修Web源暂存/锁的D-906-1、修Man CLI的D-910-2、UI布局旧红、031主题实现/资源摘要、全部历史禁向边清零、ACE/闭包/A2R路线、细拆crate/改包owner或全机回归/远端发布。

## 2. 架构方案

沿现有`web_delivery.py`、`test_four_repository_fixed.py`、`modularity-audit.py`及`.autoos/module-boundaries*`有限修复，不引入新选测系统/动态消费者图/通用审计框架。

D-910-1修的是**语言侧收据消费者与控制门**：解析真实nextest日志ID并检查合法唯一执行集合，而不是接受summary计数。Web发行producer仍归Web；本包只读核其schema/日志格式，若现producer和冻结收据合同存在真实版本差额，给具名owner交接，未获明确合同修订不在Web端改发行算法/分母。

D-906-3分为三件事：a)标准库级执行设施合法消费如何精确编码；b)当前baseline旧锚缺失/陈旧如何有证重冻；c)其他边/ownership差额逐项分类。禁止先refresh再问架构是否合理，也禁止扩大整个lang-infra→man-ecosystem放行或新造有退出阶段的假grandfather。

## 3. 技术栈

Python标准库、现收据schema与nextest人读日志/原生ID映射、tomllib/Git、现模块声明边/ownership与self-test。未来定向验证以Python unittest和固定`cargo t web-delivery-controls`/`infra-fixed`为主；无默认Rust产品行为改动，Cargo metadata只作锁定来源一致性对照，不视为编译证据。

## 4. 需求分析与背景调查

### 4.1 授权、来源与基线

**实施授权（2026-11-03）**：用户消息「完成计划918」经 /auto-plan-goal 授予整链授权：work（含建唯一组树）、review、bounded修复、merge（canonical Specs、自动索引诊断非阻塞、archive、工作树清理、landing检查）。该授权不改变本合同的目标/验收阈值/兼容承诺/仓库与行为范围/预算；后续阶段继承，不再重复询问。§0"暂不实施"的旧限制被此授权取代。

已用auto-plan-new，读verification-contract；已读`docs/specs/overview.md`、[四仓交付](../specs/repository-boundaries/web-four-repository-delivery.md)、[布局/777审计](../specs/design/repository-layout.md)、[auto-cache](../specs/auto-cache/project.md)、[906独立边界](../specs/repository-boundaries/standalone-language-build.md)、[910生成生命周期](../specs/repository-boundaries/generated-cargo-build-lifecycle.md)，以及777/906/910归档与实际脚本/配置/清单。`docs/specs/auto-tool-execution/project.md`未找到，不杜撰现成Spec，以auto-cache的PLAN-896段和crate实际std-only清单为依据，SD-02补关系合同。

只读HEAD：lang `44a774998cebec68b2b0dd26a844b23a4f08e247`；UI `816b88015dd16a1863ad7ef49dbc03e18a9e7cd6`；Web `3de5524e28bba6eeb8ff2f7f7666676b58c874fa`；Man `47d8f7b34e584dccc90fefe340629bbe318c6405`；OS `65fc70de8b505657db660d91024b2cf4425b7759`。这是观察快照，不钉死未来执行；各仓docs/031实现WIP不消费、不合并或还原。

| 输入 | 本次SHA256 |
|---|---|
| scripts/test-governance/web_delivery.py | d4c573bece724aa8cda7173cc59960fb1ed43ae96566489cb6feaa7ad18d2104 |
| .autoos/module-boundaries.toml | c2eb60abc6fb8a36ffea4c31ff35ec17b01344bc2826350e2ec85f87b4408e44 |
| .autoos/module-boundaries-baseline.json | 4dccbb938ab81b37e1e1694814f1d1ba0772e8018666e1809dab0a3d511bc776 |

### 4.2 D-910-1实际剩余

`scripts/test-governance/web_delivery.py::validate_distribution_receipt`在约131行调用`execution_ids(raw)`，本文件无定义/import，当前scripts/test-governance内未找到该实现。新跑`independent-sdk-tests`与`reused_receipt`最终都汇到此分支，不仅是复用路径。`scripts/test_four_repository_fixed.py::FourOwnerControls.test_distribution_consumer_source_and_closure_controls`进入原`WebOwnershipTests.test_distribution_rejects_partial_missing_steps_changed_sources_and_unexecuted_commands`，其合法fixture即会到达缺失调用；本次只读定位，未重跑该测试，历史复现见D-910-1/910收据。

当前语言consumer契约包括217个执行、2个原skipped、45个distribution步骤，日志PASS行形如`PASS [ 0.1s] sdk-binary sdk_case_0`，复用收据schema=`web-independent-tests/1`并绑定SDK/config/log哈希。Web实际producer `scripts/web-repository-distribution.py`亦存在217/2及直接/复用两路径；`auto-web/test-http-e2e=[]`当前仍为Web本域空兼容feature，不得误当语言七feature残留清理。计数/清单是现合同，不在本包悄改。

缺失函数修复须保binary+test组成的ID，拒绝相同ID重复终态、缺/多/失败/未运行、summary自洽但实际ID不符；同test名不同binary是合法不同ID。直接路径日志还要校验row的hash，复用路径保前序收据/SDK/config/log身份。现fixture主要证明构造控制，不是Web产品真执行PASS；后续记录严格分开。

### 4.3 D-906-3实际剩余

- 当前`.autoos/module-boundaries.toml`把auto-cache归lang-infra、auto-tool-execution归man-ecosystem；`lang-infra-no-upward`禁止normal/build边。auto-cache清单确有该normal非optional path依赖，`methods_pack.rs`两处与`sandbox.rs`一处调用owned设施。
- [auto-cache规范PLAN-896段](../specs/auto-cache/project.md)明确允许std-only执行设施持有编译树、保原预算/参数/错误，真实依赖由896提交`956967923`加入。故不能靠删边、回退裸子进程、改optional或test-only“修违规”。真正上层Man/UI/VM边仍禁止。
- auto-tool-execution清单`[dependencies]`为空，是std-only设施；所属历史域不能代表可依赖整个Man生态。拟用§5.3的逐边设施准入，不改其包owner/归属。
- baseline不含auto-cache→auto-tool-execution；其config_sha256与当前配置相同；但base_revision=`9d96d0f7602aad96231b875d812b81683da04d10`在本机`git cat-file -e <rev>^{commit}`返回128，不可解析。`staleness_paths_changed`对此明确判stale。本次不推断对象丢失原因，不fetch/改历史恢复。
- 因旧锚不可解析，本轮不能声称已对旧提交做完整manifest diff。T-01须用保留baseline edge集合、当前实际manifest/ownership、已交付规范和来源逐项重建决策账；其他新增边未实查，不能把D-906-3收窄成“肯定只有2项红”。

### 4.4 T-01 实施冻结（2026-11-03）

- 输入哈希与 2026-11-02 合同快照逐字节一致：web_delivery.py `d4c573be…`、module-boundaries.toml `c2eb60ab…`、baseline `4dccbb93…`；HEAD 已推进（快照不钉死）。
- 唯一组建成：`D:/autostack/.wt/lang-918/auto-lang`（plan-918-dev，base 593c33480）+ auto-ui（detached fa01bd79）/auto-web（3de5524）/auto-man（47d8f7b）正式 HEAD，全部 porcelain 空；外仓 031 WIP 留在原主检出，不消费。
- 真实 producer 复核（auto-web `scripts/web-repository-distribution.py`）：直接腿 `cargo nextest run --manifest-path sdk/Cargo.toml -p auto-web-contract -p auto-web --lib --features auto-web/test-http-e2e … -j 1 --no-fail-fast`，与 consumer 共用同一 summary 正则（217 run/217 passed/2 skipped）；`independent-tests.json` schema `web-independent-tests/1` 只带计数+log/输入哈希，**不携带执行 ID 集**。
- nextest 人读日志格式按真实样本冻结：PASS/FAIL 行带 8 空格缩进、Summary 行 5 空格缩进，`PASS [ <dur>] ( n/m)? <binary> <test-full-name>`（样本 `docs/plans/reports/755-landed-independent-tests.txt`：`        PASS [   0.136s] (  1/106) auto-lang back_proxy_tests::http_e2e_…`）；非 TTY 重定向输出无 ANSI（producer 经 subprocess 捕获，其 summary 正则同样假设无 ANSI）；解析必须容前导空白（R1 复审 F-918-01/02 实证修正）。
- required 集合来源冻结：现声明 SDK targets（`-p auto-web-contract -p auto-web --lib --features auto-web/test-http-e2e`）的 nextest listing；当前 receipt 不携带该集合 → **producer/consumer 兼容差额入本合同**（§5.2/§10.6）：consumer 在现 schema 内完成唯一实际执行集、summary、源/配置/日志哈希的严格校验；ID 对 ID 级对账需 `web-independent-tests/1` 增加版本化 ID 集证明（Web schema 决定，本包未擅改）。
- 当前边集冻结：observe 66 边（ledger `plan-artifacts/plan-918/t01-edge-ledger.txt`）；strict 现状=2 违例（auto-cache→auto-tool-execution 新禁向边 + stale_baseline 旧锚不可解析），无其他新禁向边；ownership 覆盖 7 组全在、paths_missing 空；文本候选漂移（250→3 等）为警告不判红。
- 设施真实调用点核验：`crates/auto-cache/src/methods_pack.rs:230/314` + `sandbox.rs:725` 共 3 处消费 `auto_tool_execution::run_command_capture`（TreeScope::ProcessTree/ExitKind），与 PLAN-896 合同一致。

## 5. 详细设计

### 5.1 开工门和唯一实施组

预定`D:/autostack/.wt/lang-918/{auto-lang,auto-ui,auto-web,auto-man}`，语言分支`plan-918-dev`；外仓先固定公开正式来源，必要down同组补入，各repo仅一树，禁止链接。唯一Plan簿记在语言主检出，实现/内部核查/final review顺序共用权威组树，独立上下文不另建实现checkout。

实施门=用户明确批准本合同；不硬等031主题全链交付。T-01接受此时实际正式上游和固定外仓来源，登记所有审计扫描repo/hash/manifest/锁/ownership及工作目录，避免读到031 WIP；真实共享manifest/锁/source-owner基线变化按精确输入身份处理。未建树前本轮只读文件和Git对象，不运行门或刷新基线。

### 5.2 收据准入修复

在原consumer模块或已存在公共日志解析点增加单一`execution_ids`等价实现；T-01先核所有使用者/日志producer，避免另建全量parser。ID至少保binary与test全名；不能只`set(PASS)`丢掉重复信息，也不能按行数或summary创造执行。

支持当前正式nextest输出（按实际工具版本/ANSI控制字符/时长格式冻结），malformed/歧义/FAIL/重复/未终态不能被清洗成PASS。合法同名跨binary、参数化全名及原skip条件有控制。输出ID与独立冻结的required集合/原过滤条件对账，不能以同一坏日志生成自己的期望集合；现receipt未携带集合时，T-01沿现声明的SDK targets/有效listing确定来源，需增加版本化ID证明时先在本合同记录producer/consumer兼容差额，不擅改Web schema。

保217/2与45步现分母；如当前SDK实际清单已变化，必须Web owner给逐ID等价/新增/原条件映射和相应合同决定，不能把数字改成最新count消红。语言正反fixture即使全绿，也只证明consumer边界，不能证明外仓当前发行流程通过。T-02需覆盖直接和复用receipt，错hash/source/config/log拒绝后修复输入可再次接受；真实老临时证据已清理，不伪造回放旧原始文件。

**T-02 实施决议（2026-11-03）**：按 890 谱系既有公共解析点恢复 `execution_ids` 并加固——冻结 PASS 行 → 唯一 `binary\ttest` 身份；重复/FAIL/未终态/畸形行/ANSI 控制字符与 summary 不符均精确拒绝（不 set 去重、不按行数或 summary 创造执行）；合法同名跨 binary 与参数化全名受控。现 `web-independent-tests/1` 不携带 ID 集，consumer 内可验证的唯一实际执行集/计数/源哈希边界如上；ID 对 ID 级对账（"等计数缺原ID+加伪ID"）需版本化 ID 集证明，该 producer/consumer 兼容差额记入本合同 §10.6，归 Web schema 决定，本包不擅改 Web schema——V-02 在现 schema 内的可判子项（重复/FAIL/截断/伪 summary/ANSI/畸形/同名跨 binary 合法）全部落入新正反例（`scripts/tests/test_web_test_ownership.py`），全路径日志内容变异并入原 fixture 门。

### 5.3 精确设施准入与边界守卫

**拟准入恰一条已获规范支持的设施边**：请求包auto-cache、本仓`crates/auto-cache`→本仓`crates/auto-tool-execution`同一package identity，normal、非optional；该设施正常/build依赖闭包须仍为空/仅std（dev/bin测试条件如实分开），禁止借同名外部path或Man re-export替身。保owned退出/取消/reader合同，不改Rust算法。

优先在既有配置/审计内增加“逐边、带设施身份与std-only约束的合法规则”（新配置机制，T-01冻结精确字段和序列化）；判定输出说明命中的合法设施边，**不是有退出阶段的grandfather**。如果现规则可无新增schema等价精确表达，T-01给证明后采用，但不得扩大from/to域、把auto-cache重标顶层、把整个man-ecosystem改底层或给目录前缀豁免。

必须拒绝：auto-cache→auto-man-runtime或auto-man、lang-core/data→auto-tool-execution（原禁向保持）、同名设施的错误来源、auto-tool-execution新增正常/build第三方/语言/UI依赖、optional/build/target形态伪装、通配放行。至少一条真实合法控制和这些受影响反向/同域邻居负控进入原self-test。

### 5.4 基线重建与可复现落地

旧锚缺失与真实边变化分开报告，保旧baseline hash/记录值和“不可解析”事实，不修成伪造祖先。观察当前所有正常/build/dev/optional/target边、外置包、ownership与迁移路径；用现baseline记录比较只作集合差额，不伪称旧commit源码已经复现。对每项标规范支持的合法变化/已关闭旧边/真实违规/未决；未知不入合法冻结。

按014/900/906/007实际owner移动，只改真实陈旧ownership路径/域事实，保required组、code-fence/覆盖义务、原窄grandfather和revoked边；不能把缺失对象/路径删掉来省检查。若发现超本包的真实新违规，具名列出准确owner/输入，必要needs_replan/前置修复，不能刷新后声称strict通过。

先提交规则/必要路径修正，独立核边决策；然后在**干净已提交的本组来源树及干净参与仓**用原`--refresh-baseline`生成新锚。baseline文件落库自身不触陈旧是原规则，不能豁免后续manifest/配置变更。若用单次候选工作区暂存审计结果，最终必须在真实交付SHA再检锚/输入，不伪造HEAD。禁止`--no-cargo-check`结果代替最后锁定metadata一致性；该参数只可在明确纯声明负控中使用。

最终strict应可完整通过且无未解释新违规；合法设施由窄规则而非新baseline吞掉才能通过。在自有临时repo做新违规/缺组/宽豁免/脏树刷新/锚缺失等负控，保源码树不变。

### 5.5 与031、Web、Man的交接

| owner/合同 | 本包责任 | 保留在原owner的责任 |
|---|---|---|
| [UI031](../../../auto-ui/docs/plans/031-theme-scope-publication-and-package-closure.md) | 基线接受正式已合入manifest/来源；交互只按真实输入重冻。其实际generation-core/descriptor接线由031负责，不另造重复实现 | scope/原子发布/正式主题包/包摘要/003010验收，不等918完成；两包都不能借对方在途WIP合入 |
| Web D-906-1 | 核语言consumer与现producer格式的兼容，交付准确日志/ID/schema/来源校验约定和必要拒绝反例 | `web-install.py` vendor源暂存/锁、真实Web发行fresh/locked/升级/恢复执行；本包不修该门/扩大Web测试或将语言控制绿当producer全绿 |
| Man D-910-2 | 保Man作为legacy/current消费收据的身份和退出规则，读回兼容 | `auto build -d <dir> -r rust`错误派发及产品CLI回归；本包不改Man命令实现 |
| 本路线记录 | 当前与历史状态分层、真实owner/交接和具名未闭清单；D910-1/D906-3仅本包交付后有实证才标闭 | 不把031/UI033/OS053/ACE/AG/全部产品当新增前置，不清910缓存持有或自动跑批回归 |

本包允许先于031交付执行治理，baseline必须选择当时可用正式版本；031后来改变manifest/配置时strict按规则再次stale，由变更owner按同样流程更新，不能承诺一次基线永久不变。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/repository-boundaries/delivery-admission-and-boundary-policy.md（新） | 缺日志ID解析/规则语义差额→实际ID准入、逐边设施合法规则及不可解析锚恢复纪律 | 与Code/控制反例一致，不造索引 | AC-01..04 |
| SD-02 | modify | docs/specs/design/repository-layout.md、auto-cache/project.md（仅补实际设施边规则指针） | 777原边界+896 owned设施→明确合法边与上层禁向、干净树基线维护 | 保原owned寿命，不假称所有Man依赖合法 | AC-02/03 |
| SD-03 | modify | docs/specs/repository-boundaries/web-four-repository-delivery.md | receipt输入/旧控制→当前consumer实际ID、直接/复用证明与Web producer边界 | 控制PASS不等外仓执行PASS | AC-01/04/05 |
| SD-04 | modify | docs/specs/repository-boundaries/delivery-admission-and-boundary-policy.md（SD-01新页） | 无后继维护→031/各owner触发条件、来源锚/限制/路线指针 | 实际交付后沉淀；设计/Plan非现状实现 | AC-05..08 |

当前只proposal，不修改canonical/配置/基线或legacy `.autoos/specs.json`。边界baseline JSON是审计输入，未来只在T-04干净冻结树更新，不是旧Spec账本。

## 6. 测试设计

V全部未来工作，未执行；T-01使精确支持格式/fixture/基线控制可运行，不新建动态选测/通用runner。

| Case ID | AC / 输入与真实入口 | 可区分错误实现的关闭oracle / owner_stage |
|---|---|---|
| V-01 | AC-01；原FourOwnerControls命名测试→validate_distribution_receipt；直接217/2日志和复用收据合法控制 | 不再NameError，两个合法路径接受，schema/45步骤/来源/config/log哈希原规则仍逐项生效。work/T-01/02 |
| V-02 | AC-01；等计数缺原ID+加伪ID、重复PASS、同名跨binary、FAIL/缺terminal/summary假成功、ANSI/畸形行 | 原ID/真实唯一执行集合核对：错误精确拒绝、跨binary合法，不能由set去重或只summary冒绿；实际expected集合独立来源。work/T-02 |
| V-03 | AC-01/05；复用收据SDK/config/log/源身份错与恢复、直接row hash错 | 失败确定性拒绝，合法恢复后下一次读取接受，不能把修fixture当产品发行完成。work/T-02/05 |
| V-04 | AC-02；真实auto-cache→std-only facility edge/现normal调用；邻居/反向/替身/设施加依赖负控 | 合法精确边可见地按新规则接受，其余禁向仍红；不改Rust owned调用/预算和domain大范围。work/T-03 |
| V-05 | AC-03；旧锚不可解析、当前边集/owner路径/规范决策账，新干净已提交锚 | 缺失事实保留，新锚实际存在、祖先与来源hash/锁一致，strict真实绿；其他未知新增边不被重采样合法化。work/T-04 |
| V-06 | AC-02/03；baseline刷新脏树/跨仓脏态、宽豁免、缺required组、复活revoked/伪新合法边 | 原self-test与真实临时repo负控仍非零，新规则不扩大豁免；负控路径不修改权威冻结源。work/T-03/04 |
| V-07 | AC-03/04；相同HEAD+相同输入两次observe/strict、metadata --locked及交付后新manifest变化 | 固定payload稳定；锁不满足/新锚变更不静默写锁或跳过；真实改输入导致stale，恢复到固定来源复验绿。work/T-04/05 |
| V-08 | AC-04/05；Web/Man/031边界及当前/历史路线/未闭债读回 | 各能力/测试owner与状态准确，语言门正常不冒全Web发行/Man派发已修，不新加全线前置。work/T-05/06 |
| V-09 | AC-06；全部work证据、健康/遗漏/规则/日志敏感性与SD | independent final review同组绑定提交、原始反例可复算，执行者自查不代替，必需未知0。review/T-07 |
| V-10 | AC-07/08；冻结职责门、干净树真实合入、canonical与债务/全输出清理 | 新锚在落地来源可复现，分列本包PASS/外仓pending、archive/guard和实际scope清理完成。merge/T-08/09 |

可执行入口（未来）：`python -I -m unittest scripts.test_four_repository_fixed.FourOwnerControls.test_distribution_consumer_source_and_closure_controls`；`cargo t web-delivery-controls`；按原`WebOwnershipTests`测试入口完整控制。审计`python scripts/modularity-audit.py --self-test`、`--mode observe --output <scope>/observe.json`、`--mode strict --output <scope>/strict.json`；`--refresh-baseline`仅T-04已提交clean树，输出路径与实际baseline变更登记，绝不在本轮主检出运行。

`infra-fixed`是否包含相同原测试T-01按`scripts/test-groups.toml`去重选择完整职责，不能为了检查Python小改编全workspace。若不改Rust仅审计/准入，编译/auto-cache全单测不自动触发；如方案确需产品源码变更先修订触面并选完整职责组。Web产品发行/Man CLI/031实际构建只在其owner明确合同需要时执行，不为本包合入跑全四仓/全ACE/AG。批量回归已到期的user-pending独立保留，本包不擅自重跑或把用户豁免记PASS。

### 6.1 实跑记录（2026-11-03，work 冻结）

| Case | 结果与证据 |
|---|---|
| V-01 | `cargo t web-delivery-controls` 13/13 PASS（原 12/13+`NameError: execution_ids`；run-ycs0vt3k / R1 修复 run-4g5bl9tz）；直接（独立-sdk-tests 带 command/log）与复用（reused_receipt）两路径合法收据通过，schema/45 步/来源/config/log 哈希原规则逐项仍生效；R1 修复后正式缩进格式正例（8 空格 PASS + 5 空格 Summary）在单元层与真实 755 样本核验通过（105 缩进 PASS 可解析、含 FAIL 的失败运行正确拒绝） |
| V-02 | `scripts/tests/test_web_test_ownership.py` 新增 `test_sdk_execution_ids_precise_admission`：217 唯一合法、冻结 (n/m) 进度形态、正式缩进形态、同名跨 binary 合法（sdk-binary/sdk-binary-2 各共享 sdk_case_* 仍 217 唯一）；重复/FAIL/伪 summary/缺 terminal/ANSI/畸形/空日志/缩进 FAIL 清洗/非法 UTF-8 精确拒绝；全路径日志内容变异 5 例并入原 fixture 门（duplicate/failed/fake-summary/ansi/malformed 均 ValueError） |
| V-03 | 复用收据 SDK/config/log 身份错与恢复沿用原 fixture（`reused SDK execution source/config identity differs` 等）；直接 row log_sha256 错→拒绝（原 mutations 保留）；修复输入后可再次接受（原 fixture 末段改写后恢复） |
| V-04 | 真实 auto-cache→auto-tool-execution 边在 strict 输出具名 admitted（`std-only-facility-consumption`）；self-test v13 五负控全红（邻居 fixture-tool-exec-2、闭包破坏→facility_std_only_closure_broken、optional→facility_form_masked、build→new_forbidden_edge、外置替身→new_forbidden_edge+不被准入）；原禁向/邻居 0 误放；Rust owned 调用/预算零改动（3 调用点核实） |
| V-05 | 旧锚不可解析事实保留；新锚 base_revision b41c83931 实际存在、config_sha256 绑定、66 边与当前 manifest 一致；strict 真实绿（EXIT 0，violations 0）；v13 证明未知新增边不被重采样合法化 |
| V-06 | self-test v04d/v04b/v09（宽豁免/revoked/缺组）+ 本次实证：主树脏刷新拒绝、跨仓 auto-ui tracked 脏刷新拒绝（HEAD fa01bd79 点名）、恢复后重刷成功；v13 证明新规则不扩大豁免 |
| V-07 | 相同 HEAD+相同输入双 observe payload 字节一致（DA0C0A46）；metadata --locked 66/66 无差异；提交 manifest 变更→stale_baseline 判红（点名文件）→revert→复绿；脏树不静默写锁/跳过 |
| V-08 | 债务台账读回：D-910-1/D-906-3 实证闭合（b41c83931），D-906-1/D-910-2/031/OS/批回归 user-pending 全部保原 owner；语言门绿不冒 Web 发行/Man 派发/031 交付（§10.5/§10.6） |

开发/修复定向反控和必要职责；review通过后merge冻结同配置职责并集，身份未变的有效证据按输入复用，不多轮重复全目录。明确选择/执行/原条件排除数，0选择不PASS。未执行/首次红/环境重放/外部阻塞分列。

## 7. 验收标准

| ID | 可观察验收 / 验证 | owner_stage |
|---|---|---|
| AC-01 | D-910-1真实consumer定义完整，直接/复用合法收据通过；实际ID/分母/来源/哈希与拒绝/恢复反控有效，无count/set/summary绕行，V-01..03 | work |
| AC-02 | D-906-3设施边按896规范精确准入，上层/替身/反向/闭包变化仍拒绝，无扩大域/弱化owned/宽grandfather，V-04/06 | work |
| AC-03 | 基线旧锚缺失诚实记录、所有差额逐项有据，新锚clean提交可解析/可复现，strict完整绿且原守卫敏感，V-05..07 | work |
| AC-04 | consumer/审计真实职责门稳定，031/Web/Man来源边界无新增反向依赖或重复实现，外仓未执行不冒PASS，V-07/08 | work |
| AC-05 | 当前路线/战略/Design53/具名债务和交接准确；D910-1/D906-3只按实际结果闭合，其余owner/真实未闭保持，V-03/08 | work |
| AC-06 | 独立final review逐AC/完整通路/反例/健康/遗漏/workaround/SD pass，verdict绑定实现/输入，V-09 | review |
| AC-07 | 冻结门/新锚落地读回/真实合入/canonical与交付状态沉淀，V-10 | merge |
| AC-08 | archive/全输出路径/材料/树/分支guard清理与持有登记真实完成，不清其他会话或未知缓存，V-10 | merge |

## 8. 执行步骤

| ID | 实际触面/关闭结果 | 依赖 / AC | owner_stage |
|---|---|---|---|
| T-01 | [x] 授权后建唯一组；冻结真实consumer/producer日志/required集合、baseline旧锚/全边/owner差额、精确规则/测试/外仓来源/预算；新差额超scope先needs_replan —— **完成 2026-11-03**：组建成（lang-918 四仓，外仓 detached 正式 HEAD）；输入哈希与合同一致；producer/consumer 差额入 §5.2/§10.6；66 边 ledger 冻结（§4.4） | 用户明确实施授权；全部work AC前置 | work |
| T-02 | [x] 原web_delivery消费路径补日志ID准入/直接复用完整控制；保持schema分母和source/config/hash规则，补真实判别正反例 —— **完成 2026-11-03**：`execution_ids` 恢复并加固（5a40437a5）；web-delivery-controls 13/13 PASS（原 12/13+NameError）；test_web_test_ownership 2/2（新增单元级+全路径日志内容变异 V-02 反例）；直接/复用两路径均达真实校验 | T-01；AC-01，V-01..03 | work |
| T-03 | [x] module-boundaries配置/审计有限设施准入及std-only身份约束，保其余禁向/ownership/原自测，不改Rust执行设施 —— **完成 2026-11-03**：`[[rules.facility_admission]]` 逐边机制+配置（9ec98e0ad）；self-test 16/16（新增 v13：合法控制+邻居/闭包破坏/optional 伪装/build 形态/外置替身五负控）；真实仓设施边具名 admitted；Rust 执行设施零改动 | T-01；AC-02，V-04/06 | work |
| T-04 | [x] 审全边决策账/owner路径、提交规则后clean树重冻baseline，新锚/strict/metadata/稳定与陈旧负控 —— **完成 2026-11-03**：新锚 87b02e2ea→re-anchor 15ff91953（base_revision b41c83931，66 边，config 绑定）；strict 全绿+设施边 admitted；metadata --locked 66/66 无差异；双 observe payload 字节一致（DA0C0A46）；提交 manifest 变更→stale 判红→revert→复绿；主树脏/跨仓(auto-ui)脏态刷新拒绝实证 | T-02/03；AC-03，V-05..07 | work |
| T-05 | [x] 完整受影响consumer/self-test职责与精确来源并集，Web/Man/031交接、回滚和外仓pending分列 —— **完成 2026-11-03**：`cargo t web-delivery-controls` 13/13、modularity-audit --self-test 16/16、test_web_test_ownership 2/2、py_compile 全过、diff 无 dbg/TODO；债务台账闭合 D-910-1/D-906-3（b41c83931），D-906-1/D-910-2/031/OS 保 owner；回滚=revert 交付提交即回 NameError/旧锚态（历史哈希在案） | T-04；AC-01..04，V-01..08 | work |
| T-06 | [x] 健康/遗漏/全部work矩阵、SD及路线/债务最终读回，execution_done6/9 —— **完成 2026-11-03**：work 矩阵与 SD proposal 已折（§4.4/§5.2/§6.1/§9），路线/债务读回见 KNOWN-DEBT-AND-RISKS.md 交付记录；无未闭 work 项、无阻塞问题 | T-05；AC-05 | work |
| T-07 | [x] 同组冻结树独立final review，finding回work/失效相关证据 —— **完成 2026-11-03**：独立审查代理终审 needs_fix（F-918-01/02 真实缩进格式缺陷 + F-918-03/04/06）→ R1 修复 cff7e6182/4a2dd38da → 同一代理 R1 复审 **pass**（独立重放 5 项全过、门全绿、AC-01..06 全过）；收据入 §9 | T-06；AC-06，V-09 | review |
| T-08 | [x] 冻结职责门及新锚/锁/配置身份、真实合入、canonical/路线/具名债务实证闭合；不代Web/Man/031发布 —— **完成 2026-11-03**：SD-01..04 canonical 落地（ca9bcafb9）；交付提交门 `cargo t web-delivery-controls` 13/13（run-647qr5w5）+ strict EXIT0 violations0 + metadata --locked 66/66；ff-only 落地 master **ca9bcafb9**（10 提交线性，零 merge commit）；canonical 路径/hash 核验（delivery-admission-and-boundary-policy.md BFCBDD8E…、baseline 6D4E2B8F…）；债务台账闭合随落合 | T-07 pass；AC-07，V-10 | merge |
| T-09 | [x] 摘要折Plan/全实际输出路径对账删统一scope，所有树先wt-guard clean后移除分支/空组，归archive9/9 —— **完成 2026-11-03**：guard clean 实证；scope `plan-artifacts/plan-918` 整删（含 review/facility/tiny 探针与 SD 草稿，无具名 hold）；树/分支/空组移除；.retention/policy.json 刷新 | T-08；AC-08，V-10 | merge |

材料唯一`D:/autostack/plan-artifacts/plan-918/`按repo身份分区，创建登记owner/用途/在途；报告/fixture/日志/基线候选/外部副本均在此，不写attachments或其他盘散落根。正常target在唯一组，外置仅统一scope。merge必要命令/退出码/数字/边决策摘要/节选/hash折本Plan和canonical后删整scope与自有树产物；仅具名未闭持有owner/理由/关闭条件，不清未知/他方目录/活动进程，不回收910历史持有。

## 9. 复审记录

2026-11-02 / stage:new / PLAN-918 r1 / outcome:blocked（仅当前暂不实施/未给执行授权） / 只读核D910-1两条收据路径均到缺失函数；D906-3既有896 std-only调用合同与旧域禁向冲突、baseline旧base对象不可解析、config哈希未变；未把历史“2违例”冒当前完整审计。静态预检pass：0–10章节、9任务/8AC/10V/4SD唯一连续、Plan/Design53/路线v21与战略v21当前段本地引用有效，限定diff检查通过；918跨活动/归档唯一、号源单调。已立有限合同并刷新路线源，未改Python/Rust/清单/审计配置/基线、未建树/跑门/构建/清缓存。next：用户明确实施后work/T-01冻结精确输入/集合/规则/全部差额；超范围真实违规先重规划。

2026-11-03 / stage:work / PLAN-918 r1 / outcome:pass / worktree_path: D:/autostack/.wt/lang-918/{auto-lang,auto-ui,auto-web,auto-man} / branch: plan-918-dev + 三外仓 detached（fa01bd79/3de5524/47d8f7b 正式 HEAD） / base_commit: lang 593c33480 / code_commit: lang 5a40437a5(web_delivery)→9ec98e0ad(facility)→87b02e2ea(基线)→df26f7bf8/75a651514(陈旧负控 probe/revert)→b41c83931(债务台账)→15ff91953(re-anchor 基线) → 当前 HEAD 待 T-06 收尾提交 / dependency_revisions: 外仓固定正式 HEAD；工具链 rustc/cargo 1.98.0 / task_ids: T-01..T-06 完成（6/9），T-07 review、T-08/09 merge 待 / evidence: §4.4 冻结记录；§5.2 实施决议；raw scope `D:/autostack/plan-artifacts/plan-918/`（t01-observe/strict/edge-ledger、t03-strict、t04-strict-pass/stab1/stab2/stale-neg/repass、t06-strict-final）；`cargo t web-delivery-controls` 13/13 PASS（原 12/13+NameError，run-ycs0vt3k）；modularity-audit --self-test 16/16；test_web_test_ownership 2/2；py_compile 全过；diff 无 dbg/TODO；strict 全绿+设施边 admitted、metadata --locked 66/66、双 observe 字节一致（DA0C0A46）、陈旧/脏树/跨仓脏态负控实证；债务台账闭合 D-910-1/D-906-3 / blockers: 无 / next_responsible_role: review（T-07，同组冻结树独立上下文，核 AC-06/V-09：逐 AC/全通路/反例/健康/遗漏/workaround/SD 稿与绑定提交）

2026-11-03 / stage:goal / plan_id: PLAN-918 / plan_revision: r1 / round: 1 / from: drafting / to: execution_done / status: execution_done / outcome: pass / worktree: D:/autostack/.wt/lang-918/auto-lang / commit: 15ff91953 / next: review（T-07，同组冻结树独立上下文）

2026-11-03 / stage:review / review_scope: final / reviewer_identity/limitations: 独立审查代理（general-purpose，独立上下文，只读+定向重放） / plan_id: PLAN-918 / plan_revision: r1 / outcome: needs_fix / worktree_path: D:/autostack/.wt/lang-918/auto-lang / branch: plan-918-dev / reviewed_commit: 15ff91953 / base_commit: 593c33480 / dependency_revisions: 外仓 detached fa01bd79/3de5524/47d8f7b；rustc/cargo 1.98.0 / spec_inputs: SD-01..04 稿（Plan §5） / acceptance_results: AC-01 FAIL（F-918-01/02：真实缩进 nextest 输出被拒、缩进 FAIL 被清洗）；AC-02 PASS（独立重放 11/11）；AC-03 PASS（旧锚 9d96d0f7 exit128 保留、新锚 b41c83931 可解析、config 绑定 75d76e91、66 边、双 observe 05ce4c3a 字节一致、脏树/跨仓守卫核实）；AC-04 PASS；AC-05 PARTIAL（D-910-1 闭合绑定合成格式证据）；AC-06 本审 needs_fix / pending_closeout_gates: R1 修复后重跑 cargo t web-delivery-controls + test_web_test_ownership（需真实缩进格式正例）；merge 前修正 SD-01/SD-03 稿格式转录 / findings: F-918-01 high（^PASS/^FAIL 锚定不含前导空白，真实 8 空格缩进日志拒绝，0/105 匹配）；F-918-02 medium（缩进 FAIL 静默忽略可清洗为 PASS，同根因）；F-918-03 low（load_config kinds 校验含 build/dev 与"只准 normal"合同不符）；F-918-04 low（SD-01/SD-03 与 §4.4 格式转录缺前导空白）；F-918-05/06/07 info（(N slow) summary 双方共享脆弱性归 Web note；U+FFFD decode 边；执行者双 observe 哈希取于中间脏 HEAD，终 HEAD 可复现性成立）/ evidence: 门 web-delivery-controls 13/13、self-test 16/16、strict EXIT0 violations0 admitted 具名、py_compile -W error 0、双 observe 05ce4c3a；反例包 plan-artifacts/plan-918/review/{facility-replays.py(11/11),tiny/run.log,obs1/obs2.json,strict-review.json,review-report.md} / next: work（R1 有界修复：PASS/终态行容前导空白并补真实格式正反例，重验后回 review）

2026-11-03 / stage:work / plan_id: PLAN-918 / plan_revision: r1 / outcome: pass（R1 修复轮） / worktree_path: D:/autostack/.wt/lang-918/auto-lang / branch: plan-918-dev / code_commit: cff7e6182 / base_commit: 593c33480 / task_ids: T-02（F-918-01/02/06）、T-03（F-918-03）、T-06（F-918-04 转录修正） / evidence: `_PASS_LINE`/`_TERMINAL_LINE` 恢复 `^\s*` 前导空白锚定（890 谱系公共解析点），拒 U+FFFD；单元测试补正式缩进正例+缩进 FAIL 清洗负例+非法 UTF-8 负例；全路径变异块不变；v13 补非 normal kinds 配置拒绝；真实 755 样本 105 缩进 PASS 可解析、含 FAIL 失败运行正确拒绝、217 缩进正例 accepted；门重跑：web-delivery-controls 13/13（run-4g5bl9tz）、test_web_test_ownership 2/2、self-test 16/16、py_compile 0 / blockers: 无 / next_responsible_role: review（同组冻结树 R1 复审：重放 F-918-01/02 真实格式正反例，确认 SD-01/SD-03 转录已修）

stage: review | review_scope: final_r1 | reviewer_identity/limitations: 独立审查代理（general-purpose，独立上下文，只读+定向重放） | plan_id: PLAN-918 | plan_revision: r1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-918/auto-lang | branch: plan-918-dev | reviewed_commit: 4a2dd38da | base_commit: 593c33480 | repair_commits: cff7e6182 + 4a2dd38da（15ff91953 之上；树 porcelain 干净核验） | dependency_revisions: 外仓 detached fa01bd79/3de5524/47d8f7b；rustc/cargo 1.98.0 | spec_inputs: SD-01..04 稿（Plan §5，转录已修） | acceptance_results: F-918-01 FIXED（`^\s*` 锚定：真实格式 217 正例 ACCEPT n=217；完整 755 样本在缩进 FAIL 行正确 REJECT）；F-918-02 FIXED（混合日志缩进 FAIL 不再被清洗，REJECT）；F-918-06 FIXED（非法 UTF-8 行 REJECT）；F-918-03 FIXED（直接探针：kinds ["build","dev"]/["normal","build"] ConfigError，缺省 normal 接受）；F-918-04 FIXED（§4.4/V-01/SD-01/SD-03 转录含 8/5 空格缩进，债务台账 D-910-1 绑定真实格式证据）；AC-01..05 现全部 PASS；F-918-05/07 保留 info 不阻塞 | pending_closeout_gates: merge（T-08）落地 SD-01..04 与台账读回；无残留 work 项 | findings: 本修复轮未引入新 finding（diff 仅锚定/UTF-8/kinds 校验+测试/文档）；F-918-05 info（(N slow) summary 双方共享严格性→Web owner note 交接）、F-918-07 info（DA0C0A46 为中间脏 HEAD 证据，终 HEAD 可复现性成立） | evidence: 审查者独立重跑：cargo t web-delivery-controls PASS 13/13（run-ozirslvs，非执行者 run-4g5bl9tz）；test_web_test_ownership 2/2；modularity-audit --self-test 16/16（v13 含 kinds-invalid）；真实树 strict EXIT0 verdict pass violations0；py_compile 0；F-918-01/02/06 五重放（755 样本/真实格式 217/缩进 FAIL/非法 UTF-8/未缩进对照）全部符合预期；kinds-probe.py 4/4；报告 review/review-report.md §7 | next: merge（T-08，冻结门+新锚落地读回+SD 合入）

2026-11-03 / stage:merge / plan_id: PLAN-918:r1 / outcome: pass / checkpoint: prepared+landed / evidence: 冻结门在交付提交 ca9bcafb9 实跑：`cargo t web-delivery-controls` 13/13 PASS（run-647qr5w5）、`modularity-audit.py --root . --mode strict` EXIT0 violations0 admitted 具名、metadata --locked 66/66（cross_check ok 沿用）；guard（wt-guard.sh）clean；rebase 于 master（base==master 593c33480，up to date 零改写）；主检出 `git merge --ff-only plan-918-dev` → **master tip ca9bcafb9**，10 提交线性、零 merge commit / checkpoint: canonical_verified / evidence: 落地提交含 SD-01..04：`docs/specs/repository-boundaries/delivery-admission-and-boundary-policy.md`（新，SHA256 BFCBDD8ECAD555689C640CB473767C63DCA968CB86C63EB4DE067A4210CD6102）、repository-layout.md / auto-cache/project.md / web-four-repository-delivery.md 增量；三仓链接解析核验；无 legacy ledger 写入；canonical 与债务台账（KNOWN-DEBT-AND-RISKS.md D-910-1/D-906-3 闭合、D-906-1/D-910-2 保 owner）随落合 / checkpoint: archived / evidence: 本文移入 docs/plans/archive/918-cross-repository-admission-and-boundary-closure.md，status: archived，9/9 任务核对，docs 提交 / checkpoint: cleaned / evidence: plan-artifacts/plan-918 整 scope 删除（无具名 hold），tree/branch/空组移除，.retention/policy.json 刷新 / next: done

2026-11-03 / stage:goal / plan_id: PLAN-918 / plan_revision: r1 / round: 3 / from: reviewed / to: archived / status: archived / outcome: pass / worktree: （已移除） / commit: ca9bcafb9 / next: done

stage: review | review_scope: final | reviewer_identity/limitations: 独立审查代理（general-purpose，独立上下文，只读+定向重放） | plan_id: PLAN-918 | plan_revision: r1 | outcome: needs_fix | worktree_path: D:/autostack/.wt/lang-918/auto-lang | branch: plan-918-dev | reviewed_commit: 15ff91953 | base_commit: 593c33480 | dependency_revisions: 外仓 detached fa01bd79/3de5524/47d8f7b；rustc/cargo 1.98.0 | spec_inputs: SD-01..04 稿（Plan §5） | acceptance_results: AC-01 FAIL（execution_ids 锚定 ^PASS，无法解析真实 nextest 缩进格式，直接/复用真实收据被拒；门绿仅基于未缩进合成日志）；AC-02 PASS（独立重放 11/11：合法具名准入/optional→facility_form_masked/dev 诚实/闭包→facility_std_only_closure_broken/邻居→new_forbidden_edge）；AC-03 PASS（旧锚 9d96d0f7 exit128 保留、新锚 b41c83931 可解析、config 绑定、66 边、strict 全绿、双 observe 字节一致、脏树/跨仓刷新守卫在代码核验）；AC-04 PASS（无 Rust/无 specs.json/无 Web schema 改动，外仓 clean）；AC-05 PARTIAL（D-910-1 台账闭合绑定于合成格式证据）；AC-06 needs_fix | pending_closeout_gates: 修复 F-918-01/02 后重跑 cargo t web-delivery-controls + test_web_test_ownership（需真实缩进格式正例），再合入；merge 前修正 SD-01/SD-03 稿格式转录 | findings: F-918-01 high：_PASS_LINE/_TERMINAL_LINE 锚定 ^ 不匹配真实 nextest 非 TTY 输出（PASS 行前 8 空格、Summary 前 5 空格；证据：755 样本被拒 0/105、live nextest 0.9.138 抓取 review/tiny/run.log、全仓 6/6 实日志、producer 原样捕获）→ 直接/复用真实收据 ValueError；F-918-02 medium：缩进 FAIL 行被静默忽略（合成混合日志中可被清洗为 PASS，反例复算）；F-918-03 low：load_config 允许 facility kinds normal/build/dev 与"只准 normal"合同不符；F-918-04 low：SD-01/SD-03 稿转录省略前导空白；F-918-05/06/07 info（(N slow) summary 严格性、U+FFFD 解码、DA0C0A46 为中间脏态证据） | evidence: 门：web-delivery-controls 13/13 PASS、self-test 16/16、strict EXIT0 violations0 admitted 具名、py_compile -W error 0、双 observe 05ce4c3a 字节一致；反例包：review/facility-replays.py（11/11）、review/tiny/run.log、review/obs1/obs2.json、review/strict-review.json；完整报告 review/review-report.md | next: work（同组 bounded 修复：PASS/终态行容前导空白并补真实格式正反例，重验后回 review）

## 10. 待澄清事项

1. 实施授权已由用户「完成计划918」经 /auto-plan-goal 授予（2026-11-03，§4.1）；拟准入为逐边std-only设施而非域重分类，已按合同实施；公开规则/范围扩大需给精确差额。
2. 当前217/2、45步是语言consumer与Webproducer现合同，实际required集合来源/日志版本T-01已核；未按今日数量改分母；真实producer变更时先与Web owner具名合同接续。
3. baseline旧锚不可解析事实已保留（旧SHA256 4dccbb93…与 git 对象 exit128 记录在案）；新锚已在干净提交树重冻、可解析可复现；strict 全绿且未知新增边未随刷新合法化（v13 负控证明）。
4. 031可独立继续，不等918；它改受管manifest的正式交付后基线自然会stale，由实际变更owner按此规则接续，不做永久豁免。
5. Web D906-1、Man D910-2、UI旧红、真实OS事件和批回归user-pending均保原owner，不因语言准入/路线刷新宣称全部产品通过。
6. **producer/consumer 兼容差额（T-02 实证入账）**：现 `web-independent-tests/1` 收据只携带计数+输入/日志哈希，不携带执行 ID 集；语言 consumer 已完成现 schema 内可判的全部校验（唯一实际执行集、summary、源/配置/日志哈希、重复/FAIL/截断/伪 summary/ANSI/畸形拒绝、同名跨 binary 合法）。"等计数缺原ID+加伪ID"的 ID 对 ID 级对账需要 Web producer 在收据 schema 增加版本化 ID 集/有效 listing 证明——归 Web schema 决定（具名 owner 交接），本包未擅改 Web schema，也不把 consumer 门绿当 Web 发行全绿。
