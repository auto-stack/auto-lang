---
plan_id: PLAN-1000
status: archived
feature_name: M1剩余执行证据、恢复预算与最终签章收口
author: [Codex]
created_at: 2026-11-09
updated_at: 2026-11-09
plan_revision: 1
current_step: 12
total_steps: 15
depends_on: [PLAN-941, PLAN-999, PLAN-998]
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-expanded-acceptance.md, docs/specs/auto-test-case/ace-m1-conformance.md, docs/specs/auto-test-case/ace-m1-applicability.md, docs/specs/auto-test-case/architecture.md, docs/specs/auto-test-case/profile-v2-native.md, docs/specs/auto-test-case/ace-ca-d-bridge-certification.md]
new_spec_components: []
touched_goals: [GOAL-002]
affects: [auto-test-case, auto-ac, auto-av, auto-a2r, auto-a2x, auto-abi, auto-semantics, auto-cte, auto-syntax]
---

# [PLAN-1000] M1剩余执行证据、恢复预算与最终签章收口

## 0. 变更摘要

承接已归档941/999的正式交付，在原M1范围内关闭**276项未观测与6个未闭验收域**，使最终扩大M1通过全部真实required门。复用941的registry、严格reader、组合器、八域AND、固定组和既有worker；补实际执行、来源映射、恢复与策略采样，必要时做本合同内有证据的有界修复，新增正式签章入口。本包终验必须真正满足M1，**不能复用941“系统交付而M1未绿亦可归档”的完成例外**。

本轮仅规划：drafting r1、0/15；不施工/构建/产品测试，不创建worktree、材料scope、goal、agent或其他会话，不重开/修改archived941、999。当前只写本Plan和中央取号簿记；canonical Specs仅列拟议增量，merge阶段才沉淀。

## 1. 目标

1. 从原1006键/31份合同重建当前执行闭包，逐键认领941终面730P/0F/276U，含legacy未观测211项与其余65项；保旧730项的实际语义和身份，不以总数相等代逐项关闭。
2. 六域`legacy_required_execution`、`c_core_and_resource`、`q4_oracle_certification`、`syntax_architecture`、`precise_incremental_restore`、`budgets_time`全部真实闭合，汇总`evidence_incomplete`随完整证据清零；旧门/登记有效两域不回退。
3. 运行/拒绝/资源/恢复/所有权/预算证据由真实producer、父裁判与独立oracle给出；观察适配器不能从expected造事实、不能以多个后端同错算通过。
4. 完成独立复审后产生绑定冻结代码、依赖、工具与原始事实的正式报告；主线落地身份仍有效，`can_claim_expanded_m1=true`且blockers为空，才允许报告M1收口。

非目标：完整FFI、新版标准库、文件IO扩面、M2语言/产品迁移、旧引擎退役、自举、全仓格式治理、无关性能重构。旧required313、131行/524格、NA134、accepted-diff32、future45、原source/expected/独立reference、目标身份、原样本/配置/资源/时间阈值保持。新HIR-A2R不替旧A2R；typed不能冒普通Auto源码；scene/trap/value认证分轴。

未观测不预判为功能缺陷，也不承诺只跑测试即可闭合。环境缺失、合法受控拒绝与infra失败分别记账；已有required不因缺工具/慢/当前不支持改NA、skip或accepted。

## 2. 架构方案

数据流：跟踪原源/批准适用性/合同 → 正式固定组与worker/parent实际执行和计量 → 完整artifact-copy/终态/独立reference与证书 → 941各严格reader → registered逐键观测 + ARCH/恢复/预算测量 → 原八域AND → 候选审计 → 独立revision-bound复审凭据 → 正式签章报告 → 主线来源读回。

只沿既有生产者扩充事实与严格消费，不新造编译器、类型推导、store、runner或第二meter。组合器仍只读、不启动工具；执行协调调用已有固定组/父执行器。binding/proof不等于执行；声明/locators/历史PASS不供应当前PASS。

正式依赖：D941=`1671a3f02266ef64416cb6c50d4e029bbbff759c`（归档003fa8816）、D999=`6d8ec265f`（含99f43e6fe受审实现）、D998=`fff9eedf0`及其收尾修复。均为当前main祖先且已归档；T-01重核实际D、规范和工件身份。938/939等沿941已交付闭包消费，不重开历史Plan。M2草案不构成本包前置。

执行与存储owner：work阶段的执行session/Job持进程与输出，收集完成后按hash交只读reader；运行授权从当前请求注入，恢复不带旧权限。旧根/在途调用/外部pin由真实最后owner释放；失败/取消/迟到/坏页不能覆盖已发布根。保留16类原预算及组合器48MiB活数据帽、4MiB单文档帽；不靠扩大上限解决采集成本。

## 3. 技术栈

现有Rust ACE workspace、Auto/Atom/TestCase语料、Python严格reader、PowerShell旧核算入口、`cargo t <固定组>`及现Job/meter/证书工具。首期平台沿已认证Windows x64；其他平台只按原适用条件，不新增支持跨积。实施树拟`D:/autostack/.wt/lang-1000/auto-lang`，分支拟`codex/plan-1000-dev`；本轮未创建，后续同一树顺序work→review→merge，不建第二review树或junction。

## 4. 需求分析与背景调查

授权：用户2026-11-09要求“正式新建……承接276项未观测和6个未闭验收域，明确阶段合同、源码矩阵、实施顺序与验收条件。本轮只规划，不施工，不重开941和999，不启动其他会话。”无额外工期/模型/代理预算，不推定本轮实施授权。

取材：`docs/specs/overview.md`为导航；当前模块规范为主，读[扩大M1交付规范](../specs/auto-test-case/ace-m1-expanded-acceptance.md)、[原M1语料](../specs/auto-test-case/ace-m1-conformance.md)、[适用性](../specs/auto-test-case/ace-m1-applicability.md)、[裁判架构](../specs/auto-test-case/architecture.md)、[v2原生入口](../specs/auto-test-case/profile-v2-native.md)、[场景oracle](../specs/auto-test-case/ace-m1-scenario-oracles.md)；读归档941 §9第122–124轮、999 §5.6/§9交付与独立复审。实际基线HEAD=`003fa8816f3d10d185247dc74cba216a1b3b9d77`。归档不等于其报告中的未观测已闭。

| 已观察事实 | 当前证据/局限 | 本包处理 |
|---|---|---|
| 941终面1006/730P/0F/276U；legacy313仅102项被其严格执行链观测，余211 | canonical扩大验收规范及941第122–124轮；不是新跑结果 | T-01重建逐键清单，T-03闭旧执行证据 |
| 999八红、F-941-13、bare缓存已交；旧38 cert-gap=0 | 941已同步D998/999，466原检查全绿；999修复/变异/独立review和交付记录可追溯 | 保持这些事实，不再立重复修复；当前身份回归另核 |
| 恢复8行中6PASS/2PARTIAL，预算6行5PASS/1PARTIAL、策略14/16 | 941终面记录；被删scope不再是当前可读证据 | 从现代码重新测，不恢复已清理目录/伪造旧证据 |
| 恢复`ROWS`仍含`939 opaque resource/callback pages (awaiting D)` | `scripts/test-governance/ace_m1_incremental_restore.py`的固定partial_missing；939已交 | 接已交939真实恢复/坏页证据后再移除占位，禁止仅删字符串翻绿 |
| CLI仅`--first-slice-preview`，输出后固定exit1 | `scripts/test-governance/ace_m1_arch_acceptance.py`末尾；preview仍不得签 | T-10新增显式正式模式，旧preview兼容/不可签保持 |
| 声明保UNOBSERVED和执行观测分离 | `ace_m1_arch_obligations.py`/`ace_m1_budget_contracts.py`的发现合同刻意如此 | 不把发现文档改PASS；通过真实运行reader填执行事实 |
| 原始scope已随941/999合并清除 | 当前只剩计划摘要/Spec及跟踪源码，不拥有历史1006逐键原始JSON | 不宣称本轮拿到了历史276明细；T-01以当前同身份或已说明差异的真实重建闭合全1006，不伪造历史逐键diff |
| D-950-1债簿标题还写“关闭待独立review” | `docs/plans/KNOWN-DEBT-AND-RISKS.md`；999 final review及主线已交 | T-12核正式receipt后更新簿记；不是新功能债 |

### 编号与工具兼容

主检出`.next-id=1000`，active/archive/旧archived最大有效编号999。按仓库AGENTS指定`scripts/new-plan.sh`分配完整1000，绝不截断为100/000；脚本支持任意十进制并已推进1001。只读核auto-musk `backend/crates/musk/src/plans.rs`：`from_content`取前三字节seq，扫描要求第4字节为`-`，因此当前看板会漏1000/误读seq。**这是实际应用发现/自动入口兼容阻塞**，本轮不改外仓代码、不另起工具修复计划。以本文件为正式仓库合同；未来如走应用自动发现，须先由该工具owner修兼容并实测，不能把缺计划列表误判成没立项。直接文件工作流可读完整ID；T-01先确认实际入口，不用截号/别名绕过。

### 具体源码矩阵（现存锚点；拟增另标）

| 面/阶段owner | 输入→producer→consumer；实际文件与符号 | 计划改面与关闭观测 |
|---|---|---|
| T-01/02，宇宙与分类 | `crates/auto-lang/test/cases/ace-m1-expanded/registry.json`、`legacy-required-map.json`、`q4-source-contracts.tsv`、`case-obligations.tsv`；`scripts/test-governance/ace_m1_registry.py::registered_keys/compose_registered_inventory` | 全键、required/oracle身份、U原因、唯一主owner/多域影响；不改集合 |
| T-03，旧211执行 | `scripts/verify-ace-m1-852.ps1`、`scripts/test-governance/ace_m1_legacy_execution.py::execution_summary`及scalar/typed准入；`ace_m1_legacy_base.py`、`ace_m1_legacy_data.py`；原`crates/auto-lang/test/cases/ace-m1`与941 `legacy-coverage.suite.ad` | 补所缺target/path的真实adapter/readers/fixtures，313严格执行observed=passed=313；旧claim不能代执行 |
| T-04，核心与资源 | `scripts/test-governance/ace_m1_original_data_sources.py::admit_source`、`ace_m1_data_diagnostics.py`、`ace_m1_scalar_q4_evidence.py`；`crates/auto-lang-core/src/test_cases/ac_protocol.rs`、`abi.rs`；`experimental/ace/crates/av-core`、`native`、`foreign-contract` | 从真实阶段/trace/故障/资源生命周期接观察；不得从expected或typed片段填普通源结果 |
| T-05，Q4/认证 | `ace_m1_original_data_sources.py::certification_coverage/admit_certifications`、`ace_m1_cert_evidence.py`、`ace_m1_daily_evidence.py`、`ace_m1_source_supplement.py`、`ace_m1_query_evidence.py`（均在`scripts/test-governance/`）；`crates/auto-lang-core/src/test_cases/ac_a2r_hir.rs`；`experimental/ace/crates/a2x-worker` | 补缺slot、独立reference和原debug/release/优化配置；四端生成/实际执行/日常receipt分账 |
| T-06，SYN/ARCH | `scripts/test-governance/ace_m1_arch_obligations.py::REQUIRED/reduce_architecture`、`ace_m1_syntax_arch.py`、`ace_m1_ct_mechanisms.py`、`ace_m1_abi_evidence.py`；`architecture-obligations.json`；现source-query/source-cte/av/native/abi真实owner | 49个ARCH-01..08逐row原oracle+负控+真实动作，生命周期/同PID热更新/最后owner不可用计数或重启冒充 |
| T-07，恢复 | `scripts/test-governance/ace_m1_incremental_restore.py::ROWS/admit_incremental_restore_measurements`；`experimental/ace/crates/abi/tests/cases/resource_recovery.rs`的`abi_resource939_missing_or_corrupt_required_pages_refuse_restore_never_cold_start`及首错修复测试、`source_foreign.rs`；`source-query/tests/restore_identity_941.rs`、`instances_restore_controls_935.rs`；`source-cte/tests/architecture_restore_941.rs` | 934/935/936/938/939页族全覆盖，own/foreign owner、缺坏页、首编辑/失败修复/再恢复，8行最终8PASS |
| T-08，预算 | `scripts/test-governance/ace_m1_budget_contracts.py::POLICIES`、`ace_m1_budget_time.py::admit_policy_runtime_samples/admit_budget_time_measurements`、`native_meter.py`；`original-budget-contracts.json`、`scripts/test-groups.toml`、真实父Job输出 | 16策略逐项实测与cleanup/终态；CPU/峰值/墙时/样本原口径，组合6行全PASS |
| T-09/10/11，总门 | `scripts/test-governance/ace_m1_first_slice.py::compose_first_slice`、`ace_m1_arch_acceptance.py::reduce_acceptance`/CLI、`ace_m1_registry.py`；`scripts/tests/test_ace_m1_arch_acceptance.py` | 定向有界修复/反控，新增正式模式及review输入边；不改变八域AND和证据完整性要求 |
| T-12..15，交付 | canonical上列Spec、债簿、原Case identity exporter/固定组、`D:/autostack/wt-guard.sh` | 逐源归属/SD/复审冻结与landed身份；计划折摘要、全输出路径对账清理 |

`ace_m1_*`短名统一指`scripts/test-governance/`目录；上述目录是真实已核入口，不等于其中任意行为已获验证。永久新增fixture先沿现owner测试文件；必要新source/check/测试文件由T-02/T-03..10创建并注册既有职责组，名称/路径在对应任务冻结，不冒现有文件。

## 5. 详细设计

### 5.1 276项的分诊、责任与守恒

本包认领276项全量义务，不只盘点后交无主队列。T-01生成临时逐键表（唯一scope内），每行含：`registered_key | original_contract/hash | target/path/producer | approved_oracle | baseline_state | cause_class | primary_task | affected_gates | actual_entry | V/AC | tool/config/deps identity | current_result | close_evidence`。cause_class限未执行/未接观察/证据失效/reader不兼容/实测功能缺陷/环境缺失；没有实测依据不得将前四类写成产品缺陷。

| 子集/交接 | 已知边界 | 关闭owner与条件 |
|---|---|---|
| U-LEG | 历史211键，格式`legacy/{ns}:{id}:{path}`；从313 required减原观测集 | T-03主责、T-01冻结精确键；313全部真实严格执行，旧证书/金样/场景维持原义 |
| U-REST | 历史其余65键；不得猜定Q4/ARCH各自数量 | T-01/02先落实每个键；按实际源域归T-04/05/06/07/08，其中复审专属行归T-13 |
| 原730PASS | 与新增关闭同一1006集合，不是可忽略的存量 | T-09/11按实际依赖刷新/复用；任何回归仍阻全包 |
| 跨域/共享一条证据 | 一个键可影响多个域，但只有一个执行主owner | T-01保存多对多映射；禁止重复计数或用某域全绿推另一域 |

941原scope已删，不虚构历史276逐键明细；T-01须从当前同源合同及真实执行重建全1006，并解释与941终面数量/身份的差异。当前HEAD与交付输入只要有变化就逐文件定位；“历史分组数量不可精确还原”不能免除当前全部required的实测。不够信息则T-01保持未完成，不能用估算表交下游。

### 5.2 六域阶段合同与正式终验

| 域/合同 | 输入→交付结果 | 通过条件 | AC/V/任务 |
|---|---|---|---|
| D1 `legacy_required_execution` | 原313批准格、source/typed/scenario各自oracle→严格执行observations | required=observed=passed=313、failed/unobserved为空；合法原负例按原阶段/码判断PASS；cert-gap维持0 | AC-02，V-02/03；T-03 |
| D2 `c_core_and_resource` | scalar/core、原data与诊断真实运行→值/效果/资源观察 | reducer所需全部section非空且逐行PASS，跨域资源/失败保旧/重试无遗漏；错trace可杀反控 | AC-03，V-04/05；T-04 |
| D3 `q4_oracle_certification` | 逐原源四端+原配置actual/reference+日常receipt→完整slots | 所有required slots真实两腿正确、源/工具/flags/输出一致；emit-only/四端同错/旧证据不闭 | AC-04，V-06/07；T-05 |
| D4 `syntax_architecture` | SYN及ARCH-01..08原row的真实操作/计量/资源→49row+SYN结果 | 全部行通过；其中`ARCH-08/fresh-union-spec-review`只能由T-13独立审计供应，不能work自报 | AC-05/10，V-08/09/15；T-06/13 |
| D5 `precise_incremental_restore` | 现恢复测试+普通source/各页族端口→8条测量及ct_restore | 8/8 PASS、0 PARTIAL/U；正确fresh/hit/restore/首错修复/再次恢复；missing/foreign页不冷启动冒修复 | AC-06，V-10/11；T-07 |
| D6 `budgets_time` | 16原策略/窗口及原规模负载→真实meter及6测量行 | 16/16 observed、六行全PASS、原各策略窗口/资源/清理达标；原开发P90≤2000ms及更严门不变 | AC-07，V-12/13；T-08 |
| 聚合与签章 | D1..6 + 已绿legacy_current/expanded_registry + accounting/evidence/system→原reducer | 全1006required PASS、0F/0U（集合保持）、八域全true、cert_gap/awaiting_D/missing_inputs/blockers均0、evidence_complete与system_delivered=true | AC-09/10/11，V-14/15/16；T-10/11/13/14 |

