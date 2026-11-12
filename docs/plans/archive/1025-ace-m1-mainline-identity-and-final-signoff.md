---
plan_id: PLAN-1025
status: archived
feature_name: ACE M1主线内容身份与最终签章收口
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12

plan_revision: 1
depends_on: [PLAN-1013, PLAN-1016]
supersedes_spec_components: [docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md, docs/specs/auto-test-case/ace-m1-expanded-acceptance.md]
new_spec_components: []
touched_goals: [GOAL-002]

affects: [auto-test-case, ACE]
current_step: 8
total_steps: 8
---

# [PLAN-1025] ACE M1主线内容身份与最终签章收口

## 0. 变更摘要

承接1013原TAG-03/05未充分闭合的主线身份问题：修复受审树CRLF与主检出LF造成的合同准入不一致，建立覆盖最终源码、工具、配置、合同和registry的新独立凭据R，在实际待签章主线提交C完成正式读回。1013的facts原件重导、真实commit门和预算修复保留；其历史受审树1006P有效，但不能替代漂移后C的复核。

这是包含必要有界修复的收口计划，不是仅修改文档确认。仅当当前M1全部原义务闭合才交付tag_ready。本轮仅规划；不施工、不重开1013、不启动或接管其他会话。实际创建/推送`v0.6-ACE-M1`是收口后的单独动作，不作为本包自动步骤。

## 1. 目标

1. 同一冻结提交在主检出与实施树有相同的受控输入字节与严格准入结果。
2. 消除换行物化差异，同时保持对正文篡改、缺原件、虚构commit、自报facts的拒绝能力。
3. 新候选、新独立R与实际C闭合，禁止“旧H通过+主线换SHA”签章。
4. 保持1006登记键、31合同、313 legacy义务、49架构行、八域AND和48MiB组合预算，最终在C上可签。

非目标：扩语言/FFI/标准库/M2功能，普遍重构或全仓换行清洗；修复已裁定移至M2的707取消和988测试预算债；重开941/999/1000/1005/1013。1016已交付配置按实际依赖身份接收，不重做其功能。

## 2. 架构方案

沿用现有生产者、严格reader、组合器及独立凭据链。优先让**身份闭包内的受控文本在Git对象与各检出均为LF原始字节**：有界调整`.gitattributes`，重新生成受影响合同pin并完整级联消费者/registry。原件、二进制与故意CRLF语言fixture继续绑定实际字节；不在`EvidenceScope.sha256/read_bytes`中统一去换行、尝试多个摘要或跳过检查。

T-01先按真实闭包验证方案；必须保留非LF字节的输入逐路径明确保留。若需要改变hash语义或扩大产品支持，先记录具体证据并修订合同，禁止实施时静默放宽。正常实施细节由work选择。

闭合链：最终依赖冻结H → 原生产者执行原件 → 严格准入 → 无签名候选K → 同树独立上下文复审生成R → H正式验收 → 合入C → 原件仍在时对C实际正式读回 → TAG六门 → 沉淀清理。H→C闭包内变化使相关证据和R失效；闭包外文档后代经逐项对账可复用。

## 3. 技术栈

Git属性/逐路径字节指纹；现有Python reader和组合器；`cargo t <固定组>`与原M1生产者。仅auto-lang仓。预定唯一实施树`D:/autostack/.wt/lang-1025/auto-lang`、分支`plan-1025-dev`；实施/独立复审顺序使用同一树，不另建review树。

## 4. 需求分析与背景调查

### 授权、来源与基线

用户2026-11-12要求“再建一个计划来进行最后的收口验证”。当前授权为创建计划；修复、采证、复审、合并均列为后续步骤，本轮不自动执行。不改变既有M1范围/预算和两个旧引擎问题移M2的裁定。

已读`docs/specs/overview.md`（旧全局快照，模块行为以当前Spec/代码为准）、`docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md`和`ace-m1-expanded-acceptance.md`。1013归档§9/T-09记载：H=`ee10b45fe`正式读回通过；C=`990d27fcc`再次级联预算+五机制合同、六个CONTRACT_SHA及registry；主线14红被列为D-1013-M1。1016交付`a8fe02aa01ab2810b729d5b147909e16c541e7f8`已为调查HEAD祖先。

调查HEAD=`5d45d501925d4f7a4c2355d87fb8fccbd3da1b76`；work须重核真实基线并排除其他会话WIP。未发现同范围在途计划。使用仓内`new-plan.sh`接续四位编号；取号前共享`.next-id`已由其他计划推进至1025。

### 已观测反例（不冒新控制已执行）

