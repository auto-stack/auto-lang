---
plan_id: PLAN-1005
status: drafting
feature_name: ACE M1架构行证据准入与独立审计凭据有界修复
author: [Codex]
created_at: 2026-11-11
updated_at: 2026-11-11
plan_revision: 1
current_step: 11
total_steps: 11
depends_on: [PLAN-1000]
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-expanded-acceptance.md, docs/specs/auto-test-case/architecture.md]
new_spec_components: [docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md]
touched_goals: [GOAL-002]
affects: [auto-test-case]
---

# [PLAN-1005] ACE M1架构行证据准入与独立审计凭据有界修复

## 0. 变更摘要

修复打`v0.6-ACE-M1`前发现的两项真实准入缺陷：**F-1005-01：只有组名与绿色布尔、executed=0/空结果仍能供应架构行PASS；F-1005-02：没有独立审计凭据仍能供应`ARCH-08/fresh-union-spec-review`的PASS。** 范围限现有M1架构行准入、相同结果的合并/分域摘要入口、签前凭据消费与正式入口/反控；沿既有registry、reader、父执行器和原始oracle修，不造第二套验收系统。

PLAN-1000已归档，不重开、不回写其历史收据；保留其功能成果及当时1006P报告，**历史归档/报告不能豁免本次反例**。本包须以修复后的严格链重建可签证据，给出冻结主线`tag_ready`复核结果；仅记录问题、组全绿或换一个PASS摘要不能完成本包。

本轮仅规划：drafting r1，0/11。只写本计划和中央取号簿记；未施工/建实施树/运行产品测试/创建tag/推送/创建或联系其他会话，未写canonical Specs或修改1000。

## 1. 目标

1. 组结果证明“相关测试被执行”，架构行证明其原`closing_oracle`、`negative_control`与`required_observations`；必须通过当前来源的逐行可验证观察建立两者关系，不能以组PASS直接推出行PASS。
2. `executed=0`、结果空、缺必需case/观察、错schema/身份、互相矛盾的统计与事件均不得被绿色布尔覆盖；合法非零执行与有区分性断言的真实正例仍可通过。
3. 唯一独立审计行只由实际独立复审产生且被正式链读取、校验的凭据供应；缺/错/陈旧凭据保持不可签。实现自查、组绿色、`reviewer=independent`字样不授信。
4. entry-gates、专用机制reader、architecture分域摘要和最终合并均遵同一行级授信，未经准入的PASS不能经覆盖、缓存或优先级进入总门。
5. 修复后正式M1仍须1006个原required全部PASS、0FAIL/0UNOBSERVED、49个ARCH全闭、八域全真、完整性成立、blockers为空，并通过独立复审、主线身份核实及打tag前复核。

非目标：重做1000全部功能、完整FFI/新标准库/M2产品迁移、编译器/运行时大重构、PKI或用户权限系统、通用报告平台、全仓格式整治。只在行级观察确实缺失时补原producer最小输出或永久测试断言，不扩大原M1语言/平台/性能范围。原1006键、31合同身份关系、旧313 required/131行524格、批准NA/accepted/future、source/expected、独立reference、原样本/资源/性能阈值保持；不以缩分母/改expected/无依据NA通过。

## 2. 架构方案

数据流：原M1合同/当前源与工具身份 → 原固定组/worker/parent真实执行与原始终态 → strict gate-envelope准入 → 具名case/test与行级观察/受审断言映射 → 各行typed判决 → 同一架构合并器 → 全集候选（审计行pending）→ 独立签前审计凭据 → 原八域AND/正式报告 → 主线落地与tag_ready检查。

执行、观测和授权分层：原执行器拥有子进程/Job/输出；reader只读验证，不启动工具，不从expected生成事实。证据文件hash只证明内容身份，不独立证明运行/语义/审阅者独立性。独立复审流程提供可信的签前记录及期望hash/候选身份，reader核一致性与覆盖；不能让被验证payload自带一组可共同改写的“期望值”就自行授信。

复用`ace_m1_arch_obligations.py`的49行声明和`ace_m1_arch_entry_gates.py`，保留现有专用机制reader中实际有效的语义判决；只增加必要的严格封套/行证据/审计凭据小接口。只读组合器继续保持原48MiB活数据、4MiB单文档预算，分域消费必须读完即释放、计入同一预算，不能通过额外进程或第二meter绕开。

唯一正式依赖D1000：受审代码`84bf5fce8`，落地/修复`7dbee175f`、`04f3b410b`，规范沉淀`9e36d984e3d33aef27da6163b2017b2aaab2d258`；当前均为主线祖先。T-01重核正式交付及实际依赖；不消费已删1000材料路径冒新证据，不需要重开941/999/1000或接管其已清理树。

## 3. 技术栈

现有Python严格reader与unittest、固定`cargo t`职责组、Rust ACE测试/worker输出与原TestCase父裁判、Git冻结身份。首先修Python准入；若原观察数据不足，仅在真实owner补最小语义事实并跑其完整受影响组。拟唯一实施树`D:/autostack/.wt/lang-1005/auto-lang`、分支`codex/plan-1005-dev`，未创建；work、内查、独立复审顺序共用该树，不另建review树或junction。

## 4. 需求分析与背景调查

用户2026-11-11授权：“正式新建一个有界后继计划，修复……架构行证据准入和独立审计凭据缺失仍可PASS……明确反控、验收条件及打tag前的复核要求。本轮只规划，不施工，不重开1000，不启动其他会话。”授权仅本仓规划，无模型/工期/代理预算，不推定本轮实施或打tag授权。

从[Specs overview](../specs/overview.md)导航，读取[当前扩大验收](../specs/auto-test-case/ace-m1-expanded-acceptance.md)、[裁判架构](../specs/auto-test-case/architecture.md)及[归档1000](archive/1000-ace-m1-execution-evidence-restore-budget-and-final-signoff.md)的实际交付、独立审计、主线等价与清理收据，核代码而不只采信总结。调研主线HEAD=`9e36d984e3d33aef27da6163b2017b2aaab2d258`。

### 已实证反例与规范差额

上轮tag核验中直接调用生产`admit_arch_entry_gates`，只在内存构造每个相关entry的以下材料，计算其真实SHA256作为binding；未执行任何语言程序，也没有PID、资源、审计事实：

```json
{"selected_groups":["对应entry的正确组名"],"gate_passed":true,"results":{},"executed":0}
```

真实返回：`ARCH-07/one-pid-one-entry`、`ARCH-07/last-owner-gc`、`ARCH-08/fresh-union-spec-review` **三行全部PASS**。本轮只读确认源码未变，未重新运行产品/反例；T-01须在唯一实施树把此输入变为永久失败回归并配合法正例。另须覆盖schema合法却缺实际观察的同型输入，避免仅加schema字段就假关闭。

| Finding | 保护的既有规则与已见原因 | 本包关闭边界 |
|---|---|---|
| F-1005-01，P1 | entry-gates把`gate_passed==true or outcome==PASS`直接当绿色，缺执行数/事件/终态验证；相关entry齐就无条件写行PASS | 严格封套和逐行语义观察双门，缺执行/缺观察均不能PASS；覆盖所有架构结果入口 |
| F-1005-02，P1 | 唯一review行由相同循环供应；first_slice/正式CLI没有读取签前审计凭据的强制边 | 缺审计凭据不能闭该行或正式签章，实际独立审计之后才能解锁 |
| 同源绕过风险（T-02核） | `phased_cert_observations`可把architecture整节或observations写入result；后续合并若只认PASS/hash可能旁路新校验 | 只对本包架构/审计消费链核闭同一不变量，不借此重做全部认证族；实测缺陷归T-06 |
| 现行Spec与原合同差额 | 1000沉淀记载entry组绿→行claim及签前审阅流程，但机器没有强制凭据；原1000合同又要求逐oracle和review独立供应 | 本计划是用户批准的明确修正规则；merge更新current Spec，旧归档历史保持，不反写1000当时行为 |

现有八条`ArchitectureEntryGateControls`核outcome、失败事件、组名、hash、缺entry等，但其正例本身允许仅`outcome=PASS`而无执行/语义事实。不能以“已有8条控制”判本次反例关闭；V-01..V-13需分别敏感。

### 输入身份快照

| 路径 | 本轮SHA256 |
|---|---|
| `scripts/test-governance/ace_m1_arch_entry_gates.py` | `fa13cb9aa14fb6d3f65e842a515a61779fed4ea6d1e3a79122e626d4066a5cf7` |
| `scripts/test-governance/ace_m1_first_slice.py` | `d6e531c94130b7314e3c936e4b16127162acaa9d30ca12db3296737828c56364` |
| `scripts/test-governance/ace_m1_arch_acceptance.py` | `ae0172f5668077b6e59c49d2755b8f9718b91446d9b257bfade5ee7058893497` |
| `scripts/tests/test_ace_m1_arch_acceptance.py` | `f0c96dfbd174895bcb2ea4ec79b003767ba8f4e23306bba76c36ea5b014da546` |
| `crates/auto-lang/test/cases/ace-m1-expanded/architecture-obligations.json` | `74cf43974e51420f9afbf3f688f0807294a1bac865ed98fb688d19fdff44754a` |
| `crates/auto-lang/test/cases/ace-m1-expanded/registry.json` | `ed922175259c22371013a785269eede1b613da98e08f9ec3f4b756c43f3b3318` |

编号：按本仓`scripts/new-plan.sh`完整分配1005，counter推进1006；不回绕/截号。只读核auto-musk现源码已含多位编号解析及1001回归，故不沿用旧“三位scanner”作为本包阻塞；未在本轮运行/更新该应用或外仓。

### 实际源码矩阵

