---
plan_id: PLAN-850
status: archived
feature_name: Restore cross-workspace full regression ownership
author: [Codex]
created_at: 2026-10-26
updated_at: 2026-10-26
plan_revision: 1
completion_kind: delivered
current_step: 5
total_steps: 5
supersedes_spec_components:
  - docs/specs/auto-ui/project.md
  - docs/specs/auto-lang/project.md
new_spec_components:
  - docs/specs/auto-lang/plans.md
  - docs/specs/auto-ui/plans.md
touched_goals: []
affects: [test-tiering, auto-ui]
---

# [PLAN-850] 恢复迁仓后的跨 workspace 全量回归编排

## 0. 变更摘要

847恢复optional probe feature门控并通过独立审查及冻结裸t（3181/3181），落地主提交2ff1286c5。随后主检出batch的tf在测试执行前暴露新的跨workspace配置错误；本计划承接B847-FULL-OWNER-01，不把它混入847或降低覆盖。2026-10-26 用户明确授权实施，进入 executing。

## 1. 目标

让正式完整批量入口在真实语言仓/UI独立仓布局下执行原语言+UI全部测试面，保留1M churn、语料族、画廊围栏、UI生成族与既有时限、P95、资源隔离和逐ID覆盖；所有子门失败必须传播，不能零选择假绿。完成独立审查及主检出单实例全档后，才更新全局绿色锚点。

非目标：修改生产代码、把UI重新并回语言workspace、复制测试源码、删除迁出包测试、改为只跑UI日常档以替代原full、放宽/ignore/降低输入或阈值、修复本计划之外新发现的产品错误。

## 2. 架构方案

保留真实owner：语言测试在auto-lang workspace，UI编译器/方言/运行测试在auto-ui workspace。采用有界跨仓调度（优先复用已有脚本；必要时新增scripts/run-full-regression.py）协调实际Cargo manifest和原full配置，串行运行并聚合退出码。T01先实证原/新完整测试ID和配置解析，再定具体入口；禁止仅从tf删去三个UI包。独立UI的t/tu/ti/tn合同保持；是否给UI新增full入口由T01按完整选择与可维护性证据决定，不自动扩大至生产实现。

仓根解析复用env覆盖→同组兄弟→主检出规则，不建symlink/junction。所有代码/配置修改在同一lang-850专用树；如必须修改UI配置，需同组独立UI worktree、各自提交/审查/落地，不直接改主检出。

## 3. 技术栈

Cargo/nextest配置、Python标准库或仓内既有编排工具、Windows单实例回归。

## 4. 需求分析与背景调查

- 授权：847批量技能要求新红建立fix-forward，因此允许起草本合同；用户原847授权不自动扩为修改档位配置。2026-10-26 用户已明确授权实施并完成整个 850 工作流。
- 原始失败：docs/plans/attachments/847/batch-r1/tf.log.txt，main冻结2ff1286c5，Cargo报auto-ui-compiler requires dev-dependencies and is not a member of the workspace，exit101/零测试。
- 引入：831切源abd34a355从语言workspace移出五UI包，调整t/tt/tb/tu但tf仍选external auto-ui-compiler/dialect/runtime；原.tf完整语义不能靠删除包恢复。831合同§5要求全量测试语义随owner迁移并由跨仓编排保持，不能从根别名简单删迁出包。
- 当前UI owner：D:/autostack/auto-ui aee499e87093ff727f68ac8467c506e9ecf6ecdc，.cargo/config.toml只有t/tu/ti/tn；UI日常排除ui_gen，不能冒充full。
- Specs：docs/specs/overview.md、auto-ui/project.md测试与档位、auto-ui-compiler/project.md原ID/feature/integration/单进程生成约束、auto-ui-runtime/project.md资源/测试owner、auto-lang/project.md保留原输入/时限与完整选择面。
- 原ID参考：831迁移证据与attachments/846/merge-r2/auto_ui_{compiler,dialect,runtime}-pre831-root-binary-list.txt；不能只用源码函数计数替代真实nextest/test二进制选择面。
- 847完整batch已结束：tf在编译选择前拒绝；tt3553执行=3547pass/5fail/1timeout；tb3109/3109pass。tt的6项包含4个报告写入Windows1224、1个stdout-cap控制变会话wall、1个FFI018超时；相同原测试已在847原树裸t通过，FFI018又在本轮tb1.243s通过。不得把这些当tf配置修复已覆盖或冒称tt绿；T01须区分执行状态/负载与产品缺陷，产品代码需要修复时另建明确fix-forward，不能搭便车。主原始收据attachments/847/batch-r1/receipt.json.txt。