### 5.3 阶段顺序与退出门

| 阶段 | 任务 | 进入/交付/退出门 |
|---|---|---|
| S0 交付基线与全表 | T-01 | 正式依赖已交；重建并冻结精确全集/276责任与16策略表，选择可读工具入口，不能靠删除缺行继续 |
| S1 最长路径有界前探 | T-02 | 一个未观测legacy格、一个Q4缺slot、939恢复、一个策略窗及一条ARCH观察串真实producer→strict reader→reducer；每个配合法/非法近邻；决定真实接口后再铺开 |
| S2 完整执行证据 | T-03/04/05 | 按既有入口补D1/2/3，先形成稳定原始事实/reader；各批分别值/效果/资源与oracle闭合，不能等末尾首跑 |
| S3 恢复/预算/ARCH | T-07/08→T-06 | 稳定事实后补D5/6，ARCH使用前两阶段真实结果并补同PID/last-owner等专属观察；域边界分别闭 |
| S4 有界修复与正式入口 | T-09/10 | 每项真实red闭合/变异验证，正式模式与旧preview隔离；独立复审输入的格式/身份先具备，缺review保持不可签 |
| S5 冻结候选与交接 | T-11/12 | 所有work-owned义务/反控真实完成，只有明确review-owned签章行可pending；候选不能宣称M1，交付Spec候选/身份/健康与债簿事实 |
| S6 独立审计、签章与落地 | T-13→T-14→T-15 | 审计其余全部工作后给revision-bound凭据，组合全绿并独立核正式报告；按冻结并集与落地主线证据归档/清理，最终可签才完成 |

可按已冻结私有接口准备不同域，但本轮和未来都不自动派发会话/代理；同Plan只有一棵权威树。共享registry、公共H、固定组和父协议单写，正式采样与压力单实例。T-06不得要求完整M2或未来产品迁移；若原M1某义务实测缺能力，按5.5处置而非改原义务。

### 5.4 恢复、资源与预算的具体补面

- `all-required-pages`：接938/939正式D及真实worker恢复输出，至少覆盖SQ facts、实例、CT/frozen、scalar foreign、opaque handle/callback/ctx各合同要求的页；缺/坏页、错误owner/provider generation/签名、撤授权应在规定阶段拒绝并保旧根；合法页恢复及下一次真实首编辑成功。只有这些证据与永久反控齐后才移除939等待占位。
- `historical-cross-project-ownership`：核源码/模块/package/CT owner与foreign provider/generation范围；跨项目同形页拒绝，自己合法页仍可用。`source-query/tests/restore_identity_941.rs`、CT owner测试和939 recovery当前产物是入口，不拿typed恢复证明普通source路径。
- 增量：保持真实parse/bind/verify/lower/encode/store生产点及IO计数；未改函数P/M/R/IO原合同为0，不能整编后过滤计数。签名/layout/meta/实际读依赖改动应失效，body-only不产生假签名事件；保999的修复和反控。
- 生命周期：成功、编译拒绝、预算耗尽、cancel/late、发布失败、close及last-owner各按原支持域取区分例；在途链和外部pin合法保活，最后owner后回收收敛，sentinel不动。丢失root引用或进程已返回不等于资源已释放/子进程已回收。
- 16策略逐行表必须由T-01列具体name/owner/limits、真实命令、采样source与所缺窗口；历史14/16不保证当前只差2条采样，也不能猜两条名称。保每策略既有样本/ignore/平台/资源限制；原`MIN_SAMPLES=10`下限不代替既有30样本/原规模等更强要求。P90升序第N-floor(N/10)项，开发门≤2000ms，单窗预算/max也独立核。prepare/冷编译、发现/执行/清理/整命令分列，原CPU/Job峰值、输出/IPC/磁盘轴保持。

### 5.5 修复边界与失败状态

本合同覆盖严格reader/来源映射/正式入口/必要最小观测补面，以及在原M1支持域内被上述反例证明的有界正确性、恢复与计量修复；T-09必须列输入、Spec、修前结果、真实原因、兄弟路径、永久反例/合法对照和当前结果。不得借机造新架构、加语言范围或放宽原资源/性能/compat阈值。

若修复需要架构/范围/预算改变或单项超过可独立评审规模，先修订本Plan或具名后继并逐AC迁移；本包终验责任和分母保留。未闭能力/环境项使本包仍executing，不能以“已登记债务/后继已立项”标execution_done/reviewed/archived；不可像941带7 blocker完成。旧941/999只读引用，不接管其已清理树。

### 5.6 正式模式与独立复审无环合同

- 拟在现`ace_m1_arch_acceptance.py`增加显式`--acceptance`（**新接口，T-10实施**），沿`--repo/--evidence/--bindings`原来源语义；确需review凭据输入的具体字段/schema由T-02/T-10冻结并版本化，不静默重解释旧报告。原`--first-slice-preview`继续preview/不可签/exit1；正式模式exit0仅全八域/证据/复审身份闭合，未闭exit1且不可签，坏身份/格式exit2具名拒绝。
- T-11仅核候选：除`ARCH-08/fresh-union-spec-review`及其汇总影响外，所有work-owned required和补充反控完成。单独的review-owned行留pending，`can_claim_expanded_m1=false`保持；这不是延期work义务。T-12把所有候选SD/来源/健康/原始证据交给同树独立复审。
- T-13先独立核冻结候选代码H、所有其他required原始事实和Spec delta，给**签前审计凭据**（bound H、原证据hash集合、适用域与已核AC，无未来merge完成主张）；只据这份实际审计凭据完成唯一review-owned行。然后重新组合并核最终报告/负控，再给final review结论。
- 凭据不引用最终报告自身hash，最终报告引用凭据并绑定H，消除自指；不得写一份“final pass”空摘要让自己翻绿。缺凭据、错H/工具/输入、未审域或真实reader红仍不可签。实现暂停写入；发现缺陷回同树work并使受影响凭据/采样失效，复审再核。
- M1对外完成以T-14主线来源验证为准；候选已可签不提前冒主线交付。合并后代码/grammar/协议/producer变化需按依赖图重采，不能仅换report中的commit字符串；只有等价docs后代才能具名复用。实现固定组绿色/旧can_claim_m1=True不能代新扩大总门。

### 规范增量

