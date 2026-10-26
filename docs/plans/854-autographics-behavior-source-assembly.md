---
plan_id: PLAN-854
status: reviewed
feature_name: AutoGraphics行为源码生成与版本兼容整理
author: [Codex]
created_at: 2026-10-26
updated_at: 2026-10-26
plan_revision: 1
supersedes_spec_components:
  - docs/specs/auto-graphics/behavior-api.md
  - docs/specs/auto-graphics/auto-source-driver.md
new_spec_components: []
touched_goals: [GOAL-008]
affects: [auto-graphics]
current_step: 6
total_steps: 9
---

# [PLAN-854] AutoGraphics行为源码生成与版本兼容整理

## 0. 变更摘要

M2/S0第二份优化合同，覆盖OPT-01：在853基线与证据工具交付后，整理Rust侧behavior/1..5的transport/API/entry构成，减少隐式版本链和无检查文本替换。保留最终生成字节/来源身份与旧运行语义，不修改Auto算法，不增加公共功能。

## 1. 目标

- 每个Api的受信来源、变换顺序、profile/版本/区域标签和错误边界可审计，只有一个当前生成入口。
- v1..v5合法用户源码的实际生成字节、原始用户字节、hash/regions/诊断与运行行为等价，旧项目直接可用。
- 对必需trusted片段/替换边界作明确检查，错误生成不能带着表面正确的hash进入编译/发布。
- 在853同条件基线下报告源码生成/编译/运行与资源对照；主要成果是兼容与可维护性，不能凭行数宣称加速。

非目标：改API、namespace规则、Auto数学展开、查询/运动协议、managed空间/CPU/资源预算、缓存新生命周期、SDK/AGP/依赖、Host拆分、游戏/编辑功能或HIR/ACE。[共同合同](attachments/853/planning/optimization-contract.md)组成此Plan保持面与门禁合同。

## 2. 架构方案

保持`Api→trusted部件构成→admit→namespace→SourceSnapshot→validate→Play`链，隔离纯生成与用户准入/文件读取/来源映射。可以拆入driver-local模块，保留`behavior::{Api,SourceSnapshot}`公开Rust路径；不新增对游戏作者可见的接口。

生成过程由显式每版本配方/具名步骤表达，而非依赖分散条件的覆盖顺序。是否纯函数模块或小型数据表由T-02比较决定；不造通用模板/宏语言/第二parser。生成步骤可以复用，但必须保持原顺序、空白/换行/namespace字节及原区域名；必要marker必须存在且匹配预期次数，不能失败后返回旧版本或半份字符串。

## 3. 技术栈

现有Rust/String、Parser与固定SDK，沿用853测量/证据工具。无新依赖；`.at`算法/API文件、samples、两个锁、AGP不变。

## 4. 需求分析与背景调查

授权来源为2026-10-26用户的M2设计/战略及优化规划请求。本轮仅起草，实施须启动；无另开agent/建仓/推送授权。853交付是硬前置，不凭其draft假定工具/结果存在。

先读[模块概览](../specs/auto-graphics/overview.md)、[行为API](../specs/auto-graphics/behavior-api.md)、[源码驱动](../specs/auto-graphics/auto-source-driver.md)、[运动](../specs/auto-graphics/bounded-motion-profile.md)。现行Specs定义语义/身份，本Plan只补生成结构和受检边界，不改它们的旧承诺。

M1观察与SHA见[source-observation](attachments/853/planning/source-observation.json.txt)。`src/authoring/behavior.rs`1214行；`SourceSnapshot::from_bytes_for`286–389附近用条件、compact和replace组合五版本，v5又注入derive/leaf同步。`bridge`依赖Builtin6来源分段；`segment_transport/segment_base_api/segment_compact/namespace`分别承担现有变换。行数含测试/诊断，未证明性能缺陷。

[853](853-autographics-m2-baselines-and-evidence.md)负责先导出真实生成基线/分段性能与当前闭包；本Plan执行前补其实际交付commit、工具命令和fixture清单。当前M1只是调查锚，不是尚未知的854实施冻结提交。

## 5. 详细设计

### 5.1 前置与入口

唯一树`D:/autostack/.wt/lang-854/auto-lang`、`plan-854-dev`，从853实际交付图形提交创建，最终只合`autographics-dev`。853已交付，实际基面c1961665d1a90a8705994085fd4344a1d6a9a564；T-01已核前序允许diff、SDK/锁/AGP/预算和fixture身份。854不得与853抢改工具。