## 5. 详细设计

T01先闭合上述批量执行状态归因，保留每条失败和重放条件；报告输出必须保护现有WIP，既存时限/分类/P95不能放宽。再输出一份可重放owner/manifest/profile/feature→实际测试ID映射，确认原tf缺失问题和其余ta/taa/t3等同形external-package隐患；本计划首先恢复当前批量tf/tt/tb合同，其他别名仅做明确范围判定，不能悄悄保留已知同形缺口或改门语义。若发现不能在原支持内保留完整面，先修订本计划，不默认缩面。

编排须逐子门验证非零且预期ID全集；真实子进程退出码、超时/中断与失败停止/收集策略必须可观察，环境/目标目录不得改变worker既有产物发现。主检出full全程单实例，work/review只运行选择清点和与编排故障相关定向控制，不能反复启动真实全档。

### 规范增量

| ID | 操作 | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD01 | modify | docs/specs/auto-ui/project.md | 现owner档位→明确跨workspace完整批量入口和UI full责任，保留旧ID/feature | 补迁仓后遗漏的可执行编排 | AC01..03 |
| SD02 | modify | docs/specs/auto-lang/project.md | 原全量/隔离纪律保持，登记完整owner调度和失败传播 | 防止仅删迁出测试假绿 | AC01..04 |

## 6. 测试设计

- 原反例：原tf完整命令必须在当前两仓布局确定性拒绝external成员；修复入口须选中语言和UI两真实owner的原完整ID，原输入/profile不变。
- 正控：每个子门执行正确且原ID全覆盖时聚合exit0；负控：丢一个UI owner、零选择、manifest缺失、子门非零/超时/中断均不得绿；不需运行全量即可用真实选择清点+有界进程故障控制验证调度器。
- ID守恒按实际编译后的完整列表和迁前/迁后映射逐项核对，新增ID单列、合法重命名须明确来源；不以总数相等代替集合。
- work/review不运行整档t/tf/tt/tb/tu/taa；合入后主检出单实例tf/tt/tb（以及实际必要UI full子门）完整运行，全部通过才推进绿色锚点。

## 7. 验收标准

| ID | owner_stage | 标准 |
|---|---|---|
| AC01 | work | 原external-member反例关闭；完整语言/UI ID与profile/feature逐项映射零丢失；无删除/ignore/降低输入时限 |
| AC02 | work | 调度正负控、零选择/缺owner/失败传播/单实例路径可观察；真实owner与目标布局正确，无symlink |
| AC03 | review | 同一专用树独立上下文复审实际代码/ID/对照/SD；merge完整门明确pending |
| AC04 | merge | 冻结必要日常门、主检出完整批量及原UI full面实际通过；若新红先归因建fix-forward，不更新绿色锚点；Specs/ledger/归档/guard完成 |

## 8. 执行步骤

| ID | owner_stage | 依赖 | 结果 | AC |
|---|---|---|---|---|
| T01 | work | 用户授权 | [x] owner/manifest/profile/ID 全映射及 847 六红归因完成，见 work-r1 | AC01 |
| T02 | work | T01 | [x] F850-R1-01 修复冻结 f112d2737 | AC01/02 |
| T03 | work | T02 | [x] 各真实输入15项删除/ignore/过滤负控及正控/scope闭合 | AC01/02 |
| T04 | review | T03 | [x] 独立final repair review pass @f112d2737，F850-R1-01 CLOSED | AC03 |
| T05 | merge | T04 pass | [x] 完整batch-r4、规范台账、归档提交与三个工作树/分支/组目录清理均实证完成 | AC04 |