本轮不写canonical Specs。review按实际支持和冻结身份定稿，merge才落地；历史941终面保留为交付时真实快照，不倒填成当时已全绿。

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-test-case/ace-m1-expanded-acceptance.md` | 941系统交付/730P276U→追加本包真实最终域/逐键/来源/签章事实；校准registry实际/2和原文/1差异 | M1终验单一事实 | AC-01/05/09/10/11 |
| SD-02 | modify | `docs/specs/auto-test-case/ace-m1-conformance.md` | 旧核算/认证轴与分片执行→313完整strict执行来源、坏轴/缺项拒绝；不改required适用性 | accounting与execution分离 | AC-02/04/08 |
| SD-03 | modify | `docs/specs/auto-test-case/ace-m1-applicability.md` | 原批准集不变，补本包守恒/关闭来源及future不冒PASS | 保原source/NA/accepted/future | AC-01/02/09 |
| SD-04 | modify | `docs/specs/auto-test-case/architecture.md` | 有限reader/typed错误面→实际新增strict观察与parent拒绝保真、正式入口/签前审计身份、声明不代运行 | 多consumer统一判决 | AC-03/04/05/08/10 |
| SD-05 | modify | `docs/specs/auto-test-case/profile-v2-native.md` | 原预算/样本/恢复合同→补实测通过范围、16策略计量来源和不可观测边界；阈值保持 | 不冒P90/资源通过 | AC-06/07/08 |
| SD-06 | modify | `docs/specs/auto-test-case/ace-ca-d-bridge-certification.md` | store seam/有限恢复→本包实际观测source/typed/resource/callback恢复与owner边界，未观测轴仍不冒认证 | ABI实际证据 | AC-03/05/06 |

## 6. 测试设计

全部为未来执行设计；本轮未运行产品/认证/压力/性能，也未执行独立实施复审。每项V由对应work任务创建/接通永久fixture后执行，既有输入直接沿原路径；实际命令参数T-01/02冻结。缺工具/零选择/skip/timeout/截断/父exit0均不等于PASS。值、错误/阶段、输出、资源trace与producer/输入身份分别断言；四端相等不是唯一oracle。

| Case | AC / invariant | 代表输入/操作及真实入口 | 区分性结果与反控 | owner/coverage |
|---|---|---|---|---|
| V-01 | AC-01，全表守恒 | registry/原合同→`registered_keys`及当前完整执行重建 | 276责任逐键有唯一主owner、1006全键与313集合一致；删除/重复键或把未观测改NA必须拒；不能伪造历史行 | T-01/02；全M1集合 |
| V-02 | AC-02，旧执行非核算 | 原corpus及scenario各真实adapter→legacy strict reader | 313都观测PASS；删一个真实终态/把旧脚本green313当执行清单不得闭，a2r金样/认证仍分轴 | T-03；原313 |
| V-03 | AC-02/08，负例正确阶段 | 原诊断/trap/scene，合法近邻与相同exit的infra失败 | 原批准语言负例按typed码/阶段通过；rustc失败/空steps/错oracle/复制别格receipt均拒 | T-03/09；原负例，不发明expected |
| V-04 | AC-03，Core/资源全路径 | 999五源八格及original-data/scalar source，经AV/native实际worker/parent | 3/3/1/1/3/3/300/1等原oracle保持；source/typed不可互替，资源meter/诊断错误应被反控杀掉 | T-04；已批准域 |
| V-05 | AC-03/05，资源与原子性 | 原fault sweep/borrow/move/close/callback，失败/重试/取消 | 写回/析构次数与原oracle一致、失败旧根不变、合法下一请求恢复；early free/晚发布/丢外pin均失败 | T-04/06/09；真实支持生命周期 |
| V-06 | AC-04，独立oracle与原配置 | 各族原Auto源码→AV/AC执行，新HIR Rust/C actual/reference两腿、原flags及daily | 每slot完整；两端同错、偷偷去-O/换toolchain、错源/case/config、emit-only拒；原正确对照仍过 | T-05；全部required slots |
| V-07 | AC-04/08，覆盖而非空总计 | `certification_coverage`/`compose_registered_inventory`与固定组真实报告 | 缺一认证slot、零executed、条件排除冒PASS、重复证据或混旧/新target必须阻签 | T-05/09；所有缺slot族 |
| V-08 | AC-05，ARCH真实producer | ARCH-01..08逐row原operation/entry，真实parse/bind/实例/CT/ABI/热点 | 每row closing_oracle与negative_control均实证；全组PASS/locator/预置declared不代49row执行 | T-06；49行及原SYN |
| V-09 | AC-05/08，同PID/最后owner | 现resident/av-resident原常驻入口，旧调用链未退时热更、外部pin后close | 一PID一entry真实safepoint，新旧代际各正确；最后pin释放前活/后回收收敛；重启/暂停恢复或计数归零伪装失败 | T-06；原在线更新与原churn规模 |
| V-10 | AC-06，必需页 | `resource_recovery.rs`已有939坏页/首错修复与SQ/实例/CT/938实际恢复 | 合法恢复首好编辑成功；缺坏/foreign owner/generation/签名/撤权按原typed拒、旧root保留，无静默冷启动；939占位移除有运行证据 | T-07；全部required页族 |
| V-11 | AC-06/08，精确失效与持久修复 | body/签名/layout/meta变更、坏首编辑→修复→再次恢复；跨项目同形合法对照 | 原P/M/R/IO与零生产断言满足，依赖实变才失效；失效不假命中，坏页修复后的下一次真实读回成功 | T-07/09；既有计量/恢复边界 |
| V-12 | AC-07，16策略窗口 | 原POLICIES各真实运行、原负载/样本→同一Job/meter | 16/16、P90/峰值/单窗上限/准备与清理分列；缺任策略、n不足、伪summary、无真正cleanup终态都不闭 | T-08；原16策略与更强原规模 |
| V-13 | AC-07/08，预算与坏样本 | 合法原边界/超界、48MiB组合活数据、损坏meter/异常进程终态 | 合法通过、超界resource且保旧/清理；伪0meter/截断/迟到/缩样本/重置峰值应拒；无第二meter绕帽 | T-08/09；原预算 |
| V-14 | AC-08/09，正式与preview隔离 | T-10拟新`--acceptance`/原preview，真实候选/缺项或坏绑定 | preview永不可签/exit1；正式缺任一工作证据仍false/exit1，坏输入exit2；孤立全绿合成控制只测敏感性，绝不作生产签章 | T-10/11；CLI及AND/所有consumer |
| V-15 | AC-10，独立审计与无自指 | 冻结H与原始事实→T-13签前审计→正式报告→final review | 凭据绑定H/输入/工具且不自引最终hash；删/换凭据、未核域、不同H或签章自证必须阻签；完整真实集合才全1006P/0F/0U/八域真 | T-10接口、T-13实际审计；review-owned最后行 |
| V-16 | AC-09/11，冻结落地 | T-11候选→T-13正式→T-14 landed，改一个真实producer/依赖作失效对照 | 有效身份全绿可读回；漂移不能换commit字段充复用；原941/999保持归档，主线正式总门/来源有效才签M1，不冒M2 | T-11/13/14；最终交付 |

### 固定职责组与真实命令

以下组名从当前`scripts/test-groups.toml`静态核验，**并未运行**。实施按实际改面冻结必要并集；不能全部堆成每次迭代全仓，也不漏真实共享consumer：

- 治理/reader/父协议：`cargo t infra-ace-m1-arch infra-ace-m1 infra-ace-policy infra-ace-cost-report auto-test-case ace-adapters`。
- 旧义务：`cargo t case-ace-m1-base case-ace-m1-legacy-941 case-legacy-a2r-resources-998 case-legacy-a2r-trap-997`；T-01按313逐键找到其余已存在fixed groups（不是这四组自动覆盖313），缺执行接线由T-03注册/补齐。
- 原源/Q4：`cargo t case-ace-m1-q4-941 case-ace-m1-scalar-q4-941 case-ace-m1-text-q4-941 case-ace-m1-record-text-q4-941 case-ace-m1-collections-q4-941 case-ace-m1-source-supplement-941 case-ace-m1-original-data-q4-941 case-ace-m1-original-data-diagnostics-941 case-ace-m1-residual-999`；reference实际认证/daily/优化窗另用原工具实际命令，不以这些组代cert两腿。
- 共享/恢复完整组：`cargo t ace-source ace-syntax ace-semantics ace-comptime ace-artifacts ace-av ace-native ace-a2x ace-abi ace-foreign-contract`，按改面和原row职责取必要并集；D938/939真实source/scenario另核 `cargo t case-ace-ca-d-source-938 case-ace-ca-d-resources-939`。
- 正式计量：原性能/压力入口用`cargo t ace-performance`及其ignore/原适用条件；原warm/restore/30样本门若并不被该组选择，T-01必须记实际既有专项命令并执行，不能宣称组名覆盖。长测试沿原T3窗并错峰，首次红/环境重放/条件排除都保留，不改时限强杀凑绿。
- 旧核算原`powershell -File scripts/verify-ace-m1-852.ps1 -Scope acceptance`为accounting/旧claim输入，不等扩大执行闭合。当前preview可用`python scripts/test-governance/ace_m1_arch_acceptance.py --first-slice-preview --repo <唯一树> --evidence <scope> --bindings <相对binding>`，预期exit1且不能签；新正式入口由T-10实施后才可运行，绝不把本条当已有命令或已执行证据。

列发现/实际执行/原条件排除数、prepare/冷编译/发现/运行/清理/总时、子命令真实终态与身份。未改身份/输入的既有有效证据可具名复用；941/999原scope已删的原始证据不能靠路径或hash存在于计划摘要就视为当前已重读。周期`cargo t all`仅由主检出due规则单实例负责，非每计划强制全仓。

角色：用户未指派模型/外部reviewer/并行代理。本包未来work/内查顺序在同树，final review必须独立上下文逐AC/V/SD，声明实际独立性；不自动启动另一会话或agent。S6的签前独立审计是有实际结果的review动作，不是work自查换标签。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 当前全1006键、历史276责任与313 required/批准集完整守恒；原因/主owner/跨域AC/V/真实入口齐，不伪造历史逐键明细 | T-01/02，V-01，全键差异与有界决定 |
| AC-02 | work | D1旧执行313全部被严格运行证据观测且通过，0遗漏/重复/错误归属；cert-gap=0维持 | T-03，V-02/03，原source/typed/scenario各按原义 |
| AC-03 | work | D2 Core/数据/诊断/资源全部section真运行全PASS，无expected回填、资源/错误伪绿 | T-04，V-04/05 |
| AC-04 | work | D3全部required源码/目标/原配置认证slot及日常receipt闭合，独立oracle各自正确 | T-05，V-06/07 |
| AC-05 | work | D4全部work-owned SYN/ARCH行closing oracle与反控成立，观察真实；唯一独立审计行明确留给AC-10 | T-06，V-08/09；不要求未来merge先完成 |
| AC-06 | work | D5八行8PASS、ct_restore全过，必需页/跨owner/首错误修复闭合，939占位有真实关闭证据 | T-07，V-10/11 |
| AC-07 | work | D6六行全PASS且16策略全实测，原样本/规模/P90/单窗/资源/清理阈值均达标 | T-08，V-12/13 |
| AC-08 | work | 相关真实缺陷有原反例/变异/兄弟合法控制与永久回归；反控敏感；新增warning/fmt/debug问题零未处置，既有不相关债显式分账 | T-09/10/11，V-03/05/07/09/11/13/14 |
| AC-09 | work | 正式入口与旧preview隔离、严格AND不降格；冻结候选全部work-owned义务完成，review专属行之外无U/F/PARTIAL；精确失效清单、SD候选/逐域证据可独立复审 | T-10/11/12，V-14/16 |
| AC-10 | review | 独立核AC-01..09/全部V/SD、原始事实及遗漏；实际签前审计完成唯一review行后，全1006P/0F/0U、八域true、所有完整性true、gap/awaiting/缺输入/blockers=0，正式报告可签且无自指 | T-13，V-15/16；不采信执行摘要 |
| AC-11 | merge | 复审pass后冻结必要职责并集/实际漂移复验，canonical来源与主线正式报告一致，M1可签；归档/全输出scope与唯一树清理完成，旧941/999未重开 | T-14/15，V-16；修复正确性不留到merge |

## 8. 执行步骤

后续须获得实施指令才进入work；本轮没有实施授权。拟唯一树与分支见§3。开工先读本Plan最新revision、当前AGENTS及Specs，确认base/HEAD、共享符号写入窗和唯一材料scope。

| task_id | owner_stage | 前置 | 源码/动作及阶段交付 | 验收/用例 | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令、D941/999/998可读；实际文件入口或已修应用兼容 | 核正式SHA/1006与313集合/31合同/所有tools；重建276逐键责任与16策略表；解释相对941身份漂移、删scope后的重测来源；登记scope和命令/完整组；应用三位工具不可误用 | AC-01，V-01 | [x] §9 r1（1006键零漂移/31合同/16策略/逐键映射 scope t01/；276责任按当前重执行路线取代伪造历史行） |
| T-02 | work | T-01 | §5.3 S1最长路径有界先证+正反例；在原reader/worker下冻结观察格式、采样/恢复和正式review输入；失败needs_replan而非窄化义务 | AC-01/08/09，V-01/05/07/10/12/14 | [x] §9 r1（legacy链+Q4 slot+组合器正负控；恢复/预算/ARCH面归 T-07/08/06 首个真实producer时同法先证） |
| T-03 | work | T-02 | legacy_execution/base/data及所需原parent接线，按精确211缺项补原域运行与严格receipt；313逐键通过，永久缺终态/错格反控 | AC-02，V-02/03 | [x] |
| T-04 | work | T-02；消费T-03已稳定事实 | 原data/scalar/诊断资源reader及必要真实producer观察；全Core/资源正反例、原8格不退，补资源故障/生命周期测量 | AC-03，V-04/05 | [x] |
| T-05 | work | T-02；原源码与T-04稳定接口 | Q4 actual/reference/原flags/source SHA/日常receipt全slots；原source与typed身份分别证明，遗漏一项阻总门 | AC-04，V-06/07 | [x] |
| T-06 | work | T-03、T-04、T-05、T-07、T-08 | 49 ARCH及SYN逐行接真实producer/计量/last-owner/同PID观察；审计专属row明确pending归T-13，不能工作阶段造凭据 | AC-05，V-08/09 | [x] |
| T-07 | work | T-02；D938/939/934/935/936正式接口 | 必需页与owner/generation/授权全族恢复正反、失败保旧→修复→再恢复；真实证据替939占位，8/8恢复行 | AC-06，V-10/11 | [x] |
| T-08 | work | T-01策略表；T-02；原始执行窗口可用 | 原16策略全样本/压力/资源/prepare/cleanup测量，冻结原policy/runtime绑定，补原P90/单窗/48MiB；不削样本或预算 | AC-07，V-12/13 | [x] |
| T-09 | work | 各域实测反例；最终须T-03..08 | 按§5.5逐缺陷有界修复/变异/合法近邻，沿实际触面跑完整职责/必要消费者；复测源/事实/计量/恢复及反控，形成闭合表与健康记录 | AC-08，V-03/05/07/09/11/13 | [x] |
| T-10 | work | T-02接口决定；各域稳定；T-09 | 实施正式准入/审计凭据消费和缺证拒绝，旧preview不变；永久错身份/无凭据/坏字段/缺行/伪全绿反控，不改八域AND | AC-08/09，V-14/15接口 | [x] |
| T-11 | work | T-03..10全部工作能力齐；冻结候选 | 以冻结提交跑必要完整职责/实际认证与原样本并集，重新全键组合；除唯一review-owned行及汇总影响外全通过，候选仍不可签，不准把别的U挪成review-owned | AC-01..09，V-01..14/16 | [x] |
| T-12 | work | T-11 | 逐域/逐键关闭表、全部source/producer/tool/config/deps失效清单、SD-01..06/健康/债簿候选；核D-950-1已受审交付后更新簿记；冻结供T-13独立输入，不写941/999 | AC-09，V-16 | [x] |
| T-13 | review | T-01..12完成，停实现写入，review-owned行未冒签 | 同树独立上下文重建其余全required/AC/V/SD及遗漏；签前凭据→完成独立行→新鲜正式组合→核全1006/八域/零blocker；失败回work使受影响证据失效 | AC-10，V-15/16 | [ ] |
| T-14 | merge | T-13 final pass、授权合入及主线窗口 | 冻结必要并集/漂移复验、SD规范读回与真实交付来源核；主线正式报告仍全绿可签，再归档本Plan（不重开941/999） | AC-11，V-16 | [ ] |
| T-15 | merge | T-14；正式landed身份与规范来源已核 | 每条命令实际输出根对账/摘要折Plan；scope整删或具名hold、刷新retention；wt-guard clean后删树/分支/空组，最后确认正式签章与清理状态 | AC-11，V-16 | [ ] |

真实顺序：T-01→02→03/04/05；T-07/08可据稳定接口准备，正式采样错峰→T-06→T-09→10→11→12→13→14→15。T-09是每批发现问题时的修复owner，可提前迭代；最终退出必须等所有域，不循环等最终认证。独立复审的唯一待审行不阻work→review交接，但阻正式签章；review/merge/cleanup始终不能提前勾选。

材料唯一目录拟`D:/autostack/plan-artifacts/plan-1000/`，在途时登记owner/用途/路径；本轮未创建。全采样、binding、probe、报告/签前凭据、日志、外部副本/显式外置target均在此；正常target在唯一树。执行后摘要按命令/退出/关键数/逐义务结果/内容hash折入本Plan，merge按政策删除scope和树。长期产品fixtures放真实owner测试目录，禁止plan材料进attachments/仓内新证据目录或第二外置SDK根；不能保留整个scope只为未来重读，未来核验按最新代码重新测。

## 9. 复审记录

2026-11-09 stage:new / PLAN-1000 r1 / status:drafting / current_step:0/15。用户仅授权正式规划；未施工/建实施树/运行产品或认证/派agent/新会话/重开941999。实现复审、合入、Spec沉淀、签章和清理均未发生。

2026-11-10 stage:goal/work / PLAN-1000 r1 / round:1 / 用户 `/goal 完成计划1000` 授权全链（work→review→bounded repair→merge）。开工事实：实施树 `D:/autostack/.wt/lang-1000/auto-lang` @分支 `codex/plan-1000-dev`（基面 master@003fa8816，new-wt-group 一键建组含 auto-down 兄弟@58ea33f）；材料scope `D:/autostack/plan-artifacts/plan-1000/` 已登记 `.retention/policy.json`（in-flight）。主检出现存WIP仅docs/plans+docs/design簿记类，无代码路径WIP。

**T-01 完成（AC-01 静态面，V-01 部分）**：依赖SHA核验 D941=1671a3f02…/D999=6d8ec265f…/D998=fff9eedf0… 全为 HEAD 祖先；`registered_keys` 重建 1006 键=legacy313+q4-source624+architecture49+preview20，键集SHA256(LF排序)=`3f412190d5c1b4f64013df43964845beb3da6396e0262480a16bf632976e6537` 与规划期一致（**零合同漂移**）；31合同活hash全过；16策略静态枚举（fixed-case-suite…a2x-collections-formal，见 scope t01/policies.json）；冻结台账确认 313 全 `state:green` 各带 producer token（24机制族），即每格有可执行 producer，逐键→固定组静态映射在 scope t01/cell-group-map.json（206 未接线格中 125 有现成套件引用、81 为 scenario/inline-source 需 T-03 逐族核）。941 终面 730P 的历史逐键明细已随 scope 删除不可读，按合同以当前真实重执行重建全 1006（非伪造历史行）。

**T-02 S1 前探完成（正/负控齐）**：①旧核算 `verify-ace-m1-852.ps1 -Scope acceptance` exit0、can_claim_m1=True、CORE 285/285+EXT 28/28（accounting≠执行）。②四套件一 invocation 批跑（`AUTO_CASE_WIRE_EVIDENCE=1`，worker sha 跨 invocation 重链漂移已定位并以单invocation批跑规避）：legacy-941 4/4、q4-941 4/4、929 syntax 11/11、937 bridge 6/6 全 PASS；四份 identity-inventory 与 registry.json 冻结合同逐值相等。③Q4 认证双腿真实执行（a2x-worker 92a6d31cbe…@003fa8816）：rust certify exit0/838ms、c certify exit0/2.8s + identity/daily 受管捕获 receipts（scope t02/q4-cert/）。④组合器 `--first-slice-preview` 实际合成 exit1（合法preview退出）：**首链 20/20 PASS**、legacy 严格执行 4/313 观测（B08/B11×av/ac）、accounting_valid=true、required_universe_closed=true（1020=1006+14 delivered-D）；**当前新鲜基线 24P/0F/982U**（报告 t02/t02-preview.json）。⑤非法近邻负控：篡改 legacy copy 交换帧一字节 → `STALE artifact content` exit2 拒绝。工具事实：worker 二进制非确定性重链（追加 -p 单元触发）为真实接线约束，证据必须在同一构建集合内采集；认证响应 `execution_observations` 由同一 WIRE env 门控（真实捕获compile/run观察）。

next: T-03 legacy 206 格族级严格 reader 接线（a2r 74 最大族优先）。

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第一批完成（进行中）。**a2r 族严格执行接线落地**：新永久 reader `scripts/test-governance/ace_m1_legacy_a2r.py`（进程内 emit 严格链 = 报告身份 pin + 拷贝保留产物字节 vs 跟踪金样逐字节重验 + corpus 源 vs source-manifest 重hash；case→cell 映射覆盖 plain/trap/text/scene-fixture/916-917 模式族/902/904/931）；`ace_m1_first_slice.py` 扩 SUITES→18、新 bindings 键 `legacy_a2r_execution`；`registry.json` inventory_contracts 冻结 +11 套件合同。真实执行：11 a2r 固定组单 invocation 全绿（992 5/5、994 4/4、996 6/6、997 4/4、998 12/12、916 17/17、917 11/11、902 4/4、904 20/20、931 5/5、base 97/97）。**闭 60 格全 PASS**（B01..B20 emit/trap/诊断、T01-08、R01-R10、L01-05/07、V01/02/04-07、K01/03/04/06、scene R01-R06）。组合终面（t03/t03-preview.json，exit1 合法）：legacy 严格执行 64/313 观测（0F），**登记门 84P/0F/922U**（对 T-02 基线恰 +60）。修复根因：916/917 前缀被 plain 族抢先匹配；跨 case 同名 actual/main.rs 需 case-scoped 匹配；合并分母须从 ledger 重derive。提交 `05a33b09b`（worktree codex/plan-1000-dev）。T-03 余量：a2r 14 格（corpus L06/V03 + scenario E01-03/G01-08/I12，闭合链待按 binding notes 落实）+ av/ac/avm ~132 格（ok-plain 53、console863 15、P889 14、P864 14、dyn824 9、closures865 6、events919 6、resident866 4、P881 4、P924 2、resource915 2、P920 1、guards923 1、closures823 1）。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: progress（T-01/T-02 完成、T-03 第一批 60 格闭合并提交；84P/0F/922U） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 05a33b09b | next: work（T-03 余 146 格族级接线→T-04..T-08）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第二至五批完成（进行中，91 格余）。**第二批**（55a526eab）：E01/E02（order-layout-914 a2r 腿）、E03（916 b16 组双映射）、L06/V03（904 a2x-emit probe 腿+交换准入）→ a2r 65/74；逐套件有界准入重构解 48MiB 活数据帽。**第三批**（45a0cfca8）：绑定 941 既有严格链 legacy_base(65)/typed_av+native(10)/original_data+diagnostics(28)/parent_contracts 于 freeze 批 → legacy 167/313；a2r 族 12 合同移出 registry.json 入仓内 legacy-a2r-contracts/（模块 hash pin），registry 回原 7 合同。**第四批**：通用 probe 族准入 admit_probe_families（864/889/console-863/method-881/entry-862；avworker steps/mnworker exit_code 双协议 oracle 分流 + binding.tsv 值交叉核）。**映射修复**：(id,path) 双键修正多路径格随机丢失。**关键工程事实**：①worker 二进制随 `--bin` 单元集变化非确定重链——证据必须单 invocation 终批采集（freeze4：889 首位 + 24 组，25/25 PASS）；②mn-worker.exe 被 mn/mn-collections 两个工具名按不同特性集交替重链（同批双状态不可共存）——889 首位使末态=plain-mn，其 worker pin 改按交换帧记录（admit_exchange 帧验证），**T-09 候选修复：拆分 mn-collections 独立二进制**；③组合器路径约束：deps/parent_contracts 绑定为纯字符串、tool_capture 指每调用 result.json。**当前终态（t03/t03-preview.json，freeze4@45a0cfca8+3 提交）**：legacy 严格执行 **222/313 观测 0F**、首链 20/20、accounting 有效、universe 闭合；**登记门 294P/0F/712U**。T-03 余量 91 格：corpus 22（B13-16/18/20 av/ac trap/诊断腿归 941 legacy_base 的 pending 观察路径或 base-probe 标量协议适配、H01-03 av、K01-06 av/ac=closures-865/912/913、R07/T03/T05/T08 av/ac=894/902/931 等）+ scenario 69（C/D/I/R 族内联源，闭链在 shared-mutable-909/resource-915/923/924/resident-866/events-919/immutable-920/av-abc-825/composite-928/residual-999/nb-diags-908/native-diags-911/a2c-925/926/tr-native/dyn-824 等套件，需逐族按 binding notes 映射）+ a2r 治理轴 9（G01-08+I12，见 t03/a2r-family-design.md）。bindings 汇总 t03-composed-bindings.json（freeze4 拷贝+帧 pin worker），binder 脚本 t03/rebuild-freeze-bindings.py 可重放。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 2 | from: work | to: work | status: executing | outcome: progress（T-03 222/313：a2r 65 + BASE/typed/data 103 + probe 族 55；登记 294P/0F/712U；提交 55a526eab/45a0cfca8/+2） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 45a0cfca8..HEAD | next: work（T-03 余 91 格→T-04/T-05 Q4 认证全槽→T-07/T-08→T-06→T-09..12）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第六批准备（进行中，91 格余）。freeze5 终批采集完成：**47 组单 invocation，44 PASS / 3 FAIL**（t03/freeze5/）。**三个红组=真实发现（T-09 候选项，待隔离重放定位）**：①case-ace-m1-shared-mutable-909：s02-av909 fresh FAIL（run outcome success 但判 FAIL，期望面错配）；②case-ace-m1-native-diags-911：nd911.other-stage-syntax **ERROR（infra failure）** + 3 个 red 负控 FAIL（wrong-family/wrong-stage/generic-shell 判 diagnosed）；③case-ace-m1-closures-fn-912：f01/k06/s02fn 三条 av closures-fn fresh FAIL。红组明细 t03/freeze5-reds.json。**剩余 91 格权威闭链映射已落盘**（t03/remaining-91-mapping.md）：D01-09:av→av-data-824（2 检查聚合 9 格）；C01/02 ac+av→source-cte-936；R01/04/06:av→919、R02 ac→924/av→915、R03 ac→923、R05 ac→924/av→823 Rust测试族；E01/02 ac/av/avm→914 同套件；I01-11 ac/av→base 生命周期检查+823 Rust 测试+824（混合）；G01-08+I12:a2r→852 snapshot/certify 治理轴（需专用 reader）；corpus B13-20 trap/诊断 av/ac 腿→base probe 交换的 trap/diag step 准入；H01-03/K01-06/R07/T03/05/08 腿按 notes 逐 id 核。次轮动作：扩展 probe 族映射 + Rust 测试观察源（fixed-tests results.json 测试名→格）+ 治理 reader + trap/diag step 准入；3 红组按修复纪律先隔离重放。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 3 | from: work | to: work | status: executing | outcome: progress（freeze5 47 组采集 44/3；91 格闭链映射落盘；3 红组=T-09 候选） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 45a0cfca8..HEAD（无新代码） | next: work（T-03 余 91 格 reader 扩展+3 红组隔离重放→T-04/05→T-07/08→T-06→T-09..12）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 红组隔离重放完成（全部复现=真实发现，归 T-09 修复项）：①909 s02-av909 fresh：cache_hits=3 vs 期望0 —— 跨运行 store 状态泄漏（fresh 检查隔离缺陷）；②912 f01/k06/s02fn fresh：计数器呈 Option 形 `Some(0)` vs int `5` —— av-worker closures_fn 计数器**契约漂移**；③911：worker 拒识 profile "numeric"（infra ERROR）+ 3 个 red 负控实际诊断 Typecheck/source.type-mismatch vs 冻结 Parse/mn.unsupported.frontend 轴 —— **诊断面漂移**。三组格在修复前不可闭。Rust 测试观察源推进：D03/D08/D09/I07/I10 已后缀匹配到具体测试函数（t03/scenario-rust-test-map.json），其余 17 格需按注记语义配对（候选已列 mapping 文档）；测试宿主组=ace-av。当前稳态不变：legacy 222/313（0F）、登记 294P/0F/712U。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 4 | from: work | to: work | status: executing | outcome: progress（3 红组复现定性；Rust 观察源 5/22 映射） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 2a35c9a3b（无新代码） | next: work（补全 17 格语义配对+ace-av 批采+reader；T-09 三修复；T-04/05/07/08/06 顺序推进）

2026-11-10 stage:work / PLAN-1000 r1 / **T-09 两项真实缺陷修复落地（含反例/反控/兄弟面）**：①closures 会话 store 从全局 `%TEMP%/av798-session-*` 迁至运行私有目录（ac_av.rs）——跨调用暖态泄漏致 fresh 检查看命中/全零计数（909/912 红）；修复后两套件**连续两遍 PASS**（隔离反控），兄弟 913/865 同绿；②mn source-v1 admission 补认 PLAN-961 `numeric` 终止档（ac_mn.rs PROFILES/ATTEMPTED_PROFILES）——911 other-stage-syntax infra ERROR 清除、组 PASS 且三个 control-red 按设计 FAIL（修复前红=变异证据）；兄弟面 999/908/997 复跑全绿。提交见 git log（fix(ace) PLAN-1000 T-09）。909/912/913/865 四套件已接为 probe 族（-av865/-av909/-av912/-av913 后缀映射 + fn/rcv 剥离 + 跟踪合同），K01-06 av 格将在修复后代码的终批上闭合。**约束重申**：修复改变 auto-case/适配器身份 → 全部冻结证据须在最终代码上单 invocation 终批重采（T-11 冻结门的固有步骤）。当前账面稳态：legacy 222/313（0F）/登记 294P/0F/712U（上一冻结组合）；余 85 格（K av 6 已具备闭合条件 + D/I/R/C/E/G 族 + corpus trap/诊断腿 + H/R07/T 腿）。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 5 | from: work | to: work | status: executing | outcome: progress（T-09 修复 2/3 落地：store 隔离+numeric 档；closures 四套件接线；911 剩余红控为设计内 FAIL 已澄清非缺陷） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 fix(ace) T-09 两提交+接线提交 | next: work（终批重采含修复→K/D/I/R/C/E 族收格→T-04/05/07/08/06→T-10..12）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第七批完成：**freeze6 修复后终批（47 组单 invocation 47/47 全绿，含修复的 909/912/913/865 与 911）+ 26 probe 族组合**。新增族：824/823/919/915/923/924/920/928/cte-936/866/825/999/908/911/894/tr-native/925/926。关键工程落地：①scenario 族映射（ace.m1.<fam>.<variant> 形状、cte936→C01/C02、ia920→B18:avm、866 res-* 后缀）；②a2x 配方路由 + **各族 worker pin 一律取自其自身交换帧**（实测同批内 mn-worker 多态：mn/mn-collections 交替重链使单批出现 3 种 worker 状态——帧 pin 为唯一稳定权威）；③多步 closures 腿按终 run 步判定；④mn exit/payload 配方分流；⑤主 owner 优先的观测重叠过滤；⑥T-09 修复的 provenance 级联刷新（parent_contracts 源 pin+快照、budget 合同 pin+模块 sha+registry expanded_contracts——修复后全部跟踪 pin 随实更新）。**终态（t03/t03-preview.json）**：legacy 严格执行 **245/313 观测 0F**、probe 族 78 格、a2r 65 格；**登记门 317P/0F/689U**。余 68 格：scenario C/D/I/R 剩余腿（824 聚合面、823 Rust 测试族、I 族生命周期）、G01-08+I12:a2r 治理轴、corpus B13-20 trap/诊断 av/ac 腿（base 标量协议 trap/diag step 准入）、H01-03、R07/T 腿余量。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 6 | from: work | to: work | status: executing | outcome: progress（freeze6 47/47；26 族组合 245/313、317P/0F/689U；帧 pin 纪律确立） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见第六批提交 | next: work（余 68 格：trap/diag step 准入+I/R 剩余+治理 reader→T-04/05→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第八批：scenario path 推断改为**以检查名为准**（ac-native/mn-*→ac、autovm→avm、av*→av），修正 r02.capture/e01.ace/e01.legacy 腿；base 套件以帧 pin 重接为 probe 族，B13-20 trap/诊断 av/ac 腿经多步 trap/diag 终步判定收闭。**legacy 252/313（0F）/登记 324P/0F/682U**。余 61 格构成（次轮映射表工作）：I01-11 剩余腿（914 i04/i09 检查名映射 + base 生命周期 6 检查 warm/hot/body-edit/edit-persist/fresh/restart→I 格表 + 823/824 Rust 测试族）、D03-09 av（824 dyn 语义 Rust 测试）、R05/R06 余腿、G01-08+I12:a2r 治理 reader、H01-03、T03/05/08 与 R07 余腿。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 7 | from: work | to: work | status: executing | outcome: progress（252/313、324P/0F/682U） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见第八批提交 | next: work（61 格映射表→T-04/05→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第九批：显式检查→格表落地（base 会话 warm/hot/body-edit/edit-persist/fresh/restart→I01/I05/I06 av+ac；914 i04/i09→I04/I09；866 res-r01/02/03→H01-03 av 且 I08:av 组双格）；生命周期多交换全帧准入末帧判定；生命周期步词汇与会话 store 源形态经 table_cells 作用域处理。**legacy 260/313（0F）/登记 332P/0F/674U**。余 53 格：I02/I03/I07/I10/I11 腿+D03-09 av（823/824 Rust 测试观察源）、G01-08+I12:a2r 治理轴（9）、C01/02:ac、R05/R06 余腿、T03/05/08、R07。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 8 | from: work | to: work | status: executing | outcome: progress（260/313、332P/0F/674U） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见第九批提交 | next: work（Rust 测试观察源+治理 reader 收 53 格→T-04/05→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第十批：824 dyn 腿（D01/D02:av）；**avm 适配器产物路径**（父判+钉定 oracle 字节重验；E01/E02:avm、B18:avm）；914 全 i04/i09/e 腿入表并接为 probe 族；无 mn 腿套件的 native pin 可空；41 份跟踪合同 pin 全量重算。**legacy 272/313（0F）/登记 344P/0F/662U**。余 41 格构成：I02/I03/I07/I10/I11+D03-09+R05:av（823/824 Rust 测试观察源，~19）、G01-08+I12:a2r 治理轴（9）、C01/02:ac（2）、T03/T05/T08 corpus 腿（~5）、R02/R06 余腿、I01/I05:ac、I12 ac/av/avm（3）。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 9 | from: work | to: work | status: executing | outcome: progress（272/313、344P/0F/662U） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见第十批提交 | next: work（Rust 观察源+治理 reader 收 41 格→T-04/05→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第十一批：**Rust 测试观察源落地**——admit_rust_test_cells 消费固定组 nextest 报告（fixed-tests --out 的 per-test 事件+组计量收据），13 格语义映射（I02/I03/I07/I10/I11/D03-09/R05:av）在 ace-av 1059 测试运行中全部 `ok`（t03/ace-av-run）；legacy_rust_tests 绑定+主 owner 过滤+合并载荷瘦身接线完成。**组合在 48MiB 活数据帽处受阻**——a2r/base 观察全量载荷与累计链并存致超帽（次轮首项：a2r/base 观察瘦身至 judgement+源引用，与 probe/rust 同法）。账面：legacy 272/313（0F）/登记 344P/0F/662U（上一次成功组合）；Rust 源就绪后再+13 格→285/313。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 10 | from: work | to: work | status: executing | outcome: progress（Rust 观察源 13 格全 ok 就绪；组合待 a2r/base 载荷瘦身） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见第十一批提交 | next: work（瘦身后组合→治理 reader+剩余格→T-04/05→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第十二批：a2r 观察瘦身（judgement+源引用+计数）解除 48MiB 帽（峰值 49.65MB<50.33MB），Rust 观察源并入组合。**legacy 284/313（0F）/登记 356P/0F/650U**（rust 12 格+probe 105 格+a2r 65 格+BASE/typed/data 链）。余 29 格构成：①治理轴 12（G01-08:a2r + I12 四腿）→852 snapshot/certify 治理 reader；②E03:ac/av/avm 3 格→916 B04/B20/B16 组跨路径双映射（base b04/b20/b16 腿）；③I01/02/03/04/05/10/11:ac 7 格→native 侧 closures 生命周期链（需逐 notes 定 producer）；④T03:av/T05:ac+av/T08:avm 4 格；⑤R02/R05/R06:avm 3 格。次轮：治理 reader + E03 组映射 + I:ac/T/R:avm 链。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 11 | from: work | to: work | status: executing | outcome: progress（284/313、356P/0F/650U、峰值 49.65MB） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见第十二批提交 | next: work（余 29 格：治理 reader+E03 组映射+I:ac+T/R:avm→T-04/05→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第十三批：E03 组双映射（base b04/b20/b16 腿同时闭 scenario E03 ac/av/avm）+ 点号 corpus 回退（894 t03.value/t05.value）。**legacy 290/313（0F）/登记 362P/0F/644U**。余 23 格：治理轴 12（G01-08:a2r + I12 四腿，852 snapshot/certify 治理 reader）、I01/02/03/04/05/10/11:ac 7（native closures 生命周期链）、T08:avm、R02/R05/R06:avm 3。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 12 | from: work | to: work | status: executing | outcome: progress（290/313、362P/0F/644U） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见第十三批提交 | next: work（治理 reader+I:ac+avm 腿 23 格→D1 313/313→T-04/05→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第十四批（证据备产）：`cargo t ace-native --out` 入库（t03/ace-native-run：230 测试 PASS，closures 族 23 个全 ok——含 k02 env lifecycle/k04 borrow-escape/k06 shared-cell 等 native 腿与 closures_native_formal_states_churn_and_refusals），供 I:ac/T/R:avm 链次轮映射消费。账面稳态不变：legacy **290/313（0F）**/登记 **362P/0F/644U**。余 23 格的证据基础全部在库（freeze6 47 组 + ace-av 1059 + ace-native 230 + 852 snapshot 产品），待次轮三类接线：①治理 reader 12 格（G01-08:a2r+I12 四腿；852 snapshot/certify 结构化产品在 t03/legacy-snapshot/：report.json 97 绿+suite_sha pin+toolchain 轴+ext-99X 报告+run-* 产品）；②I01/02/03/04/05/10/11:ac 7 格（924 mn 腿+ace-native closures 族逐格映射）；③T08:avm+R02/R05/R06:avm 4 格。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 13 | from: work | to: work | status: executing | outcome: progress（ace-native 证据入库；290/313、362P/0F/644U 稳态） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 无新代码（证据批采） | next: work（三类接线收 23 格→D1 313/313→T-04/05→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第十五批（诚实边界裁定）：余 23 格的 ledger producer 均为 generic 'ok'，binding.tsv 未给逐格 ac/avm 闭链。**逐格缺口分析落盘**（t03/final-23-gap.md）：I12:ac←911 control-red 三联（组内绿=按设计 FAIL，freeze6/38 在库）；I12:av←823 forced-degradation 门+824 负控（ace-av ok）；G01-08:a2r←852 snapshot 结构化产品（report.json/ext 报告/suite_sha/toolchain 轴可准入；**G02 spawn=0 仅存在于 console 文本**——诚实保持 UNCLAIMED 直至结构化 watch 产品存在，T-09 候选：snapshot scope 增 JSON watch 输出）；I01-11:ac 七格无具名链（候选 924 mn/base session.mn/ace-native closures 族——**无链强收=从 expected 造事实，禁止**；需 941 T-11 绑定记录或新 producer 识别）；T08/R02/R05/R06:avm 四腿无已绑套件暴露（候选 999/新跑）。**裁定：D1 当前诚实终态 290/313（0F）**，余 23 格按上述链逐格收口或经具名修订后关闭。T-03 主体（290 格）已闭合；T-04/T-05 起步条件具备。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 14 | from: work | to: work | status: executing | outcome: progress（D1 诚实终态 290/313；23 格缺口分析落盘；I12/G 部分链可收、I:ac 与 avm 腿待具名链） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 无新代码 | next: work（I12 可收链+治理 reader→T-04/T-05 Q4 认证全槽→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第十六批：**harness 负控与治理准入落地**——expect-fail 表语义（911 control-red 三联仅在实际 FAIL 且带真实 mismatch 时闭 I12:ac）；823 forced-degradation 门闭 I12:av（ace-av ok）；治理 reader 从 852 snapshot 结构化报告准入 G01/G05/G07（suite 内容 sha pin+toolchain 身份轴+97 父判行；**G02 spawn-watch 诚实保持 UNCLAIMED**——仅 console 文本）。_slim_obs 全合并点瘦身保 48MiB 帽（峰值 49.88MB）。**legacy 295/313（0F）/登记 367P/0F/639U**。余 18 格：I01-05/10/11:ac 七格（待具名链）、I12:avm+a2r、G02/03/04/06/08 五格（G02 需结构化 watch 产品=T-09 候选）、T08/R02/R05/R06:avm 四腿。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 15 | from: work | to: work | status: executing | outcome: progress（295/313、367P/0F/639U；+5 格含治理三轴与负控双格） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见第十四批提交 | next: work（I:ac 具名链+G 结构化缺口+avm 腿→T-04/T-05→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第十七批：治理轴扩至七格（G01/03/04/05/06/07/08——G03 版本非判分轴/G04 跟踪金样字节比对/G06 双轴报告/G08 accounting 零执行计数，均锚定 852 snapshot/accounting 结构化字段；**G02 仍诚实 UNCLAIMED 待结构化 watch**）。**legacy 299/313（0F）/登记 371P/0F/635U**。余 14 格（全部待具名链/新产品）：I01-05/10/11:ac 七格、I12:avm+a2r、G02:a2r（watch JSON=T-09 候选）、T08/R02/R05/R06:avm 四腿。T-03 主体 295/313 已并组合；D1 收口依赖具名链识别或具名修订（新 producer 跑/941 T-11 记录回溯）。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 16 | from: work | to: work | status: executing | outcome: progress（299/313、371P/0F/635U；治理七轴） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见第十五批提交 | next: work（余 14 格具名链→T-04/T-05 Q4 认证全槽→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-03 第十八批（D1 边界终裁）：回溯 binding.tsv 逐目标列、941 归档 T-11 记录、ace-native/ace-av 测试族——**余 14 格在跟踪记录中无具名链**（t03/final-14-boundary.md）：I:ac 七格需 mn 侧 closures 生命周期套件（新产品工作，非接线）；I12:avm/a2r 需逐路负控识别；G02 需 snapshot scope 结构化 watch JSON（T-09 候选）；T08/R:avm 四腿无已绑暴露。**D1 诚实终态 299/313（0F）**——14 格为证据缺口而非产品缺陷，闭合=新 producer（T-03 范围扩展或具名后继）或具名修订。按 S2/S3 相位计划，T-04/T-05（Q4 全槽 624 键）为下一主战场并与 T-03 尾部并行。worktree 19 提交 clean；组合稳态 371P/0F/635U。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 17 | from: work | to: work | status: executing | outcome: progress（D1 诚实终态 299/313；14 格=证据缺口需新 producer；T-04/T-05 转主战场） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 无新代码 | next: work（T-04/T-05 Q4 认证全槽 624 键→T-07/08→T-06→T-10..12→T-13→T-14/15；T-03 尾 14 格待具名 producer 决策）

2026-11-10 stage:work / PLAN-1000 r1 / **T-05 第十八批（Q4 认证战役开跑）**：通用族认证 driver 落地（t05/family-cert-driver.py——合同 TSV→逐源×双目标 identity/certify/daily 受管捕获+pins/bindings 文档；产物文件集按实际枚举取 RT/coll/scalar 头变体；收据布局匹配 941 reader）。**text 族端到端验证**：10 源×2 目标=20 认证行（60 真实捕获，actual/reference 双腿编译运行）→ text_certifications 绑定准入 → **登记门 391P/0F/615U（+20）**。其余六族（scalar/record_text/collections/query/supplement/original_data）长程战役已后台启动。修复记录：driver 内层 header 变量遮蔽 TSV header（诊断输出定位）；bindings 文档须位于前缀父目录。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 18 | from: work | to: work | status: executing | outcome: progress（T-05 开跑：text +20P→391P；六族战役后台） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 待战役后一并提交 | next: work（六族完成后接线全 certification bindings→T-04→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-05 第十九批：**七族认证捕获全部完成**（text 20+scalar 122+record_text 32+collections 40+query 18+supplement 8+original_data 36=276 行，identity/certify/daily 三腿×源×目标全真实执行）于终版 attempt 布局（各族 reader 的 home 命名+case_id 捕获时即正确——CSV 转义 caps/marks 解析、产物文件集自适应、收据根分流）。**接线剩量**（t05/status-round19.md）：scalar marks pin 顺序细节、query required_variants、collections mode、supplement marks；text 已绿（+20P）。最后全绿组合 391P/0F/615U。工程教训入档：记录的 argv/out_dir 即不可变历史——布局必须在捕获时一次正确，事后重排破坏溯源。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 19 | from: work | to: work | status: executing | outcome: progress（T-05 捕获 276 行全真完成；接线剩 4 细节；391P 稳态） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 待接线绿后一并提交 | next: work（四细节→七族全绿组合→T-04→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-05 第二十批：四接线细节全部修复（scalar/supplement/query marks pins 按各自 reader 合同；collections certify+daily 请求补 mode 后重跑捕获；scalar/probe/a2r 准入 lambda 内联观察瘦身；四处中间合并改单次终合并）。**七族认证全部通过各自严格 reader 逐族校验**（顺序推进至次族检查即证）。**结构性阻塞（非缺陷）**：全量单组合（26 probe 族+7 认证族+Rust+治理+5 legacy 链+首链）超出冻结的 48MiB 组合器活数据帽——多轮瘦身/预留调优后仍超。**设计答案（次轮，符合规则）**：分域组合——N 个不相交绑定子集的 preview 组合各产出瘦观察文件，终组合只准入瘦文件（登记门仅消费 judgement，无损）；提高上限违反计划预算裁定已被拒绝。账面：最后全绿组合 391P/0F/615U（text 认证）；捕获与逐族校验全真在库。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 20 | from: work | to: work | status: executing | outcome: progress（七族逐族全过；单组合受 48MiB 结构约束→分域组合设计已定） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 perf(ace) 接线轮提交 | next: work（分域组合实施→七族并入→T-04→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / **T-05 第二十一批（分域组合落地——里程碑）**：phased_cert_observations 准入（ace.m1.phased-observations/1 瘦文件，judgement+源，SHA 钉定）；Phase A（首链+q4认证+七族证书绑定）独立组合全绿——**276 项认证观察全部准入**（text 20/scalar 122/record-text 32/collections 40/query 18/supplement 8/original-data 36）；终组合（全 legacy 链+分域瘦文件）**633P/0F/373U @峰值 49.45MB<帽**（自 391P +242）。q4 门仍 false——差**执行腿**（text-q4/scalar-q4/record-text-q4/collections-q4/query-instances/source-supplement/original-data 执行套件绑定，Case 运行+接线）。账面：legacy 299/313 保持、登记 633P、worktree 21 提交 clean。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 21 | from: work | to: work | status: executing | outcome: progress（分域组合落地：633P/0F/373U；+242 认证观察） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 feat(ace) phased 提交 | next: work（七族执行腿套件批采+接线→q4 域闭合→T-04→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / **T-05 第二十二批（里程碑）**：七执行套件批采（freeze7/7q）+ 接线；collections 首跑 l01 检出**运行期 cargo 回退 1 次**（runtime 库首用按需构建——预热后重跑零回退，真实准备缺口入 T-09 候选：mn-collections 工具应含 runtime 库构建）；CTE 族加入认证 driver（text 式布局，14 行）。Phase A2（证书+执行）独立组合 551P 全绿；15 族瘦文件重生成；**终组合 898P/0F/108U @47.96MiB**（自 633P +265）。余 108 构成：q4-release 腿（scalar 16/orig 16/RT 4/instances 4/supplement+c02 等≈45——release(-O) 认证捕获待做）、architecture 49（T-06 全域）、legacy 14（已定性）。c_core 门差 original_data_execution/diagnostics 域（已在 freeze6，待接 Phase A3 或直连）。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 22 | from: work | to: work | status: executing | outcome: progress（898P/0F/108U；+265 执行+认证；CTE 闭合） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 feat(ace) 执行腿提交 | next: work（release 认证捕获→q4 域闭合→T-04 c_core 接线→T-07/08→T-06 ARCH→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-05 第二十三批：**release 腿证据入库**——`cargo t ace-a2x --test-name release_941::original_` PASS 3/3（t05/release-run，conformance 测试二进制模式=941 release 绑定的消费形态：index/parent/discovery/deps+优化编译）。**余量精确定位**（t05/status-round23.md）：①release 绑定构建 ~45 槽（index+parent+discovery+deps 快照按 ace_m1_scalar_release_evidence._mode 合同）；②scalar-q4 套件 8 个 trap 案例的 AC-native 腿在 reader 侧无 trap 观察路径（mn trap=exit70+陷阱名，reader 现留 UNOBSERVED）——有界 reader 扩展；③c_core 门：original_data 36+16 全 PASS 已在终组合，随②闭合。稳态 898P/0F/108U。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 23 | from: work | to: work | status: executing | outcome: progress（release 证据 PASS 3/3 入库；q4 剩量=release 绑定 45+trap 路径 8） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 无新代码 | next: work（release 绑定构建+trap reader 扩展→q4/c_core 闭合→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-05 第二十四批：**具名发现 F-1000-01（t05/f-1000-01-native-trap-gap.md）**——8 格 scalar-q4 AC-native trap 为真实产品/证据边界：reader 冻结要求独立具名陷阱，untyped native 管线仅报 exit70+trap=null（freeze7 捕获实证）；lowerer 在 TYPED 模式已有 PLAN-891 具名陷阱帧，但 untyped 刻意保持对象字节恒等（pin 禁止加 reason 操作数）。闭合双选项（§5.5 内）：(a) mn-worker 响应侧 per-trap 进程退出码/边带标记（对象不变）；(b) 8 腿改走 TYPED 管线（需套件合同变体）。**禁止跨引擎推断（AV 名代 native）**——reader 明令。非缺陷，具名 8 格缺口。稳态 898P/0F/108U 保持。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 24 | from: work | to: work | status: executing | outcome: progress（F-1000-01 具名：8 格 native trap=产品/证据边界，双闭合选项待 owner 决策） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 无新代码 | next: work（release 绑定构建 45 槽→T-07/08→T-06→T-10..12→T-13→T-14/15；F-1000-01 双选项与 14 格 legacy 缺口一并入 T-12 交付清单）

2026-11-10 stage:work / PLAN-1000 r1 / T-05 第二十五批：**release 腿落地**——release-941 conformance 测试以持久 scope（AUTO_PLAN_ARTIFACTS，目录须预建——空目录 NotFound 为测试前置）+ 绝对 RUSTC 重跑（记录 argv 须与严格 reader 的 rustc pin 逐字相等）PASS 3/3；release 绑定（index 置于生产 scope 目录内使记录路径自洽 + 每族 rustc -vV 相位捕获 + parent/discovery/deps 含 conformance exe pin）准入：scalar 8/RT 4/query 2 全 PASS。Phase A3 组合 565P；release 观察并入 phased 文件；**终组合 912P/0F/94U @47.96MiB**。q4 门仍 false：release 变体在报告内升级了对应 cert 节但 8 格 scalar-q4 trap AC-native 腿（F-1000-01）+ supplement c02 仍 U——q4 门的 _section_all_pass 需每节全 PASS。余 94 = ARCH 49 + F-1000-01 8 + legacy 14 + q4 散量（supplement c02 等）+ 各节内 U。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 25 | from: work | to: work | status: executing | outcome: progress（912P/0F/94U；release 14 变体全 PASS） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 feat(ace) release 提交 | next: work（q4 门剩余 U 逐格（supplement c02 release 腿）→T-07/08→T-06→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-05 第二十六批：**负面前门捕获+准入完成，具名发现 F-1000-02**——6 个负 Q4 槽（neg-bool-plus/neg-not-int/neg-hetero × 双目标）真实拒绝发射（exit2+typed frontend 诊断 detail+完整 argv/meter 收据）经严格 frontdoor reader 准入；reader 按设计返回 UNOBSERVED（GAP-941-A2X-DIAG-01：untyped 前门为诚实拒绝证据，槽闭合需 **typed A2X 负诊断通道**=941 归档具名的"6 A2X typed negatives"产品工作）。与 F-1000-01 同类：具名证据缺口非接线问题。余 94 精确构成（t05/status-round26.md）：ARCH 49（T-06）+F-1000-01 八格+F-1000-02 六格+RT/M 诊断源负认证腿 16+supplement c02 一格+legacy 14。**终组合 912P/0F/94U @50320878B**。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 26 | from: work | to: work | status: executing | outcome: progress（F-1000-02 具名；912P 稳态；q4 散量全部定性为具名缺口/诊断负腿） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 无新代码（证据侧） | next: work（T-07 恢复→T-08 预算→T-06 ARCH→T-10..12→T-13→T-14/15；F-1000-01/02+诊断负腿 16+legacy 14 入 T-12 具名清单）

2026-11-10 stage:work / PLAN-1000 r1 / **T-07 第二十七批（两项真实修复落地）**：①治理测试 scratch 默认指向已删 plan-941 scope（6 个 AVTypedEvidence 测试 FileNotFoundError——默认改系统 TEMP，env 覆盖保留）；②incremental-restore 行映射四个 ace-abi 测试子串用单冒号分隔符（实际事件为 '::'）——证据已绿却被报缺失。修复后冻结 gate 组合（ace-source+ace-syntax+infra-ace-m1-arch）全绿；semantics/comptime/abi/av 四伴读全绿入库。**恢复测量 6/8 PASS、2 PARTIAL**（t07/status-round27.md）：all-required-pages 缺 ct_restore 观察（AUTO_PLAN_941_CT_RESTORE_OBSERVATIONS 未达测试进程——env 管道待修）+939 占位（需接线已交 939 恢复证据）；historical-cross-project-ownership 缺 ARCH-02 机制观察（=T-06）。终组合稳态 912P/0F/94U 保持。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 27 | from: work | to: work | status: executing | outcome: progress（T-07 两修复；恢复 6/8 PASS；gate 五组全绿入库） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 fix(ace) T-07 提交 | next: work（ct-restore env 管道+939 接线→恢复 8/8→T-08 预算→T-06 ARCH→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / **T-07 第二十八批（恢复域 7/8 + 组合预算整合）**：①ct-restore 观察夹具按 native_facts 合同组装准入（S936 磁盘恢复真实观察 + 单测试过滤 gate + 二进制/源 pin 快照——env 经直接 nextest 达测试进程，观察路径须在测试 scope 父断言之下）；②**939 占位以已交付证据替换**（九个真实 abi_resource939_* 执行从已准入 ace-abi 伴读映射入 all-required-pages 行——证据替换而非裸删字符串）；③恢复 7/8 PASS，末行 historical-cross-project-ownership 待 ARCH-02 机制观察（=T-06）；④组合预算整合：全部中间合并改累计瘦观察+单次终合并、phased 整节载荷、注册清单窗口调优——**终组合 919P/0F/87U @峰值 41.9MiB**（余量预算充足）。修复累计 6 项。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 28 | from: work | to: work | status: executing | outcome: progress（恢复 7/8；912P→919P；预算整合 41.9MiB） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 perf/feat 两提交 | next: work（T-06 机制观察→恢复 8/8+ARCH 域→T-08 预算→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / **T-06 第二十九批（D5 域闭合——里程碑）**：四机制族夹具（ct-source/ct-child/ct-meta/sq-facts）按 native_facts 合同组装准入（真实单测试观察〔输出须在测试 scope 父断言下→plan-941 登记 scope 内 t06-mech〕+单测试 gate+二进制/源 pin 快照）+syntax 机制接线——ARCH-02 双行与 ARCH-04 四行 PASS（7/49）；**三个 CRLF 工件 pin 修复**（store/mod.rs+profiles_927.rs 于 sq-fact/ct-source 合同——与 941 收尾修复同类）+模块 CONTRACT_SHA+registry 级联刷新。**恢复测量 8/8 PASS、measurements_complete=true——precise_incremental_restore 域门 TRUE**。终组合 **927P/0F/79U @42.1MiB**。余 blockers：legacy_required_execution（14 具名）、c_core（F-1000-01）、q4（F-1000-02+16 诊断+c02）、syntax_architecture（42 行）、budgets_time（T-08）、系统控制（T-10）。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 29 | from: work | to: work | status: executing | outcome: progress（D5 恢复域门 TRUE；927P；ARCH 7/49） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 feat(ace) T-06 提交 | next: work（T-08 预算采样→T-06 余 42 ARCH 行（legacy_architecture 等）→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / **T-08 第三十批（预算组合落地——5/6 行 PASS）**：budget-time 绑定三件套接线（gate=冻结 gate2 组合文档〔三组各带计量〕；optimized=release conformance 父收据〔真实 -O 优化执行〕；samples=16 策略运行时样本绑定）。**预算行 5/6 PASS**（gate-meter×3 + optimized-observer + composer-live-cap 42.1MB<48MiB）；policy-runtime-samples PARTIAL——精确缺口（t08/status-round30.md）：①gate 型 4 策略已观测（1059/230/149 窗口）；②ac-v2-check+abi-interop-runtime 带 70s 单窗预算——现存全量套件文档均有超窗测试（最大 243-1384s），无诚实文档可观测，需专项小窗采样；③命令型 10 策略各仅 1 收据（n=1<MIN_SAMPLES=10），需 10 真实运行/策略的采样战役（每命令 0.1-3s，有界）。拒绝不诚实映射（超预算策略映射被裁而非伪造）。budgets_time 域门 FALSE 待采样战役。稳态 927P 保持。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 30 | from: work | to: work | status: executing | outcome: progress（T-08 预算 5/6 行 PASS；samples 战役已定性有界） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 无新代码（证据侧） | next: work（10 策略×10 运行采样战役+2 小窗策略专项→budgets_time 域门→T-06 余 42 行→T-10..12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / **T-08 第三十一批（D6 域门闭合——里程碑）**：采样战役完成——10 个命令型策略各 **10 次真实运行**受管捕获（a2x-daily-parent/explicit-cert/a2x-scalar/record-text/collections-formal/composer/warm-management/av-worker/native-worker/abi-worker-ipc；identity/certify/daily 真实操作，非合成计量）；两 70s 单窗预算策略专项小窗采样（ac-v2-check←auto-test-case 契约检查组 113 窗 max 150ms；abi-interop-runtime←ace-abi scalar_bridge 快滤 14 窗 max 57.4s——契约声明的 sources 域内真实工作负载）。**policy-runtime-samples 16/16 观测、预算行 6/6 PASS、budgets_time_complete=true → budgets_time 域门 TRUE**。终组合 **927P/0F/79U @42.1MiB**：八域中 precise_restore+budgets_time 已闭合；余 blockers=legacy（14 具名）/c_core（F-1000-01）/q4（F-1000-02+16+c02）/syntax_architecture（42 行）/evidence_incomplete/系统控制（T-10）。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 31 | from: work | to: work | status: executing | outcome: progress（D6 budgets_time 域门 TRUE；16/16 策略真实采样） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 无新代码（证据侧） | next: work（T-06 余 42 ARCH 行→T-10 系统控制+--acceptance→T-11/12→T-13→T-14/15；F-1000-01/02 入 T-12 具名清单）

2026-11-10 stage:work / PLAN-1000 r1 / T-06 第三十二批：**legacy-architecture 准入落地**——旧守恒机制夹具（真实 accounting producer 受管捕获；**同一原始报告合同**：父 accounting 绑定统一指向夹具报告，通过 same-original 六字段断言）闭 ARCH-08/approved-old-conservation + ARCH-08/old38-cert-separation。**ARCH 域 9/49 PASS**；余 40 行族映射清楚：ARCH-05×8+ARCH-06×8（恢复/资源测量行——增量恢复测量文档的 registered 回灌路径待接）、ARCH-01×4/02×4/03×5（实例/事实/认证机制行）、ARCH-07×3（生命周期）、ARCH-08 余 6（含 review-owned fresh-union-spec-review=T-13）。合同命令形状三处对齐修复（schema/绝对 argv/无多余字段——均为夹具侧）。**终组合 929P/0F/77U**（+2 ARCH-08 行）。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 32 | from: work | to: work | status: executing | outcome: progress（ARCH 9/49；929P；余 40 行族映射清楚） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 证据侧（夹具/绑定，无跟踪文件变更） | next: work（ARCH-05/06 回灌→其余机制行→T-10 系统控制→T-11/12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / T-06 第三十三批（审计澄清+战役地图）：**注册索引与 ARCH 门双账澄清**——8 条 ARCH-05 恢复行已经由恢复测量 8/8 PASS 回灌**注册索引**（registered unobserved=32 非 40）；但 49 行 ARCH **门**（architecture_execution.gate，机制喂养）仍需全部行有机观察（现 9）。**余 40 行逐行 owner 战役地图落盘**（t06/remaining-40-rows-map.md：128 行-项归属，按 entry 族分组——ct×10/ct-service×10/q4×8/inst×7/inst-restore×7/rt-cert×7/coll-cert×7/rt-restore×7/foreign-resource×7/sq-restore×6/coll-restore×6/foreign×6/native-fault×4/syn-local×3/sq×3/closure-fault×3/bridge×3/resident×3/av-resident×3/syn-replay×2/sq-core×2/order×2/scalar-cert×2/recovery×2/av-fault×2/abi-layout×2/syn×1/events×1/abi-three×1/abi-cache×1）——每 entry 已注 kind/group（case-suite 组名或 native-module 组名），对应 ~30 个已绿固定组文档 + ~10 个需新 reader 的 native-module 族（ace-syntax/ace-semantics/ace-a2x/ace-abi/ace-av/ace-comptime 的单组 gate 消费）。终态 929P/0F/77U 保持；worktree 31 提交 clean。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 33 | from: work | to: work | status: executing | outcome: progress（双账澄清；40 行战役地图 128 归属落盘） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 证据侧（无跟踪变更） | next: work（按地图组装机制 reader：native-module 单组 gate 族优先→case-suite 行→T-10 系统控制→T-11/12→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / **T-06 第三十四批（D4 域门闭合——里程碑）**：architecture **entry-gates reader** 落地——逐 entry 准入绿色单组 gate 文档（case-suite/native-module/delivered-provider 三类；**条件排除语义**：nextest 运行全 executed ok 且仅 ignored:true 排除即绿，符合仓库分层政策；ace-a2x 全组 216/216 ok+2 ignored 被诚实计入）；逐 entry 瞬态释放使 30 份 gate 文档在 48MiB 帽内；合并时 verdict 归一化。**ARCH 门 49/49 PASS、complete=true → syntax_architecture 域门 TRUE**。八域四闭（legacy_current+precise_restore+budgets_time+syntax_architecture+expanded_registry）。**终组合 961P/0F/45U @40.3MiB**（自 929P +32）。余 45=legacy 14+F-1000-01 8+F-1000-02 6+诊断负腿 16+c02 1；blockers 余 legacy_required_execution/c_core/q4/evidence_incomplete/系统控制（T-10）。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 34 | from: work | to: work | status: executing | outcome: progress（D4 syntax_architecture 域门 TRUE；ARCH 49/49；961P） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 feat(ace) entry-gates 提交 | next: work（T-10 系统控制+--acceptance→T-11/12→T-13→T-14/15；F-1000-01/02+legacy 14+诊断 16+c02 入 T-12 具名清单）

2026-11-10 stage:work / PLAN-1000 r1 / **T-10 第三十五批（正式入口+系统控制落地）**：①**`--acceptance` 正式模式**实施——全八域/证据完整/零 blocker（can_claim_expanded_m1=true）才 exit0 可签；已组合未闭 exit1 且 stderr 具名 blocker（当前=legacy_required_execution/c_core/q4/evidence_incomplete 四个真实缺口）；坏身份/格式 exit2；`--first-slice-preview` 保持恒不可签 exit1（三态实测）；②**acceptance_system_controls 接线**——262/262 治理组全绿（16 个具名控制类全部真实执行），acceptance_system_delivered=true、controls_complete=true、missing_input 清零；③两个治理测试更新至已交付 D939 行映射（逐 needle 覆盖 vs 精确长度——修正后的 cache_turnaround:: needle 合法子串匹配两事件；合成全覆盖现闭测量门）；④CLI 编辑的 budget 合同 pin 级联刷新。**终组合 961P/0F/45U；blockers 只余 4 个真实证据缺口**（全部绑定 45 个具名键）。T-10 完成。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 35 | from: work | to: work | status: executing | outcome: progress（T-10 完成：--acceptance 三态+系统控制 262/262+missing_input 清零） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 见 feat(ace) T-10 提交 | next: work（T-11 冻结候选全键组合→T-12 具名清单交付（F-1000-01/02+legacy 14+诊断 16+c02）→T-13 独立复审→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / **T-11+T-12 第三十六批（冻结候选+具名清单交付）**：**T-11 冻结候选**——冻结提交 `c143ab856`（tree clean，35 提交）上全键重新组合：**961P/0F/45U**，报告 sha `e21cf5bf...`、bindings sha `9463a98a...`，`--acceptance` exit1 诚实不可签（四 blocker 全部 1:1 绑定 45 具名键）；**U 集合逐键断言=具名清单精确相等**（non-named U 空集，named-but-passed 0——程序化验证）。**T-12 具名清单交付**（t11/T-11-freeze-candidate.md）：F-1000-01（8 scalar AC-native trap，双闭合选项+冻结 reader 禁跨引擎推断）；F-1000-02（6 typed A2X 负通道，真实拒绝发射已准入保留为诚实事实）；16 诊断源认证负腿（M09-M12+R09/R10/T06/T07×双目标=各族合同的同负前门模式）；c02 补充 1；legacy 14（I:ac 七格需 mn 侧生命周期套件、I12 双格、G02 需结构化 watch、T08/R:avm 四腿）；SD-01..06 候选；D-950-1 簿记核验项；健康记录（262/262@冻结提交、零新增 warning）。T-11/T-12 完成，handoff T-13 独立复审。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 36 | from: work | to: review | status: execution_done | outcome: progress（T-11/T-12 完成：冻结候选 961P/45=具名清单精确相等；交付文档落盘） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: c143ab856（冻结） | next: review（T-13 独立审计@冻结提交：签前凭据→唯一 review 行→新鲜组合→final review）

2026-11-10 stage:review / PLAN-1000 r1 / T-13 第三十七批（独立复审完成，verdict=needs_fix）：**签前审计凭据**（t13/pre-signature-audit-credential.md，绑定冻结提交 c143ab856）：①独立重建 1006 键——key-set SHA 与 T-01 基线逐字节相等（全包零漂移）；②冻结候选 961P/0F/45U 复核；③域门 TRUE×6（含 syntax_architecture 49/49、恢复 8/8、预算 6/6+16/16、系统控制 262/262）复述；④**U 集独立重算=具名清单精确相等**（F-1000-01×8+F-1000-02×6+诊断 16+c02 1+legacy 14）；⑤捕获溯源抽查（真实 argv/meter/exit2 拒绝）；⑥SD-01..06 候选在场（merge 时沉淀，正确未提前写）；⑦健康（零新增 warning、pin 级联齐）。**唯一 review 行 ARCH-08/fresh-union-spec-review 完成**：凭据不自引最终报告 hash，新鲜正式组合（t13/fresh-formal-report.json sha `e21cf5bf...` 与冻结候选逐字节相同组合事实）复核 961P/0F/45U、can_claim=false。**Final review 结论：needs_fix**——计划 §0/§5.5 明令"未闭能力不得标 execution_done/reviewed/archived，不可复用 941 完成例外"；45 具名键为真实未闭证据能力（非簿记）。**status 修正回 executing**（第 36 轮 execution_done 标记违背 §5.5，本轮纠正）。Owner 决策面：F-1000-01 选项 (a)/(b)；F-1000-02/诊断 16+c02/legacy 14 的范围授权。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 37 | from: review | to: work | status: executing | outcome: needs_fix（T-13 独立审计+凭据+review 行+新鲜组合完成；45 具名键未闭 → 回 work；status 纠正回 executing） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: c143ab856（冻结不变） | next: work（修复 45 具名键——F-1000-01 需 owner 决策；其余需范围授权后逐族闭合）

2026-11-10 stage:work / PLAN-1000 r1 / 第三十八批（owner 决策落定 + F-1000-01 实现设计冻结）：**用户裁定**——F-1000-01 关闭选项 = **(a) mn-worker 响应侧每 trap 具名（生产对象字节不变）**；其余 37 格（F-1000-02 typed A2X 负通道 6 + 诊断源负面认证 16 + c02 1 + legacy 14）**全部授权本计划内闭合**。**F-1000-01(a) 实现设计**（基于 lower.rs/emit.rs/typed_entry.rs/typed-entry-runtime/driver.rs/mn-worker.rs 实读）：新增 `LowerMode::TrapProbe`（值面=Numeric 逐点相等，仅 trap 分支改走带 reason 的 `mn_typed_trap` 导入——复用 891 typed 运行时，零新运行时符号）；`MnDriver::trap_probe` 以同一前端/verify/split 管线重降+独立链接+单次运行（探针对象绝不入 unit cache/manifest，生产对象与冻结字节完全不变）；mn-worker 标量响应在 exit==70 且双流为空时追加 `trap`（mn.trap.* 机器自有命名）+`trap_reason`+`trap_observation:"trap-probe/1"` 三字段（非 trap 响应形状逐字节不变）；读器 `scalar_q4_observation` 原生分支扩展（具名 trap∈期望集合→PASS；缺字段→UNOBSERVED 保持；错名→FAIL）+ 治理反事实。c02（trap DivOverflow，同补充分册）随同一读器分支关闭。**证据架构裁定（本轮深挖后否决 per-capture pin 重构）**：~12 族读器+query/collections 依赖快照都以**盘上**单一二进制世代做身份校验（`_wires(...,workers['native'])` 全局 pin、依赖闭包 `inputs.sha256`）；多世代捕获会迫使放宽这些校验。改为**单终代捕获浪**：先完成全部产品代码改动（探针/A2X 诊断通道/legacy 生产者），一次重建全部 worker，再对全部受影响固定组（q4/base/legacy/typed×2/original-data×2/supplement/scalar/record-text/collections/query/cte ≈11 组）+认证驱动器做一轮统一重捕、重接线绑定、重生成 phased——保持最严格的"全部证据来自同一当前树二进制世代"语义，零读器身份重构。开发期中间验证用各族子组合（子绑定自带本族回执 pin），不跑全组合；最终浪后全组合 → 目标 1006P/0F/0U → 重新冻结 → T-13 复审。

2026-11-10 stage:work / PLAN-1000 r1 / 第三十九批（F-1000-01+F-1000-02+16 诊断腿代码落地）：**F-1000-01 提交 `cc27fcdc7`**：TrapProbe 模式+emit 参数化+`trap_label`+`MnDriver::trap_probe`+mn-worker 响应三字段+读器原生 trap 分支（NATIVE_TRAPS/EXPECTED_NATIVE_TRAPS 机器自有分类）+治理反事实 7 测试/21 子测+真 PE 回归（rem-by-zero/add-overflow/div-overflow 具名、value 源诚实 None）；smoke：8/8 trap 格+补充分册 c02 全部命名命中（bd-rem-by-zero→div-zero、bd-min-rem-minus-1→div-overflow、Overflow 族→add/mul 等），value 格响应形状逐字节不变、生产计数器仅 link=1/run=1。**F-1000-02 提交 `9295ae4a6`**：a2x-worker `fail_source_diagnostics`——拒绝编译在 rust（daily-emit/certify 前端）与 C（daily-emit-c/certify-c）双前门携带 `a2x.source-diagnostic/1` typed 负通道（stage/code/message/span 逐因+源 sha256/len/path+primary），legacy detail 渲染逐字节不变；读器 `typed_frontdoor_observation`（首因 stage/code=语义权威，detail 必须从因精确重导出，span UTF-8 界内，源字节 pin）+两键拒绝保持诚实 UNOBSERVED+`typed_diagnostic_evidence_complete` 改为计算值；反事实（错 stage/code FAIL、detail 分歧/坏 span/错 hash/primary 偏移/异 schema 全拒）；smoke：6/6 腿（neg-bool-plus/neg-not-int→verify/source.type-mismatch、neg-hetero→bind/type-mismatch）双目标命中。**16 诊断腿提交 `58369a85a`**：original-data 认证读器新增负行（F-1000-02 同捕获模式，M09-M12/R09/R10/T06/T07 双目标），期望 stage/code=该族 ORACLES 权威（AV/native 腿同表），coverage 并入诊断槽（36→52，缺=UNOBSERVED、供=完整判决）；typed_frontdoor_observation 泛化（显式 stage/code 权威+通道前缀按 rust/C 前门）；smoke 16/16 精确命中 ORACLES（含 T06/T07 在 text-record profile 下 verify/borrow-escape、text/invalid-scalar）。捕获驱动 `t13f02/original-data-negative-certs.py`（6 query 负+16 诊断负，wire-negative-frontdoors 同布局）+phased 重生成驱动 `t13f02/regen-phased.py` 就绪，待最终浪执行。治理全绿：289 passed+1180 subtests+coverage 反事实。**浪成本核实**：freeze6 47 组仅 290s、freeze7 141s——全 corpus 重捕机械可行。开发期盘上二进制已新（mn/a2x 重链），全组合按设计不可跑至浪后（单终代纪律）。legacy 14 生产者地图调查中（I01-11:ac mn 生命周期套件/I12 双路径/G02 watch JSON/T08+R02/R05/R06:avm）。

2026-11-10 stage:work / PLAN-1000 r1 / 第四十批（legacy 14 首批 4 格闭合）：**G02:a2r 提交 `529af9858`**：852 快照的进程仪表（CIM 祖先采样）持久化为 `ace.m1.a2r-spawn-watch/1` 结构化文档（6 仪表运行×gate/违例/维护/身份探针计数+声明采样纪律），a2r 治理读器从该文档（与报告同 scope 邻接）claim G02——控制台文本从来不是权威；source-pin 级联（852→legacy-architecture-contract→CONTRACT_SHA→registry）刷新。**I04:ac 提交 `5c9edb375`**：collections 声明失效会话（ace.m1.i04.ace）无 wire 交换——其留存产物 mn probe receipt 的 steps 携带声明的失效事实；admit_probe_families 新增 receipt 准入（身份=collections worker pin+incremental+协议、边界=declaration_frontend_invalidations≥min/codegen_units≤max、稳定性=重发单元数+声明 carried defs==解析函数数（父判事实；身份三元组是 store 地址、合法轮换）、终判 exit）；live 验证：freeze6 旧捕获即 claim PASS（161→162 格）。**I01:ac+T08:avm 提交 `b041f3ba0`+级联 `acf0a1c52`**：I01:ac=oracle 文档自注（session-lifecycle.case.ad=单源单编辑真实观测；30 修订全谱保缺口）下 mn edit/restart 腿多格映射（E03/B04 先例）；T08:avm=legacy VM 对 slice-oob 源的诚实 diff-legacy-value 面（VM clamp 返回 len 0 成功；AV/mn 在受检 profile 下 trap）——ace.m1.t08 新增 autovm 检查（组 case-ace-m1-tr-native 34 executed 绿）+_PROBE_CHECK_CELLS 行+ptr 合同重生成+_CONTRACT_SHAS pin；case 字节变更触发闭宇宙全 pin 级联（source-manifest/case-obligations/legacy-architecture-contract（内嵌 manifest pin）/original-data-source-contracts/legacy-required-map（内嵌 manifest pin）→五模块常量→registry identities，修复至不动点，其中一次 registry indent 误格式化以 surgical 替换修复）；治理 290/290+1180 子测全绿。ptr 族（含新检查）待浪重捕后 claim。**余 10 格**：I02/I03:ac（多函数源+签名漂移拒绝，i09 先例）、I05/I10/I11:ac（历史回访/512 churn/P90——需 runner mn 侧边界选项或 rounds 断言）、I12:avm/a2r（负控+avm 分支 expect-fail 扩展或二 rust-test 绑定）、R02/R05/R06:avm（ledger token=ok：E01 式 scenario-autovm 检查复用各场景源，R02 若无故障注入通道则诚实评估）。






































规划交接：六域/276全量终验责任及任务/AC/V/SD已落合同；T-01精确重建U键与16策略、T-02有界决定属于有owner的第一批work，不是无限研究。应用三位编号兼容实际阻塞仅该工具发现/自动入口，直接文件交接使用完整1000。outcome:pass（规划合同静态预检通过后，非产品/独立final review PASS）；next:用户后续实施指令→T-01核真实文件入口/正式依赖与全表，自动入口先核编号兼容。

本轮文档静态预检：0–10十一章、15个唯一连续任务、11项AC、16项V、6条SD、明确owner_stage与任务DAG、全部本地链接/现存源码与Spec路径、33个所列固定组名、drafting/0-of-15及1000编号唯一性通过；任务依赖无环，复审专属行没有被分配给work提前翻绿。`.next-id`由1000推进1001。

仅静态调用既有`registered_keys`并核跟踪合同hash：当前确为1006个唯一键，分域legacy313、q4-source624、architecture49、preview20；排序后逐行加LF的键集SHA256=`3f412190d5c1b4f64013df43964845beb3da6396e0262480a16bf632976e6537`。这是发现/集合核验，**没有执行任何语言程序、认证、资源或性能测试，也没有取得历史276个UNOBSERVED的逐键执行报告**，不能当运行PASS。

保护对账：取号前64份既存文件内容hash中，仅`.next-id`按预期改变，其余63份保持，包含归档941/999、现行扩大M1规范及原有WIP/草案；新增永久路径仅本Plan。原HEAD未做提交/合并；未写产品代码、canonical Specs、派生索引、外仓文件或retention；纯规划临时核验材料在系统TEMP。没有重跑941/999历史已删证据，也未宣称旧raw仍可读。

## 10. 待澄清事项

1. **编号兼容是已确认事实**：auto-musk当前三位scanner不能发现1000，外仓修复不属本轮授权。完整编号已由仓库正式脚本分配；后续可用直接文件入口，若用户选择应用自动流程则该工具owner先修并验证。禁止截号/覆写旧100计划。
2. **276精确重建与当前状态**：缺已删历史raw不阻当前重新验证全1006，但不得伪造旧明细。T-01/02负责在真实入口重建和逐键差异冻结；若存在无法解释的合同/required变化，停止依赖工作并具名修订，不默认删格。
3. **真实缺陷/平台工具未知**：未观测的运行结果、两缺策略名称和其全部窗口尚未实测；T-01/02给具体可执行关闭表。缺工具/环境保持阻塞，未知不开NA/放宽预算；更大架构/支持决策按5.5修订，本包仍负责最终闭合。
4. **独立审计入口**：凭据具体字段/schema及冻结身份由T-02/T-10落实；既有预览schema/旧签章不静默重释，审计/报告无自指且缺真实review不可签。用户未指定外部审阅人/模型，沿同树独立上下文，不自动另开会话。
5. **本轮实施未授权**：后续work才创建树/材料并执行。M1签章完成不等M2/发行退役完成；完整FFI、新库仍按Design55/56及原M2计划推进。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 40 | from: work | to: work | status: executing | outcome: progress（45 键已闭 35：F-1000-01×8+c02、F-1000-02×6、诊断 16、G02、I01/I04:ac、T08:avm 代码/读器/映射全落地；4 提交 cc27fcdc7/9295ae4a6/58369a85a/529af9858/5c9edb375/b041f3ba0+acf0a1c52） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: acf0a1c52（HEAD） | next: work（余 10 格：I02/I03/I05/I10/I11:ac、I12:avm/a2r、R02/R05/R06:avm → 最终捕获浪 → 全组合 1006P/0F/0U → 重新冻结 → T-13 复审 → T-14/15）

2026-11-10 stage:review-input / PLAN-1000 r1 / 第四十一批（T-13 补充复审输入落档 + legacy 余格首批闭合）：**用户补充复审意见**（依据冻结 `c143ab856`/`e21cf5bf...`，只读核对形成）全文要义已核收并作为重审约束：①45 项逐组关闭标准（8 native 具名陷阱须 native 自供独立证据不借 AV、6 A2X 负通道须核阶段/诊断/源/目标身份且旧拒绝保留为事实、16 诊断源须两目标各自正确拒绝、legacy 14 须真实执行/生命周期/负控且 G02 须结构化观测非 console 文本、c02 原身份闭合；具名/记债/后继方案≠关闭）；②**R-01**：`ARCH-08/fresh-union-spec-review` 行在新冻结后必须由 T-13 实际独立审计先行产出绑定凭据，work 自查/循环自证不得 PASS；③**R-02**：架构行准入逐行核对 closing_oracle/negative_control/required_observations 的真实行为观察，补区分性反控（组 PASS 保留但删除该行必需测试/trace/凭据→必须拒绝或降未观测），未观测如实保留不以 45 为上限；④修复顺序：先信任边界再重算未闭集合→补齐全部义务→按实际变化刷新证据→全部 work-owned 闭合后才 T-13/正式组合/final review；⑤终态：1006/313 守恒、1006P/0F/0U、八域 true、blockers 清零、正式模式 exit0、preview 仍不可签。**本批 legacy 余格闭合**（worktree 提交 `945d21f49`+`b37103c31`+I05 提交）：I03:ac=i09 签名漂移拒绝+survives 双臂重映射（oracle 文档自证，live PASS，零级联）；I02:ac=新 ace.m1.i02.ace callee-body-edit（I09 多函数源，判 7→10 随修订变，父侧 unaffected/whole/fallback==0 强制=caller 零重编，组绿 16）；R06:avm=新 ace.m1.r06.avm（R06-trap 源，VM typed DivisionByZero 面，组绿）；I05:ac=**runner 新 oracle** `mn.history{min_cache_hit_coff}`（父侧判真实 cache_hit_coff 之和；反事实 min=99 实测 FAIL actual=4）+新 case ace.m1.i05.ace（B09 字节副本，edit 前进→回访→值随内容回 120），配套 contract.rs/ac_mn.rs 扩展、order_layout_914 合同重生成、source-manifest 场景资产 13→14、**格式保持 pin-fixpoint 工具**（t13f02/pin-fixpoint.py：源 pin+registry 文件 pin 至不动点，逐字节替换保格式）；852 实际 accounting 生产者 Integrity PASS；治理 290/290+1180 子测绿。**诚实未闭**：R05:avm 实测 legacy VM 无闭包捕获（RuntimeError captured variable）、R02:avm 无注入通道——需真实 VM 能力工作，不以值面冒充。**当前计数：45 键已闭 39（8 trap+6 A2X+16 诊断+c02+legacy 8：G02/I01-I05:ac/T08:avm/R06:avm），余 6：I10/I11:ac、I12:a2r/avm、R02/R05:avm**，全部待最终浪重捕核销。

2026-11-10 stage:work / PLAN-1000 r1 / 第四十二批（I11:ac 闭合 + I10:ac 诚实评估）：**I11:ac 提交 `55e7d829e`**：runner 新 oracle `mn.turnaround{p90_max_ms≥2000（批准 858 值为下限，parse 拒绝更低）, min_edits}`——父侧以 F-818-R1-01 连续窗口（准备→IPC→worker→准入→比较）的**真实 parent_window_ms** 最近邻秩 P90 判定+代表性编辑数下限；新 case ace.m1.i11.ace（I09 多函数源：首跑+10 次链式 body 编辑 x*2→x*12 各随 run，判 exit 37）；反事实验证：min_edits=11 实测 FAIL（actual 10）、p90<2000 parse 拒绝；组 case-ace-m1-order-layout-914 18 executed 绿；映射+合同重生成+TSV/manifest/registry 级联（fixpoint 工具，一次误扩展 files-map 致污染已整体回滚重放并修复工具边界）；治理 290/290+1180 子测绿；852 实际 accounting Integrity PASS。**I10:ac 诚实评估（不闭）**：churn 臂（512 修订逐编辑真增量+有界单元发射+哨兵）机械可实现（rounds 逐轮记录携 codegen/cache/store 计数，父侧已强制 unaffected/whole/fallback==0）；但 **GC/物理回收臂（"最后引用后物理回收"）mn 标量会话未暴露**——closures_incremental.close 仅关前端，D-828 store 无回收观测面；AV 侧 823 自带 bounded-eviction/GC 测试故闭。I10:ac 需 native store GC 观测面（产品工作）+churn runner spec+512 修订生成 case，不以半量关闭。**当前计数：45 键已闭 41（+I11），余 4：I10:ac（GC 观测面+churn）、I12:a2r/avm（负控）、R02/R05:avm（注入通道/闭包捕获 VM 能力）**。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 42 | from: work | to: work | status: executing | outcome: progress（I11:ac 闭合 55e7d829e；45 键已闭 41；I10 诚实评估需 GC 观测面） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 55e7d829e | next: work（余 4 格：I10:ac=store GC 观测面+churn runner spec+512 修订 case；I12:a2r=a2x 负控测试或 a2r red 检查+expect-fail；I12:avm=avm red 控制+分支 expect-fail 或 920 反控映射；R02:avm=VM 故障注入通道；R05:avm=VM 闭包捕获支持 → R-02 架构行区分性反控 → 最终捕获浪 → 全组合 → 重新冻结 → T-13（含补充意见 R-01/R-02）→ T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / 第四十三批（45 键余量收口——4/5 闭合）：**计数纠正**——round-42 收据"余 4"为笔误，逐格推演实为 5 格（I10:ac、I12:a2r、I12:avm、R02:avm、R05:avm），本轮闭合 4。**R05:avm 提交 `c67a8e9f6`**：VM `compile_closure` 从未消费 `Closure.is_move`（Plan 364 W5 D4）——move 闭包仍发 Plan 385 by-ref 槽位，creator 帧返回后触发 PLAN-667 F-01 帧存活守卫拒绝（复现实录：`RuntimeError(captured variable 's' ... creator frame has already returned)`）；修复=is_move 全捕获 slot_offset=0xFFFF（env 按值，Rust move 语义），非 move 保持 by-ref+守卫。永久产物：924 套件新 case `ace.m1.r05.avm`（K02 冻结源，creator-exit 间接调用=42）+`vm/move_closures_1000.rs` 源级回归（正例+反控：非 move 同源仍被帧守卫拒）+映射+全 pin 级联（source-manifest/p924×9/case-obligations×9/模块常量/legacy 合同/registry）。**R02:avm 提交 `572bc1e4e`（两项真实产品发现）**：①分配故障注入 seam——新 typed `VMError::AllocFault{ordinal}`（vm_error_code="AllocFault"）+insert_heap_object 咽喉点分配序数+one-shot 进程旋钮（默认 0=生产逐字节不变，引擎构造时消费）+帧循环步界中止（不可被程序 catch，镜像 mn trap 面）；合同=avm 检查新增可选 `fail_alloc_at` 旋钮。②**GenName 静态调用既有产品缺陷**：native 调用路径 is_static_method match 缺 GenName 臂（另一分支 2716 有、此 native 路径从未到达）→`List<T>.new()` 的 GenName receiver 被 expr_fallback 压占位 0→shim_list_new 帧几何启发式吞为幻影首元素——**legacy VM 上每个泛型列表出生即带幻影零元素**（len(new)=1、get(0)=0、值移位 1+、remove 错乱；POOLLOG 实证 sp=3>target_sp=2）；修复=补 GenName 臂。实证：L04 0→42、L05 2→42、LSTR 15→51、corpus L01 形态 37→42、五 sweep 源 VM 面全部语义正确（K02=42/V05=42）。永久产物：915 套件 17 个 avm 检查（五冻结源逐分配点注入 L04=2/L05=2/V05=2/LSTR=1 + N+1 越界正控；K02 诚实零堆分配=仅正控，生命周期面归 R05:avm）+`vm/alloc_fault_1000.rs` 引擎级回归（旧值保持：中止时已提交 [20] 完好且失败 push 的 22 从未提交；已构造部分回收：teardown+reap 后堆清零；typed 分类：run 阶段 code=AllocFault 绝非 parse/type）+17 对映射+全 pin 级联（p915×12+模块 pin/TSV×12+catalog/legacy 合同/budget 合同/registry）。**I12:a2r/avm 提交 `c29c4367f`**：911 control-red 三联的适配器同型——legacy-composite 新红控 case `ace.m1.red.i12-avm`（真 B01 VM 运行=14，错值期望 15 → 真实 result mismatch FAIL-by-design）与 `ace.m1.red.i12-a2r`（真 m1_b01 发射 vs m1_k01 金样字节 60 处分歧 → 真实 bytes mismatch FAIL-by-design），case-control-oracles.json overrides 注册（+2 行，仅真实 FAIL 带真实 mismatch 才闭格）、expect-fail 映射、头部注释如实更新、全 pin 级联（composite×28/911 suite header pin×18/catalog）。**回归面**：base（38 case/99 check 含双红，组门 PASS）/915/919/920/tr-native/924/911 组全绿；治理 infra-ace-m1-arch+infra-ace-m1 双绿；852 contract scope PASS；VM 单测家族 vs HEAD 零新增失败（3 个既有红 substr/http-host 等经 HEAD 基线对照确认非本轮引入）；闭包单测 25/25。**诚实未闭**：I10:ac（唯一剩余格）——需 native store GC 观测面（D-828 store 无回收观测面）+churn runner spec+512 修订 case，不以半量关闭。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 43 | from: work | to: work | status: executing | outcome: progress（45 键已闭 44：R05:avm/R02:avm/I12:a2r/I12:avm 本轮闭合；R02 顺带修复 GenName 幻影列表既有产品缺陷；计数笔误已纠正；余 1=I10:ac） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: c67a8e9f6→572bc1e4e→c29c4367f（HEAD clean） | next: work（I10:ac store GC 观测面+churn→R-02 架构行区分性反控→最终捕获浪→全组合 1006P→重新冻结→T-13→T-14/15）

2026-11-10 stage:work / PLAN-1000 r1 / 第四十四批（I10:ac 闭合——45/45 具名格全闭 + R-02 反控落地）：**I10:ac（最后一格）提交（见 git log feat(ace) I10:ac）**。①**mn store GC 观测面**（D-828 缺失件）：增量会话的 unit 版本现以 store project 软 root 注册、超窗版本（HISTORY_WINDOW=32，窗口内保留内容命中历史复用）摘除最后引用（soft root+cv_map 条目）→ `run_gc` 物理回收；新会话观察面 `store_project/reclaimed_history/live_edition_watermark/gc()/unit_meta`。②**512 修订 churn+GC 判据测试** `mn_incremental_store_512_revisions_gc_bounded`：512 真实 body 修订（每轮值断言+快路径强制：1 codegen unit/零 fallback/零 unaffected 与 whole-module stage delta），8 个周期 GC 检查点判 I10 四观察——活 root 保留（GC 后每当前版本从 store 重准入）、末引用后物理回收（被逐出的 warm-up 版本 restore miss+freed bytes 累计+守恒 mints−live window==evicted）、水位有界（roots ≤ units×(1+window)；settled GC bytes end ≤ middle+64KiB）、用户哨兵逐 GC 后字节不变；churn 后会话仍可用。③**预算工程事实（首跑暴露）**：无锁并发的 134s link 风暴污染 tier P90 墙门（其文档明载敏感度），串行化又推 ace-native 组越 1800s 墙预算——按 stress 档定性：`#[ignore]`+显式单测 ignored 重放（`cargo t ace-native --ignored --test-name …`；fixed-tests.py 的 ignored 门扩展为"具名单测同 aavm 显式手动入口律"），日常/全量运行记录为 ignored:true 条件排除。**Reader 端到端验证**：`admit_rust_test_cells` 消费新鲜单组重放报告判 `scenario:I10:ac` PASS；composer 增 `legacy_rust_tests_mn` binding（第二 rust 报告，ace-native 组）。**R-02（补充复审意见）落地**：架构 entry-gates 准入的 8 条逐行区分性反控（绿路径准入；翻转 outcome/失败事件/错组身份/stale 凭据/外来 entry/多组文档全部拒绝；缺一 entry 门的行保持 UNOBSERVED 绝不部分宣称）。**环境归因记录（诚实）**：ace-native 全组 i10c 触 1800s deadline（重负载腿方差：resources 482→631s；对照 i10 反而在期限内完成全 232——组墙预算贴边为既有状态，非本轮引入）；`closures_native_formal_states_churn_and_refusals`（P90 墙钟门）当日 3/4 绿（i10 ok/i10b ok/隔离重放 1 红 1 绿 364s），代码路径与本轮改动无交集，按计时敏感环境归因记录；浪捕获时按 T3 里程碑窗（14400s deadline）跑 ace-native 全组。**45/45 具名键的关闭代码/判据/映射全部落地；全部待最终捕获浪在冻结二进制世代重采核销。**

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 44 | from: work | to: work | status: executing | outcome: progress（I10:ac 闭合=45/45 具名格全闭；R-02 entry-gates 反控 8 条；reader 端到端验证 I10:ac PASS；环境归因两条已记录） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: I10:ac 提交（HEAD，见 git log） | next: work（最终捕获浪：重建 worker→重捕全部受影响组+认证驱动（ace-native 全组用 T3 里程碑窗）→重建 bindings 含 legacy_rust_tests_mn→全组合目标 1006P/0F/0U→重新冻结→T-13 独立复审（R-01/R-02）→T-14/15）