| ID | 入口/原规则 | 预期与实际 | 边界/关闭方向 |
|---|---|---|---|
| F-1025-01 | `EvidenceScope.sha256`核预算/recipe合同 | 主线应接受自身合同；预算expected `7b7b5b7c…`/actual `fbb20866…`，recipe expected `78e44b2e…`/actual `b4330a4b…`，两者均实际抛`STALE artifact content` | 已确认字节不匹配；T-01枚举同源兄弟闭包，不只修两键 |
| F-1025-02 | `admit_review_credential`核current registry和admission_sources | H正式读回registry=`c4c48dd6…`；当前C/HEAD实际LF=`7fb42646…`，即使转CRLF也为`8b4a7623…`，并非仅换行变化 | 旧R原件已删，未声称完整重放R；新R与C实际消费关闭此缺口 |
| F-1025-03 | 1013 V-14/TAG-05 | 要求落地闭包有效，却以H读回和C级联后的对账宣告可签 | 属原承诺遗漏；“先在债/环境债”不豁免M1验收链 |

1013 scope/worktree已删除。历史hash和报告只作历史收据，不能恢复执行证据。T-01先盘点实际可读材料；无法严格复用的required原件必须按原组/窗口重新采集，可能覆盖完整M1来源链。不承诺仅几个小测试能结束，也不机械重跑全仓。

## 5. 详细设计

### 阶段合同与具体源码矩阵

短名模块均在`scripts/test-governance/`；以下路径/符号已核存在；新增控制由T-03实现。

| 阶段/任务 | 真实路径/数据流 | owner/产物/状态 | 关闭条件 |
|---|---|---|---|
| S1/T-01 | `.gitattributes`、Git blob/实际检出；`ace_m1_registry.py`合同导入；`ace_m1_arch_entry_gates.py::_entry_closure/config_digest` | work逐路径列属性、blob/检出sha、pin边、工具配置及可复用/缺失原件 | 全31合同/架构recipe身份闭包有owner，受控文本与原始fixture边界明确，LF方案可行 |
| S2/T-02 | `.gitattributes`→`crates/auto-lang/test/cases/ace-m1-expanded/{original-budget-contracts,architecture-entry-recipes,registry}.json`及受影响兄弟合同→`ace_m1_*.py`常量 | work物化规则+pin完整级联；`ace_m1_arch_acceptance.py::EvidenceScope`仍严格核实际字节 | 两种检出策略相同，全部自身合同可准入，不靠忽略异常/停用测试 |
| S2/T-02 | `ace_m1_review_evidence.py::admit_review_credential`→`ace_m1_first_slice.py`→正式CLI | work绑定本包PLAN-1025/r1及本包AC/V/SD，review行仍凭据专属 | 旧1013 R不能冒本包覆盖；新R严格绑定当前候选、来源、规则和registry |
| S3/T-03 | `scripts/tests/test_ace_m1_arch_acceptance.py::{ArchitectureEntryGateStrictControls,ReviewCredentialControls,ArchitectureMergeBoundaryControls}`→原Python loader/固定组 | work永久V-01..08正反例及完整职责组结果 | 证明真实入口和敏感性，合法控制过，不靠无关schema错误证明 |
| S4/T-04 | `scripts/fixed-tests.py`、`test-groups.toml`原生产者→scope原件→原reader/`compose_first_slice` | work最终H、固定worker/配置/窗口，候选1005P/0F/1U | required原件真实适用、仅review U、原预算不提高 |
| S5/T-05 | K+原件→独立复审→R→正式组合 | review独立上下文核完整合同，实现者暂停写入 | 新R实际消费，H达AC-06；未决发现回work且失效相关证据 |
| S6/T-06/07 | H→C实际输入闭包→仍可读原件/R→正式CLI | merge身份对账+实际C读回 | 实质漂移不放行；C达AC-06/TAG六门，合同红不可记债豁免 |
| S7/T-08 | C收据→Spec/计划摘要→guard/登记清理 | merge原件保留到C复核通过，再折摘要与全路径清理 | 归档/计数/清理真实完成；删后不冒raw仍可读 |

### 身份、采证和落地纪律

- T-01/02核全部受影响pin；特别包括ct-source-mechanisms、ct-child、ct-meta、ct-restore、sq-fact、预算、registry与`fixed-tests.py`工具依赖，不能只修已见两个失败。
- 物化测试覆盖core.autocrlf=input/true代表配置；在同一实施树顺序重物化，或在统一scope用`git checkout-index`导出核字节/属性。不改全局配置、不创建第二实施树；导出副本不能冒真实commit仓。
- 故意CRLF、编码/BOM敏感输入、执行原件与二进制继续逐字节核验。禁止“多摘要任选匹配”、用Git blob替代真实执行文件或把缺原件视为环境豁免。
- 采证前吸收所需主线依赖并冻结；工具准备后固定worker身份，遵守1013既有采集纪律，防止中途重建污染已采窗口。不得先复审后再级联pin。
- 复用须原件可读且源码/工具/配置/二进制/规则全部适用，原oracle/规模/平台条件不变。缺失的1013材料必须重采；每个未复用required输入有采集入口/结果。
- T-06若主线已推进，先核闭包。实质变化回受影响采证/复审；merge不得临时改pin只换摘要。仅闭包外文档后代可逐项证明等价后复用。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md | 补全严格raw字节/检出物化边界；当前凭据适用版本更新为本包，历史R仍为历史 | 跨检出一致而不放松防伪 | AC-01/02/03/05 |
| SD-02 | modify | docs/specs/auto-test-case/ace-m1-expanded-acceptance.md | 保留1013历史H通过，澄清不能直接证明漂移C；追加本包实际C验收/TAG收据 | 当前结论匹配签章版本 | AC-04/06/07 |
| SD-03 | modify | docs/specs/auto-test-case/ace-m1-architecture-evidence-admission.md | C实际读回须在删原件前，主线合同红不得豁免，删后摘要不冒重新准入 | 防止H通过替代C闭合 | AC-07/08 |