写面限driver的`src/authoring/behavior.rs`、新增`src/authoring/behavior/assembly.rs`（拟定新路径）、相关`tests`、当前source/consumer清单和必要README。API/bridge/entry最终字节不变；不以减少Auto helper或增加缓存改变运行成本/owner。

### 5.2 先锁输出，再整理实现

T-01消费853实际产物并在改assembler前重算，固定五Api＋LF/CRLF/Unicode/合法局部名称fixture的真实source bytes独立SHA、API/bridge/combined、版本/profile、regions与managed长度。新增`tests/behavior_source_identity.rs`（T-02建立）使用已冻结输入/期望，不从待测生成器自行更新golden；若853交接缺项先补前置证据，不能改后再补“基线”。

T-02写`attachments/854/work/assembly-decision.md`：实际部件数据流、namespace/诊断依赖、受检marker列表及次数。最多比较“具名纯函数”与“显式版本配方表”两种局部组织；确认可逐字节保持后再进入T-03。若发现必须改变合法源码、字节身份或Auto算法，则停依赖实现，回new给具体不兼容证据，不擅自升版本。

T-03建立一个生成入口，返回完整受信transport/API/entry及对应来源元信息；版本选择显式且不允许未支持版本隐式fallback。因外部输入引发的read/parse/admission/compile/callback/output阶段、path/line/column和无span说明保持；新增内部生成一致性错误不得伪装成用户语法错误。合法输入不会触发该内部错误。

marker断言需要实际负控：在测试局部输入中删/重复必需片段或改变目标，让相应构成步骤拒绝；恢复原片段成功。不能只数replace次数而未检最终字节，更不能绕过准入直接信任快照hash。

### 5.3 保持完整消费路径

继续由同一`SourceSnapshot`检查user/API/bridge/combined、regions、base scene与版本；实际`Play::new_user`消费结果。原始用户字节不重写；同回调read-your-writes、常量初始化、嵌套返回边界、sticky error/last-good、Stop/新Run/绑定项目与v2..v5token/预算原样。

T-04新增当前`research/plan854-identity.json`/verifier，沿853明确当前入口更新消费导出；旧清单不改。再用853命令同机同配置同输入测量，记录编译/生成与tick/资源结果，不能用M1低分辨率聚合对比新细分指标。没有新百分比收益门；若硬预算/语义退化必须修复，不以测量噪声辩解。

### 规范增量

| delta_id | add/modify/retire | docs/specs/... target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-graphics/behavior-api.md | 旧五版本身份/诊断保持，明确受信部件单入口、marker拒绝与来源区域责任 | 可审计且不改变公共契约 | AC-02/03/04 |
| SD-02 | modify | docs/specs/auto-graphics/auto-source-driver.md | 更新854源码/consumer入口与生成等价证据，保持旧profile/SDK/预算 | 当前实现定位 | AC-01/05/07 |

## 6. 测试设计

实施/自查与独立最终review按共同合同，同一实现树顺序进行；完整门归merge。未分配模型或其他agent。

| Case ID | AC/不变量 | 实际入口/输入 | 区别错误实现的oracle | work任务/边界 |
|---|---|---|---|---|
| V-01 | AC-02 字节/版本 | 新behavior_source_identity，五Api与853固定合法fixture | 实际source独立hash/长度/regions/版本/profile全部等于前置基线；只改空白/替换次序也红 | T-02/03；已支持fixture，不把有限hash表当任意源码全证明 |
| V-02 | AC-03 受信构成 | 局部生成输入删/重复marker、原片段恢复 | 构成阶段可定位拒绝，未进入compile/Play；恢复字节等价 | T-03；测试变异不改固定API文件 |
| V-03 | AC-04 用户规则 | `tests/authoring_behavior.rs`现有合法名称/常量/嵌套返回、保留名/坏回调/坏UTF8/边界源码 | 实际Parser准入、诊断阶段/用户span和managed标签一致；合法局部名称不能误拒绝 | T-03/05；原单文件profile |
| V-04 | AC-04 完整消费 | authoring_behavior_host、motion/segment/sweep API/Host、绑定项目及新Run | 真实Auto数值/read-your-writes/token/sticky错误/last-good/Stop与原owner一致，/5接受/未来版本拒绝 | T-05；五版本原范围 |
| V-05 | AC-01/05 闭包/性能 | 当前verifier、独立consumer、853基线工具 | 漏新模块拒绝；消费者实际运行；同条件原始测量+资源并列，无硬预算/owner退化 | T-04/05；非任意GPU性能承诺 |