## 9. 复审记录

起草阶段：原始tf失败与831配置差异已核，未实施/测试新入口/独立审查。不得把拟议映射与控制当实测。因847 batch冻结主HEAD不变，本草案与.next-id待本轮batch结束后立即提交，未创建实现树。

## 10. 待澄清事项

2026-10-26 用户已明确授权实施，无待批准范围变更。所有原测试规模/预算/时限保持。

实施入口核对：主 HEAD 1c453e8166e574d9e2eae023eed1758b3f02f3e9；创建 lang-850 专用树。已确认 tf/taa/ta/t3 均仍引用迁出的三个 UI 包，UI 当前无 full 别名。T01 的逐 ID 清点与批量失败归因尚未完成；未执行整档测试，未宣称任何 AC 通过。


T01 续查：UI owner HEAD aee499e87093ff727f68ac8467c506e9ecf6ecdc。原 ID 输入为 attachments/846/merge-r2/auto_ui_{compiler,dialect,runtime}-pre831-root-binary-list.txt（实际 libtest ID 列表）。847 tt 六红已定位为 mir_native_rounds_failure_liveness_and_caps、ffi_dual_018_dep_fields、mir_native_turnaround_{body_edit_gate,fresh_cold_30,restart_warm_30,signature_rebuild_gate}；其中后四项日志明确 Windows 1224，尚需定向重放，不标已关闭。建树调用系统 WSL bash，检出完成后须以 Windows Git repair 修复 /mnt/d 管理路径，再断言身份，期间禁止实现写入。T01/T02/T03 仍未完成。

### 2026-10-26 work 实施记录（进行中）

- 用户授权已扩为完成整个 850 工作流。唯一实现树 `D:/autostack/.wt/lang-850/auto-lang`，分支 `plan-850-dev`，基面 `1c453e8166e574d9e2eae023eed1758b3f02f3e9`；UI 只读同组 detached `aee499e87093ff727f68ac8467c506e9ecf6ecdc`，未修改 UI 配置/生产实现。WSL 建树完成后已用 Windows `git worktree repair` 修复两仓管理路径，并断言语言根/分支及 clean 初态。
- 实证旧 tf 命令（nextest list，仅编译选择）exit 1，内部 Cargo exit 101：external auto-ui-compiler requires dev-dependencies and is not a member。新双 owner 完整清点：language 3718 IDs/3218 必选，UI 3091 IDs/3073 必选，迁前 UI 原始清单 3100 行含 9 个重复 ID，按 suite+测试名去重后集合零缺失/零新增。UI compiler/ui 显式恢复原 auto-lang/ui-iced 特征合一；runtime 默认 ui-iced 保持。语言 2ff1286c5..1c453e816 的 crates/.config/experimental diff 为空。
- 入口设计收口：cargo tf/taa/ta/t3 经零依赖 standalone Rust launcher 调用 Python 标准库编排，保留四档各自 feature/profile、tf 三集成目标；显式子串作用域仍支持，作用域证据不冒充 full。UI full 共用原 full/t3 配置，未新增/修改 UI 本仓配置；tt/tb 仍用原日常配置，各自清点必选 ID，不能拿 tf 的必选集替代。正式 batch=`cargo tf --batch`，序列 language→ui-full→tt→tb，首红即停。
- 执行契约：alias→launcher→真实 owner nextest list JSON→逐 ID/必选状态检查→同参数 run→exit/超时/中断收据；机器锁覆盖完整生命周期，未过滤 full 只允许主检出。根 env→组内兄弟→主检出，metadata 核对真实 UI 依赖路径。输入/时限/P95/资源配置未改，target 未重定向。已存在的 796/818 producer-only 报告先安全移开，保留生成产物、恢复原字节与 hash；文件被占用无法保护时在测试前拒绝。
- 定向区分性控制（Python unittest，当前 9 例 pass）：合法双 owner/完整 ID；缺 owner/过滤原必选 ID/零选择；无效显式根拒绝 fallback；首红 exit7 留收据且后门不执行；真实 exit0/7、超时及真实孙进程死亡；独立子进程互斥；失败后 WIP 恢复、Windows 映射文件无法移动时拒绝。真实错误根 CLI 控制已观测 metadata 指向组内 UI 与 AUTO_UI_ROOT 主 UI 不一致，拒绝而非测错 owner。
- 847 六红定向重放首轮冷态 worker 构建触发原 120s deadline，停止该轮，未放宽预算；补齐与 847 相同的 mn-worker release 前置构建后，第二轮 stdout-cap/liveness 已 pass 29.898s，其余仍进行中。未运行任何整档测试。现有编译警告来自未修改的语言/UI 基线，未纳入本计划生产代码修复。
- T01/T02/T03 尚未全部关闭；T04 独立上下文复审的子代理授权已异步请求，未收到前不启动代理。merge 全档仍 pending。