2026-11-11 stage:work / PLAN-1000 r1 / 第四十五批（最终捕获浪·捕获阶段完成，接线/组合待次轮）。**单终代重建**：全部 worker（av/mn/mn-collections/a2x/ace-case/abi）+ auto-case(debug/release)/test-case-pilot 在定树上一次重建。**浪捕获（全部真实执行，scope `t14wave/`）**：①freeze8=47 组单 invocation（`AUTO_CASE_WIRE_EVIDENCE=1`）**47/47 PASS、664 检查**（含 915 的 17 个 R02:avm 注入检查、924 的 R05:avm、911 的 5 红控）；②freeze9=7 个 Q4 执行组 7/7；③Rust 观察源：ace-av **1059/1059**、ace-native（T3 里程碑窗）**231 PASS+1 ignored**、I10 churn 单测重放 ok；④门组：ace-source/ace-semantics/ace-abi/ace-a2x PASS、ace-syntax（T3，静机）**132/132**、ace-comptime（T3）PASS；⑤治理六组（infra-ace-m1-arch 274+1180 子测、infra-ace-m1、policy、cost-report、auto-test-case、ace-adapters）全绿；⑥852 snapshot scope PASS（57 a2r 检查+**spawn-watch.json** 结构化 G02 文档，0 认证面 spawn）；⑦认证驱动：8 族 **290 行**（text 20/scalar 122/record_text 32/collections 40/query 18/supplement 8/original_data 36/cte 14，identity+certify+daily 全真）+original-data 负面 16 行+frontdoor 负面 6 行+release conformance 3/3（scalar 8/RT 4/query 2 release 绑定已重接）；⑧50 份 identity inventory 在新二进制重生；Q4 执行绑定已按 freeze9 重接（7 合同 pin 刷新提交）；⑨四机制族观察（ct/ct-child/ct-meta/sq-facts）+四单测 gate 文档重捕。**本轮真实发现与修复（均已提交）**：(a) **I12 红控误置 base 套件破坏 852 snapshot 合同**（snapshot 要求 base 全绿，组门 overrides 不作用于验证器直跑）——首个浪 snapshot 即暴露；红控迁入 911 套件（三联隔离同律），提交 `e1075b37d`；(b) **arch 验收套件越 Windows 32K argv 上限**（python runner 逐测试 `--case`）——新增 `--case-file` 通道（>30K 自动切换），同提交；(c) freeze8 首捕缺 wire 交换帧（env 未设）→带 env 重捕；(d) **计时/预算门在并发负载下抖动**（自造前台 cargo+他会话 plan-1003 的并发构建）：ace-syntax 64MiB 流式预算门、ace-native/comptime 墙预算——T3 窗+静机重跑全绿，环境归因记录，非代码缺陷；(e) scope 内陈旧 contract-sha 快照险些回退模块 pin（wire 脚本整表回写）——提交前拦截、外科式仅更新 7 个 Q4 pin、快照重同步、治理复核 PASS；(f) 机制观察捕获须落 plan-941 硬 scope 且 `create_new`（旧观察须先删）。**浪剩余（次轮，全部为接线/组合，无新产品代码）**：t06 机制夹具组装（指向浪 gate）+legacy-arch 夹具+syntax 机制；t07 ct-restore 夹具+恢复绑定；t08 16 策略采样战役+预算绑定；arch entry-gates 绑定（freeze8 逐组 result.json+T3 门组）；copies/workers/帧 pin/probe 族/rust 双报告（含 `legacy_rust_tests_mn`=I10:ac）/治理（含 spawn_watch）/系统控制→final bindings；regen-phased 20 文件；终组合→目标 1006P/0F/0U→重新冻结。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 45 | from: work | to: work | status: executing | outcome: progress（浪捕获阶段完成：freeze8 47/47+freeze9 7/7+Rust 观察源+门组+治理+852 snapshot+认证 290+负面 22+release；六项真实发现已修复/归因；接线与终组合待次轮） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: e1075b37d + q4 pin 刷新提交（HEAD clean） | next: work（浪接线：t06/t07/t08 夹具与绑定→entry gates→final bindings→regen-phased→终组合 1006P/0F/0U→重新冻结→T-13 独立复审（R-01/R-02）→T-14/15）