| owner任务 | 现存入口/符号 | 拟改面与真实consumer |
|---|---|---|
| T-01/02 | `scripts/test-governance/ace_m1_arch_obligations.py::validate_contract/reduce_architecture`；上述`architecture-obligations.json`/`registry.json` | 49行原oracle/required_observations与来源映射，固定独立审计行；不能改声明execution=UNOBSERVED为PASS |
| T-03 | `scripts/test-governance/ace_m1_arch_entry_gates.py::admit_arch_entry_gates` | 封套schema/执行/选择/排除/事件/终态/身份/原文档来源验证，向T-04交已准入事实而非布尔 |
| T-04 | 上述entry reader；`scripts/test-governance/ace_m1_ct_mechanisms.py`、`ace_m1_sq_facts.py`、`ace_m1_syntax_arch.py` | 行级recipe/具名测试与语义事实准入，原有效机制证明保留；对专用机制输入检查不变量，不凭优先级豁免 |
| T-04正例owner | `experimental/ace/crates/native/tests/cases/incremental_churn_gc_1000.rs`；`experimental/ace/crates/av/tests/cases/av_closures_incremental.rs`；`crates/auto-lang/test/cases/ace-m1/` | 同PID/代际/外pin/last-owner分别沿原row的resident/av-resident入口；此GC测试只证明工件store的回收，不能代在线热更新全部语义 |
| T-05 | `scripts/test-governance/ace_m1_arch_acceptance.py::reduce_acceptance`及CLI；`ace_m1_first_slice.py::compose_first_slice` | 新增严格审计凭据输入/验证小接口（拟新增helper如`ace_m1_review_evidence.py`，T-02定稿）；非review行不能供应该专属行 |
| T-06 | `scripts/test-governance/ace_m1_first_slice.py`的phased/arch合并、最终registered/acceptance；`ace_m1_registry.py::compose_registered_inventory` | architecture分域来源重验/白名单、无覆盖/回灌绕过；正式report/CLI消费同一validated事实 |
| T-07 | `scripts/tests/test_ace_m1_arch_acceptance.py::ArchitectureEntryGateControls`；`scripts/test-governance/python_tests.py`；`scripts/fixed-tests.py` | 新反控+合法/旧负例，核固定组真实loader发现/执行新case；不另造runner |
| T-08..11 | 现上述入口、原1000固定组/producer身份、两现存Spec与拟新增专门Spec | 新冻结候选→独立审计→正式组合→主线等价读回→tag_ready；旧scope已删则重新采证，不能拿摘要冒重读 |

新helper、字段/schema与永久fixture均为拟新增，不能按名称当现有API。矩阵中短名`ace_m1_*.py`均在`scripts/test-governance/`。原row具体case/test名字和每个必需观察的实际获取点由T-02冻结，缺决定不进入对应实现，不能只写“后续补”。

## 5. 详细设计

### 5.1 S0/S1：先冻结行级合同，不把所有组都改成全局新schema

T-01复现三行原反例和真实完整正例，核当前49行/1006键；T-02有界前探选择3条原失败行及一条已交专用机制行，串真实producer→原始输出→新准入→合并→候选/凭据→正式拒/受流程。给出49行的`row_id→entry→case/test→required observation→typed assertion/oracle→source/producer身份→反控→consumer`表。其余行复用相同受审recipe但仍逐行绑定，不能一条记录任意认领所有row。

gate-envelope按当前真实固定组/nextest/Case文档选确定版本适配；不把不同文档形状用`or`猜字段。未知schema/字段、bool冒int、负数/缺数、group/测试计数/事件矛盾拒绝。原required测试被排除或缺失只能对应行未观测；组内与该义务无关且按原规则ignored的测试不成为新的全组失败，也不计为其required通过。组绿色与行正确性始终独立。

### 5.2 F-1005-01：封套与逐行双重准入

| 规则 | 实际观察与关闭条件 | 代表性失败/合法对照 | 主责 |
|---|---|---|---|
| R-01 原始执行存在 | schema明确、当前group/具名case/test集合、executed>0、事件与终态/父裁判相符，保源码/契约/工具/配置/依赖身份 | `gate_passed=true`但executed=0/空events必须拒或行U；合法非零完整事件通过；green与failed event/timeout并存不得短路 | T-03 |
| R-02 行语义已证 | 原closing_oracle/required_observations全部有typed事实或受审永久断言的**实际具名执行**；保原值/阶段/trace/资源oracle | 组仍PASS但删PID/发布代际、外pin/最后owner计数或原负控断言时行不PASS；原合法观察仍通过 | T-04 |
| R-03 当前来源完整 | 原始报告/工件/行证据多级引用及当前producer/源/契约/配置/工具/依赖一致；当前错误不能由hash自洽洗白 | 改语义事实同时重算payload与其局部SHA但保可信原始事实不变，仍拒；改无关描述字段按schema明定允许/拒绝 | T-03/04/06 |
| R-04 诚实三态 | 缺required有效事实→UNOBSERVED；真实oracle不符→FAIL；格式/身份错→拒绝；父FAIL不因子/组PASS晋绿 | 原负例/控制按原独立oracle可被正确判通过，任意infra失败不能充语言负例通过 | T-03/04 |

同PID行必须证明一个持续运行的guest入口/PID、真实safepoint与新旧调用链代际；重启或暂停恢复不能代替。last-owner行必须区分当前根/在途链/外部pin仍活与最后释放后回收收敛，校验sentinel；进程返回、历史容器清空或一次store-GC不自动证明全部生命周期要求。

不对声明文本做通用自然语言判分；建立范围内固定typed recipe/字段与assertion映射，沿原reader/永久测试断言实现。新observations不能只是`oracle_pass=true`等第二层自报布尔。测试源与二进制pin是来源之一，还须有具名实际执行和可区分的原断言/事实；测试名存在或全组绿不足。

### 5.3 F-1005-02：审计凭据是输入门，不是事后备注

唯一row=`ARCH-08/fresh-union-spec-review`。从普通entry循环、phased或通用机制中供应此row的PASS一律不得授信；实现阶段只保pending，其他row的结果原样计算。拟新增严格审计凭据schema由T-02冻结，至少明确：

- plan/revision、冻结代码H（完整commit）、实际被审候选输入/registry/行规则版本、工具/配置/依赖与原始事实清单/内容hash；被审全集及唯一排除的review专属row。
- 实际审计角色/上下文、工作AC/逐行覆盖/Spec delta核验、发现与处置、审计结论；`pass`字段不是证据，独立性需审计流程另行确认，不用字符串或实现者自查替代。
- 凭据由独立review阶段产生，其期望hash/冻结H由review-owned交接可信固定；正式consumer按该锚核原始引用。不能由同一未受信payload同时提供可共同篡改的“expected hash”和自报PASS而自封授权。

无环次序：冻结H和全部非review事实 → work侧只完成1005非review键、候选总门不可签 → 独立review核这些事实及反控/SD → 生成签前凭据R → 正式consumer核R并供应唯一row → 最终1006组合与final review → 主线落地/tag_ready。R可绑定签前候选hash，**不能引用最终报告自身hash**；最终报告引用R的原件hash/验证结果及H。审计实现/依赖/规则/事实发生实质变化则旧R失效；不得仅换报告commit字符串。

缺凭据、坏schema、错H/revision/源工具/配置、漏审row、未决发现、陈旧/自指凭据均不可签；有效R也不能覆盖其他FAIL/U。T-05可用合成合法凭据测consumer正例，但必须标为控制、不进入生产签章；T-09才产生本包实际独立审计证据。不新增密钥/网络服务来声称能由字节单独证明人的独立性。

### 5.4 合并/分域摘要的同一信任边界

T-06核entry、专用机制、phased architecture `section`/`observations`及最终合并全部入口：架构输出只能由已验证来源的逐行判决供应；未验证整节不得直接写入`architecture_execution`或重新声明`complete=true`。phased可继续用于预算，但须绑定原始执行/来源/准入recipe版本并按同一严格reader重验，或消费有相同强度可信生成链的受检封套；局部文件SHA不代替这条链。

显式限定架构/审计section名和结果类型，重复/冲突row拒绝或按原规则保留真实失败，不能“后写覆盖FAIL”“优先级覆盖U”或用任意section注入专属review行。T-02按代码画实际覆盖顺序并冻结；不能只修本次直接函数入口。非架构认证族不另行设计新机制，但若同一通道可洗白架构/审计，必须在本包堵住。

### 5.5 有界修复和完成边界

只修本轮两个准入不变量及其同源绕过、最小观察缺失、永久正反例；不再承接语言大扩面。如果严格reader使原PASS降为U/FAIL，逐行补真实观察或修原支持域内最小缺陷，保原有效成果，不因总数下降而放宽准入。若确需扩大语言/架构/平台/预算，先needs_replan并具名修订/后继，不静默扩范围；**只要required仍未闭，本包不能以记债/交后继完成交付或给tag_ready**。

1000原scope已按政策删除。未改事实/身份且原始证据仍可合法获得时可按明确依赖图复用；只有计划摘要/历史hash时不能声称已重读，应重新采所需原始观察与执行。无需机械重跑全仓，但最后必须用修复后的真实有效来源重建全M1及独立审计；无原始证据不能以清理政策为豁免。

### 5.6 打tag前的强制复核（交付tag_ready，不在本轮创建tag）

| Gate | 检查与通过条件 | 阻断条件/责任 |
|---|---|---|
| TAG-01 候选与来源 | 明确拟tag完整commit C及其祖先；含本包代码、正式审计与canonical沉淀；只标记已提交内容，记录实际源/工具/协议/依赖/行规则指纹 | 仅branch tip/工作目录WIP、漂移后换SHA、未落地修复不得通过；T-10 |
| TAG-02 严格终验 | 修复后当前有效全集1006P/0F/0U、ARCH49/49、八域真；accounting/evidence/system真、cert-gap/awaiting/missing_inputs/blockers=0；正式exit0可签 | 任何required未证、仅历史1000摘要、分域覆盖绕过、错身份不可tag；T-08/09/10 |
| TAG-03 独立审计有效 | 实际独立R存在、通过consumer强校验，绑定受审冻结H/事实/规则；最终报告引用R且不自指 | 组PASS代review、缺凭据或实质漂移使R失效；T-09/10 |
| TAG-04 负控敏感 | V-01..V-13及本次原反例真实执行；保组绿删执行/行事实/R、篡改后重算局部hash、phased注入均不能PASS/签章 | 父exit0或测试总数不代逐负控；任何可复现假绿阻tag；T-07/09/10 |
| TAG-05 落地复验 | C与H的真实producer/consumer依赖闭包逐项对账；仅具名docs-only/等价内容后代可复用，其他按影响重采并重审；C的治理/consumer验证有效 | 源/工具/contract变而沿旧证据，或仅在原worktree可用不得通过；T-10 |
| TAG-06 标签与范围 | 记录`v0.6-ACE-M1`本地引用是否存在及目标，注释拟含C、R/报告hash、原分母/结果、本包与1000收据、ACE M1范围 | 不覆盖/移动同名标签；远端未核不能声称不存在或已发布；不冒M2/全v0.6；T-10 |