### work 最终交接 — 2026-10-26

stage: work | plan_id: PLAN-850 | plan_revision: 1
completion_kind: delivered | outcome: pass | worktree_path: D:/autostack/.wt/lang-850/auto-lang | branch: plan-850-dev | base_commit: 1c453e8166e574d9e2eae023eed1758b3f02f3e9 | code_commit: 1db29917b7f7b26925cc1432fb3a74a0f4c01129 | dependency_revisions: auto-ui aee499e87093ff727f68ac8467c506e9ecf6ecdc / auto-down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4 | task_ids: T01,T02,T03 | evidence: attachments/850/work-r1/readiness.md | blockers: none work-owned | next_responsible_role: independent final review (T04)

- AC01/02 work-owned：通过。tf 两 owner 3718/3091 IDs；tt 4469 IDs/3553 必选，tb 3773/3109；taa 3835/3214、ta 4537/3668、t3 4537/3669（均另含 UI 3091/3073）。所有原 required IDs 保持，tt/tb 的 765/69 增量是相对语言 tf 的 feature 面，不冒称新增产品用例。13/14 之类不作通过：最终 controls.log.txt 实际 14/14 全通过。
- 847 原六红：同代码重放第二轮 5 pass/1 fail（签名 P95 1165.1593ms），其他构建结束后只重放该签名门，原 30 样本/1000ms 不变，P95 786.2717ms、1/1 pass。历史中已有 D-846-INCR-PERF/F-846-M1 同类记录；846 的专属性能政策不沿用到 850。此处仅关闭原错误归因，不能把分次定向通过当完整批量绿。
- MN worker 族经 test-cases 仅入 tt；真实 show-config 已验证 tt 原串行组命中。full/t3 新增组无匹配后已撤回，最终两配置与基面字节一致。四完整别名的各 feature/profile、原输入和各时限保持；无 crates/ 或产品改动。三次列表配置解析不是整档测试运行。
- 所有实现提交、Git clean、Rust launcher fmt、diff 检查通过；无新增代码 warning。基线语言/UI warning 已明确来自未改生产源，未借此扩大到生产修复。冻结选择清单可复用到最终提交：后代仅增强双向 owner/输入身份、主检出 WIP 守卫及控制用例，不改命令/feature/配置/ID输入；对应新机制已定向验证。
- T04：必须独立上下文复审；当前实现会话自查不授 final pass。子代理明确授权问题已呈现，仍待答复。T05：独立 pass 后才可执行主检出单实例完整 batch、绿锚点、沉淀归档和 guard 清理。Category A（无 crates Rust 改动）不额外跑裸 cargo t 或单独 docs_gen；显式批准的原完整 batch 仍保留其原集成目标。当前未合入、未修改 live ledger/绿锚点、未归档或移除工作树。