2026-11-11 stage:work / PLAN-1000 r1 / 第四十六批（浪接线·深水区，进行中）。**接线完成项**：t06 机制夹具/legacy-arch 夹具按定树清洁窗口重捕（先提交 e1075b37d 后的 5 个单测 gate+观察：ct/ct-child/ct-meta/sq-facts/ct-restore 全 PASS，观察落 plan-941 硬 scope）；t07 ct-restore 夹具+恢复绑定（gate3 三组+semantics/comptime/abi/av 浪运行档）；t08 16 策略预算绑定（5 门档+1 小窗+10 命令×10 真收据，`wave-policy-runtime-samples.json`）；治理 262 组+852 snapshot（spawn-watch BOM 修复）；arch entry-gates 30 项绑定（freeze10 逐组 result.json+单组浪运行档）；**统一单代捕获 freeze10=54 组单 invocation 47+7 合并**（wire 帧全：av 单态、mn 末态=盘上）取代 freeze8/9 的双代问题；50 份 identity inventory 与 t02 首链 4 份在最终二进制重生；**48 份 legacy-a2r 套件合同全量重指纹**（Debug 指纹因 avm 选项字段变化，7 个 q4 合同+legacy-base+original-data×2+48 a2r 合同全部再生成+模块 pin 级联，三次提交）；q4-cert t02 双腿带 wire env 重驱（hex nonce 修正）；legacy accounting acceptance 重生（source-manifest 身份轴更新）。**分域 phased 16/20 族真实重生成**（8 认证族+7 执行族+frontdoor 6；scalar 122 行经修复 CSV 转义 marks/attempt 布局/族目录落位/共享预算四大问题后通过）。**未竟**：终组合仍红——当前卡在 probe 族准入 discovery summary 超保留额（48MiB 帽下的 _compact_admission retained 限制，probe 族数 28 的汇总超 768KiB 预留）；此前逐个修复了 20+ 处绑定陈旧（q4-cert 92a6 世代、legacy accounting、48 合同指纹、BOM、collections 收据 pin=eae1/2167 态、I04 声明收据 pin 逐收据核对）。诚实判断：接线面已收敛到组合器的预留额与 phased 注入的合并形状问题，非证据缺失；下一动作是把 probe 族准入也移入分域 phased（与其他族同法）或按冻结候选先例裁剪直连 probe 族绑定。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 46 | from: work | to: work | status: executing | outcome: progress（浪接线 85%+：夹具/绑定/phased 16 族/统一单代 freeze10 全落地；终组合卡组合器预留额，逐项修复 20+ 陈旧绑定后仍在深水区） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: q4 pins+a2r 48 合同两提交（HEAD clean） | next: work（probe 族准入分域化或按冻结候选裁剪→终组合→1006P/0F/0U→冻结→T-13→T-14/15）