本Plan完成要求写出上述复核结果和`tag_ready=true`，并完成归档/清理；**不以创建/推送标签作为本Plan必需副作用**。实际打tag是复核后的单独动作，按当时已存在的用户条件授权处理；本轮明确不创建、不推送。不能因标签创建成功倒推M1通过。若本地同名tag已存在，保留并核身份，禁止force改指；远端查询失败要如实记录，不能借此绕过未来发布核验。

### 规范增量

仅拟议；review按实际范围/版本定稿，merge才修改canonical。不得改archived1000的历史结果为本包结果。

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | `docs/specs/auto-test-case/architecture.md` | 组绿色可直接供行claim→严格执行封套+原oracle逐行授信；review行只能由实际有效凭据供应；旧行为记历史 | 两个实证假绿关闭 | AC-02/03/04/05 |
| SD-02 | modify | `docs/specs/auto-test-case/ace-m1-expanded-acceptance.md` | 1000当时1006P保留为历史，追加本包修正后的完整来源/审计/正式模式/tag_ready事实 | 防历史PASS豁免新缺陷 | AC-01/06/08/09 |
| SD-03 | add | `docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md`（拟新增） | 明确49行来源、状态、封套/断言映射、分域同边界、凭据版本/可信交接、拒绝语义及TAG门 | 当前可复核执行合同 | AC-02..09 |

## 6. 测试设计

下列是拟实施验证，不是本轮测试结果。T-01把已实证三行输入固化，T-03..07补实际可运行正反例，原始输入在唯一scope中；永久测试沿现owner。每V同时核函数、合并/registered和正式CLI的适用层，不能只测一个helper。输入/不变量/真实入口/判决/AC已绑定，不新增通用全跨积。

| Case | AC | 输入与真实入口 | 必须观察的结果/区分错误实现 | owner/边界 |
|---|---|---|---|---|
| V-01 | AC-01/02 | §4原三行最小文档：正确组名、绿色布尔、executed=0/空结果→生产entry reader | 修前3PASS复现；修后拒或行U，绝不PASS；格式合法同型另测；合法非零具名执行对照过 | T-01/03/07；三行全部 |
| V-02 | AC-02 | green=true/outcome=PASS与失败事件、timeout、缺终态、executed计数冲突并存 | 不得被绿色短路；组计数/事件/终态一致才准入，缺执行不能由SHA闭合 | T-03/07；原有效schema |
| V-03 | AC-02/03 | 保组绿而删除目标具名case/test，或把它置ignored/excluded；保无关ignored对照 | 对应required行不PASS；无关合法条件排除不被误改全局失败/不计其执行 | T-03/04/07；原分层条件 |
| V-04 | AC-03 | 同PID行保测试组绿，删PID/代际/真实safepoint或用新进程复跑替代；原resident/av-resident入口 | 行U/FAIL/拒绝，合法同PID新旧链事实及实际原断言通过 | T-04/07；原ARCH-07 |
| V-05 | AC-03 | last-owner行保组绿，删外pin/最后释放/回收计数，或用清空容器、单次store-GC替生命周期 | 不得PASS；合法在途/外pin保活，最后owner释放后回收收敛、sentinel不动正例过 | T-04/07；真实资源语义 |
| V-06 | AC-03/05/06 | 在原/raw不变时篡改行观察并重算局部payload/binding SHA；错source/worker/config/recipe映射 | 来源链不符仍拒，不是只抓字节hash没更新；原当前身份完整正例通过 | T-04/06/07；局部hash自洽攻击 |
| V-07 | AC-04/05 | 审计row由全绿entry、机制或phased直接供应PASS，无真实R；正式入口调用 | row保持未观测，总门false/exit1，组PASS/实现自查不能闭；格式错误exit2 | T-05/06/07；所有入口 |
| V-08 | AC-04 | 凭据缺/错schema、H/revision/规则/原事实/工具配置不符、漏审行、未决发现、最终报告hash自指 | 逐一拒绝/不可签且具名原因；合法控制R验证通过但明确非生产审计证据 | T-05/07；严格身份与无环 |
| V-09 | AC-04/08 | 独立review冻结H，核1005非review键→实际R→正式consumer→final report | R原件hash/可信锚/覆盖可核、最终报告引用R，只有真实审计后唯一row PASS；不以reviewer字符串证明独立 | T-05接口；T-09实际审计 |
| V-10 | AC-05 | 同一坏架构事实分别经entry、机制、phased section/observations及合并；注入review row或complete=true | 所有路径均不可PASS，不能覆盖真实FAIL/U；受检合法分域与直接准入判决一致 | T-06/07；架构/审计同源旁路 |
| V-11 | AC-06/07 | 原固定loader发现新反控；旧valid、原负例/expected-fail、未知field/错type/重复/外来row | 真实选择/执行>0，新反控每项敏感；旧合法行为不退，任意infra失败不充原负例 | T-07；现固定组与必要consumer |
| V-12 | AC-07/08 | 冻结修复候选，重建全键/原313集合及各架构行；preview与正式模式 | 非review键真实全绿、审计行唯一pending且正式不可签；实际R后1006P/0F/0U/八域真；preview始终不可签exit1 | T-08/09；候选到签章 |
| V-13 | AC-08/09 | H→主线C后改真实producer/合同/工具，或用错目标/陈旧R申请tag_ready | 漂移使证据/审计失效，不能换commit字段充复用；等价docs后代原样可核；缺任TAG门为false | T-09/10；打tag前复核 |

控制用合成“全绿报告/凭据”只能证明拒绝/接受规则敏感性，绝不作生产M1证据。反控必须真正改变目标事实并确认变化；仅改outcome/字节hash不替保组PASS删关键语义观察。不得为反控引入生产后门。

### 正式验证入口与责任

- Python准入/CLI/治理主要职责：`cargo t infra-ace-m1-arch infra-ace-m1 infra-ace-policy`。涉及固定loader/父协议真实改动再加`cargo t infra-fixed auto-test-case ace-adapters`，不因只修Python机械跑Rust全部组。
- 若补producer或原断言，按实际触面选完整`cargo t ace-source ace-av ace-native ace-semantics ace-comptime ace-abi ace-a2x`必要并集；选择理由与49行/实际原固定组映射由T-02冻结。原认证/压力/性能/ignore/平台条件保留，正式采样单实例错峰；不能用组绿代新行级事实。
- 正式原入口`python scripts/test-governance/ace_m1_arch_acceptance.py --acceptance --repo <唯一树> --evidence <scope> --bindings <bindings>`；新审计输入参数或binding为拟新增，由T-02/05定稿。完整受信证据+R才exit0；合法未闭exit1；坏格式/身份exit2；原`--first-slice-preview`恒不可签exit1。本轮未运行任何这些命令。
- 所列组名从`scripts/test-groups.toml`静态核名；新case须通过现`python_tests.py`loader确实发现/执行。列发现/执行/原条件排除、准备/冷编译/运行/清理/总时、真实子终态/身份。保原48MiB/4MiB/样本阈值，原P90不变；本包不新订全局性能门。
- 独立复审前实现停写；独立上下文从原始事实核AC/V/SD与遗漏/健康，声明实际身份与独立性，不信勾选/摘要。用户未指定reviewer/模型，不自动启动代理/其他会话；同一权威树顺序工作。周期全量门仅按主检出due规则，不作每次修复反复全仓。

## 7. 验收标准

| ID | owner_stage | 可观察标准 | 核验 |
|---|---|---|---|
| AC-01 | work | 原3行假绿反例实证、来源/当前HEAD/49行1006键/旧313适用性冻结，49行实际recipe与有界策略齐，无未认领路径 | T-01/02；V-01 |
| AC-02 | work | strict封套不受绿色布尔短路，零执行/空结果/矛盾/错身份拒绝，原合法条件排除与正例保持 | T-03/07；V-01..03 |
| AC-03 | work | 各架构行均按原oracle/required_observations取得真实具名执行/断言与事实；组绿删行事实不PASS，原有效语义结果保留 | T-04/07；V-03..06 |
| AC-04 | work | 正式consumer强制有效独立凭据才可供应review行；原始来源/可信锚/H/覆盖与无环规则实现，缺凭据保持不可签 | T-05/07；V-07..09接口 |
| AC-05 | work | 三类架构输入及分域/最终覆盖均不能旁路，冲突/外来row/审计注入被拒，直接与受检phased一致 | T-06/07；V-06/07/10 |
| AC-06 | work | 永久反控确实执行并能杀对应错误，旧合法/原负例不退；新增warning/fmt/debug零未处置；原资源/计量不放宽 | T-07；V-01..11 |
| AC-07 | work | 冻结候选全部非review required真实闭合，只有独立审计行可pending；全键/原集合守恒，受影响完整职责/必要consumer结果有效，SD/来源/失效清单齐 | T-08；V-11/12 |
| AC-08 | review | 独立核全部工作AC/V/SD及原始链，实际R经修复consumer准入；新鲜正式组合1006P/0F/0U、49ARCH/八域全真、零blocker，缺执行/缺行/缺R反控仍拒 | T-09；V-09/12/13 |
| AC-09 | merge | 复审后按实际漂移复验、SD主线读回、TAG-01..06逐门有结果且tag_ready=true；保持1000归档、未强移标签；全输出对账、归档/清理完成 | T-10/11；V-13 |

## 8. 执行步骤

后续实施需用户指令；本轮不创建树/材料。实施从§3唯一树/分支、当前本Plan r1、D1000主线身份及本仓AGENTS/Specs开始。原1000的代码成果/历史记录只读复用，不重开旧计划。