stage: review | review_scope: final | reviewer: /root/review_850 independent read-only context | PLAN-850:r1 | outcome: needs_fix | reviewed_commit: 1db29917b7f7b26925cc1432fb3a74a0f4c01129 | finding: F850-R1-01 P1 AC01/02 T02/03，各 language 子门 required 与 tf expected 相交，未守卫 tt/tb/taa/ta/t3 的513/69/10/464/465个必选 ID。实际选择完整；删除或ignore真实a2r_rustc_real_compile_gate/book_ch01_01仍绿。其余根/生命周期/WIP/选择/Spec证据保留。最小修复复审仅各子门基线与缺失/ignore/profile反控及合法scope，不重复性能/整档。next: work。同树所有权已交回实施。



stage: work | PLAN-850:r1 | outcome: pass | repair: F850-R1-01 | code_commit: f112d2737e6fae3d4a7bed8fecbdeb85682f9890 | 同树 plan-850-dev Git clean。每个tier/leg独立完整IDs+required，不再与tf集合相交。原始JSON正控5项、合法scope5项及删除/ignore/profile15反例全部通过，且同15反例旧参数仍错误接受，证明区分性。实际raw输入SHA见 attachments/850/repair-r1/replay.log.txt；永久main流水线反控覆盖全部5tier×3突变，15 unittest方法通过。原未改选择、原profile/feature及性能证据复用；未重跑整档或性能族。Spec仅明确各feature完整基线，不改变合同。next: 同独立review上下文定向复审；完整门仍merge pending。



stage: review | review_scope: final repair | reviewer: /root/review_850 independent read-only context | PLAN-850:r1 | outcome: pass | reviewed_commit: f112d2737e6fae3d4a7bed8fecbdeb85682f9890 | AC01/02/03 pass; F850-R1-01 CLOSED。真实七组完整集合与原JSON恒等、5正控+5scope+15反例通过，原错实现同15例假绿永久可重放。SD auto-ui project SHA ff19acaf953b6fe5b5f08fc436dcc74aa968114e2e8ca04ab2e2a6be79974bb0。Git clean / UI aee499e / down c3464d0。独立pass不覆盖后续rebase受影响差异。next: merge；完整batch、绿锚点、知识/ledger、归档guard pending。用户2026-10-26已明确授权独立子代理及整个收尾工作流。



stage: merge | PLAN-850:r1 | prepared/landed: independent post-rebase pass @83c875d1d5011cb86f7e4493b5db40477a10cb94, base100172980；6/7 range-diff等价，首提交仅832/850共享docstring冲突已保留双方说明，独立复核通过。fresh四实际list集合与独立各基线恒等。guard clean后master ff-only 100172980→83c875d1d。Category A无crates Rust新改动，不额外cargo t；正式批准完整batch仍pending，未交付完成。最终运行期832 schema输入auto-man实体 D:/autostack/auto-man @1b7a9ce0de9055f7953129d78a25777662508258，clean；UI aee499e/down c3464d0保持。其余主检出docs WIP与既有oracle Cargo.lock保留。



merge batch-r1: language 3218/3218 pass 62.032s；UI 3072pass/1fail/18skip 50.875s，首红停止，tt/tb未运行，绿锚点不推进。M850-UI-ROOT: Plan831迁仓测试helper从UI crate回组根误用5个parent（应3个），实际指D:/auto-lang/examples/ui，launch011-calculator app not found。L0 fix-forward复用独立同组UI树，分支codex/fix-850-ui-fixture-root；仅删2级parent与订正测试注释，不改生产/3真实App断言/选择/阈值。当前原失败target已1/1 pass0.550s；待提交及独立review后UI own master落地、更新依赖身份，再执行冻结完整batch。


M850-UI-ROOT L0独立review /root/review_850 pass，冻结UI b4fe0000e56cddaf7d3cf70e348431f2258aacde（baseaee499e），两布局新helper均指同组语言仓、3个App均存在，原5parent在main错指D:/auto-lang，在组内误读主语言仓。仅测试路径修复，无生产/断言/ID/时限改变。guard clean后UI master ff-only aee499e→b4fe000；更新850依赖身份为b4fe000。Lang83c875d1及guard/SD未变，独立允许复用旧证据。正式batch-r2待执行，r1语言通过不冒称r1整体通过。


