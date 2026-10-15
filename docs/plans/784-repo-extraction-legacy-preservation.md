---
plan_id: PLAN-784
status: drafting
feature_name: 旧仓保全与拆分迁移准备
author: [Codex]
created_at: 2026-10-15
updated_at: 2026-10-15
plan_revision: 1
current_step: 0
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
| [ ] | T-01 | work | 779交付+本合同确认 | 核对三仓状态与在途写入范围，冻结 refs/WIP 清单 | 01/02/03 |
| [ ] | T-02 | work | T01 | 不改变原WIP地保存payload/bundle，列明可恢复位置与hash | 01/02 |
| [ ] | T-03 | work | T02 | 建legacy refs及准备树；执行V01..03恢复/幂等/冲突反控 | 01/02 |
| [ ] | T-04 | work | T03 | 复用777刷新基线，核对781/783与后续计划路径owner | 03 |
| [ ] | T-05 | work | T04 | 完整收据、V04反控、三仓前后指纹与SD素材；提交docs | 01/02/03 |
| [ ] | T-06 | review | T05 | 独立恢复与保护范围审查，SD冻结 | 04 |
| [ ] | T-07 | merge | review pass | docs沉淀/归档及执行树清理；准备树显式交接 | 05 |

## 9. 复审记录

2026-10-15 stage=new | plan_revision=1 | outcome=pass（草案preflight） | next=确认后work | 0/7；只建文档，未创建 refs/树或备份。Category A 不运行Rust门禁。执行期间若出现未登记WIP，先扩保全清单，不自行清空。

## 10. 待澄清事项

当前无须用户先决定分支技术细节；如同名refs占用、目标仓有外部写入或恢复payload不可读，T01/T02给出具名冲突后再处理。迁移完成前是否改master为main、何时push，留给正式跨仓交付计划。