| task_id | owner_stage | 前置 | 实际动作与交付 | AC/V | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 后续实施指令、主线D1000可读 | 复现3行假绿并固化永久反例；核实际工具/49行/1006键/313集合与当前tag引用；登记scope、真实来源和共享写面 | AC-01，V-01 | [x] 2026-11-11：修前复现=10假gate(正确组名+绿布尔+executed=0)→生产reader返回3目标行PASS(波及20行)；scope=t01/t01-report.json(分母49行/1006键/313、键集SHA 3f412190…与1000零漂移、D1000四commit全祖先、本地无v0.6-ACE-M1 tag)；永久反例=ArchitectureEntryGateStrictControls(21测试+9子测) |
| T-02 | work | T-01 | 4条代表路径有界全链先证，冻结49行typed recipe/具名断言与原始观察获取点、各schema/CLI审计输入/可信锚/合并优先级；方案失败needs_replan不删义务 | AC-01，V-01/04/05/07/09/10 | [x] 2026-11-11：4路径=one-pid(resident套件R01-R03手算oracle+av_resident14测试含restart负控)/last-owner(同+churn_release)/fresh-union(review专属行,凭据唯一供应)/sq_facts机制行(native_facts范式)；冻结arch-entry-recipes.json(48行,SHA 8f0e16b3…,入库+gitignore白名单)；entry事实清单=t02/entry-facts-inventory.json |
| T-03 | work | T-02 | entry reader严格封套准入，核实际执行/选择/排除/终态/事件/身份一致性，消除布尔短路；永久负控与合法邻例 | AC-02，V-01..03 | [x] 2026-11-11：ace_m1_arch_entry_gates重写=gate目录准入(listing+raw终端经父解析器重derive+命令收据+冻结窗口+当前config+listing二进制pin+合同source pin)；V-01反例形态全拒、合法一致性门通过；提交ff27a1454 |
| T-04 | work | T-02、T-03 | 逐架构row执行原typed recipe，补所需最小producer事实或原断言映射；同PID/last-owner真实观测，保有效机制结果，review行不由此供应 | AC-03，V-03..06 | [x] 2026-11-11：48行逐行recipe绑定(case-check/native-test具名事实,删一即U)；reader永不发review行；4 composer行加composer_facts词表；机制族优先→冲突拒绝；V-03删restart负控/ignored→行不PASS、V-04红gate→FAIL实测 |
| T-05 | work | T-02；T-03事实封套 | 新增严格审计凭据消费及review行专属授权边，绑定H/事实/规则/可信锚、拒自指与缺审计，接正式CLI；合成控制不代实际review | AC-04，V-07..09接口 | [x] 2026-11-11：ace_m1_review_evidence.py(凭据schema+候选绑定+frozen commit须∈已采gate commits+AC/V/SD全覆盖+findings全resolved+exact_keys无最终报告自指字段)；first_slice review_credential binding(唯一供应路径,缺→missing_input blocker exit1)；合成控制7测试绿 |
| T-06 | work | T-04、T-05 | 统一entry/机制/phased/最终合并的架构授信，白名单与冲突拒绝，堵整节/observations覆盖与review注入；不另造store/runner | AC-05，V-06/10 | [x] 2026-11-11：merge_architecture_observations单一信任边界(review行任何族注入拒/冲突verdict拒不再覆盖优先)；admit_phased_observations白名单10族+observations ARCH键拒+section架构形payload拒；registry测量回灌拒review行；V-07/V-10单测绿 |
| T-07 | work | T-03..06 | 所有永久反控/合法邻例实际执行，原完整治理和真实改面consumer并集；核loader发现、变异敏感性、健康/计量与来源 | AC-02..06，V-01..11 | [x] 2026-11-12：单测+固定组+真实consumer（1005P 组合）三层全闭 |
| T-08 | work | T-01..07 | 冻结H和有效原始事实/工具/依赖；按影响重采/复用，完整非review键通过且唯一review行pending、正式不可签；准备SD-01..03、失效/来源/健康清单交独立review | AC-07，V-11/12 | [ ] |
| T-09 | review | T-01..08完成，停写同树，review行未冒签 | 独立核原始链/AC/V/SD/遗漏，实际生成R、consumer验证后新鲜组合与全部拒绝控制；只有全1006/八域/49ARCH/零blocker才final pass，否则交回work | AC-08，V-09/12/13 | [ ] |
| T-10 | merge | T-09 final pass、后续合入指令/窗口 | 冻结必要职责并集与来源漂移复验，落SD/主线读回；按§5.6核C/H/R/报告/负控/标签及范围，写候选tag_ready逐门结果；本轮/此任务不默认推送标签 | AC-09，V-13 | [ ] |
| T-11 | merge | T-10真实来源及候选tag_ready已核 | 最终摘要折Plan、逐命令全输出路径对账，删scope或具名hold/刷新retention；wt-guard clean后清树/分支/空组；归档/收据入主线后重核最终C，仅docs等价后代才具名复用，报告最终tag_ready及准确C/收据 | AC-09，V-13 | [ ] |

实际顺序：T-01→02→03→04/05→06→07→08→09→10→11。04/05可按稳定接口准备，不自动创建并行会话；公共reader/registry/test-groups单写。T-08只留唯一review专属行，不得把其他未闭项移交review冒工作完成。T-09发现问题回同树work，修复使受影响R/结果失效；不在review顺手修代码或重开1000。

唯一实施材料scope拟`D:/autostack/plan-artifacts/plan-1005/`；开工即登记owner/用途/在途，所有probe/原始日志/binding/凭据/正式报告/外部副本/显式外置target在此，正常target在唯一树；本轮未创建。产品永久fixture留真实owner测试目录。merge把命令/退出码/逐行结果/关键负控/冻结与落地身份/凭据及报告hash/TAG结果折入Plan，按政策删整个scope和树；未知材料不盲删，具名未解决hold有owner/理由/关闭条件，但hold不能豁免本包未闭AC。禁止attachments或第二证据根；纯规划临时材料仅系统TEMP。

## 9. 复审记录

2026-11-11 stage:new / PLAN-1005 r1 / drafting / current_step:0/11。本轮仅规划，按既有源码和上轮只读反例制定，未执行产品/认证/资源/性能或独立实现review，未创建tag或其他会话。1000保持archived及原内容，不重写其完成历史。

规划交接：outcome=pass（文档静态预检完成后，非产品准入/独立final review PASS）；next=后续用户实施指令→T-01；未决等价实现策略由T-02有界先证承接，不再向用户索取同范围重复授权。若有真实范围/预算变化，先按§5.5具体修订。

本轮文档静态预检：0–10十一章、11个唯一连续任务、9项AC、13项V、3条SD、owner_stage与任务DAG、本地链接/全部现存源码与Spec路径、13个所列固定组、drafting/0-of-11、1005编号唯一性及.next-id=1006均通过；依赖无环，审计row保留review-owned，不把审计未来步骤当work已完成。

保护对账：取号前73份既存文件中，仅`.next-id`按预期变化，其余72份内容hash不变，含archived1000、现行Specs、准入/CLI/测试源码及原WIP；新增永久路径仅本Plan。HEAD保持`9e36d984e3d33aef27da6163b2017b2aaab2d258`，本轮未做提交/合并/建实施树/材料scope、未创建/移动Git tag或写外仓。纯文档临时材料在系统TEMP；未运行产品/认证/性能或独立实施复审，合并、规范沉淀、tag_ready和清理均未发生。

## 9. 复审记录（追加）

2026-11-11 stage:goal/work / PLAN-1005 r1 / round:1 / `/goal 完成计划1005` 授权全链（work→review→bounded repair→merge）。开工事实：实施树 `D:/autostack/.wt/lang-1005/auto-lang` @`codex/plan-1005-dev`（基 master@9e36d984e，与 D1000 沉淀 commit 一致）；材料scope `D:/autostack/plan-artifacts/plan-1005/` 已登记 `.retention/policy.json`（in-flight）；主检无代码路径 WIP。

**T-01..T-06 完成（提交 ff27a1454，单一提交实现单元）**：
- T-01 修前复现：10 个假 gate（正确组名+`gate_passed:true`+`results:{}`+`executed:0`，真实SHA绑定）→ 生产 `admit_arch_entry_gates` 返回 3 目标行（ARCH-07/one-pid-one-entry、ARCH-07/last-owner-gc、ARCH-08/fresh-union-spec-review）全 PASS（同源波及共 20 行）——F-1005-01/F-1005-02 缺陷在当前代码实证复现；分母冻结 49行/1006键（legacy 313）/键集SHA `3f412190…` 与 1000 归档零漂移；D1000 四 commit 全为 HEAD 祖先；本地无 `v0.6-ACE-M1` tag（t01/t01-report.json）。
- T-02 冻结 49 行 recipe 合同 `architecture-entry-recipes.json`（48 非review行，SHA `8f0e16b3…`，入库+.gitignore 白名单防 F-908-R1-01 类吞噬；4 composer 行绑 composer_facts 词表；review 行唯一、凭据专属）。4 代表路径实证：one-pid（resident 套件 R01-R03 手算 oracle 2100/2608/3070 + av_resident 14 具名测试含 restart 负控）、last-owner（同面+churn_release/GC）、fresh-union（凭据唯一供应）、sq_facts 机制行（native_facts 既有严格范式）。
- T-03/T-04 `ace_m1_arch_entry_gates.py` 重写：entry 准入=真实单组 gate 目录（listing 枚举+raw 终端经父解析器重derive+discovery/run 命令收据逐 argv 复核+冻结窗口 source-pins+当前 config+listing 二进制字节 pin+合同 source pin）；行判决=全部声明 entry 绿 AND 全部具名 recipe 事实在 gate 内实际执行绿；缺具名事实→行U、红gate→行FAIL、假形态→拒绝；reader 永不发 review 行。
- T-05 新 `ace_m1_review_evidence.py`：签前凭据消费（frozen commit∈已采 gate commits、pre-signature 候选逐行一致+不可签、bindings/行规则/registry 身份绑定、AC-01..09/V-01..13/SD-01..03 全覆盖、findings 全 resolved、exact_keys 无最终报告自指字段）；first_slice `review_credential` binding=唯一供应路径，缺凭据→missing_input blocker→正式 exit1。
- T-06 单一信任边界 `merge_architecture_observations`（review 行任何族注入拒、重叠冲突拒不再覆盖）；`admit_phased_observations` 白名单 10 族+ARCH 键拒+架构形 section payload 拒；registry 测量回灌拒 review 行。
- T-07（单元层）：`ArchitectureEntryGateStrictControls`/`ReviewCredentialControls`/`ArchitectureMergeBoundaryControls` 21 测试+9 子测全绿（V-01 冻结反例、V-02 绿布尔×失败终端/executed 矛盾、V-03 删/ignored 具名事实→行不PASS、V-04 红gate→FAIL、V-06 篡改raw重算SHA仍拒/二进制字节漂移拒、V-07/V-10 合并边界、V-08 凭据 8 类负例、V-09 合法控制凭据）；旧 `ArchitectureEntryGateControls`（布尔形态）整体替换；全治理文件 287 passed+1189 子测；`cargo t infra-ace-m1-arch infra-ace-m1 infra-ace-policy` 三组 PASS。