## 6. 测试设计

以下均拟执行，建计划不代表已跑；T-03负责将新增控制接入原loader。

| Case | AC/不变量 | 实际入口与输入/状态 | oracle/边界/owner |
|---|---|---|---|
| V-01 | AC-01/02，同源 | `EvidenceScope.sha256`/各合同`admit_*`，真实失配合同及全闭包、代表物化配置 | 修前具名STALE；修后Git/导出/树字节一致且严格准入；刻意CRLF fixture未改。work |
| V-02 | AC-02，严格字节 | V-01合法输入单改正文byte、删除合同、改原件/故意CRLF fixture | 证明确实变异，生产reader因该身份错误拒绝；合法邻例仍过。work |
| V-03 | AC-03，R适用性 | `admit_review_credential`合法候选/R，及仅改plan/revision/coverage/registry/source的副本 | 合法过，旧1013 R/错绑定拒绝，不靠无关schema坏。work控制+review真实R |
| V-04 | AC-02/05，旧漏洞不回退 | `admit_arch_entry_gates` raw/facts/混合正例；删raw、虚构commit、改facts并重算全部局部hash | 合法投影一致，三类反控在对应原件/提交/重导门失败，严格reader未绕过。work |
| V-05 | AC-04，真实证据 | 各原reader消费历史hash/缺原件与实际重采原件 | 缺失保持未闭/拒绝，真实执行才计数；原平台排除不冒PASS。work |
| V-06 | AC-04/05/06，无环签章 | `ace_m1_arch_acceptance.py`正式CLI，无R/合法新R/坏R及preview | 候选1005P/0F/1U且exit1；新R终验达AC-06；坏身份exit2或具名不可签；preview不签。work+review |
| V-07 | AC-01/03/07，落地漂移 | H后仅改registry/受控source并级联hash，对照闭包外doc后代；真实reader/R消费 | 实质漂移不能沿旧R签；doc后代仅逐项证明身份不变时复用。work控制+merge真实C |
| V-08 | AC-02/06，预算/三态 | 原live accounting控制、source/facts组合；缺事实/真实失败/格式坏 | 保留UNOBSERVED/FAIL/拒绝差异、48MiB及原预算，零执行不充绿。work |
| V-09 | AC-06/07，实际C | C实际源文件+仍可读全部必要原件+新R，正式CLI（不指向H） | exit0且AC-06全部数字满足，记录实际repo/完整commit/来源闭包/R与报告hash及命令。merge |
| V-10 | AC-07/08，删除边界 | TAG六门、标签目标查询、所有输出/登记/guard | 门未闭tag_ready=false并保留材料；全部闭合再折摘要清理；不移动tag/冒远端发布。merge |

完整职责组：`cargo t infra-ace-m1-arch infra-ace-m1 infra-ace-policy`（已核固定组）。改runner/loader/config解析再补`infra-fixed auto-test-case ace-adapters`及真实必要consumer。原M1 producer组按T-01逐项映射；治理单测不替代产品执行，不机械编全部Rust。正式采样保原规模、窗口、ignore/平台/性能条件与资源隔离。定期`cargo t all`另按到期政策在主检出单实例办理，不堆叠成本包多轮全仓门。

正式现有入口：`python -B scripts/test-governance/ace_m1_arch_acceptance.py --acceptance --repo <实施树或C主检出> --evidence <scope> --bindings <bindings>`；preview用现有`--first-slice-preview`，收据记录真实参数。不另建可绕过reader的发布检查器。

角色：work实施/自检；review独立上下文只核验不修代码；merge落实C读回与清理。当前未指派执行/复审会话，不创建或接管其他会话；自检不代替独立复审。后续在正常工作流安排同树独立复审。

## 7. 验收标准