batch-r2正式失败收据：language3218/3218、UI3073/3073通过；tt3552pass/1fail/916skip，399.726s，signature_rebuild_gate P95 1060.6103ms >=1000ms（30原样本）；tb首红停止未跑。绿锚点保持840。独立review只读诊断：前16样本中位752.225ms，后14样本943.127ms，后半段持续抬升；功能/重编闭包/缓存计数断言已过；850未改原10秒稳定等待、warmup、样本/预算/scratch/worker/target，wrapper子门无重叠。外部841 rustc及821 resources_tier存在，但无对齐窗口CPU/IO证据，不能直接断言因果。等待外部重任务结束后带负载记录做同30样本定向验证；原阈值与observe政策不动，完整批量仍pending。

诊断复核：821原资源/turnaround重任务结束后，原850树原signature单项1/1 pass57.572s，count30、P95 787.3877ms、1000ms原阈值。七报告逐hash恢复、树83c875d1 clean。独立只读确认旧代码/guard/SD审查可复用；负载日志仍见短暂821/841构建（84107:38:07–11与测试重叠），因此仅称“所记录负载条件下通过”，不称完全空闲，也不据此断言r2竞争因果。失败r2保留，r3在同冻结基线上正式全批量，带时间戳CPU进程监测；全绿/归档仍pending。

batch-r3失败：language3218/3218 78.224s、UI3073/3073 51.090s；tt3552pass/1fail/916skip 405.389s，原signature30样本P95 771.422ms、57.454s pass。唯一新红M850-REPORT-738：既有738矩阵JSON输出被映射占用，fs::write OS1224；tb未跑、绿锚点未推进。保护器遗漏了docs/plans/reports/738-stdlib-matrix.{json,md}，同源审计还识别827 inventory纯producer；p728bench显式ignore不入正式面。原850树 bounded repair @5acd60ab8仅追加这3个输出路径及SD具体枚举，无生产/测试ID/输入/配置/阈值改动。永久部分prepare失败/锁矩阵/原WIP及生成结果四路径反控，16方法pass；原738+827定向2/2pass0.264s；实际主检出可逆保护预检逐hash恢复通过。现executing，待独立受影响复审；旧代码/ID/性能证据复用边界不变。完整batch-r4 merge-owned pending。
M850-REPORT-738独立pass @5acd60ab8；main新增c3ad09602仅821计划及SD草案，原树rebase至7750f8d3399fe8aa620b7eecb6c7390ca810df05，range-diff1/1全等，独立复核更新pass绑定7750。guard clean后master ff-only c3ad09602→7750f8d33。16控制、原738/827两测试及10份报告hash证据复用；完整batch-r4 pending。

## 11. PLAN-850:r1 合入沉淀收据（实际归档）