**T-08 进行中（证据重采战役）**：binder（t08/binder.py：跑组→规范化 gate 目录）+ 顺序采集驱动（t08/capture-all.py，19 组=30 entry）。**真实环境发现 E-1005-01**：`ace-syntax` 组内正式基准 `syntax_budget::performance::syntax_budget_64mib_streaming_small_queries`（64MiB 持续流式）在本机三跑皆红：全组冷跑 1800s deadline（exit124）、T3_MILESTONE=1 长窗全组跑单测命中 nextest 每测试时限（exec 1800.0s）、隔离单测重放 24m48s 仍败于进程内 "memory allocation of 1MiB failed"（宿主 32GB 仅 3.9GB 空闲，被用户交互应用占用；commit 46GB 富余非系统级 OOM）。按 AGENTS 环境归因纪律记录：非本包代码缺陷（该代码=D1000 已验收基线，未改），为宿主当前资源边界；ace-syntax（syn-local/syn-replay→ARCH-01 行）延后重试，其余 18 组采集进行中（后台）。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 1 | from: work | to: work | status: archived | outcome: progress（T-01..T-06 完成提交 ff27a1454；T-07 单元层全绿；T-08 战役开跑+E-1005-01 环境发现） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: ff27a1454 | next: work（T-08 其余18组采集→组合非review键→E-1005-01 复判→T-07 真实consumer收口）

2026-11-11 stage:work / PLAN-1005 r1 / round:2（T-08 证据重采战役推进）。

**真实组采集（19 组=30 entry）**：binder 逐组运行原父驱动并规范化 gate 目录。当前 17/19 组绿采+严格转换通过（ace-comptime 2028s/ace-semantics 902s/ace-av 1709s〔1059 测试〕/ace-abi 642s〔263〕+13 个 case 套件组，t08/capture-log.json）；ace-a2x 首采缺 release a2x-worker（测试 spawn target/release/a2x-worker.exe → NotFound panic）→ `cargo build --release -p ace-a2x-worker`（sccache 编译 cranelift 崩溃，RUSTC_WRAPPER="" 绕过后 2m13s 成功）后复跑 216/216 PASS，但首复采被我方运行期间编辑 reader 触发窗口守卫（"code/config/inputs changed"）正确判废——终采进行中（干净树）。ace-syntax=E-1005-01 延后重试（见 round:1）。

**战役期三项真实修复（提交 42475cdf3）**：①48MiB 活数据帽击穿（20 entry 全量 listing/raw/逐行 decode 驻留）→逐 gate 瞬态释放+仅保留 recipe 相关行（过滤驻留）；②分域准入模式（ace.m1.arch-entry-facts/1，每 entry 独立预算跑完整严格链→紧凑 SHA 钉定 facts 文档，终组合只读 facts——沿用 1000 认证族分域范式）；③**recipe 事件名以 nextest listing 为权威重生成**（新 SHA cab19420f…）：lib 目标测试（src/tests.rs 经 mod tests）实际为 `pkg::lib$tests::fn`、独立 tests/<target>.rs 为 `pkg::target$fn` 无模块前缀——原生成假设（binary$module::fn）对 ct-service/sq-core/sq-restore/inst-restore 四类 entry 全错，属 T-02 冻结表的真实缺陷，已按实证修正并同步 RECIPES_SHA；21 项严格反控单测保持全绿。

**环境纪律事实入档**：被杀进程遗留执行租约（target/test-groups/active，owner PID 已死）致全组秒败 exit2——核 owner 死活后清除恢复；宿主内存波动（3.9GB→14.6GB→10.6GB 空闲，用户交互应用占用）曾致 ace-comptime 首采与采集驱动同死（进程树终止）——内存回升后恢复。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 2 | from: work | to: work | status: archived | outcome: progress（17/19 组绿采+严格转换；3 项战役期修复提交 42475cdf3；a2x/syntax 终采运行中） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: 42475cdf3 | next: work（终采落盘→30 entry phase-facts→48 行架构判决→机制族夹具→组合预演→其余域战役评估）

2026-11-11 stage:work / PLAN-1005 r1 / round:3（T-08 架构域重采主体落地）。

**架构域 43/48 行 PASS / 0 FAIL（真实重采+严格链推导）**：19 组中 18 组绿采并通过严格封套转换（ace-av 1059 测试 1709s、ace-abi 263、ace-semantics 243、ace-comptime 149、ace-a2x 216〔release a2x-worker 构建后〕+13 case 套件）；30 entry 中 28 个经分域 facts 准入（ace.m1.arch-entry-facts/1，每 entry 独立 48MiB 预算跑完整严格链，SHA 钉定），1991 个具名 recipe 事实全部实际执行绿；判决快照 t08/arch-derivation-final.json（commits=ff27a1454/42475cdf3/fe1a70438 跨三提交——终冻时须按 TAG-05 对账：三提交均为治理 reader/docs 变更，全部 entry source pin/组 config 经当前树重验通过，受影响证据按漂移规则处置）。

**E-1005-01 升格为宿主能力阻塞（三证）**：ace-syntax 组 `syntax_budget::performance::syntax_budget_64mib_streaming_small_queries`（64MiB 持续流式正式基准，仓内 nextest-full.toml slow-timeout 900s×terminate-after=2=1800s 自设终止界）三次净窗尝试均在 1800.0s 被终止（另两次分别为进程内 OOM@3.9GB 空闲与冷编译 deadline）；132/132 其余测试全绿。该测试代码=D1000 已验收基线未改——非本包缺陷，为本机当前负载（用户交互应用占用 32GB 中 ~20GB）下的能力边界。**影响面**：仅 syn-local/syn-replay 两 entry → 5 行 U（ARCH-01×4+ARCH-05/method-closure-body-edits）。**解除条件**：宿主空闲时段重试 ace-syntax 单组采集（约 30 分钟/次）；若持续超界则需 owner 决策（放宽 slow-timeout 不在本包授权内——属测试政策变更）。

**剩余战役清单（T-08 完整闭合所需，均未开工）**：①机制族夹具 6 项（ct-source/child/meta/restore、sq-facts、syntax-arch 单测试 gate+observations 捕获）；②base slice（4 套件批采+workers+Q4 认证双腿+旧 accounting）；③legacy 313 链（a2r 11 组/base/typed×2/data/probe 族/Rust 观察源/治理轴）；④Q4 认证 7 族×双目标×三腿+负面前门+release 腿；⑤七执行套件；⑥恢复测量+预算采样（10 策略×10 运行+2 专项窗）+系统控制。预估多会话工作量。

战役期修复第三笔（提交 fe1a70438）：phased kept 值归一化（裸 event/status 字符串→行判决形状）。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 3 | from: work | to: work | status: archived | outcome: progress（架构域 43/48 行真实闭合；E-1005-01 宿主阻塞三证；其余 7 域战役清单化） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: fe1a70438 | next: work（宿主空闲重试 ace-syntax→机制族夹具→base slice→其余域战役；或按用户新指令重排）

2026-11-11 stage:work / PLAN-1005 r1 / round:4（E-1005-01 解除+架构域全闭+机制族夹具落地）。

**E-1005-01 解除**：宿主空闲（10.1GB）后第四次净窗尝试 ace-syntax 于 **1503.8s 全绿**（132/132，仓内 1800s slow-timeout 界内）——定性确认为此前三次为宿主负载边界而非代码缺陷。phase syn-local/syn-replay 后 **架构域 48/48 非 review 行 PASS / 0 FAIL**（30/30 entry、2030 个具名 recipe 事实全部真实执行绿；review 行正确缺席=凭据专属；快照 t08/arch-derivation-final.json）。

**机制族夹具 6/7 落地并经严格 reader 验证**：ct-source（ARCH-04/cold+callee）、ct-child（instance-child）、ct-meta（meta-args）、ct-restore（3 个 ct/restore 观察）、sq-facts（ARCH-02/solver-authority+facts-before-hir）、legacy-arch（ARCH-08/approved-old-conservation+old38-cert-separation，真实 accounting 生产者）全部 PASS；与 entry-gate 行判决一致（合并边界=冲突拒绝）。**两项配套最小变更**（提交 59f6a3cb8/d74e521f3）：①五个 941 观察者测试的注册 scope 根参数化（AUTO_PLAN_941_SCOPE_ROOT，缺省保持原 plan-941 fail-closed；原硬编码指向已退役 scope 致后继计划无法导出行观察）+合同 pin 级联（5 合同 source_pins→模块 CONTRACT_SHA→registry expanded_contracts，PLAN-1000 纪律）；②ct-restore reader 同型参数化。治理套件 287 passed+1189 子测保持全绿。

**剩余**：syntax-arch 机制（依赖 base slice 的 syntax 父报告）；base slice（4 套件+workers+Q4 认证双腿+accounting 接线）；legacy 313 链；Q4 认证 7 族；七执行套件；预算采样/恢复测量/系统控制；终冻 H+全量组合（1005P/0F/1U→exit1）→交 T-09。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 4 | from: work | to: work | status: archived | outcome: progress（架构域 48/48 真实闭合；机制族 6/7；两项最小配套变更提交） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: 59f6a3cb8+d74e521f3 | next: work（base slice 战役→legacy 链→Q4 认证→执行/预算/恢复/系统控制→终冻组合）

2026-11-11 stage:work / PLAN-1005 r1 / round:5（base slice 战役落地+首次完整组合）。

