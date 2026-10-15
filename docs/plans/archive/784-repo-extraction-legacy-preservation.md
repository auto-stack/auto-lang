---
plan_id: PLAN-784
status: archived
feature_name: 旧仓保全与拆分迁移准备
author: [Codex]
created_at: 2026-10-15
updated_at: 2026-10-15
plan_revision: 1
current_step: 6
total_steps: 7
supersedes_spec_components: []
new_spec_components: [docs/specs/repository-boundaries/project.md]
touched_goals: []
affects: [repository-boundaries]
depends_on: [PLAN-779]
---

# [PLAN-784] 旧仓保全与拆分迁移准备

## 0. 变更摘要

保留既有 auto-ui/auto-man 仓身份、旧历史及未提交改动，建立 legacy 恢复锚与隔离迁移分支；刷新 777–780 合流后的迁移入口清单。主分支维持旧版本，既不清空也不切换默认分支；本计划不搬生产代码。

## 1. 目标

交付两个旧仓的可验证恢复收据、隔离准备分支，以及后续迁移使用的仓/模块/资源/消费者清单。
非目标：独立 UI/Man 已交付、删除旧源码、重写历史、force-push、发布、去重、改产品协议、建立 common 大包。

## 2. 架构方案

沿用 Design 37 §8.0。旧主分支完整历史继续存在；本地 legacy 分支/标签固定当前 tip，迁移开发基于原历史，在新提交中替换树。auto-lang 来源路径/commit 映射待真正迁出时建立，不能现在制造空 orphan 主分支。

## 3. 技术栈

Git refs、bundle/原始字节备份、SHA256 清单；既有 777 边界工具与 Cargo metadata 只读盘点。恢复演练在 disposable clone；不编译旧项目。

## 4. 需求分析与背景调查

依据 Design 37、docs/specs/overview.md、project-model/project.md 及 auto-man/project.md；内容版本与代码快照见 [取材指纹](attachments/modularization-planning-20261015/inputs.md)，auto-lang@6154f4734e23e8fa8edcf8cd9706d9a06185c61f。
实查：auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e，master，Cargo.lock 未提交；auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e，master，干净。只检查本地状态，未 fetch，不声称远端同步。执行时重新检查实际 tip/WIP，不能沿用本快照覆盖新工作。
用户本轮授权建立计划；尚未授权本轮实施或更改旧仓 refs。后续确认本计划实施后，可在三仓内做本合同约定的本地保全/分支准备；push、默认分支切换和生产树替换不在此授权范围。
779 已交付；781/783 无需先结束才能做保全。UI 实际路径迁移另以 781 最终 review+merge 为硬门。

## 5. 详细设计

- T01登记三仓绝对路径、origin、默认/当前分支、HEAD、refs、WIP 和所有者；同时读取当时各仓适用 AGENTS。不得把用户主检出切到其他分支。
- 先保全再建立工作树。auto-ui Cargo.lock 保存原始字节、二进制差异及原路径/hash；其他新增 WIP 按同样办法处理。未提交/未跟踪内容不能仅靠 git bundle 保全；ignored 内容先列清单，需保留项单独备份。收据只写元信息，payload 放独立恢复位置，不把敏感文件内容提交进报告。
- 拟 refs：分支 `legacy/pre-v06-extraction` 和标签 `legacy-pre-v06-<tip短SHA>`；若已存在且同 tip 则复用，异 tip 不强制移动。迁移准备分支拟 `codex/v06-extraction`，从已保全旧 tip 建立，当前计划不替换其代码。实际名称/tip 写收据。
- 在 `D:/autostack/.wt/lang-784/{auto-lang,auto-ui,auto-man}` 各建唯一树（auto-lang 为 plan-784-dev；目标仓准备分支与旧主检出分离）。无 junction/symlink；若已有合适准备树先核对所有权再复用。先检查组内跨仓 path 是否解析，禁止为方便而改依赖到临时绝对路径。
- 收据：新增 `docs/plans/reports/784-repo-preservation.md` 与 attachments/784/，含 bundle 校验、refs/恢复路径、备份 hash、演练结果。临时恢复 clone 不位于任何被保护仓内；演练不改原工作区。
- 复用 777 的盘点/ownership/边界门刷新，不另造同类检查器；产出迁出候选路径→未来 owner→资源/生成物/测试/消费者→相关在途计划表。以当时可重放基线记录现有兼容边，不要求本计划消掉所有边。
- merge 清理 auto-lang 执行树；目标仓隔离准备树/refs 若供后续搬迁使用，明确转交及恢复锚，不误当“已清理”或删除其中 WIP。待后续接管记录完成后按各仓 guard 规则清理。