定向入口：`cargo check --locked`；`cargo test --locked --test behavior_source_identity -- --test-threads=1`（T-02新增）；`cargo test --locked --test authoring_behavior --test authoring_behavior_host -- --test-threads=1`；相关版本的`motion_api/segment_api/sweep_api`和受改Host命名case。853基线/当前verifier命令由T-01核实后记录；不得把尚不存在工具当已跑证据。最终完整两端/原native/owner/content按共同合同由merge运行。

## 7. 验收标准

| ID | owner_stage | 可观察通过条件/验证 |
|---|---|---|
| AC-01 | work | 853实际交付与本树/SDK/依赖身份闭合，允许写面受限、当前清单完整；V-05 |
| AC-02 | work | 五版本配方/数据流显式，只有一个生成入口；实际输出字节/hash/区域/标签/版本完全等价；V-01和源码审查 |
| AC-03 | work | 必需片段缺失/重复受控拒绝且恢复成功，无fallback/半生成；V-02 |
| AC-04 | work | 旧合法/错误源码、诊断与Play/查询/运动/项目语义、预算/owner全部保持；V-03/04 |
| AC-05 | work | 新模块独立consumer消费；853同条件性能/资源对照齐备，未冒领未测收益；V-05 |
| AC-06 | review | 独立核所有work AC/V/SD与warnings/fmt/遗漏；merge-owned完整门pending |
| AC-07 | merge | 冻结两端完整面、全部受影响native/owner/content/新面通过；复用逐项证明，失败不豁免 |
| AC-08 | merge | 源码落autographics-dev、实际SD/ledger/归档和guard清理收据；交接855真实提交 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 实际位置/操作与结果 | AC | 状态 |
|---|---|---|---|---|---|
| T-01 | work | 853交付 | 核前置基面/工具/fixture与所有保持面，创建唯一树，补实施入口和允许diff；先复现原输出 | AC-01/02 | [x] |
| T-02 | work | T-01 | assembly-decision与新behavior_source_identity，固定原实际字节/诊断基线后才改生成 | AC-02 | [x] |
| T-03 | work | T-02 | behavior.rs及拟定assembly.rs；单入口/显式版本/受检marker，V-01..03 | AC-02/03/04 | [x] |
| T-04 | work | T-03 | 当前854 identity/verifier/consumer与853同条件对照，V-05 | AC-01/05 | [x] |
| T-05 | work | T-03/04 | 真实Auto/项目/版本Host定向回归、V-04，提交最终代码/SD/证据，不跑整档 | AC-01..05 | [x] |
| T-06 | review | T-05 | 独立最终review R1 pass；[完整AC/V/SD证据](attachments/854/review-r1/review.md)，merge门仍pending | AC-06 | [x] |
| T-07 | merge | T-06 pass | 冻结两端完整/受影响原native-owner-content/新面，核不可变runner/选中ID | AC-07 | [ ] |
| T-08 | merge | T-07 | 源码只合autographics-dev；main沉淀SD/overview/plans/ADR/ledger/索引并归档 | AC-08 | [ ] |
| T-09 | merge | T-08 | guard clean后清树/临时分支，保留恢复产物/实际9/9，交接855 | AC-08 | [ ] |

## 9. 复审记录

2026-10-26 new r1：stage=new；outcome=blocked（依赖等待，不是设计否决）；next=853交付后核冻结提交/工具/fixture并进入T-01。范围/AC/V/T/SD均已草拟，未实施/测试；status保持drafting。禁止将“853计划已建立”写成其工具已可用或854实施基线已冻结。

2026-10-26 work entry：用户明确授权实施；853已 archived/delivered 9/9，实际交付与 autographics-dev HEAD 均为 c1961665d1a90a8705994085fd4344a1d6a9a564。唯一树 D:/autostack/.wt/lang-854/auto-lang / plan-854-dev 从该提交创建，顶层/分支已断言。SDK固定681文件已从Git对象重建。主检出既有未跟踪 crates/Cargo.lock 和 test/fixtures/modularity 输入已披露并保持不动。当前 T-01 正在重算原输出，尚未改assembler；review/merge门未执行。

2026-10-26 T-01/02完成：本树原实现先独立构建并导出21文件，与853保留runner重导出及交付baseline-r2逐字节一致；[基面/实际输出SHA](attachments/854/work/entry.json.txt)。[局部组织决策与高风险执行合同](attachments/854/work/assembly-decision.md)已先于生成器修改记录，选择具名纯函数+穷尽Api匹配，历史v5无效替换只移除不启用。十组原输出metadata冻结入tests/fixtures/behavior-source-853.json（无更新模式）。目前新identity测试1例与实际trusted边界负控2例通过；初次整profile/版本文本替换碰到return10前缀而被新次数检查拒绝，已改完整函数声明边界，最终五版本字节等价通过。cargo fmt --all因固定SDK嵌套workspace不适用，改本package fmt通过；未改SDK/锁。T-03..05继续，未送review。