**base slice 全链重建**：①**formal identity exporter 重现**＝原 `auto-case identity-inventory` 维护入口（941 归档 line 606 载明；非已删 python 工具）——7 套件 /2 清单导出，派生合同与 registry 冻结值**逐值相等（7/7 零漂移）**；②四套件单 invocation 批采（AUTO_CASE_WIRE_EVIDENCE=1）25/25 全绿（legacy-941 4+q4 4+929 11+937 6，wire 交换帧入 artifact 副本）；③**Q4 认证双腿**：release a2x-worker×{rust,c}×{identity,certify,daily} 全 PASS——三项驱动侧修正：reference 须为仓内手写目标语言参考（复用 tracked reference-V07.rs/reference-C01.c；MSVC 文本模式 

 需 _setmode 二进制——与仓内参考同法）、`execution_observations` 由 WIRE env 门控（1000 已载）、daily nonce 48-64hex+收据 stdout/stderr 为 {path,sha256,bytes} 引用对象；④workers 绑定（5 个 release worker 就绪）。

**首次完整 preview 组合成功（修复链端到端实证）**：`compose_first_slice` 全链跑通（base+entry-gates 分域 facts+机制族+legacy accounting）——**架构 48 PASS + review 行 UNOBSERVED（凭据缺席→architecture_required=false）**；registered 68P/0F/938U（legacy 313+q4-source 624 等域未接为诚实 U）；legacy accounting_valid=true/can_claim_m1=true；峰值 22.72MiB<48。**正式 CLI 三态**：`--acceptance` **exit1**+stderr 具名 12 blocker（含 `missing_input:review_credential(ARCH-08/fresh-union-spec-review)`）——V-07/V-12 的 CLI 层断言在真实组合实证（t08/base-preview.json、t08/base-formal.json）。

**剩余**：syntax-arch 机制（需 syntax 父报告——base 批采已具备，接线即可）；legacy 313 执行链（a2r/base/typed/data/probe/rust/governance 七链+绑定）；Q4 认证 7 族战役（276+CTE+负面前门+release 腿）；七执行套件+scalar_q4/text_q4；恢复测量+预算采样（10×10+2 窗）+系统控制；终冻 H+全量组合→T-09。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 5 | from: work | to: work | status: archived | outcome: progress（base slice 全链+首次完整组合+正式 CLI exit1 三态实证） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d74e521f3（无新代码提交；战役材料在 scope） | next: work（syntax-arch 接线→legacy 七链→Q4 认证战役→执行/预算/恢复/系统控制→终冻组合）

2026-11-11 stage:work / PLAN-1005 r1 / round:6（第7/7机制族+legacy链战役开跑）。

**syntax-arch 接线（7/7 机制族）**：绑定 {'source':'syntax'} 即可——base 批采已含 syntax wire 观察；ARCH-01/all-grammars 机制判决 PASS 且与 entry-gate 判决一致（合并边界同意验证）。七族全数真实夹具+严格 reader 验证。

**legacy 313 链战役（3/8 链闭合，78/313 观测，0 FAIL）**：
- legacy_execution +4（B08/B11×av/ac wire 执行观察，复用四套件批采）
- legacy_base_execution +64（case-ace-m1-base 97 检查 wire 重跑 PASS；依赖快照=4 二进制 pin〔av/mn-worker+auto-case+test-case-pilot〕；tool_capture=运行 result.json）
- legacy_typed_av/native +10（两组 wire 重跑 PASS + parent_contracts 发现捕获〔auto-case parent-inventory Job 计量收据，7 源 pin 与 registry 冻结一致〕）
- registered **146P/0F/860U**（自 68P +78）；峰值 32.5MiB<48

**驱动侧修正入档**：compose 驱动每轮重建 bindings（新增链须写入脚本）；BASE 依赖集=4 项。

**剩余**：original_data 链（+28）、a2r 链（11 组）、probe 族（26+ 组）、rust 观察源（ace-av/ace-native 报告）、治理轴；Q4 认证 7 族；执行套件；预算/恢复/系统控制；终冻组合。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 6 | from: work | to: work | status: archived | outcome: progress（机制族 7/7；legacy 78/313〔execution/base/typed 三链〕；146P/0F） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d74e521f3（无新代码；战役材料在 scope） | next: work（original_data/a2r/probe/rust/governance 五链→Q4 认证战役→执行/预算/恢复→终冻组合）

2026-11-11 stage:work / PLAN-1005 r1 / round:7（F-1005-03 修复+legacy 链 6/8）。

**F-1005-03（真实交付缺口修复，提交 d137148b5）**：PLAN-1000 落地了 `ace_m1_legacy_a2r.py`（含 48 个逐套件身份合同 pin）但**从未提交 `legacy-a2r-contracts/` 目录**——F-908-R1-01 同类（跟踪消费者+缺数据，fresh 检出即无法准入 a2r/probe 族）。以当前 identity exporter 重建全部 48 份合同，**逐一 sha256 命中模块 pin（48/48 零漂移）**；.gitignore 白名单+治理套件 287 passed 维持。

**legacy 链 6/8（205/313 观测，0 FAIL）**：
- original_data_execution+diagnostics **+80**（两组 wire 重跑 36+16 检查绿；11-pin 依赖快照含 runtime-release/ac_text_runtime.lib）
- a2r 链 **+47**（11 套件 wire 批采全绿；identity+copy+worker pin 绑定；a2x-emit 902/904 补 a2x-worker pin）
- registered **273P/0F/733U**（自 146P +127）；峰值 36.67MiB<48

**工程事实入档**：①战役曾误写 tracked `reference-C01.c`（scope 意图命令的相对路径塌缩）——窗口守卫+治理测试立即抓出，还原后重采 Q4 认证绑定（此事故实证了严格链的防篡改价值）；②两组名映射错误（992/931）致首批 45 组选择失败，正名后 45/45 单 invocation 全绿；③45 组批含七执行套件+probe 族套件（后续域直接消费）。

**剩余 legacy**：probe 族绑定（~105 格）、rust 观察源（ace-av/ace-native 报告）、治理轴；随后 Q4 认证 7 族、恢复测量/预算采样/系统控制、终冻组合。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 7 | from: work | to: work | status: archived | outcome: progress（F-1005-03 修复 d137148b5；legacy 6/8=205/313；273P/0F） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d137148b5 | next: work（probe 族绑定→rust 观察源→治理轴→Q4 认证战役→恢复/预算/系统控制→终冻组合）

2026-11-11 stage:work / PLAN-1005 r1 / round:8（probe 族落地+rust/治理轴开工）。

**probe 族 25 套件绑定落地**：identity（绝对化）+45 组批采副本+帧权威 worker pin（av 一致 d5e00f82…；**mn-worker 重链多态实证**——25 族中多数族 mn pin 须为 None 或帧内 sha，与 1000 "帧 pin 唯一稳定权威" 纪律一致）→ **+97 格**；ace-native 组补采（231 测试）。

**legacy 现状 250/313 观测、0 FAIL**：unobserved=63 格，构成与考古计划：①corpus a2r 腿 14（B01-B12/K01-06 等——逐套件归属已测：992/994/996/997/998/902/904/916/917/914 各自 claim 集=55 格、931=0；B01-B12 腿的 owner 需从 binding.tsv 生产者 token+残留套件〔a2r-hir-892?/999/ptr〕考据）；②rust 观察源 13（D03-09:av+I02/03/07/10/11+R05:av+I12:av←ace-av 报告；I01-I11:ac←ace-native closures 族具名测试——名匹配初查命中少，需按 binding notes 语义配对重建 1000 的 t03/scenario-rust-test-map）；③治理轴 G01-08:a2r（852 snapshot 结构化捕获——legacy_governance 绑定）；④E01-E03:ac/avm 腿（914/916 双映射+base b04/b20/b16 腿+avm 适配器——部分已在 probe 内待重验）。

**registered 370P/0F/636U**（自 273 +97）；峰值 37.82MiB<48。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 8 | from: work | to: work | status: archived | outcome: progress（probe 族 +97；legacy 250/313；63 格考古清单化） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d137148b5 | next: work（63 格考古：a2r 残腿 owner 考据→rust 映射重建→治理轴→E 腿；随后 Q4 认证战役→恢复/预算/系统控制→终冻组合）

2026-11-11 stage:work / PLAN-1005 r1 / round:9（**legacy 313/313 全闭合**）。

**考古三步全落地**：①corpus a2r 残腿 owner=BASE 套件自带 `a2r-snapshot` checks（adapter a2r/action emit）——legacy_base 加入 a2r 绑定 **+18**；②E01-E03 九腿+多 I 腿=`_PROBE_CHECK_CELLS` 双映射（b04/b16/b20→E03；ace.*/av.*→I）——base+914 以 probe 键补入（914 需 collections pin；worker pin 路径须绝对）**+23**；③rust 观察源 14 格按 binding.tsv notes 语义具名匹配（D03-09←av_dynamic_*；I02/03←av_dynamic_incremental body/signature；I07←paused_call_pins_old_body；I10←churn_512_bounded；I11←warm_edit_p90；I12←forced_degradation_gate；I10:ac←native churn）**+14**；④治理轴 G01-08：852 snapshot scope 结构化产物（97+5 绿+spawn-watch.json）**+8**。

**单终代纪律实战**：852 snapshot 的 PLAN-818 维护构建改写 release worker 字节→三代 worker 并存实证非确定性；按纪律把钉盘字节的 9 个 case 组（base/orig×2/typed×2/四套件）在当前代**单 invocation 终浪重采**（9/9 PASS）+Q4 认证重采+依赖快照重导+组合脚本重定向终浪。probe/a2r 族帧钉不受影响（帧内一致）。

**组合状态**：registered **433P/0F/573U**；blockers 余=Q4 认证域/c_core(scalar_q4 执行)/syntax_architecture(review 行凭据解锁)/恢复测量/预算/系统控制/evidence/missing_inputs——全部即剩余战役清单；峰值 39.22MiB<48。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 9 | from: work | to: work | status: archived | outcome: progress（**legacy 313/313=execution PASS**；433P/0F；单终代终浪落地） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d137148b5 | next: work（Q4 认证 7 族战役→scalar_q4/text_q4 执行→恢复测量+预算采样+系统控制→终冻组合→T-09）

2026-11-11 stage:work / PLAN-1005 r1 / round:10（Q4 认证战役开跑——text 族 20 行全过）。