### 规范增量

| delta_id | 操作 | target | before/after rule | 原因 | AC |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/repository-boundaries/project.md | 新增实际仓身份/legacy保全与准备状态；主分支仍旧版，切换须后续验收 | 可恢复、可追溯；不冒称搬仓完成 | AC01/02/03 |

touched_goals 为空：仓库治理准备，不关闭语言/UI能力目标。

## 6. 测试设计

实施者未指派；独立 final review 在同一组树查验，merge 单独收尾。用例由 T03/T04 变为可执行，不把规划算证据。

| Case | owner_stage | AC | 入口/输入 | oracle与边界 |
|---|---|---|---|---|
| V01 | work | 01 | 两仓 legacy refs+bundle，disposable clone恢复 | commit/tree与保全前相等，旧tag可追溯；不编译历史依赖 |
| V02 | work | 01/02 | auto-ui dirty Cargo.lock及其他WIP原字节 | 恢复副本hash相等；原工作区status/hash不变 |
| V03 | work | 02 | 重跑准备、同名不同tip ref控制（临时仓） | 同tip幂等；冲突明确拒绝且原ref不动 |
| V04 | work | 03 | 当前777清单+管理的读取失败反控 | 源码/资产/依赖身份可追溯；必需输入不可读失败，不吞为空 |

Category A：不改 Rust，不运行 cargo t/tv/tf/docs_gen；仅refs、hash、恢复与文档检查。独立复审确认保全载荷实际可恢复。

## 7. 验收标准

| ID | owner_stage | 可观察结果 |
|---|---|---|
| AC01 | work | 两仓旧提交/标签及未提交payload可恢复，V01/02通过 |
| AC02 | work | 主分支、原WIP与remote不变；准备树/refs隔离、幂等，无链接，V02/03通过 |
| AC03 | work | 当前合流基线与迁出owner/消费者表可复放，781/783重叠分明，V04通过 |
| AC04 | review | 独立逐项验证、无未授权删除/跳项，绑定三仓tip/依赖/收据与SD |
| AC05 | merge | SD/ledger与计划归档；准备树明确接管，完成的执行树guard-clean后清理 |

## 8. 执行步骤

