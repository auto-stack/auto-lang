---
plan_id: PLAN-777
status: execution_done
feature_name: v0.6 模块边界基线与增量检查
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-15
plan_revision: 1
current_step: 4
total_steps: 6
supersedes_spec_components: [docs/specs/design/repository-layout.md]
new_spec_components: []
touched_goals: []
affects: [docs/specs/design/repository-layout.md]
---

# [PLAN-777] v0.6 模块边界基线与增量检查

## 0. 变更摘要

建立可复现的规模、依赖与资产基线，以及新增坏依赖检查。仅脚本/配置/文档，不改Rust、Cargo构建或语言行为。

## 1. 目标

调查可重复，现有焊点有owner/退出阶段，新增跨界依赖可确定性判红。非目标：搬代码/建仓/换AST/VM/性能优化或统计全仓精确重复率。

## 2. 架构方案

区分normal/build/dev/optional声明边、源码文本候选和人工确认；观察模式覆盖全仓，strict仅判冻结的新增边。存量窄baseline不豁免整个vm/lib目录。依据Design37，目标态不得当成当前已经达成的事实。

## 3. 技术栈

Python标准库、Git tracked files、Cargo metadata（不编译）。

默认以 `cargo metadata --no-deps --locked --format-version 1` 读取；如锁状态不满足，记录依赖调查失败/缺口，不自动更新。实际 feature 链抽查同样用 `cargo tree --locked`，且不把该命令当编译验收。

## 4. 需求分析与背景调查

用户在2026-10-13授权调查与报告/规划；本轮不执行重构、不创建新仓、不发布/合入。实施需确认本Plan revision 1。其他在途Plan/WIP保留。

前置报告 reports/v06-modularization-research.md、逐文件hash reports/v06-modularization-baseline.jsonl、Design37、Specs overview/README与repository-layout。调研HEAD 5e21561d2c30，src物理行704000；work重采HEAD，不固化该行数为永久阈值。

2026-10-15状态就绪核查：[启动核查收据](reports/777-readiness-20261015.md)。基线master@17bc8431457848b040bffa5c261e13b057771559；742/744/773/774/771/776已archived且交付提交为master祖先。770/781在独立树继续，782仍待前置。777可并行开始，不等待全生态清空；T-01启动时再绑定实际最新提交，未合入代码单列为在途范围。

## 5. 详细设计

拟新 `scripts/modularity-audit.py`、`.autoos/module-boundaries.toml`、`test/fixtures/modularity/audit/`。输出schema/method/revision/file hashes、声明边/文本候选/unknown。稳定payload不含timestamp/绝对检出路径；baseline绑定base revision，过期或宽豁免不能默默通行。输入只读，生成报告另存。

ownership表覆盖报告的六类UI焊点、服务/runtime非UI归属、资产、外部consumer。测试/生成无法精确分类时写unknown，不报虚假SLOC。新增边与存量边分开，允许合法UI→SDK与dev-only，不把文本match当语义解析。

### 规范增量

| delta_id | 操作 | docs/specs target | before/after | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/design/repository-layout.md | 增加真实审计入口/口径/baseline规则，不把目标仓写成已拆 | 可复现治理 | AC-01/02/03 |

## 6. 测试设计

实现者未指派；work自检后交独立上下文review，在同一唯一worktree逐项验收，不修代码；merge仅负责已验收的知识/归档/清理。本草案未派遣agent/chat。

Category A：禁止cargo t/docs_gen。T01实现后 `python scripts/modularity-audit.py --help`；T03实现拟新 `--self-test`；T04运行真实扫描与绑定revision的差分。metadata成功不代表编译成功。

| V | AC/不变量 | 输入/入口 | 可观察oracle | 边界 |
|---|---|---|---|---|
| V-01 | AC-01/稳定只读 | 同HEAD扫描两次，前后hash/status | payload相同且输入不变 | tracked物理行 |
| V-02 | AC-02/新增边 | fixture实际新增core→UI正常边 | 非零+具体path/edge；合法反向/dev-only不误红 | manifest声明边 |
| V-03 | AC-03/候选诚实 | 注释/测试/alias/宏/string对照 | 分类与盲区明确，不声称完整Rust解析 | 文本scanner |
| V-04 | AC-02/baseline | 窄豁免/撤销边/过期revision/全目录豁免 | 存量不能盖新边；过期/宽豁免失败 | 先证明fixture被改 |