**认证驱动模式落地并验证**：族合同（repo 跟踪 TSV/JSON，逐行 source/reference/oracle/caps/marks 钉定）× {rust,c} × {identity,certify,daily} 真实捕获（release a2x-worker，ace.m1.command/1 Job 计量收据，evidence 根相对 stdout/stderr 引用）→ 族绑定文档 → 严格 reader 准入。**text 族先行**：10 case × 2 靶=20 行全 PASS（certs/text/bindings.json）→ compose **+20（453P/0F）**；绑定路径父目录=prefix 根的布局规则实证。

**剩余族扩展（驱动骨架已通用化，按族差异参数化）**：scalar 61×2=122（attempt-N 子目录前缀+marks→expected_stderr+release 腿 8+required_variants trap 集）、record_text 16×2=32、collections 20×2=40（mode debug+execution_config）、query 9×2=18、supplement 4×2=8、original_data 18×2=36、CTE 7×2=14——共约 270 行×3 腿≈810 次 worker 调用（预计 1-2 小时纯计算）。随后 scalar_q4/text_q4 执行接线（c_core 门）→恢复测量+预算采样+系统控制→终冻组合。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 10 | from: work | to: work | status: archived | outcome: progress（认证驱动模式验证+text 族 20 行；453P/0F） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d137148b5 | next: work（scalar/RT/coll/query/supp/orig/CTE 七族扩展→执行接线→测量/采样/控制→终冻组合→T-09..T-11）

2026-11-11 stage:work / PLAN-1005 r1 / round:11（认证三族战役——635P 里程碑）。

**scalar+RT+coll 三族 194 行真实捕获全 PASS**：scalar 61case×2（attempt-1 前缀+marks→expected_stderr+trace caps）/RT 16×2（family--case 键+rt 文件集）/coll 20×2（mode debug+coll 文件集+execution_config）——identity/certify/daily ×194×3 全真实执行；**daily 路径规则修正**（reader 派生=certification prefix 父目录内 daily-<target>——194 个 daily 按正确位置重跑）；零失败零条件排除。compose **+182（635P/0F/371U）**——与 1000 Phase A 分域里程碑（633P）同量级；峰值 40.88MiB<48。

**剩余**：query 18/supp 8/orig 36/CTE 14 四族（76 行）+负面前门+release 腿；scalar_q4/text_q4 执行接线（c_core）；恢复测量+预算采样+系统控制；终冻组合。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 11 | from: work | to: work | status: archived | outcome: progress（三族 194 行；635P/0F） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d137148b5 | next: work（query/supp/orig/CTE 四族→执行接线→测量/采样/控制→终冻组合→T-09..T-11）

2026-11-11 stage:work / PLAN-1005 r1 / round:12（认证七族全接——673P/0F）。

**query/CTE/supp 三族 40 行真实捕获全 PASS 并经严格 reader 准入**：query 9×2（family--case attempt 前缀）/supp 4×2（仓内手写参考，含 reference-C01.c）/CTE 7×2（平铺前缀+旁侧收据+WIRE env——三项布局事实修正：收据位于 prefix 同级、stdout/stderr 引用=evidence 根相对、execution_observations 由 WIRE env 门控）。**root-cause 修复**：capture-q4-cert 驱动残留一行旧 write 持续改写 tracked reference-C01.c（工作树字节漂移的真正来源——三次 STALE 的根因；已删除该行并在净树上重采全部受影响认证；此事故链实证严格 pin 链对 tracked 输入漂移的拦截）。

**组合状态：registered 673P/0F/333U**；峰值 41.46MiB<48。已闭域：legacy_current+legacy_required（313/313）+syntax_architecture 的 48/49+q4 认证 7 族 254 行+first-slice 20。

**剩余（精确交接）**：①original_data 认证 52 行（26 case×2，含 16 负面前门行——驱动模式同七族+负面前门布局 certs/orig/<family>--<case>/<target>）；②scalar_q4/text_q4 执行接线（c_core 门——绑定模式同 supplement admit_source，批采副本已在 legacy-batch）；③恢复测量+预算采样（10 策略×10 运行+2 专项窗）+系统控制（治理组收据）；④终冻 H+全量组合（预期 ~990P→1005P/0F/1U，审计行唯一 pending→正式 exit1）→T-09 独立复审+真实凭据 R→T-10 TAG 复核+tag_ready→T-11 归档/清理/合入 master。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 12 | from: work | to: work | status: archived | outcome: progress（七族 254 行准入；673P/0F；驱动 write 缺陷根除） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d137148b5 | next: 后续会话继续（orig 认证→执行接线→测量/采样/控制→终冻组合→T-09..T-11）

2026-11-11 stage:work / PLAN-1005 r1 / round:13（**961P 冻结候选复现**）。

**original_data 认证 52 行 + 六执行族接线——registered 673P→961P/0F**：
- orig 认证：36 正行（record_text_certification，attempt-1）+16 负面前门（daily-emit 拒绝 exit2+typed_frontdoor ORACLES 权威）全捕获准入；保留界 512→768KiB（提交 d4c07ae3b）
- 执行六族（scalar_q4 122/text_q4/record_text/collections/source_cte 14/source_supplement 8）在当前终代重采（旧批 worker 代不符）+identity+副本接线；**collections 冷启动 cargo 回退实证**（PLAN-1000 已知边界——runtime 预热后重采零回退）；三族依赖快照 schema 各异（rt-q4 11pin/collections 12pin/text-q4 4键）逐一对齐
- **961P/0F/45U——与 PLAN-1000 T-11 冻结候选逐数一致**；峰值 47.87MiB<48（贴近上限）

**剩余 45U 构成（精确）**：review 行 1（凭据解锁，设计使然）+ **release 腿 44**（`rust_release_runs==1` 案例的 ACE-A2R-HIR 槽——需 scalar_release_execution/record_text_release_execution 绑定：release-941 conformance 测试+AUTO_PLAN_ARTIFACTS+绝对 RUSTC argv pin，PLAN-1000 t05/release-run 模式）。

**blockers**：q4_oracle_certification（release 腿）、syntax_architecture（review 行）、precise_incremental_restore、budgets_time、系统控制、evidence、review_credential missing_input——全部即剩余清单。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 13 | from: work | to: work | status: archived | outcome: progress（**961P/0F**；剩 release 腿 44+测量+控制+终冻） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d4c07ae3b | next: work（release 腿绑定→恢复测量+预算采样+系统控制→终冻组合 1005P/0F/1U→T-09..T-11）

2026-11-11 stage:work / PLAN-1005 r1 / round:14（**V-12 冻结候选达成：1005P/0F/1U**）。

**release 腿+查询执行+负面前门全落地**：
- release 三族（scalar 8/RT 4/query 2）=release-941 conformance 真实运行（AUTO_PLAN_ARTIFACTS 持久 home 须预建；工件目录不得改名——index 内 cwd/argv 逐字 pin 原位）；parent=invocation result.json+discovery 旁侧+每族 8/9/9-pin 依赖快照 → **+14**
- query/instances 执行（13-pin 依赖+两套件终代重采+emit-only leaf42 金样例外）→ **+24**
- 六负面前门（3 诊断源×双靶 daily-emit 拒绝 exit2+typed_frontdoor 通道）→ **+6**

**registered 1005P/0F/1U——V-12 冻结候选目标达成**（唯一 U=review 行，凭据解锁）；峰值 47.92MiB<48。**正式 `--acceptance` exit1**（合法未签）+具名 blocker：syntax_architecture(review 行)、precise_incremental_restore、budgets_time、evidence_incomplete、系统控制、missing_input×3（review_credential/恢复测量/预算测量/治理控制）——全部即剩余清单（t08/freeze-candidate.json）。

**AC-07 主体达成**：非review键 1005 全部真实闭合；剩余=三个测量/控制域（恢复测量+预算采样+系统控制）+终冻+T-09..T-11。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 14 | from: work | to: work | status: archived | outcome: progress（**1005P/0F/1U 冻结候选**；正式 exit1） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d4c07ae3b | next: work（恢复测量+预算采样+系统控制→T-08 全闭→终冻 H→T-09 独立复审+凭据→T-10/T-11）

2026-11-12 stage:work / PLAN-1005 r1 / round:15（三测量域接线推进——E-1005-02 宿主阻塞三证）。

**测量域结构与依赖全部厘清**：①恢复测量+预算时间共用同一冻结 gate 文档（GROUPS=ace-source/ace-syntax/infra-ace-m1-arch 单 invocation，`admit_incremental_restore_measurements` 与 `admit_budget_time_measurements` 各自消费）+优化观察者（release-941 run.command.json，已有）+采样绑定（10 命令型策略×10 真实运行+2 单窗策略）；②系统控制=**infra-ace-m1-arch 组的报告字段**（实测本轮 gate 文档中 report: executed=287/failures=[]/errors=[]——**16 控制类全绿可用**，与 gate 文档同源）；③采样战役驱动已修正待跑（源路径改合同真值 M01.at/collections-926--L01.at）。

**E-1005-02（宿主能力阻塞，三证）**：measure-gate 三次净序重跑，`syntax_budget::performance::syntax_budget_64mib_streaming_small_queries` 全部在 **1800.0-1800.1s 被 nextest-full.toml 冻结 slow-timeout（900s×terminate-after=2）终止**（exec_time 三证齐；该基准 round:4 单机空闲时 1503.8s 通过——当前宿主负载下持续超限）。ace-source/infra-ace-m1-arch 两绿稳定。**gate 文档须单 invocation 三组全绿**，不可拼接（拼接=伪造）。**解除**：宿主空闲窗重试（~30min/次）或 owner 对基准放置的政策决定（超出本包授权——冻结测试语义不可放宽）。注意 samples 战役勿与 gate 并发（并发负载即 round:15 首败根因）。

**剩余精确链**：E-1005-02 解除→gate 绿→采样战役（驱动就绪）→三域绑定接线（optimized=release run.command.json；controls=gate 文档 report 字段）→正式组合 blockers 清零（除 review_credential）→终冻 H→T-09 独立复审+真实凭据 R（凭据消费链已就绪并经 21 项单测）→1006P/0F/0U 八域真→T-10 TAG-01..06+tag_ready→T-11 归档/合入 master。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 15 | from: work | to: work | status: archived | outcome: progress（三测量域结构全厘清+驱动就绪；E-1005-02 三证入档） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d4c07ae3b | next: work（E-1005-02 宿主空闲重试→gate 绿→采样→三域接线→终冻→T-09..T-11）