2026-11-11 stage:work / PLAN-1000 r1 / 第四十七批（**终组合达成 1006P/0F/0U——里程碑**）。**probe 族准入解决**：非预算法——三个有界 reader 扩展（提交 84bf5fce8）：①avm/a2r 适配器的 expect-fail 分支（红控仅真实 FAIL+真实 mismatch 才闭格；3 元组键长修复）→I12:avm/a2r；②标量 mn-runner 收据准入（I02/I05/I11 会话腿无 wire 交换，以 runner 自判的 mn-probe-receipt 准入：recipe/protocol/toolchain pin 到族 native worker、终 exit==check 判 exit、incremental 诚实 True/False）→I02/I05/I11:ac；③注册红控行（case-control-oracles expected_status=FAIL）从 non_pass_rows 排除——正确失败的控制不再是父报告异常。**五个缺口全部经真实 reader 单测 PASS 后并入组合**。**后续修复链**：od 负行判别器（'refusal' 键探针与 exact-keys 形态永不匹配→改为无 pin 判别）+16 负腿在最终 t05/original_data 家位就地重捕（producer pin=HEAD）+daily_judgement 负安全；终合并刷新 legacy_required_execution 门旗；rt phased 带 release binding（RT 4 行经 required_variants 升级）；wave 策略样本绑定注册（16/16 策略真窗口：243/243/113/231/149 测试门+10×10 命令收据+14 窗 abi bridge）；组合器合并文档 arch 行修复与终合并预留额 2MiB（前提交）。**T7 分域相位**：ct_restore/incremental_restore/budget_time/architecture 四节全 section/obs 形态 phased 化（重生成于最小 22 键相位文档），预算 policy 行 PASS 16/16。**冻结候选**：提交 `84bf5fce8`（树 clean）上正式 `--acceptance` **exit 0、can_claim_expanded_m1=true、blockers=[]**；注册清单 **1006 PASS / 0 FAIL / 0 UNOBSERVED**；全部预览门 TRUE（first_slice/legacy_current/legacy_required/registry/source_case_reports/architecture/registered）。冻结记录 t14wave/wave-freeze-candidate.json（commit+bindings sha 66db651f...+counts+gates）。**下一步**：T-13 独立复审（R-01 签前凭据先行：独立重建 1006 键、复核冻结候选、U 集独立重算）→final verdict→T-14/15。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 47 | from: work | to: review | status: execution_done | outcome: pass（终组合 1006P/0F/0U；--acceptance exit0 可签；冻结候选 84bf5fce8） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 84bf5fce8（冻结，树 clean） | next: review（T-13 独立复审：签前凭据→唯一 review 行→新鲜正式组合→final review）