## 7. 验收标准

| ID | owner_stage | 可观察结果与验证 |
|---|---|---|
| AC-01 | work | 可复现真实报告schema/method/hash/revision；V01 |
| AC-02 | work | 窄baseline+退出owner；V02/04实际判红 |
| AC-03 | work | ownership覆盖全部调研焊点/资产/consumer；V03未知范围明确 |
| AC-04 | review | 独立逐AC审计/遗漏检查，绑定commit与revision |
| AC-05 | merge | 真实SD01沉淀、归档/账本更新、wt-guard clean后清理 |

## 8. 执行步骤

| 状态 | Task | owner_stage | 依赖 | 路径/操作与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 确认r1 | 新audit脚本，重采HEAD/文件，help+V01。[R1–R3历史保留]；**R4（5e1794bcc）**：根外已存在输出目标按 inode 身份对照全部受管输入+实读锁（F777-R3-01）；r4 冻结态双扫描 payload 字节一致（sha 732ae3b6709a…）、git status 前后不变 | 01 |
| [x] | T-02 | work | T01 | 新boundary配置/ownership，逐项核对真实路径和存量退出阶段。[R1/R2历史保留]；**R3**：域映射表按包名补根外实现包 auto-ui=ui（F777-R2-01），未知外置包保持真实 unclassified；跨根规则正向拦截由生产域表+规则控制（v10/S01）| 02/03 |
| [x] | T-03 | work | T02 | 新audit fixtures/self-test，V02–04实际正反控。[R1–R3历史保留]；**R4**：新增 v12（硬链接→受管源码/实读锁拒绝且字节不变、根外新路径另存正控），`--self-test` **15/15** | 02/03 |
| [x] | T-04 | work | T03 | 真实扫描/diff/check输入不变，受管evidence；不跑Rust tests。[R1–R3历史保留]；**R4（代码 5e1794bcc，基线 677055420 不变——配置/边集零变更）**：观察双扫一致（sha 732ae3b6709a…，2052/945894）；cargo 对照 36=36 零差异；**四套复审控制全过：r1 replay 7 控 + r2 replay + r2 supplemental 5 控 + r3 input-matrix 4/4（M01-hardlink-source exit 3 输入不变）**；strict pass/0违例/2 grandfathered/coverage ok；证据 attachments/777/work/（t04r4-*、t03r4-selftest.txt） | 01/02/03 |
| [ ] | T-05 | review | T04 | 独立review，禁止baseline偷换目标 | 04 |
| [ ] | T-06 | merge | review pass | Spec/归档/合流/受保护清理；batch到期按现行规则 | 05 |

拟tree `D:/autostack/.wt/lang-777/auto-lang`，branch `plan-777-dev`；脚本实现也必须在worktree。Plan簿记在master；依赖用new-wt-group，禁止链接。

## 9. 复审记录

2026-10-13：stage=new；r1；outcome=pass（草案preflight）；next=确认实施后work。0/6 task，脚本/fixture/测试均未创建执行；不是独立实现复审pass。

2026-10-15：stage=readiness；r1保持；outcome=ready（技术前置）；证据见[启动核查](reports/777-readiness-20261015.md)。只核查状态/Git历史与范围，未建777树、未改实施文件、未运行Cargo测试；status仍drafting/current_step仍0。本轮用户请求的是是否可启动核查，不把本记录作为已经执行或完成的证据。Category A仍仅跑脚本/差分目标验证，禁止cargo t/docs_gen；无需因最新整档后置裁定变更本Plan语义合同。