2026-11-12 stage:work / PLAN-1005 r1 / round:16（**E-1005-02 解除+三测量域全闭——blockers 仅剩 review 凭据**）。

**E-1005-02 解除**：宿主空闲（9.5GB）重试 measure-gate 三组单 invocation **全绿**（ace-source+ace-syntax 132/132+infra-ace-m1-arch 287——64MiB 基准本次界内通过；前序采样战役并发负载即失败根因的最后实证）。

**三测量/控制域全部接线闭合**：①恢复测量=gate 文档+四伴读组（semantics 243/comptime 149〔空闲重跑绿〕/abi 263/av f94109 过滤 3 测试——全量 av 文档超 48MiB 峰值，按行需测过滤是伴读合同允许的具名导航）+ct-restore 机制观察 → **8/8 PASS measurements_complete=true**；②预算时间=gate+optimized（release 收据）+采样战役（12 命令策略×10 真实运行+a2x-scalar-formal 10 certify 窗+两单窗策略专项小窗 auto-test-case 113 窗/ace-abi scalar_bridge 14 窗——大窗策略不得挂 gate 大文档即超 70s 预算）→ **6/6 PASS budgets_time_complete=true**；③系统控制=gate 文档 report 字段（287 执行/0 失败/16 控制类）→ **controls_complete=true**。

**正式组合 blockers 收敛至 3=全部 review 行凭据链**：syntax_architecture（review 行 pending）+evidence_incomplete（同因）+missing_input:review_credential——**其余全部为零**（legacy×2/c_core/q4/restore/budget/registry/系统控制/八域中七域全真）。registered **1005P/0F/1U** 保持；峰值 47.98MiB<48。正式 exit1（合法未签，凭据专属）。

**AC-07 全闭达成条件已满足**：全部非review required 真实闭合（1005）、唯一 review 行 pending、全键守恒、受影响完整职责/必要consumer有效、SD/来源/失效清单见 §9 历轮收据。**T-08 work 侧完成**；next=T-09 独立复审（停写同树→独立核→生成凭据 R→consumer 验证→新鲜组合 1006P/0F/0U）。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 16 | from: work | to: review | status: execution_done | outcome: pass（T-08 全闭：三测量域绿+blockers 仅剩凭据链；1005P/0F/1U） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: d4c07ae3b | next: review（T-09 独立复审@冻结树→凭据 R→1006P 八域全真→final review）

2026-11-12 stage:review / PLAN-1005 r1 / T-09 独立复审完成——**final pass**。

**独立复审执行**（冻结树 d4c07ae3b+abb6baa7f 停写；同权威树顺序独立上下文）：①反控全量重执行 287 passed+1189 子测；②冻结候选新鲜重组复现——registered counts 与 49 架构行判决逐值相同、review 行唯一 UNOBSERVED、preview 复核恒不可签 exit1；③1006 键分母从冻结 ledger 再derive；④admission-source/recipe/registry 身份 pin 逐项核对；⑤三项 finding（F-1005-03/E-1005-01/E-1005-02）全部 resolved 有据；⑥工程修正一项：消费链候选 bindings 等值检查改为无环（签前候选无凭据 binding——其 bindings 身份逐字绑定凭据；R 绑定候选 bindings+报告双 sha；最终报告引用 R 原件 sha；提交 abb6baa7f，治理 287 全绿保持）。

**签前凭据 R**（t08/review-credential.json，sha `b9304ef038ec…`）：frozen_commit=42475cdf3（∈gate commits）、候选双 sha（bindings 95e4ec9f…/报告 87a2ae97…）、行规则 cab19420/registry 310a107b、audited 48 行/1006 键/唯一排除 review 行、AC-01..09+V-01..13+SD-01..03 全覆盖、3 findings resolved、verdict=pass。

**最终正式组合（t08/final-report.json，sha `ee7785ee59b3…`）：凭据经严格消费链接纳 → review 行唯一解锁 → registered 1006P/0F/0U、八域门全 TRUE、blockers=[]、can_claim_expanded_m1=true、峰值 47.98MiB<48 → `--acceptance` exit 0 可签**。V-09/V-12/V-13 全闭合：无凭据不可签（exit1 具名）→ 凭据消费链验证 → 唯一行解锁 → 1006 全绿。

**AC-08 达成**。next=T-10（TAG-01..06 逐门复核+tag_ready=true+SD 主线读回）→T-11（归档/清理/合入）。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 17 | from: review | to: merge | status: archived | outcome: pass（T-09 final pass：凭据 R b9304ef0→1006P/0F/0U 八域真 exit0） | worktree: D:/autostack/.wt/lang-1005/auto-lang | commit: abb6baa7f | next: merge（T-10 TAG 复核+tag_ready→T-11 归档/清理/合入 master）

2026-11-12 stage:merge / PLAN-1005 r1 / T-10 完成——TAG-01..06 逐门复核+tag_ready=true+SD-01..03 主线读回。

**TAG 复核（t08/tag-review.json）**：TAG-01 候选 C=abb6baa7f（树净；frozen 42475cdf3 为祖先；含全部修复代码+审计+canonical 沉淀 ec8340820 之前身）；TAG-02 严格终验 1006P/0F/0U、ARCH 49/49、八域全真、accounting/evidence/system 真、cert-gap/awaiting/missing/blockers=0；TAG-03 凭据 R b9304ef0 经消费链强校验、绑定 frozen/事实/规则、最终报告引用 R 且不自指；TAG-04 反控真实执行（治理 287+1189@abb6baa7f；无凭据正式 exit1 具名；修前反例 t01 复现档案）；TAG-05 组合内 44 admission sources 逐 pin 重验、producer 闭包单终代对账；TAG-06 本地无 v0.6-ACE-M1 标签（**tag_ready=true**；实际打/推 tag 为复核后单独动作，本计划不创建不推送，不冒 M2/全 v0.6）。

**SD 主线读回（worktree 提交 ec8340820）**：SD-01 architecture.md（严格封套+逐行 recipe+凭据授权边；旧行为记历史）；SD-02 ace-m1-expanded-acceptance.md（1000 历史保留+本包修正后全量重建/凭据/三态/TAG 事实）；SD-03 新增 ace-m1-architecture-evidence-admission.md（49 行来源/封套/映射/分域同边界/凭据版本/拒绝语义/TAG 门）。

stage: merge | prepared: reviewed 基线 abb6baa7f+canonical ec8340820 | next: T-11（land→archive→clean）

2026-11-12 stage:merge / PLAN-1005 r1 / T-11 完成——landed/archived/cleaned，**delivered**。

**T-10 收尾补充**：跨检出 pin 身份修复（提交 05b7c9d7e，PLAN-1000 先例）：本包再生的工件以 CRLF 工作字节写入而仓 blob 为 LF——rebase 后两检出读 LF、CRLF 计算的 pin 断裂（master 9 红）。修复=工件 LF 规范化+模块 CONTRACT_SHA/RECIPES_SHA+registry expanded_contracts 重钉+机制夹具观察者源 pin 重绑；凭据 R 重绑（8fe85f8d）于 LF 基线；最终正式组合复核 **1006P/0F/0U exit 0**。**master 双侧治理 287 passed+1189 子测全绿**。

**T-11 执行**：①最终摘要折入本计划（§0/§9 历轮收据 68P→1006P 全链+凭据链+LF 修复）；②SD-01..03 已随 ec8340820（rebase 后 05b7c9d7e 祖先）落 master 主线（architecture 47f19067/expanded-acceptance 7de0286c/evidence-admission aebb2260）；③`git mv` 归档 docs/plans/archive/ + status: archived；④wt-guard clean（两检）→worktree/分支移除；⑤plan-artifacts/plan-1005 整删（证据已折本计划；无具名 hold）；⑥retention 刷新。

**交付事实**：delivery commit `05b7c9d7e`（master ff-only，range-diff 等价全等）；10 提交链（ff27a1454→05b7c9d7e，rebase 映射 3a007e7fc 等 9 项+LF 修复）；正式报告 t08/final-report.json（已折摘要：1006P/0F/0U、八域全真、blockers=[]、exit0、凭据 8fe85f8d、峰值 47.98MiB）；**tag_ready=true**（TAG-01..06 全过；本地无 v0.6-ACE-M1；实际打/推 tag 为后续单独动作）；1000 保持归档不回写。

stage: goal | plan_id: PLAN-1005 | plan_revision: r1 | round: 18 | from: merge | to: done | status: archived | outcome: pass（delivered：1006P/0F/0U 八域真 exit0+tag_ready+SD 沉淀+归档+清理） | worktree: （已移除） | commit: 05b7c9d7e | next: done（批量回归门 due→regress 单列）

## 10. 待澄清事项

1. **schema/recipe具体实现**：缺少行级事实类型、真实case/test/原始观察位置及可信审计锚的细节由T-02在4条最小真实全链确定；无可执行决定不进入对应实现。它们不能降成新的PASS布尔或测试名存在性检查。
2. **独立性与机械校验边界**：程序校验R内容、来源/冻结身份和覆盖，实际独立review由同树独立上下文流程担保并记真实身份；不宣称hash或reviewer字符串能独立证明人的独立性，不建设PKI。用户未指派独立模型/会话，不自动派发。
3. **历史raw已清**：T-01核可合法获得的当前证据，必要时用原producer重新测；若新严格链暴露原M1语义缺口，不假定旧1006P能豁免。不能只有修复反例绿而缺真实M1正例闭合。
4. **tag是后续动作**：本轮只规划。未来须TAG-01..06及独立review/主线来源真实成立，方按既有条件授权另行创建本地annotated tag；推送依当时明确授权，不在本计划自动追加。旧同名tag保留不强移；远端不可读不冒发布成功。
5. **无扩大路线授权**：本包不重新打开1000/941/999，不接管其他会话，不施工M2。若修复确超有界准入/最小观测范围，维持原终验责任，先具名修订而不悄悄放宽。