- prepared：独立final pass绑定7750f8d3399fe8aa620b7eecb6c7390ca810df05；UI b4fe0000e56cddaf7d3cf70e348431f2258aacde；Down c3464d0a0d36e4f8a7988b0c0c26448fc52361b4；Man1b7a9ce0de9055f7953129d78a25777662508258及cmd_vue输入hash前后一致。两次rebase映射/range-diff及受影响独立pass均存证。
- landed：实现7750f8d33已ff-only落地主分支；后代仅规范索引/台账沉淀6dd05a0a78385349ece99c3c11dffc13fa272810，master与plan-850-dev同tip。UI L0 own master已ff b4fe0000。生产、选择ID、原profile/feature/输入/时限/阈值不变。
- merge-owned gates：Category A（lang无crates Rust实现）不额外裸t/docs_gen；正式主检出单实例batch-r4四子门全部实际pass：language3218/3218（75.044s，skip500）、UI3073/3073（51.230s，skip18）、tt3553/3553（382.705s，skip916）、tb3109/3109（54.289s，skip664）。每子门10份报告前后hash与当前字节一致；完整receipt=pass、所有list/run exit0、输入identity不变。原signature30样本P95 796.4324ms<1000ms。无AAVM触发路径，不额外taa/t3。原r1/r2/r3失败、定向诊断与负载记录完整保留，无豁免、缩面或阈值放宽。
- ledger_refreshed：docs/specs/auto-lang及auto-ui project/plans沉淀；.autoos/specs.json guarded offline projection（无可达store writer，检查端口8080及进程），v89→90，SHA256 69d24aa29ad257adb4b6ca4f2f8816b3614e902aa71056d35496cb44ad069acf。受影响designs=P840-P2-AUTO-LANG/P831-AUTO-UI-REPO，tests=P850-FULL-GUARDS，reviews=P850-REVIEW，reports=P850-MERGE；file/related/tags/来源SHA完整，其他1212条原始片段逐字相同。byte-exact v89备份、投影脚本、schema/readback及独立核验有据。
- 绿锚点：仅在四门实际全过后推进last_covered_plan_id850/covered_commit7750f8d33；旧840锚点字节备份及845/847/850失败历史保留。modulo5本轮义务已清偿，随后仅文档/台账后代，无重复批量触发。
- archived：当前文件已实际移至docs/plans/archive/850-restore-cross-workspace-full-regression.md，status archived、completion_kind delivered；两个模块索引指向此处。归档/主证据已提交120b419993429127122db2339be977c3eb876c2a，实际链接与Git追踪核验通过。
- cleaned：verified。三个工作树、两个临时分支和组目录均已按guard流程移除，T05完成、总计5/5；各项实际证据见最终清理与核验。
- 生产产物：变更仅测试编排、standalone Cargo入口、测试fixture路径与规范；无生产代码、后端/daemon/web bundle变更，无部署或服务重启需求。实际cargo tf入口已编译执行launcher，不宣称无关运行中服务更新。
- 其他会话现有docs/design/roadmap/报告与fixture Cargo.lock WIP均保留；“clean”仅指本计划专用树及交付范围，不冒称主检出全局clean。

[独立复审](../attachments/850/merge/independent-review.md) · [完整批量收据](../attachments/850/batch-r4/receipt.json.txt) · [回归统计准入](../attachments/850/merge/batch-admission.json.txt) · [台账投影](../attachments/850/merge/ledger-projection.json.txt) · [语言规范](../../specs/auto-lang/project.md) · [UI规范](../../specs/auto-ui/project.md)
原始range-diff/测试日志保留原空白及字节，不为whitespace检查改写证据；所有非raw .txt的编排/规范/计划变更diff检查通过。

### 最终清理与完成核验

归档/主证据提交120b419993429127122db2339be977c3eb876c2a已验证追踪；实际归档6个证据/规范链接均存在。lang、UI、down三个专用工作树清理前均重新核绝对路径、owner、clean状态、已落地祖先关系（down detached身份）；逐树wt-guard均clean后移除。plan-850-dev和codex/fix-850-ui-fixture-root均已删除。组内73份scratch逐hash与已追踪证据一致，组guard clean；按明确清单逐文件移除，仅删除确认空目录，D:/autostack/.wt/lang-850实际不存在。

stage: merge | PLAN-850:r1 | outcome: pass | completion_kind: delivered | tasks: T01,T02,T03,T04,T05 = 5/5 | AC01-04: pass | tested_commit: 7750f8d3399fe8aa620b7eecb6c7390ca810df05 | delivery_commit: 6dd05a0a78385349ece99c3c11dffc13fa272810 | archived_commit: 120b419993429127122db2339be977c3eb876c2a | ledger: v90 | cleaned: verified | remaining_owned_work: none

prepared/landed/ledger_refreshed/archived/cleaned五检查点全部实证闭合。B847-FULL-OWNER-01/B847-TT-STATE-02已关闭，F850-R1-01、M850-UI-ROOT、M850-REPORT-738均修复独立复审及完整门闭合。本轮850到期批量已完成，之后只有本计划文档/台账收尾，无需重复整档。后续更高计划的批量义务按其自身到期判断，不提前代跑。
[最终实际状态核验](../attachments/850/merge/final-closeout.json.txt)