2026-10-15：stage=work | PLAN-777 | r1 | outcome=pass（work-owned 全部完成，交独立review） | worktree D:/autostack/.wt/lang-777/auto-lang | branch plan-777-dev | base 17bc8431457848b040bffa5c261e13b057771559 | code 0b5e79950381（脚本+配置+fixtures）+ 97df2f997（基线冻结落库） | deps: auto-down 组内兄弟 detached@c39ac7f（只读，未改） | tasks T-01..T-04=4/6 | evidence: [T-04 工作证据](attachments/777/work/t04-summary.md)（self-test 7/7；双扫描 sha 54e93962964a… 字节一致；strict pass/0违例/2 grandfathered；cargo 对照 36=36 零差异） | blockers: 无 | next=review（T-05：独立逐 AC 审计，重点=baseline 是否偷换目标、ownership 覆盖 vs 调研焊点全集、strict 判红路径真实性）。实现要点偏离记录：声明边主源=manifest tomllib 直解（V-02 要求在锁未重生成的工作树上点名新边），`cargo metadata --no-deps --locked` 降为默认一致性对照（失败记 gap 不自动更新）；strict 判当前工作树边集（未提交变更也在判定面内），脏树只在 refresh-baseline 拒绝；基线文件 `.autoos/module-boundaries-baseline.json` 因根 `.gitignore:161` 的 `*.json` 全局规则一次性 `git add -f` 入库（此后持续受管）。merge-owned 门禁：无（Category A 不触发 cargo 整档；无需 batch 前置）。

2026-10-15 R1：stage=review | review_scope=final | reviewer_identity=本评审会话（参与规划/就绪核查，未参与777实现；由提交和独立反例重建，未另建agent/树） | plan_id=PLAN-777 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-777/auto-lang | branch=plan-777-dev | reviewed_commit=97df2f99785be53fd8a1a6ef70124725d304d576 | base_commit=17bc8431457848b040bffa5c261e13b057771559 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；Python3.14.2/Cargo1.98.0 | spec_inputs=repository-layout SHA256 3800856129eb9b5bd742fef9ea2aa0c94f0db780ab2a8ba01fc7ae8965dde49b，冻结SD01见附件 | acceptance_results=AC01 partial(F03)、AC02 fail(F01/02/04/06)、AC03 partial(F05/06)，AC04审计完成但未pass，AC05 pending | findings=F777-R1-01..06 blocking（target表复用、workspace path基点、输出覆盖输入、optional激活误豁免、责任遗漏、跨根optional规则误红） | evidence=[R1完整报告](attachments/777/review-r1/REVIEW.md)、[可重放反例](attachments/777/review-r1/replay.py)、[原始结果](attachments/777/review-r1/controls.jsonl)、[输入指纹](attachments/777/review-r1/fingerprints.jsonl) | verification=自测7/7与真实双扫稳定/36=36通过；独立合同控制1阳性通过+6失败；源/锁hash未变；无Cargo编译/测试 | pending_closeout_gates=Category A无Rust整档门，T06 Spec/投影/合流/归档/到期判定/guard-clean仍pending | task_state=T01..04因原合同缺口重开，executing 0/6（保留历史work证据，不变r1） | next=/auto-plan:work同树修复全部findings并补永久控制，提交后独立复审；本次不修实现、不合入、不发布Spec或账本。

2026-10-15 R2：stage=review | review_scope=final | reviewer_identity=本评审会话（规划/R1参与，未参与实现；从提交及独立组合反例重建） | plan_id=PLAN-777 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-777/auto-lang | branch=plan-777-dev | reviewed_commit=aaf70201110ed8afcfbbc6e1ccd99c4e82019982 | base_commit=17bc8431457848b040bffa5c261e13b057771559 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；Python3.14.2/Cargo1.98.0 | spec_inputs=repository-layout SHA256 3800856129eb9b5bd742fef9ea2aa0c94f0db780ab2a8ba01fc7ae8965dde49b，冻结SD01见附件 | acceptance_results=AC01 partial(R2-02)、AC02 fail(R2-01)、AC03 pass、AC04审计完成未pass、AC05 pending | findings=F777-R2-01(P1，生产auto-ui外部域遗漏假绿)/F777-R2-02(P1，已读取外部manifest可被报告覆盖) | closure=R1-01/02/04/05关闭；R1-03配置旧例关闭但完整只读partial；R1-06旧误红关闭但新生产规则遗漏见R2-01 | evidence=[R2完整报告](attachments/777/review-r2/REVIEW.md)、[补充矩阵](attachments/777/review-r2/supplemental.py)、[原始反例](attachments/777/review-r2/supplemental.jsonl)、[旧例7控](attachments/777/review-r2/controls.jsonl)、[全输入指纹](attachments/777/review-r2/fingerprints.jsonl) | verification=旧例7/7、自测12/12、真实双扫一致/36=36/strict正控通过；补充矩阵3pass/2fail；diff-check通过，受管hash/锁/HEAD/status未变，无Rust编译测试 | pending_closeout_gates=Category A无Rust整档，T06仍pending | task_state=T01..04因相关实现/配置及依赖验收证据重开，executing 0/6，保留历史绿项且r1合同不变 | next=同树work先提交外部域和所有审计输入的诊断/完整路径覆盖矩阵（两次final失败），修复两项并补生产配置永久控制；提交后独立复审。本次不修实现、不合入、不发布Spec/ledger、不清理树。