| 状态 | ID | owner_stage | 依赖 | 操作/产出与验证 | AC |
|---|---|---|---|---|---|
| [x] | T-01 | work | 779交付+本合同确认 | 核对三仓状态与在途写入范围，冻结 refs/WIP 清单。inventory ×3 落 attachments/784/t01/；auto-ui `M Cargo.lock`+ignored `.claude/`，auto-man 干净+ignored lock/vscode，与快照一致；781/783/787 在途均无旧仓写入交集 | 01/02/03 |
| [x] | T-02 | work | T01 | 不改变原WIP地保存payload/bundle，列明可恢复位置与hash。**r2**：快照身份纳入完整 refs→objectID 映射（state_schema 2，F784-R2-01），LATEST 代=ui `2c2ec22a85dcea56` bundle `a18a2d7c…` / man `e143d51c828956ec`；v1 松散布局（ui `c3ef028f…`/man `2fc84d89…`）与 v2 代（ui `efe2102a…`/man `1567f76…`）原样保留为已验证恢复点；Cargo.lock work/head/diff 三份原始字节；ignored 点名保留 3 文件；manifest 元信息 t02/t03r1/t03r2 三代入 attachments | 01/02 |
| [x] | T-03 | work | T02 | 建legacy refs及准备树；执行V01..03恢复/幂等/冲突反控。refs：两仓 `legacy/pre-v06-extraction`+`legacy-pre-v06-{f7d92eef,51fafe82}`（本地未push）；准备树 `.wt/lang-784/{auto-lang(plan-784-dev),auto-ui,auto-man(codex/v06-extraction)}`，wt-guard 三树 clean。**r2 重跑（state_schema 2）**：V01 recover-drill ok×2（refs 11/29 全等、tree 相等、tag 3/23 全溯）；V02 verify ok×2（原文件全矩阵+bundle list-heads 实际覆盖冻结 refs）；V03 drill **8/8**（D1幂等/D2异tip拒绝/D3漂移拒绝/D4快照分裂/D5 A→B旧快照字节不变+同态reuse/D6原文件漂移删除四反控+复绿+备份不动/D7损坏bundle拒绝零改写/**D8 同HEAD refs新增(tag指旧bundle不可达提交)/移动/删除矩阵+bare fetch实际恢复断言**）+真实仓 preserve/anchor 重跑 reuse | 01/02 |
| [x] | T-04 | work | T03 | 复用777刷新基线，核对781/783与后续计划路径owner。基线重锚 `7d51845ff`（36→39边，780交付面+autodown optional 入册），刷新后 strict pass（0违例/2 grandfathered）；V04 反控 3/3 显式 exit 3；迁出清单 attachments/784/t04/migration-inventory.md（27 ownership 项→owner→资源/测试/消费者→781/783/785/786/787 重叠分明） | 03 |
| [x] | T-05 | work | T04 | 完整收据、V04反控、三仓前后指纹与SD素材；提交docs。收据 v3 reports/784-repo-preservation.md（§8 r1/r2 对账+两连败诊断）；SD-01 快照身份描述更新为含 refs；链接复查 0 失效（t05r2/link-check.txt）；code `3f084979d`(首轮)+`199428080`(r1)+**`d2f12ca2e`(r2)**（plan-784-dev @ base 7d51845ff）；提交后树 clean | 01/02/03 |
| [x] | T-06 | review | T05 | 独立恢复与保护范围审查，SD冻结 | 04 |
| [x] | T-07 | merge | review pass | docs沉淀/归档及执行树清理；准备树显式交接。delivery `2ca68d900` ff-only 落地（rebase 映射 3f084979d→befe52975/199428080→1f1bef226/d2f12ca2e→80f11a1c3，range-diff 3/3 `=`）；SD-01+INDEX+ledger v24→v25（P784-D1/T/R3/M，受管离线投影 Regime B）入库；复审附件 r1–r3 随归档提交；执行树 guard-clean 清理；准备树+legacy refs+三代恢复载荷显式转交后续迁移计划 | 05 |

## 9. 复审记录

2026-10-15 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=确认后work | 0/7；只建文档，未创建 refs/树或备份。Category A 不运行Rust门禁。执行期间若出现未登记WIP，先扩保全清单，不自行清空。

2026-10-15 stage=work | PLAN-784 | plan_revision=1 | outcome=pass（work-owned T-01..T-05 全部完成，交独立review） | worktree D:/autostack/.wt/lang-784/auto-lang | branch plan-784-dev | base_commit 7d51845ffcca4c886698caa947cbaca93bf7fdca | code_commit 3f084979dcabf65e363e19027df9dff0c4b6e0b9 | dependency_revisions: 无代码依赖（只读盘点 auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e / auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e，Python 3.14.2 / git 2.x；777 边界工具复用未改） | task_ids T-01..T-05=5/7 | evidence: [保全收据](reports/784-repo-preservation.md) + [attachments/784/](attachments/784/)（V01 两仓 recover-drill ok / V02 verify ok×2 / V03 drill 4/4+真实仓 anchor 重跑 reuse / V04 反控 3/3 exit 3 / 基线重锚 strict pass 0违例2豁免） | readiness: AC01/02/03 work-owned 全过；open findings=无；未授权项未做（push/默认分支切换/代码树替换/生产迁移零发生）；781/783/787 在途重叠已判定无写入交集 | blockers: 无 | merge-owned pending: 无 cargo 整档（Category A 禁跑）；T-07 归档/ledger/执行树清理/准备树交接仍 pending | next_responsible_role: review（T-06：独立逐 AC 恢复与保护范围审查 + SD-01 冻结，重点=载荷实际可恢复性独立复演、未授权操作零发生、迁移清单可复放性）

2026-10-15 stage=review | review_scope=final | reviewer_identity=当前 Codex 会话（规划作者、非本提交实施者；独立恢复及自选反例，无额外agent） | plan_id=PLAN-784 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-784/auto-lang | branch=plan-784-dev | reviewed_commit=3f084979dcabf65e363e19027df9dff0c4b6e0b9 | base_commit=7d51845ffcca4c886698caa947cbaca93bf7fdca | dependency_revisions=auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e,auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e | spec_inputs/SD hash=[fingerprint](attachments/784/review-r1/fingerprint.json)、[冻结SD](attachments/784/review-r1/spec-delta-reviewed.md) | acceptance_results=AC01/02 partial,AC03 pass,AC04 fail,AC05 pending | findings=F784-R1-01(P1:重复preserve覆盖旧WIP备份),F784-R1-02(P2:verify漏检原ignored内容漂移),F784-R1-03(P3:SD三个失效链接) | evidence=[R1完整报告](attachments/784/review-r1/REVIEW.md)、[独立原始结果](attachments/784/review-r1/results.json)、[可复放反例](attachments/784/review-r1/run.py) | 正控=两仓独立恢复refs/tree/旧tag通过、真实源文件/备份hash一致、原drill4/4、两strict通过、三树guard clean、diff-check通过 | 保护=原仓/真实payload/提交实现前后未变；没有修实现/发布规范/修改ledger/merge/移树 | tasks=保留T-01/T-04，重开T-02/T-03/T-05，2/7 | pending_closeout_gates=Category A禁止cargo整档；T-07规范沉淀/归档/交接/guard清理仍pending | next=work原树修复三项、提交并更新覆盖与收据后再review；不可覆盖真实旧快照来关闭finding。

2026-10-15 stage=work | PLAN-784 | plan_revision=1（R1 needs_fix 三项已修复，重新交独立review） | worktree D:/autostack/.wt/lang-784/auto-lang | branch plan-784-dev | base 7d51845ffcca4c886698caa947cbaca93bf7fdca（不变） | code 3f084979d（首轮）+ 199428080（R1 修复） | deps: 只读 auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e / auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e；777 边界工具未改 | task_ids T-01..T-05=5/7（T-02/03/05 r1 重闭） | evidence: [保全收据 r1](reports/784-repo-preservation.md)（§8 修复对账）+ [t03r1/](attachments/784/t03r1/)（V01 ok×2 / V02 ok×2 含原文件全矩阵 / **drill 7/7** / snapshots 清单）+ [t05r1/link-check.txt](attachments/784/t05r1/link-check.txt) | 修复对账：F784-R1-01 preserve 重写为状态寻址快照（staging+原子 rename、同态完整性校验通过零写入复用、变态新增独立快照、失败 exit 4 不动旧快照；真实 v1 载荷未动，v2 快照 ui `efe2102a435580ae`/man `1567f76ca3ecb7cb`；永久控制 D5+D7）；F784-R1-02 verify 对 tracked WIP/untracked/ignored 原文件统一存在性+hash 校验（漂移/删除判红非零退出、备份不动；永久控制 D6 全矩阵）；F784-R1-03 SD 三链接 `../../plans/` 修正+3 文档全量链接检查 0 失效（t05r1 证据） | blockers: 无 | merge-owned pending: 无 cargo 整档（Category A）；T-07 归档/ledger/执行树清理/准备树交接 pending | next=review（T-06 r2：绑定 199428080 重跑受影响控制，重点=快照不可覆盖性、ignored 漂移判红、链接复核、SD 重新冻结）

2026-10-15 stage=review | review_scope=final | reviewer_identity=当前Codex会话（规划/R1作者、非实施者；独立恢复及自选反例，无额外agent） | plan_id=PLAN-784 | plan_revision=1 | outcome=needs_fix | worktree_path=D:/autostack/.wt/lang-784/auto-lang | branch=plan-784-dev | reviewed_commit=1994280805a524659ca3119ab9845e453a2cbc4a | base_commit=7d51845ffcca4c886698caa947cbaca93bf7fdca | dependency_revisions=auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e,auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e | spec_inputs/SD hash=[fingerprint](attachments/784/review-r2/fingerprint.json)、[冻结SD](attachments/784/review-r2/spec-delta-reviewed.md) | acceptance_results=AC01/02 partial,AC03 pass（相关输入hash与R1相同复用证据）,AC04 fail,AC05 pending | findings=F784-R1-01/02/03均closed；新增F784-R2-01(P2/blocking:快照身份缺refs，HEAD不变而新增标签后仍reuse旧bundle、独立恢复缺标签) | evidence=[R2报告](attachments/784/review-r2/REVIEW.md)、[原始结果](attachments/784/review-r2/results.json)、[可复放控制](attachments/784/review-r2/run.py) | 正控=真实两仓快照独立恢复11/29refs与3/23tags、tree/WIP/保留文件hash相等；永久drill7/7、独立A→B保留与同态reuse、三类源文件变更/删除6反控、SD全5链接、diff-check/三树guard通过 | 保护=两原仓/全部真实载荷/实现tracked hash与HEAD前后未变；未修实现/发布规范/修改ledger/merge/清理 | tasks=保留T-01/T-04，重开T-02/T-03/T-05，2/7 | pending_closeout_gates=Category A无Rust整档；T-07沉淀/归档/准备树交接/执行树guard清理pending | next=work同树修复R2-01、补refs状态控制并提交；两次final失败，下次readiness先提供根因及快照身份/manifest/bundle一致性的覆盖调整，再review。

2026-10-15 stage=work | PLAN-784 | plan_revision=1（R2 needs_fix 一项已修复，重新交独立review） | worktree D:/autostack/.wt/lang-784/auto-lang | branch plan-784-dev | base 7d51845ffcca4c886698caa947cbaca93bf7fdca（不变） | code 3f084979d + 199428080 + **d2f12ca2e（R2 修复）** | deps: 只读 auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e / auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e | task_ids T-01..T-05=5/7（T-02/03/05 r2 重闭） | **两连败诊断（技能检查点，先于修复给出）**：R1（快照覆盖/ignored 漂移漏检）与 R2（身份遗漏 refs）同源——快照模型把"仓状态"窄化为 HEAD+工作区文件态，refs 被当作 bundle 派生元数据，既未进身份也未进复用/复核比较面；各轮只闭合被举反例，未重审 state↔manifest↔bundle 实际内容的完整对应。覆盖调整：①身份=HEAD+完整 refs→objectID+保留内容 hash（state_schema 2）；②创建后/复用前/verify 三处接入 bundle list-heads 实际覆盖冻结 refs 的真恢复 oracle（非 manifest 自证）；③D8 用 bare fetch 实际恢复断言覆盖同 HEAD refs 新增/移动/删除三类变化 | evidence: [收据 v3](reports/784-repo-preservation.md) + [t03r2/](attachments/784/t03r2/)（V01 ok×2 / V02 ok×2 含 bundle refs 覆盖 / **drill 8/8** 含 D8 / snapshots 清单 / LATEST=ui 2c2ec22a85dcea56、man e143d51c828956ec）+ [t05r2/link-check.txt](attachments/784/t05r2/link-check.txt) | 修复对账：F784-R2-01 refs 入身份+三处覆盖断言+真实仓新代快照（v1/v2 代原样保留）；R1 三项保持闭合（D5/D6/D7 复跑绿） | blockers: 无 | merge-owned pending: 无 cargo 整档（Category A）；T-07 归档/ledger/执行树清理/准备树交接 pending | next=review（T-06 r3：绑定 d2f12ca2e，重点=D8 反例复放、bundle refs 覆盖 oracle、状态↔manifest↔bundle 对应完整性、SD 重新冻结）

2026-10-15 stage=review | review_scope=final | reviewer_identity=当前Codex会话（规划/前两轮复审作者、非实施者；独立恢复与自选反控，无额外agent） | plan_id=PLAN-784 | plan_revision=1 | outcome=pass | worktree_path=D:/autostack/.wt/lang-784/auto-lang | branch=plan-784-dev | reviewed_commit=d2f12ca2e37cab568041f2b548f6dad29e516dc4 | base_commit=7d51845ffcca4c886698caa947cbaca93bf7fdca | dependency_revisions=auto-ui@f7d92eef273c7f5592cce9d1612fa28ebe12d45e,auto-man@51fafe82e8e0c4e66325ee14088eed8c3270e07e | spec_inputs/SD hash=[fingerprint](attachments/784/review-r3/fingerprint.json)、[冻结SD](attachments/784/review-r3/spec-delta-reviewed.md) | acceptance_results=AC01/02/03/04 pass,AC05 merge-owned pending | findings=F784-R1-01/02/03及F784-R2-01全部closed，无新增阻断；两连败诊断与refs/manifest/bundle覆盖调整已核对实际实现 | evidence=[R3报告](attachments/784/review-r3/REVIEW.md)、[19/19独立结果](attachments/784/review-r3/results.json)、[可复放控制](attachments/784/review-r3/run.py) | 验证=两真实新代快照独立恢复11/29refs与3/23tags、tree与保留文件hash一致；永久drill8/8；旧A→B/源变更删除6反控/refs新增移动删除及branch新建/有效旧bundle+同步hash仍被覆盖oracle拒绝全部通过；SD5链接、diff-check、三树guard通过；实现树和主检出使用交付baseline两strict均0违例2grandfathered | 保护=原仓/全部真实payload/实现tracked hash及HEAD前后不变，准备树仍隔离clean；主检出HEAD前进仅781簿记，不影响审面 | tasks=T-01..T-06完成，reviewed 6/7 | pending_closeout_gates=Category A禁止cargo整档，无Rust回归待补；T-07规范沉淀/ledger/归档/准备树明确交接/执行树guard后清理pending | next=merge核对冻结证据并完成T-07；本轮未merge/发布规范/改ledger/移树，review pass不等于交付完成。

2026-10-15 stage=merge | PLAN-784:r1 | outcome=pass（delivered） | 收据 checkpoints：
- **prepared**：reviewed_commit=80f11a1c3（R3 pass 绑定 d2f12ca2e，rebase 等价映射后）；冻结 SD-01 SHA256 `3285de1b50564b96…` 应用前实测与 worktree 文件一致；canonical diff=新增 docs/specs/repository-boundaries/project.md + INDEX.md 注册（32 projects）+ ledger v24→v25；delivery_commit=2ca68d900（reviewed 代码的纯文档/投影后代，实现与依赖零变更）。
- **landed**：master `git merge --ff-only plan-784-dev`，tip=2ca68d900=delivery（972dddf10→2ca68d900，无合并提交）。rebase 旧→新映射：3f084979d→befe52975、199428080→1f1bef226、d2f12ca2e→80f11a1c3、44289efa9→ff7f20064；`git range-diff` 3/3 全 `=`（补丁等价=安全重写证明）。
- **ledger_refreshed**：`.autoos/specs.json` 受管离线投影（Regime B，无 store writer；字节级备份 D:/autostack/.wt/lang-784/specs-backup-784.json；序列化器对旧文档 round-trip 自证后写入，git diff 仅 +43/-1），version 24→25，新增 P784-D1（designs，file=docs/specs/repository-boundaries/project.md）/P784-T（tests，file=scripts/legacy-preservation.py）/P784-R3（reviews，file=review-r3/REVIEW.md）/P784-M（reports，file=docs/plans/reports/784-repo-preservation.md），全部带 file+tags；六段集合/顺序、旧 items 前缀、project 字段校验通过，读回 ID 验证。
- **Category A 门禁**：未运行 cargo t/tv/tu/tt/tb/docs_gen/taa（784 全程零 crates/ Rust 源码改动——仅 scripts/、.autoos/、docs/）；按 AGENTS Category A 此为禁止项而非豁免。R3 已在实现树与主检出双跑 strict（均 pass 0 违例/2 grandfathered）。
- **archived**：git mv → docs/plans/archive/784-repo-extraction-legacy-preservation.md，status: archived，T-01..T-07=7/7，completion_kind=delivered；review-r1/r2/r3 复审附件随本收据入库（K13：不入库≠归档）。KNOWN-DEBT 784 段已注 merge 收尾（该文件含他计划在途 WIP，按 777 先例不代提交，由其 owner 提交）。
- **cleaned**：wt-guard lang-784/auto-lang clean → `git worktree remove` + `git branch -d plan-784-dev`；**准备树 .wt/lang-784/{auto-ui,auto-man}@codex/v06-extraction、legacy refs（两仓 branch+tag，未 push）、仓外三代恢复载荷 D:/autostack/.recovery/784-legacy-preservation/ 显式转交后续迁移计划（785/786 及 G/H 波），不清理不删除**；组目录因准备树保留。
- **部署产物核对**：auto-lang 无发布二进制/web bundle 生产面（Category A 工具/文档交付），N/A。

## 10. 待澄清事项

当前无须用户先决定分支技术细节；如同名refs占用、目标仓有外部写入或恢复payload不可读，T01/T02给出具名冲突后再处理。迁移完成前是否改master为main、何时push，留给正式跨仓交付计划。