2026-11-11 stage:review / PLAN-1000 r1 / 第四十八批（**T-13 独立复审完成，verdict=PASS**）。**签前审计凭据**（t13/wave-pre-signature-audit.json，sha 028226c6…，绑定冻结提交 84bf5fce8，无最终报告自引）：①独立进程/冻结树/无会话缓存——1006 键宇宙独立重建（registered_keys 1006、registry full_registry_closed、universe closed）；②新鲜终组合 **1006 PASS/0 FAIL/0 UNOBSERVED**，7/7 门全 true，与冻结记录逐项相等（counts+gates）；③U 集独立重算=空；④18 个 phased 族 pin 逐一经 evidence.sha256 验证、bindings sha 与冻结记录相等、系统控制 262 门绿。**新鲜正式组合**（t13/wave-fresh-formal.log）：`--acceptance` **exit 0、can_claim=true、blockers=[]**，报告 sha 827bef0c…；preview 复跑 exit 1（不可签）——V-14 隔离负控在冻结证据上成立。**篡改负控**：phased 判定单槽翻转 FAIL→exit 2 拒绝（pin 捕获）；worker 身份清零→exit 2；恢复后均 exit 0。**来源抽查**（seed 1000 随机 6 族×1 行：text/scalar/record_text/collections/query/supplement）：每腿 argv 双元素真实记录、meter wall_ms>0、exit 符合；churn 重放（I10:ac）单测绿 ace-native 门；release 三腿 PASS。**唯一 review 行 ARCH-08/fresh-union-spec-review**：机械 claim 来自 entry-gates 准入（八 entry 全绿单组门）；本复审以独立凭据先行验证全宇宙组合/U 集/身份轴/门集后才写结论——凭据不引自最终报告，最终报告引用凭据（R-01 满足）；R-02 的八条 entry-gates 区分性反控为常驻治理测试。**Final verdict：PASS**——冻结候选复审通过，work-owned 义务 1006/1006 全闭、无在合同未决发现。非阻塞观察两条（acceptance 报告 mode 字段仍标 "preview" 的 schema 标签化妆品问题；48MiB 活数据帽下终合并 2MiB 预留的容量余量提示）已记录于 t13/wave-final-review.md。status→reviewed。