2026-10-15 R3：stage=review | review_scope=final | reviewer_identity=本评审会话（规划及前两轮参与，未参与实施，直接提交/反例重建） | plan_id=PLAN-777 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-777/auto-lang | branch=plan-777-dev | reviewed_commit=677055420203db3bc3a868a2dca697f8167361dd | base_commit=17bc8431457848b040bffa5c261e13b057771559 | dependency_revisions=auto-down@c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8；Python3.14.2/Cargo1.98.0 | spec_inputs=repository-layout SHA256 3800856129eb9b5bd742fef9ea2aa0c94f0db780ab2a8ba01fc7ae8965dde49b；冻结SD01见附件 | acceptance_results=AC01 partial(R3-01)、AC02/03 pass、AC04审计完成未pass、AC05 pending | findings=F777-R3-01(P2 blocking，根外硬链接报告目标可覆盖受管Rust输入，exit0/hash改变) | closure=R2两项具体反例关闭；R1依赖/ownership问题保持关闭，完整输入只读仍partial | evidence=[R3完整报告](attachments/777/review-r3/REVIEW.md)、[输入别名矩阵](attachments/777/review-r3/input-matrix.py)、[原始输入/结果](attachments/777/review-r3/input-matrix.jsonl)、[指纹](attachments/777/review-r3/fingerprints.jsonl) | verification=自测14/14、旧7控/R2补充5控全pass、输入矩阵3pass/1fail；真实双扫一致/36=36/strict正控pass；全受管输入和锁hash/HEAD/status未变，diff-check通过，无Rust编译测试 | pending_closeout_gates=Category A无Rust整档；T06仍pending | task_state=保留T02，重开T01/03/04及依赖证据，executing 1/6，r1不变 | next=同树work补齐所有受管/实读输入的身份保护和永久矩阵，提交后复审；这是原只读不变量的剩余路径，可局部修复，不自动needs_replan。两连败诊断已检查，但身份比较仍仅接部分输入类别。本轮不修实施/merge/发布Spec/ledger/清理树。

## 10. 待澄清事项

仓名/发布渠道不影响baseline。任何strict扫描扩面要先有可区分错误实现的oracle，再修订contract；不把此Plan扩大成Rust编译器。

2026-10-15：stage=work | PLAN-777 | r1（合同未变） | outcome=pass（R1 needs_fix 六项已全部修复，重新交独立review） | worktree D:/autostack/.wt/lang-777/auto-lang | branch plan-777-dev | base 17bc84314578（不变） | code 3b50ea63e（F01-F06 修复+永久自测 v05-v09）+ aaf702011（基线重冻，36 边/config sha 绑定） | deps: auto-down 组内兄弟 detached@c39ac7f（只读，未改） | tasks T-01..T-04=4/6（R2 证据回填） | evidence: [R2 工作证据](attachments/777/work/t04-summary.md) + review-r1/replay.py 在新提交重跑 **六控全过（exit 0）**；self-test 12/12；r2 双扫一致 sha 6fa0e43e549a…；strict pass/0/2 grandfathered/coverage ok | blockers: 无 | next=review（T-05 r2：绑定 3b50ea63e+aaf702011 重新逐 AC 审计）。修复对账：F777-R1-01 每表三元组携带 target 依赖表（v05：Windows UI 判红/Unix SDK 不误红）；F02 继承 path 按 workspace 根解析（v06：不再 unresolved）；F03 输出/刷新写入前输入冲突守卫（v07：配置/基线字节不变，合法重冻结不受影响）；F04 基线豁免绑定完整声明身份 from/to/kind/target/optional/scope/dep_key（v08：optional 激活判红不冒充存量）；F05 ownership 27 项 7 必需组 + 缺组 strict 判红（v09；parser/ast/dialect/builder/CLI 补登记，C05 六路径全注册）；F06 跨根规则改 Design37§2 域限定（语言域禁外部 UI/Man 实现），scope-only 无方向规则按惰性处理并记录告警（C06 合法 UI→外部 SDK exit 0）。勘误同步：t04-summary 计数改 2052/945894（与所附 JSON 一致）；脚本头/METHOD 脏树措辞改为“strict 判当前工作树边集、refresh 拒脏树”。未新增 baseline 豁免让反例变绿；复审 fixtures/replay 未修改。