2026-10-26 T-03完成：实现提交16fea18a8，唯一单入口/五版本具名路径、受检marker与部件、internal assembly错误已接入。新assembly3例、冻结identity1例、原8个定向binary43例全通过，[命令/选中ID/日志SHA](attachments/854/work/scoped-receipts.json.txt)。cargo check通过（SDK原146 warning，driver0新），package fmt/diff检查通过；新增部件断言后identity仍等价，原行为所用字节与SDK未变，43例证据复用。447来源/681SDK闭合，源码删/多/坏SDK/恢复负控、工具7永久例通过。T-04/05性能与独立消费者进行中，未运行整档。

2026-10-26 work最终交接：`stage: work | plan_id: PLAN-854 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-854/auto-lang | branch: plan-854-dev | base_commit: c1961665d1a90a8705994085fd4344a1d6a9a564 | code_commit: 523d27b863dbe306c89af0ea546334225a29f8e9 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467 / AGP与两锁/API/samples不变 | task_ids: T-01..05 | evidence: attachments/854/work/readiness.md + handoff.json.txt | blockers: [] | next_responsible_role: independent review`。work-owned AC-01..05完成，execution_done5/9；[逐AC/V就绪记录](attachments/854/work/readiness.md)、[最终身份/实际命令/原始日志SHA](attachments/854/work/handoff.json.txt)、[同条件性能/资源完整并列](attachments/854/work/performance-comparison.json.txt)。生产者56个具名定向例、消费者21个唯一例（含五API实际VM运行头）、工具7永久例通过；十fixture/447来源/681SDK与删改/恢复负控通过；50新进程1500暖tick与853语义/来源/像素/资源/owner完全一致。实际时间略高，不宣称加速；没有硬预算退化或放宽。测量后提交只补cfg(test)反例，非测试字节/其余身份完全一致的复用证明已绑定，不重复性能campaign。SDK原146 warning，driver0新；check/fmt/diff通过。开放finding/必需work覆盖缺口/未决决策为空。T-06独立review和T-07冻结完整两端/原native-owner-content/AGP门仍pending，T-08/09合入沉淀归档清理未做；源码只交autographics-dev。实现树clean、写权冻结并保留供同树review；本轮work自查不是独立终审。

2026-10-26 final review R1：`stage: review | review_scope: final | reviewer_identity: /root/review_854_r1（独立子agent上下文，未参与实现，未写实现树） | plan_id: PLAN-854 | plan_revision: 1 | outcome: pass | worktree_path: D:/autostack/.wt/lang-854/auto-lang | branch: plan-854-dev | reviewed_commit: 523d27b863dbe306c89af0ea546334225a29f8e9 | base_commit: c1961665d1a90a8705994085fd4344a1d6a9a564 | dependency_revisions: SDK7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467/681文件、AGP/两锁/API/samples未变 | acceptance_results: AC-01..06、V-01..05、SD-01/02 pass | findings: [] | evidence: attachments/854/review-r1/review.md + checks.json.txt + audit.json.txt | next: merge`。已实际核worktree注册/HEAD/分支/clean，重放identity1/assembly4并独立附加2例组合反例，工具7例通过；43行为+8项目及consumer21唯一例的原始SHA/具名选择/结束核验，复用基于生产前缀和依赖/输入未变。50组1500暖tick原始语义/来源/像素/资源/owner及P50/P95重算通过；447来源/681SDK的独立consumer逐文件核对，十fixture确为853原产物。冻结规范增量SHA256 `6a29da3e8857594c97f22aecf0f762ba6ee62643f533661fbef96a44e96f4b9a`，四项Spec输入hash见audit.json.txt。未写canonical/ledger，未修代码。历史853 runner构建环境metadata不足已在报告明确（实际两端PE栈均32MiB、运行配置/工具链/锁一致）；不宣称完全相同的构建环境记录或加速。最终复审次数1、失败0，T-06闭合，reviewed6/9。`pending_closeout_gates: T-07完整producer/fresh own-SDK consumer及原native-owner-content/AGP复用核准/current source gates；T-08限定autographics-dev合入与规范沉淀归档；T-09 guard清理`。禁止把review pass当作已交付或绕过上述merge-owned门。

## 10. 待澄清事项

853实际交付前置已闭合；T-02组织决策已记录。当前无work阻塞、必需覆盖缺口或未决合同问题。需要改变公共身份/接受规则/预算、Auto算法或生产仓边界时，先据证据回new修订，不在work静默扩面。