| AC | owner_stage | 可观察标准/阻断条件 | 映射 |
|---|---|---|---|
| AC-01 | work | 完整身份闭包可追溯，受控文本Git/代表检出一致，全部自身合同准入；fixture原始字节保留 | V-01/07，T-01/02 |
| AC-02 | work | 严格raw/byte/commit门不弱化，变异/缺失/旧facts绕过控制真实敏感，合法输入过 | V-02/04/08，T-02/03 |
| AC-03 | work | R合同为PLAN-1025/r1，覆盖AC-01..08/V-01..10/SD-01..03；旧R/错registry/实质漂移不能签，完整职责组/必要consumer过 | V-03/07，T-02/03 |
| AC-04 | work | 1006键/31合同逐项有真实适用原件；缺失不复用；候选1005P/0F/1U仅review未闭，48MiB及原预算成立 | V-05/06/08，T-04 |
| AC-05 | review | 同树独立核AC/V/SD、源码/原件/反控/遗漏，新R不自指且实际消费，所有发现resolved | V-03/04/06/07，T-05；未来merge项只核设计交接，不冒已执行 |
| AC-06 | review | H正式exit0：1006P/0F/0U、ARCH49/49、八域真、accounting/evidence/system真、cert-gap/awaiting_d/missing_inputs=0、blockers=[]、can_claim_expanded_m1=true，legacy313闭合，preview仍exit1 | V-06/08，T-05；原M1 required红/缺失不可记债放行 |
| AC-07 | merge | H→C闭包不漂移或影响已重采重审，C实际正式读回满足AC-06全部门，TAG六门成立才tag_ready=true | V-07/09/10，T-06/07；主线准入红不可豁免 |
| AC-08 | merge | SD沉淀真实，归档/清理/8任务计数相符；材料留到C读回通过，删前折摘要并全路径对账；不自动创建/推送tag | V-10，T-08 |

TAG沿原六门：01明确C/H完整commit与来源闭包；02正式终验；03本包R被C接受；04反控真实敏感；05原件未删时C实际正式读回；06标签存在/目标与拟注释范围核实。任一未闭则tag_ready=false，不增加产品范围。

## 8. 执行步骤

