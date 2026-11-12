---
plan_id: PLAN-1025
status: drafting
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
current_step: 0
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
| T-01 | work | 1013/1016归档，后续实施指令 | 核真实repo/树/分支/base，复现F-01..03，身份/采证闭包矩阵，LF字节方案可行性决策；AC-01/04 | [ ] |
| T-02 | work | T-01决策闭合 | 有界属性规则+合同/常量/registry完整级联，新R合同，不改产品语义；AC-01..03 | [ ] |
| T-03 | work | T-02 | V-01..08永久控制、完整职责组/必要consumer，定稿SD提案；AC-01..03 | [ ] |
| T-04 | work | T-03，所需主线依赖吸收/冻结 | 盘点合法复用或真实重采，固定worker/窗口，构造仅review U候选与预算核验；AC-04 | [ ] |
| T-05 | review | T-04/work项完成 | 独立全范围复审、生成新R，H实际正式消费，AC-05/06；失败回work，不先勾合入项 | [ ] |
| T-06 | merge | T-05 pass，身份稳定 | 合入C并逐项核代码/工具/配置/合同/registry/二进制闭包；漂移回相应任务，不临时改pin绕审；AC-07前半 | [ ] |
| T-07 | merge | T-06，原件仍可读 | C实际正式读回+TAG六门，核本地标签目标/范围，记录C/H/K/R/报告身份与结果；AC-07 | [ ] |
| T-08 | merge | T-07全部闭合 | 沉淀SD、折收据摘要、guard与全输出对账、登记清理/归档并确认8/8；AC-08 | [ ] |

当前唯一开工入口T-01，无实施任务完成。T-01..04完成才execution_done，T-05 pass才reviewed，T-06..08实际闭合后才archived/delivered。原M1义务问题保持具名阻断并有界修复，不能为“最后一包”预先承诺必过或移至M2；扩大产品范围/预算才需另行裁定。

材料统一`D:/autostack/plan-artifacts/plan-1025/`，开工登记owner/用途/在途状态；正常构建留专属树，外置target限该scope。原件留至T-07通过；T-08将必要命令、退出码、数字、C/H/K/R/报告hash、控制结果和全输出路径对账折入计划后按政策全删。未闭项持有需具名owner/原因/关闭条件，未知材料不盲删。移除实施树前执行`bash D:/autostack/wt-guard.sh D:/autostack/.wt/lang-1025/auto-lang`且clean。本轮不创建材料目录或worktree。

## 9. 复审记录

2026-11-12 stage: new；plan_id: PLAN-1025；plan_revision: 1；outcome: pass（计划合同可交接，非产品验收通过）；next: work（后续实施指令）。

规划预检：已读当前相关Spec、reader/registry/pin、测试入口与1013落地收据，将F-1025-01..03映射至T-01..08/AC-01..08/V-01..10/SD-01..03。只读观察证明两个合同STALE与registry身份变化，不声称已执行新控制或完整终验。仅写本计划与中央编号，未改既有归档或其他会话WIP。

后续按实际阶段记录完整基线/命令/退出码/结果，不预填pass。连续两次独立终审失败时，先诊断剩余不变量、此前漏检原因、实际路径与oracle调整，再进入下一轮，不能以归档/转债绕过合同。

## 10. 待澄清事项

无阻碍规划交接的产品决定。T-01核全部同源失配、当前原件与真实采集量，属于有界调查，不要求用户猜技术方案。执行/独立复审角色未指派，本轮不启动会话。实际tag创建/推送按收口后当时授权单独办理，本包不自动发布。