2026-10-15：stage=work | PLAN-777 | r1（合同未变） | outcome=pass（R2 needs_fix 两项已修复，重新交独立review） | worktree D:/autostack/.wt/lang-777/auto-lang | branch plan-777-dev | base 17bc84314578（不变） | code d6e947912（R2 修复+永久自测 v10/v11）+ 677055420（基线重冻，36 边不变/config sha 重绑） | deps: auto-down 组内兄弟 detached@c39ac7f（只读，未改） | tasks T-01..T-04=4/6（R3 证据回填） | evidence: [R3 工作证据](attachments/777/work/t04-summary.md)；self-test **14/14**；**r1 replay 7 控 + r2 replay + r2 supplemental 全过（exit 0）**；r3 双扫一致 sha 15af9eca82d8…；strict pass/0/2 grandfathered/coverage ok | blockers: 无 | next=review（T-05 r3：绑定 d6e947912+677055420）。修复对账：F777-R2-01 生产 [domains.members] 按包名补 auto-ui=ui（表语义=包名→域，含已知根外实现包；未登记外置包保持真实 unclassified，不假称所有外部依赖违规）——规则正向拦截以生产域表+规则实测（v10 五例 + S01 三例）；F777-R2-02 extract_declared_edges 返回实读 manifest 清单（含根外 path 目标），guard_output_target 输入矩阵闭合（配置/基线/实读 manifest/受管文件/根内既有文件），same_file 以 normcase+samefile 归一路径别名（v11：已读根外 manifest 与大小写别名拒绝且字节不变，根外另存正控可用）。闭合表校正：r1 C06 旧例仅证 scope-only 惰性语义，生产跨根规则的正向拦截证明由本轮 S01/v10 补齐；scope-only 惰性是 r1 建立并保持的规则合同（禁向判定基于依赖方向）。未新增豁免；复审 artifacts 未修改。两次 final review 失败后的诊断（技能两连败检查点）：R1 六项与 R2 两项同源于“具体反例闭合而完整不变量未闭合”——本轮按输入矩阵/域映射两条完整路径补面而非只补旧例；下轮如再失，按 needs_replan 升界。

2026-10-15：stage=work | PLAN-777 | r1（合同未变） | outcome=pass（R3 needs_fix 一项已修复，重新交独立review） | worktree D:/autostack/.wt/lang-777/auto-lang | branch plan-777-dev | base 17bc84314578（不变） | code 5e1794bcc（R3 修复+永久自测 v12；**基线不重冻**——配置/边集零变更，677055420 仍有效：staleness 锚定面自 base 起无变更、config sha 一致） | deps: auto-down 组内兄弟 detached@c39ac7f（只读，未改） | tasks T-01..T-04=4/6（R4 证据回填；T-02 维持复审保留的 [x]） | evidence: [R4 工作证据](attachments/777/work/t04-summary.md)；self-test **15/15**；**四套复审控制全过**：r1 replay 7 控 / r2 replay / r2 supplemental 5 控 / **r3 input-matrix 4/4（M01-hardlink-source exit 3 且输入字节不变）**；r4 双扫一致 sha 732ae3b6709a…；strict pass/0/2 grandfathered/coverage ok | blockers: 无 | next=review（T-05 r4：绑定 5e1794bcc+677055420）。修复对账：F777-R3-01 guard_output_target 对根外**已存在**目标先以文件身份（st_dev/st_ino，硬链接同 inode）对照全部受管文件与实读锁（根 Cargo.lock，cargo 对照尝试运行时读取）再放行——身份比较接入的输入类别从“配置/manifest”补齐为“全部受管输入+锁”；永久控制 v12 覆盖硬链接→源码/锁负控与根外新路径正控。摘要 V01 行已按 r3 复审勘误同步 t04r4/sha 732ae3b6…。三连败诊断延续（R1/R2 同源+R3 输入身份未接入全类别）：本轮把身份保护补成完整输入/目标矩阵后，r3 矩阵 4/4 在新提交通过；若 r4 复审再失，按 needs_replan 升界。