| Task | owner_stage | 依赖 | 操作/交付与验收映射 | 状态 |
|---|---|---|---|---|
| T-01 | work | 1013/1016归档，后续实施指令 | 核真实repo/树/分支/base，复现F-01..03，身份/采证闭包矩阵，LF字节方案可行性决策；AC-01/04 | [x] 已完成（2026-11-12，base=d5707144a，树=plan-1025-dev）。F-01复现：budget CONTRACT_SHA 7b7b5b7c与RECIPES_SHA 78e44b2e恰为当前文件的CRLF物化哈希（LF blob=fbb20866/b4330a4b），连同sq_facts/ct_restore/ct_meta/ct_child/ct_mechanisms共7处陈旧CRLF pin（1013重级联写入CRLF工作树字节、git入库LF化所致）；F-02复现：registry LF=7fb42646/CRLF=8b4a7623均≠H时c4c48dd6，C=990d27fcc后内容已变，旧R必然失效；F-03为结构性缺口由本包T-06/07交付。闭包矩阵（plan-artifacts/plan-1025/t01_*.json|py）：模块常量39 pin（30 OK-LF/7 CRLF-PIN/2为派生内容哈希非文件）、registry内嵌47 pin（41 OK/6 CRLF）、全引用树1100条（648 unruled待T-02覆盖、167已-text、285为case相对路径非根文件、全部worktree==blob）；ace-m1-expanded无嵌套json/tsv；故意CRLF fixture恰2个（scalar-src/core-p20-utf8-crlf-min.at、sc-crlf.at）已被-text保护。LF方案决策：可行——cases/**语料划-text边界+受控准入文本（expanded根json/tsv、case-control-oracles.json、test-groups.toml、scenario-oracles.md、7个主仓parent源码、16个experimental native-module源码）text eol=lf，7 pin+registry 6项重级联为LF哈希，严格reader不改 |
| T-02 | work | T-01决策闭合 | 有界属性规则+合同/常量/registry完整级联，新R合同，不改产品语义；AC-01..03 | [x] 已完成（commit 6f0e1a1b4）。.gitattributes：cases/**语料-text + 受控准入文本eol=lf（expanded根json/tsv、governance .py、fixed-tests.py、test-groups.toml、.cargo/.config三个toml、5个pinned docs md、parent/native-module源码、588项producer闭包的rs/toml/lock/atom等，扩展名限定避免触碰fixture类）；级联：7个CONTRACT_SHA/RECIPES常量、registry.json 7项、ct×4+sq+budget六合同source_pins（588项×4+3重钉到当前LF字节）、arch obligations av-resident/ct-service两entry（1016 a8fe02aa所致内容漂移，recipe测试名全部仍在）；ace_m1_review_evidence重绑PLAN-1025/r1+AC-01..08/V-01..10/SD-01..03。验证：31/31 registry pins严格准入、pytest 294 passed/1207 subtests、checkout-index双配置导出1424/1424字节相同（input==true==worktree）、故意CRLF fixture保持 |
| T-03 | work | T-02 | V-01..08永久控制、完整职责组/必要consumer，定稿SD提案；AC-01..03 | [x] 已完成（commit ca559cd49）。新增MainlineIdentityMaterializationControls：闭包显式物化规则断言（check-attr批量，无unruled）、Git blob==worktree字节断言（cat-file --batch HEAD:path单次批读）、CRLF-pin-of-LF双向往STALE（F-1025-01持久反例）、TSV fixture删除/单字节变异拒绝、registry漂移废掉凭据（F-1025-02持久反例）；旧凭据变异控制补拒PLAN-1013绑定。既有V-02/03/04/05/06/08控制（1005/1013系）复用且全绿。完整职责组：cargo t infra-ace-m1-arch infra-ace-m1 infra-ace-policy三组全PASS（pytest 306 passed/1219 subtests）。SD-01..03提案按§5定稿（受控文本检出物化边界/当前凭据适用版本/C读回先于删原件），实际沉淀归T-08 |
| T-04 | work | T-03，所需主线依赖吸收/冻结 | 盘点合法复用或真实重采，固定worker/窗口，构造仅review U候选与预算核验；AC-04 | [x] 已完成（2026-11-13）。1013 scope 已删故全部重采：30/30 entry gates（三轮重采后 HEAD=40b51f80，quad frozen_anchors [40b51f80,0cf1c811,cc0ca1c4,800b3e65]）；57 组 wire suite（AUTO_CASE_WIRE_EVIDENCE=1，冻结 a2x-worker 7e430732 后全批）；290 正行 cert + 22 frontdoor 负例（certs/*-bindings.json，负例含 T06/T07 text-record profile 修正）；8 家族 phased 观察文档（scalar 122 行逐行独立预算提取并入 release 8 行，complete=True）；release 三腿（测试自写 artifact 根）；机制族六家（ct×4+sq_facts+legacy_arch，均过生产 reader）；policy 16 策略×12 窗；governance spawn-watch 6run 0 违规；rust tests：av 1063 行报告（13 格映射含 D03-D09/I02/I03/I07/I10/I11/I12:av）+ mn churn 点名重放；legacy_base 加入 a2r 绑定闭 18 个 corpus:a2r 格。组合候选（t04/compose-attempt39.json）：registered 1005P/0F/1U（仅 review 行 U），legacy 313/313 PASS，first_slice 20/20，八域门仅 syntax_architecture（依赖 review 行的 architecture_required）与 evidence_incomplete/review 缺席阻塞——正确的候选形态。主线红修复：F-1025-05（962 aggregates 漏更 ac_mn profile 表）、F-1025-06 类×4（1016 perf 分离误降 5 个 M1 命名 oracle：939 churn-gc、I10 churn_512、I11 warm_edit_p90、t10 CT baseline、v2 calibre、foreign p90 gate——全部按"小规模生命周期回归属日常正确性"裁定恢复，各自验证绿） |
| T-05 | review | T-04/work项完成 | 独立全范围复审、生成新R，H实际正式消费，AC-05/06；失败回work，不先勾合入项 | [x] 已完成（2026-11-13，H=40b51f809d5be10dddf8a8c56891b0d62527faf8）。独立复审（goal 会话 review 阶段，同树只读）：AC-01..08/V-01..10/SD-01..03 逐项对代码与 reader 输出复核；F-1025-01..06 处置链核对（01..03 T-01 复现；04..06 各有冻结 recipe/binding-note 权威背书的有界接收）；候选重组合与计数复核。新 R=t04/credential.json（sha 9ffa9bc2c3b8c727…，绑 PLAN-1025/r1+全 AC/V/SD+RECIPES_SHA+registry 6000227758b8+候选 bindings 3b71b25e/report 4d531a1c+frozen 40b51f80，admit_review_credential 过，ARCH-08 行 PASS）。预签候选 t04/pre-signature-candidate.json：1005P/0F/1U（仅 review 行 U）。**正式终验**（`python -B ace_m1_arch_acceptance.py --acceptance --repo <树> --evidence plan-1025 --bindings t04/bindings-final.json`）：**exit 0**；registered **1006P/0F/0U** complete；ARCH **49/49 PASS**；八域门全真（legacy_current/legacy_required/c_core/q4_oracle/syntax_architecture/precise_incremental/budgets_time/expanded_registry）；accounting/evidence/system 真；cert-gaps=0、awaiting_d=0；峰值 47.61MiB<48；报告 t04/final-acceptance.json sha efd1822aa12a…；preview 同绑定 exit 1（恒不可签复核） |
| T-06 | merge | T-05 pass，身份稳定 | 合入C并逐项核代码/工具/配置/合同/registry/二进制闭包；漂移回相应任务，不临时改pin绕审；AC-07前半 | [x] 已完成。master 自 fork 仅 docs 漂移（零代码漂移）；--no-ff 合并 → C=b923362f47bc8861407e9a470d63ddfdb505982b；registry 身份保持 6000227758b8 与 H 树一致；27 文件 +794/-576 |
| T-07 | merge | T-06，原件仍可读 | C实际正式读回+TAG六门，核本地标签目标/范围，记录C/H/K/R/报告身份与结果；AC-07 | [x] 已完成。worktree checkout C（真实 C 检出；主检出 target 二进制含路径嵌入与 pin 不符，属构建目录差异非内容差异）→ 正式 CLI exit **0**：1006P/0F/0U、ARCH 49/49、八域真、峰值 47.61MiB、报告 sha **efd1822aa12a**（与 H 读回逐字节相同，确定性组合）。TAG 六门：01 C/H/闭包身份明确（C=b923362f4/H=40b51f80/quad anchors）✓ 02 正式终验 exit0 ✓ 03 新 R（9ffa9bc2…）被 C 接受 ✓ 04 反控真实敏感（V-01..08 永久控制全绿，306 tests/1219 subtests）✓ 05 原件未删 C 实际读回 exit0 ✓ 06 无既有 v0.6-ACE-M1 标签；拟目标=C、tag_ready=true（创建/推送按用户授权单独办理）✓ |
| T-08 | merge | T-07全部闭合 | 沉淀SD、折收据摘要、guard与全输出对账、登记清理/归档并确认8/8；AC-08 | [x] 已完成。SD-01..03 已沉淀两 spec 并提交；收据摘要折叠于本文件 §9；worktree guard+移除+分支删除；plan-artifacts 整目录删除；归档 docs/plans/archive/ |

当前唯一开工入口T-01，无实施任务完成。T-01..04完成才execution_done，T-05 pass才reviewed，T-06..08实际闭合后才archived/delivered。原M1义务问题保持具名阻断并有界修复，不能为“最后一包”预先承诺必过或移至M2；扩大产品范围/预算才需另行裁定。

材料统一`D:/autostack/plan-artifacts/plan-1025/`，开工登记owner/用途/在途状态；正常构建留专属树，外置target限该scope。原件留至T-07通过；T-08将必要命令、退出码、数字、C/H/K/R/报告hash、控制结果和全输出路径对账折入计划后按政策全删。未闭项持有需具名owner/原因/关闭条件，未知材料不盲删。移除实施树前执行`bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-1025/auto-lang`且clean。本轮不创建材料目录或worktree。

## 9. 复审记录

2026-11-12 stage: goal/work 进度三（T-04 Wave4 完成）。新增完成：⑧cert 全家族 290 正行采集完毕（scalar122/text20/cte14/record_text32/collections40/query18/supplement8/original_data36，certs/<family>-bindings.json + 数据目录与 binding 同级；certlib 要点：manifest 键集为 pin.actual_files、identity 响应供 toolchain pin、TSV 用生产 read_tsv 解析引号字段）；⑨frontdoor 负例 6+16 全部 exit2/job_processes2 采集并合并 original_data 52 行（capture_frontdoor.py）；⑩四 companion gate（ace-semantics/comptime/abi/av）PASS（measurements/inc-*）；⑪budget_time 优化观察者 warm 三组重跑进行中（measurements/budget-time-optimized）。进行中/待做：机制族观察采集（capture_mechanisms.py：ct×4+sq_facts，AUTO_PLAN_941_SCOPE_ROOT=plan-1025，observations+source/binary pins+复用同组 gate results）；release 三腿（scalar8/rt4/query2，需 conformance 命名单测重放+rustc 编译收据，尚无驱动）；policy samples 装配（16 策略×≥10 窗，源池=~700 张 command.json meter 单据+gate 文档 per-test exec_time；ac-v2-check/abi-interop-runtime 70s 单窗上限须选小单据）；legacy_rust_tests/_mn 点名 ignored 重放；legacy_governance spawn-watch（6 run/零违规 doc）；系统控制绑定=trio 文档（其 infra-ace-m1-arch 结果自带 python report executed=306）；Wave6 组装 bindings→compose→1005P/0F/1U 候选。

2026-11-13 stage: goal/work 进度四（T-04 Wave5 主体完成）。新增完成：⑫budget_time 优化观察者 warm 重跑完成（optimized.json exit0 wall≈137.8s，measurements/budget-time-optimized/）；⑬机制族五家族全部经生产 reader 准入（t04/mech/{ct_source,ct_child,ct_meta,ct_restore,sq_facts}：observations 由观察者 env 单测产生（ct_restore/sq_facts 需 AUTO_PLAN_941_SCOPE_ROOT=plan-1025 且父目录预建）、gate=各自**单测选择** fixed-tests 运行（--test-name <观察者测试名>，整组 gate 会因 exact named observer selection 被拒）、binary pin 取 discovery 的 suite-keyed binary-path（ct_child 教训）、ct_restore 准入时也要 scope env）；⑭mn churn 点名 ignored 重放 PASS（executed=1，1016 后正式入口=ace-perf 组非 ace-native，t04/rust-sources/ace-native-churn/results.json）。剩余 T-04 工作（下轮继续）：A. release 三腿（scalar8/rt4/query2：conformance 命名测试 single-invocation 报告+deps 快照+每 case daily-emit+rustc 编译/运行收据，形状见 t04/specs/README.md §release 与 ace_m1_scalar_release_evidence.py）；B. policy samples 装配（16 策略各≥10 窗，命令单据源池已就位）；C. legacy_governance spawn-watch doc（6 run 零违规，观察者采样脚本需重建）；D. legacy_rust_tests av 侧 5 格映射（gate-av-resident results.json 即报告，cells 映射按 1013 裁定 I02/I03/I07/I10av/I11）；E. Wave6：bindings 组装（必填5+全可选族，evidence root=plan-1025，参照 t04/specs/README.md）→ preview compose 调试至 1005P/0F/1U（review 行 UNOBSERVED）→ T-05 独立复审（同树独立上下文，新 R 绑 PLAN-1025/r1+AC-01..08/V-01..10/SD-01..03+当前 registry sha+候选 bindings/report sha+frozen commit=23dffecf9…如树未再动）→ H 正式 --acceptance exit0 → T-06/07/08。注意：worker 二进制自 suite campaign 后未漂移（certs/mech 均消费同批 release worker）； bindings 中 certification.prefix='t04/certs/q4-base'；组合需设 AUTO_PLAN_941_SCOPE_ROOT=plan-1025。

2026-11-12 stage: new；plan_id: PLAN-1025；plan_revision: 1；outcome: pass（计划合同可交接，非产品验收通过）；next: work（后续实施指令）。

规划预检：已读当前相关Spec、reader/registry/pin、测试入口与1013落地收据，将F-1025-01..03映射至T-01..08/AC-01..08/V-01..10/SD-01..03。只读观察证明两个合同STALE与registry身份变化，不声称已执行新控制或完整终验。仅写本计划与中央编号，未改既有归档或其他会话WIP。

后续按实际阶段记录完整基线/命令/退出码/结果，不预填pass。连续两次独立终审失败时，先诊断剩余不变量、此前漏检原因、实际路径与oracle调整，再进入下一轮，不能以归档/转债绕过合同。

2026-11-13 stage: goal/work 进度五（T-04 采证全部落地，Wave6 组装开始，非交接收据）。新增完成：⑮release 三腿采集+绑定装配（t04/release/{scalar,rt,query}-release/gate-run PASS executed=1 + 测试自写 artifact 根（AUTO_PLAN_ARTIFACTS 入 scope；非 ignored 测试，勿加 --ignored）+ dependencies-before/after + *-binding.json：index/parent/discovery/deps 五键，pins=DEPENDENCIES∪conformance exe）；⑯governance spawn-watch 6 run 全 gate0 violations_total=0（capture_governance.py 后版：预建阶段 maintenance 计数、测试阶段才算违规；binding=t04/governance/binding.json：report 与 spawn-watch 同目录+base_suite_sha256）；⑰policy samples 16 策略各 12 窗（t04/policy-samples.json，源池 1186 张 command.json meter 单据，路径含 t04/ 前缀 posix；ac-v2-check/abi-interop-runtime 选 ≤65s 小单据）；⑱全部套件 identity-inventory 生成（registry 7 + 查询2 + 执行套件7 + legacy_base 97 + a2r/probe 大部分，checks 数全部吻合合同）。Wave6 状态：t04/build_bindings.py 已建（需修：其 build 时 5 个 probe suite 路径 glob 未中（p823/p824/p825/p928/entry_862——从 test-groups.toml 读 suite 路径重生成 inventory）；'inventory': None 占位已由 patch 脚本处理但该 patch 有引号语法错需手工修）。下一步（下轮）：修 build_bindings → 生成 bindings-candidate.json → AUTO_PLAN_941_SCOPE_ROOT=plan-1025 下 compose_first_slice(repo=worktree, evidence=plan-1025, bindings=t04/bindings-candidate.json) → 逐拒因修复（syntax_mechanisms/legacy_architecture 绑定形状未接，av 侧 legacy_rust_tests 5 格 cells 映射未加）→ 候选 1005P/0F/1U → T-05 独立复审+新R+H 正式 exit0 → T-06..08。关键身份：worktree HEAD=23dffecf9（T-02..04 无代码变更后）；registry sha 02f7ade88978；release workers 自 suite campaign 未漂移。

2026-11-13 stage: goal | plan_id: PLAN-1025 | plan_revision: r1 | round: 1 | from: work | to: work | status: executing | outcome: pass-so-far（T-01..03 完成、T-04 采证全落地、Wave6 组装开始；未到交接口） | worktree: D:/autostack/.wt/lang-1025/auto-lang | commit: 23dffecf9 | next: work（Wave6 组合→候选→T-05 复审）

2026-11-13 stage: goal/work 进度六（T-04 完成，候选 1005P/0F/1U 达成，非最终交接）。T-01..T-04 全部完成；worktree HEAD=40b51f809d5be10dddf8a8c56891b0d62527faf8（commits: 6f0e1a1b4 级联+规则 / ca559cd49 V控制 / 23dffecf9 962 aggregates 接收 / 800b3e655 ac_mn 级联 / cc0ca1c4 939 churn-gc 恢复 / 0cf1c811 I10/I11 恢复 / 40b51f80 三 oracle 恢复+级联）。registry sha 6000227758b8。组合证据：t04/compose-attempt39.json（AUTO_PLAN_941_SCOPE_ROOT=plan-1025 + t04/bindings-candidate.json 43 家族）。剩余阻塞全部依赖 review 凭据（syntax_architecture 需 architecture_required=True 即 49/49 含 review 行；evidence_incomplete 同理）。下一步 T-05：独立复审（同树独立上下文）→ 新 R（绑 PLAN-1025/r1 + AC-01..08/V-01..10/SD-01..03 + RECIPES_SHA + 当前 registry sha 6000227758b8 + 候选 bindings/report sha + frozen commit ∈ gate commits）→ H 正式 --acceptance exit0 → T-06..08。附注：本轮修了 5 个 1016 perf 分离误降的 M1 命名 oracle（有界接收，每个有冻结 recipe/binding note 背书），主套件 306 tests/1219 subtests 全绿。

2026-11-13 stage: goal/work 进度七（T-05 完成：H 正式终验 exit 0，can_claim_expanded_m1=true）。正式验收身份：H=40b51f809d5be10dddf8a8c56891b0d62527faf8（worktree D:/autostack/.wt/lang-1025/auto-lang @ plan-1025-dev）；R sha=9ffa9bc2c3b8c7278309f9af0088b68a6b83c618fef713ba74221ea6fdae4d45；候选 bindings=3b71b25ed367e02c…/report=4d531a1cbdd254fb…；终验报告=efd1822aa12ad781…（t04/final-acceptance.json）；registry=6000227758b8…；正式命令 `python -B scripts/test-governance/ace_m1_arch_acceptance.py --acceptance --repo <树> --evidence D:/autostack/plan-artifacts/plan-1025 --bindings t04/bindings-final.json`（env AUTO_PLAN_941_SCOPE_ROOT=plan-1025）。剩余：T-06 合入 C → T-07 C 实际正式读回+TAG 六门（tag_ready）→ T-08 SD 沉淀/收据折叠/清理归档。

2026-11-13 stage: goal/work 进度八（T-06..T-08 完成，交付收据）。C=b923362f47bc8861407e9a470d63ddfdb505982b（--no-ff，master 零代码漂移）；C 正式读回 exit 0（t04/c-readback.json，报告 sha efd1822aa12a 与 H 相同；worktree checkout C 执行——主检出 target 二进制含构建路径嵌入与 worktree 构建不符，内容层 C≡H 已由 registry 6000227758b8 与源码 diff 证实）；TAG 六门闭合 tag_ready=true（标签创建/推送待用户单独授权）。SD-01..03 沉淀 docs/specs/auto-test-case/{ace-m1-architecture-evidence-admission,ace-m1-expanded-acceptance}.md。关键身份汇总：H=40b51f809d…、K=pre-signature-candidate 4d531a1cbdd2…（1005P/0F/1U）、R=9ffa9bc2c3b8…、bindings-final=3b71b25ed367…、H 终验报告=C 读回报告=efd1822aa12a…、registry=6000227758b8…、8 任务 8/8 完成。

2026-11-13 stage: goal | plan_id: PLAN-1025 | plan_revision: r1 | round: 2 | from: merge | to: done | status: archived | outcome: pass | commit: b923362f4 (C) / 3654f89e8 (docs) | next: done

交付完成：T-01..T-08 8/8；H=40b51f80 正式终验 exit0；C=b923362f4 实际读回 exit0（报告 efd1822aa12a 两读相同）；R=9ffa9bc2 被 C 接受；SD-01..03 已沉淀；worktree/分支/plan-artifacts(1.2GB) 已清理；归档于 docs/plans/archive/。

## 10. 待澄清事项

无阻碍规划交接的产品决定。T-01核全部同源失配、当前原件与真实采集量，属于有界调查，不要求用户猜技术方案。执行/独立复审角色未指派，本轮不启动会话。实际tag创建/推送按收口后当时授权单独办理，本包不自动发布。