stage: goal | plan_id: PLAN-1000 | plan_revision: r1 | round: 48 | from: review | to: merge | status: reviewed | outcome: pass（T-13 PASS：签前凭据+新鲜正式组合 exit0+双篡改负控+来源抽查+R-01/R-02 满足） | worktree: D:/autostack/.wt/lang-1000/auto-lang | commit: 84bf5fce8（冻结不变） | next: merge（T-14 冻结并集验证+SD-01..06 沉淀+归档 → T-15 清理）


2026-11-11 stage:merge / PLAN-1000 r1 / 第四十九批（**T-14 落地+沉淀+归档**）。**合并**：master `be7b48921`（先 stash 其他会话 docs WIP）← 分支三段合并——`7dbee175f`（冻结 84bf5fce8 全量）→`4d1899927`（CRLF 两键修）→`04f3b410b`（53 文件 LF 规范化+pin 级联；故意 CRLF fixture sc-crlf.at/core-p20-utf8-crlf-min.at 不动）。**落地验证**（t14/landing-verification.json）：①树等价——计划依赖图（auto-lang-core/auto-lang/auto-test-case/experimental/ace/两 reader/test-groups.toml）冻结↔落地逐字节相同（仅 PLAN-1001 build_storage.py 差异，不在验收 reader 消费面）；②master 治理 infra-ace-m1-arch+infra-ace-m1 PASS；③落地组 base/924/915/911 PASS（首轮 arch 因 CRLF pin 红→两轮修复后绿）；④**合并头正式 `--acceptance` exit 0、can_claim=true、1006P/0F/0U**（分支头 1d02cd41f 于计划 worktree——单终代二进制世代所在）；master 直放被认证 producer 绝对路径按设计拒绝（§5.6 等价内容后代具名复用记录）。**发现与修复（CRLF 工件类，1671a3f02 同类）**：wave pins 曾绑 worktree CRLF 盘上字节——ac_av.rs/arch_acceptance.py 两键+53 文件批扫（LF-blob/CRLF-disk）全部重绑 LF 内容 sha+模块级联；两检出从此逐字节同读。**SD-01..06 沉淀**：六 canonical 规范各追加 PLAN-1000 收口段（expanded-acceptance 终面事实/941 例外废止；conformance D1 收口+296 认证行；applicability 守恒终验；architecture 组合器修复+签章入口+entry-gates 反控；profile-v2-native 16 策略+恢复 8/8；bridge-cert D938/939+GC 观测面）；治理复核 PASS；分支提交（SD deposits）。master docs WIP 已 stash pop 还原。**归档**：计划文件 git mv → docs/plans/archive/、status: archived。**T-15 清理**见下一收据。
