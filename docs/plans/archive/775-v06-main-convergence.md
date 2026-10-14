---
plan_id: PLAN-775
status: archived
feature_name: v06-main-convergence
plan_revision: 6
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-15
supersedes_spec_components: []
new_spec_components: []
touched_goals: []
affects: [auto-lang, auto-down, auto-os, auto-musk, app-repositories]
current_step: 8
total_steps: 8
---

# [PLAN-775] v0.6 已提交更新汇合与在途计划续做

## 变更摘要

按用户 2026-10-13 的明确修正，本计划先把已经提交的 v0.6 开发更新汇入各仓默认分支；不等待 Plan 742/744/773、Notes、Blog、Reader、Launcher、Weather、Explorer 等在途工作全部完成。相关 Plan 保持原状态和未完成项，合流后在各自权威工作树继续。

使用普通 merge、fast-forward 或可追溯 cherry-pick；不重写已发布历史，不 force-push，不删除恢复分支和用户 WIP。AutoLang 的默认分支名仍为 `master`，其他仓沿用各自 `main`/`master`；合流后各默认分支作为 v0.6 后续开发线。

## 目标

1. 将各仓已提交且仍有效的 v0.6 增量放入默认分支，保留发布主线第一父链与原开发引用。
2. 先发布应用子仓，再更新 AutoOS 固定 SHA；所有新写入的 gitlink 指向已推送的提交。
3. 保持在途 Plan 的实际状态、验收目标、未提交工作和原工作树，不将分支合流误记为功能完成。
4. 对未变化仓给出已有证据支持的无需合并判断；完成最终远端引用、fresh-clone 与安全清理核验后再关闭本计划。

非目标：完成各功能 Plan 的 required 条件；将 drafting/executing 改为完成；改名现有默认分支；改写 v0.5 tag 或 v0.6-dev 历史；删除旧分支、异机 WIP 或不属于本计划的工作树。

## 架构方案

AutoLang 与 AutoOS 从发布默认分支创建候选并用普通 merge 汇入开发历史；其他存在有效开发增量的仓使用 fast-forward，或在提交仅存在于本地时以 `cherry-pick -x` 保留来源。AutoDown 与 Notes 的依赖顺序为先发布可包含 `src` 的 AutoDown 包，再由 Notes 提交更新嵌套 gitlink，最后由 AutoOS 更新外层 Notes gitlink。

AutoOS v0.5 对 Tetris 与 Klondike 的源文件差异先落入产品仓并推送，再由 AutoOS 固定这两个产品提交。任何本地 checkout 的脏状态仅作调查对象，不通过 reset、stash、清理或复制覆盖处理。

## 需求分析与背景调查

用户先授权执行，后明确要求“先把 v0.6-dev 上已有更新合并回 main；未完成的计划可以保持做到一半，合并后再单独做”。此修正替代 revision 2 中“先完成所有在途工作再合并”的先决顺序，不改变各在途 Plan 自身的 required 验收合同。

基线盘点覆盖 GitHub `auto-stack` 37 个相关仓和本机 27 个顶层检出，见 [初始调查报告](../reports/v06-main-convergence-20261012/README.md)。远端变化仓在推送前刷新了 refs；最终提交和推送收据见 [2026-10-13 合流收据](../reports/v06-main-convergence-20261012/merge-receipt-20261013.md)。该报告不能证明本机以外不存在未推送工作。

Plan 编号联合表不变：发布线 767–769 保持原义；v0.6 计划现行编号映射为 772/773/774；本计划为 775。Plan 773 当前为 `execution_done`，Plan 774 为 `drafting`；Plan 772 位于 archive。合流不改变这些状态，也不把它们当作本计划已完成的功能验收。

## 详细设计

本次发布结果：

| 仓库 | 默认分支收口点 | 处理 |
|---|---|---|
| auto-lang | 审计时 `master` `d777ccbb227c41882ad5f6976c803b1e03b61135` | 纠偏 merge `3002c30ffd618f768031048742bcc700a2aff363` 以 `v0.5.0` tag 为第一父、旧 `master` 为第二父；此审计点后只追加本计划收据文档。tag 和既有提交均未改写 |
| auto-down | `master` `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8` | 小型补充提交把 engine 包清单从 `dist` 扩为 `dist` + `src`，恢复 Notes 所需的可发布源文件 |
| auto-notes | `main` `1391bd7eb2f6b3b08e9841d30b25125e47ced058` | 普通后续提交把 `vendor/auto-down` 指向上述已发布 SHA；旧不可达 pin 留在其历史祖先 |
| auto-blog | `main` `08da63359c4f8d147c0c10f2c774d6feaad79425` | fast-forward 到已推送开发头 |
| auto-reader | `main` `29b5decf878ec2cdca5c6a39575e872a4597121d` | 默认分支与开发头已一致，无额外提交 |
| auto-launcher | `main` `df207283536ff943e2e574dc886c4cb8594cce13` | fast-forward 到已推送开发头 |
| auto-weather | `main` `1626a4a16ccf4de717bc92e27b60a9184b8198f2` | 默认分支与开发头已一致；Plan 008 的未提交工作另行保留 |
| auto-explorer / auto-file-manager | `main` `59a99b9f42896728aa32a7ff3e38fa08261c7d18` | fast-forward 到已推送开发头 |
| auto-tetris | `main`、`v0.6-dev` `06b2e67f2117084853271cf194e67cc47be6b3c5` | 在产品仓保留 v0.5 源更新后推送两条分支 |
| auto-solitaire / auto-klondike | `main`、`v0.6-dev` `91192ef593cb25377a2d089542e6f040507231a9` | 在产品仓保留 v0.5 源更新后推送两条分支 |
| auto-os | `main` `853a551e681ad888f4c72c519b6ad1df94e7f313` | 普通 merge；第一父线 `e7fadc470684f611620175c722a1a22df5e9d9c9`，第二父线 `71d3b5f43ff4a20b998bca2917f7bf62b101fdd7` |
| auto-musk | `main` `cb4cb9570ab185b0a2a7a645cc80edd65a1689f4` | 两条本地唯一的 Plan skill 文档提交用 `cherry-pick -x` 迁入远端主线；原本地 main 和原提交保持不动 |
| jade-edit | `main` `5fb1e9e28e5a7e379317b7dffb400bad6c94606d` | 无 `v0.6-dev` 分支；从旧 `main` `4d49bbc45bd62da2724a4bc678983f6133087470` 快进至 `v0.5.0` tag target |

2026-10-13 再次通过 GitHub API 刷新全部 37 个默认分支头：仅 AutoLang receipt-only 文档提交与 jade-edit 的 v0.5 tag 快进改变了上一版快照。23 个远端 `v0.6-dev` 尖端全部是对应默认分支祖先（compare `behind_by=0`）；可解析的 11 个 `v0.5.0` tag 全部是默认分支祖先，另 26 个仓没有该 tag。完整头列表见 [最终远端 refs](../reports/v06-main-convergence-20261012/final-refs-20261013.json)。AutoOS 候选的根提交已推送；在独立验证副本中确认 recursive 子模块无 SHA 不匹配，`verify-app-links.py` 通过（21 个新产品、24 个子模块，另核对 3 个既有应用）。

### 规范增量

| delta_id | 操作 | 目标 | before / after | 原因 | 验收 |
|---|---|---|---|---|---|
| SD-00 | none | n/a | 此计划只汇合现有 Git 历史和依赖引用，不新增 AutoLang 运行时规则 | 功能 Spec 继续由各自 Plan 维护；本次合流保留其已提交版本 | AC-01–AC-05 |

Spec-impact frontmatter 为空是有意的：本次不直接修改 AutoLang canonical Spec；AutoOS 随开发线合入的产品规格仍以其仓库记录为准。

## 测试设计

- 历史检查：验证发布主线是 AutoLang/AutoOS merge 的第一父线，开发头为第二父线；其余推送为祖先保持的 fast-forward/cherry-pick；检查 v0.5 tag 和原 v0.6-dev refs 未移动。
- AutoLang：本轮执行记录已确认 `cargo check -p auto-lang -p auto-man --quiet`、`cargo tv` 通过；`cargo t`/`cargo tt` 的已知红按既有 Plan/基线记录区分。本次主要工作是历史整合，不重跑全套 feature 门禁。
- AutoDown：`npm pack --dry-run --json` 通过，包含 178 个 `src/*` 项及 `src/index.ts`。
- AutoMusk：迁入文档 `git diff --check` 通过；五个变更 Markdown 文件的相对链接检查通过。
- AutoOS：关键 JSON 可解析、`scripts/desktop.sh` 的 Bash 语法通过，手工冲突文件的 staged whitespace 检查通过。独立 AutoOS 检出 recursive 子模块 SHA mismatch=0；`scripts/verify-app-links.py` 通过，验证 21 个新产品和 3 个既有应用。
- 只对分支/索引执行的变更不以这些检查替代原功能 Plan 的验收。

## 验收标准

| ID | owner_stage | 当前结果 |
|---|---|---|
| AC-01 | merge | 既有提交与 tag 均未改写；AutoLang 的 `v0.5.0` tag 经追加 merge 成为 `master` 祖先；37 个默认分支已刷新，23 个 `v0.6-dev` 头均可从默认分支到达；11 个存在的 `v0.5.0` tag 均为祖先。 |
| AC-02 | work | 在途功能 Plan 原状态、原 required 目标和独立工作树继续保留；本次不要求其先完成。 |
| AC-03 | merge | AutoLang 纠偏 merge `3002c30ffd618f768031048742bcc700a2aff363` 的第一父为 `v0.5.0` tag commit `e05e2d3d7832c7f1d7e29a09a371648b5a53f692`，第二父为原 `master` `9e54f738a07c3715e055ef2a763380e8ec31253e`；AutoOS 父顺序仍按原收据核验；其他开发更新可追溯，AutoMusk 两提交带 `-x` 来源记录。 |
| AC-04 | merge | Tetris/Klondike 的 v0.5 源增量已先推产品仓；AutoOS 指针指向已推提交；Notes 的 nested pin 指向已推 AutoDown 修复。 |
| AC-05 | review | 变更仓按范围检查通过；独立 AutoOS 检出 recursive 子模块无 SHA mismatch，`verify-app-links.py` 通过。 |
| AC-06 | review | 本阶段复核历史、冲突解决、来源、debt/未完成项和工作区边界；结论限定为 branch-convergence phase，不宣称整体 Plan 最终通过。 |
| AC-07 | merge | 默认 `main`/`master` 作为 v0.6 后续开发线；37 仓远端 refs 已记录并复核。审计时 AutoLang 本地 `master`/`origin/master` 同步在 `d777ccbb…`；jade-edit main 已快进到 v0.5 tag。AutoOS dirty 候选及用户本地 WIP 均保留，不作为合并或功能完成条件。 |

## 执行步骤

| Task | owner_stage | 操作与完成证据 | 状态 |
|---|---|---|---|
| T-01 | work | 正式登记 PLAN-775、联合 Plan 编号和本地 WIP 快照。 | [x] 2026-10-13：`.next-id=776`；主检出 `auto/` 未跟踪目录保留。 |
| T-02 | work | 以调查报告为基线，刷新所有有变化仓 refs；标明用户 WIP 和本地唯一提交的保留方式。 | [x] 变化仓推送前均刷新；AutoLang/AutoOS/AutoMusk/AutoDown/Notes 和产品子仓的最终 refs 见收据。未变化仓刷新列入 T-08。 |
| T-03 | merge | 普通合入 AutoLang 发布线、本地有效提交与 v0.6-dev 更新，推送 `master`。 | [x] 原 merge `7749510…` 已发布但第一父 `f903848…` 并非 `v0.5.0` 后代；2026-10-13 追加纠偏 merge `3002c30…`（第一父 tag `e05e2d3…`，第二父原 `master` `9e54f73…`），以旧头校验后非强制更新远端 `master`。既有提交/tag 均未改写。 |
| T-04 | merge | 按依赖顺序更新 AutoDown、应用仓与产品 gitlink；保存 Tetris/Klondike 源增量。 | [x] AutoDown `c39ac7f…`，Notes `1391bd7…`，应用与产品 SHA 见表。 |
| T-05 | merge | 普通合并 AutoOS 的发布线和 v0.6-dev，发布固定 SHA。 | [x] merge `853a551…`，父顺序已独立核对；推送时 `origin/v0.6-dev=71d3b5f…` 未变。 |
| T-06 | merge | 在不覆盖原本地 checkout 的前提下将 AutoMusk 两条有效提交纳入远端 main。 | [x] cherry-pick `-x` 为 `eecac9b…`、`cb4cb95…`；原始 `8e09f5b…`、`bfcf373…` 在原本地 main 保留。 |
| T-07 | review | 复核本阶段的历史、发布 SHA、用户范围修正、未完成计划和已知验证边界。 | [x] 2026-10-13 phase pass；自审限制与待办见 §复审记录。 |
| T-08 | merge | 刷新所有仓远端 refs；只快进无 WIP 的本地默认检出；在干净 AutoOS checkout 运行 `verify-app-links.py`/recursive submodule 验证；完成最终收据和通过 wt-guard 的安全清理。 | [x] 2026-10-13：37 个默认 refs 复核；23 个 v0.6-dev 尖端与 11 个可解析 v0.5.0 tag 均为默认分支祖先；AutoLang 本地检出同步到 `d777ccbb…`；jade-edit main 快进到 tag；AutoOS fresh-clone recursive/app-link 验证通过。所有可安全清理的临时工作树已按收据清理；dirty AutoOS 候选与用户 WIP 按保全要求原样保留。 |

## 复审记录

### 2026-10-13 — revision 2 的用户范围修正

- 用户明确要求分支合流先行，未完成 Plan 可以保持“做到一半”，并在主线更新后继续；因此 revision 2 中“先完成所有 in-progress Plan”的顺序不再适用。
- 该修正只改变本计划的顺序和验收范围，不修改 Plan 742/744/773/774、Notes、Blog、Reader、Launcher、Weather、Explorer 等原合同。它们不得因本次 merge 自动改为 completed/reviewed/archived。

### 2026-10-13 — stage: work / T-01–T-06 完成

- revision: 3；outcome: executing；提交和 refs 见 [合流收据](../reports/v06-main-convergence-20261012/merge-receipt-20261013.md)。
- 当时记录 AutoLang/AutoOS 保留发布线第一父线；AutoDown/Notes 以新提交修复 Notes 不可达嵌套 pin；AutoMusk 的本地有效规约改动通过带来源标记的 cherry-pick 保留。AutoLang 的父线判断已由 revision 5 纠正，见下。
- 自动化审核曾拒绝在 dirty 子模块目录上批量执行 `git submodule update --checkout`，原因是该动作可能覆盖子模块内未提交状态。本计划不绕过该拒绝；先通过索引和远端 SHA 核验，之后在独立 fresh clone 完成验证。原 AutoOS 集成候选仍保留 dirty 子模块状态。
- 已确认 Plan 773 仍为 `execution_done`、Plan 774 仍为 `drafting`，未把这些状态提升或关闭；应用原工作树与未提交工作也未清理。
- next_responsible_role: review；review_scope: phase；下一步对 branch-convergence 阶段独立复核，记录复核限制和未完成的 closeout gates。

### 2026-10-13 — stage: review / T-07 branch-convergence phase pass

- review_scope: phase `branch-convergence`; plan_id: PLAN-775; plan_revision: 3; overall status remains `executing`.
- reviewed_commit: AutoLang `e35f9e99b6711ff6f1990ea980729d1fa521327e`; AutoLang merge `77495105284b4ba657c1bd1aa3b864d72aeddb17` first/second parents `f903848bd8a468f9c4e166a217c1c52111f689a1` / `2e9777b191f32683e8a3bbbe0b993675e38cce63`; AutoOS merge `853a551e681ad888f4c72c519b6ad1df94e7f313` first/second parents `e7fadc470684f611620175c722a1a22df5e9d9c9` / `71d3b5f43ff4a20b998bca2917f7bf62b101fdd7`; AutoMusk `cb4cb9570ab185b0a2a7a645cc80edd65a1689f4`; AutoDown `c39ac7f9dacdc769fc4a5ad3b98ec896240b28e8`; Notes `1391bd7eb2f6b3b08e9841d30b25125e47ced058`.
- reviewer_identity/limitations: same task context as execution; no separate reviewer identity is claimed. This is a phase review for the published branch convergence, not a final Plan pass. The Plan remains executing while T-08 cleanup is pending.
- acceptance_results at revision 4: AC-01 pass (37 default branch heads refreshed); AC-02 pass (unfinished Plan states and requirements preserved); AC-03/AC-04 pass as then reviewed; AC-05 pass (scoped checks and clean-clone recursive/app-link validation); AC-06 pass for this phase only; AC-07 partial because the dirty AutoOS candidate remains. AutoLang ancestry portion of AC-01/03 is superseded by revision 5 below.
- findings: F-01, resolved for verification — the first app-link check hit dirty/unmaterialized submodules; an independent clone later passed with recursive pointer mismatches=0. F-02, closeout pending — AutoOS integration worktree root status still reports 20 dirty app submodules and recursive gitlink mismatches=2 (Notes main and its nested AutoDown pin); preserve it and do not remove. Bulk checkout was rejected by automatic approval review because it could overwrite uncommitted submodule state; no bypass was attempted.
- evidence: [merge receipt](../reports/v06-main-convergence-20261012/merge-receipt-20261013.md), [final remote refs](../reports/v06-main-convergence-20261012/final-refs-20261013.json); push/remote SHA receipts; AutoDown `npm pack --dry-run --json`; AutoMusk markdown-link/whitespace checks; AutoOS JSON/Bash/whitespace checks and fresh-clone recursive/app-link result; `wt-guard` results in the receipt.
- next: Keep the dirty AutoOS candidate intact until its submodule state can be reconciled safely. Resume unfinished feature Plans separately in their existing trees.

### 2026-10-13 — revision 4 / T-08 closeout recheck

- review_scope: phase `branch-convergence-closeout`; plan_id: PLAN-775; plan_revision: 4; overall status remains `executing`.
- reviewer_identity/limitations: same task context as execution; no separate reviewer identity is claimed. This recheck verifies updated closeout evidence and does not claim the active AutoOS worktree is clean.
- verified: final refs JSON parses and contains 37 entries; GitHub API heads match the receipt's changed-repository SHAs; auto-ai/auto-shell/auto-forge fast-forwards ended at the audited SHAs; AutoOS fresh-clone `verify-app-links.py` passed and recursive mismatch count was zero; clean AutoLang/AutoMusk worktrees and verification clone were removed after guard.
- remaining: AutoOS integration candidate has no reparse points but recursive Git status shows 20 dirty app submodules and 2 gitlink mismatches. It remains intact. T-08 is still unchecked, AC-07 partial, and PLAN-775 is not archived.
- disposition: branch-convergence objective is delivered to the remote default branches; retain this Plan for cleanup closeout only. Do not resume or complete unrelated feature Plans as part of this Plan.


### 2026-10-13 — revision 5 / AutoLang release ancestry correction

- review_scope: phase `branch-convergence-lineage-correction`; plan_id: PLAN-775; plan_revision: 5; overall status remains `executing`.
- reviewer_identity/limitations: same task context as execution; no separate reviewer identity is claimed. This focused review audits the parent graph, tree delta, publication guard, and preservation of unfinished work.
- finding: the original AutoLang merge `77495105284b4ba657c1bd1aa3b864d72aeddb17` has parents `f903848bd8a468f9c4e166a217c1c52111f689a1` / `2e9777b191f32683e8a3bbbe0b993675e38cce63`; its first parent is not descended from `v0.5.0`. The common ancestor was `38e4c7c5fa5b8c7f68b8054e342f106a7ec9f5fd`; five release-side commits after it were therefore absent from the then-current `master`. The local `v0.6-dev` head `2e9777b191f32683e8a3bbbe0b993675e38cce63` was already reachable through the old merge's second parent.
- corrective_commit: `3002c30ffd618f768031048742bcc700a2aff363`; parents are `e05e2d3d7832c7f1d7e29a09a371648b5a53f692` (the unchanged `v0.5.0` tag target) then `9e54f738a07c3715e055ef2a763380e8ec31253e` (the old published `master`). The tree is `01e8f20f93dd2542f145b8dddf3c5c5884f91653`; merge was conflict-free and adds the five release-side file changes.
- verification: GitHub compare confirms old `master` and `v0.5.0` are both ancestors of the correction; the tag-to-correction merge base is the tag commit. Local candidate was clean, `git diff --check` passed, and the documented website build/e2e receipt for the imported locale-label change remains in `docs/reports/fix-language-labels-verification.json`. No Rust source changed, so no Cargo tier was run.
- publication: GitHub `master` moved from `9e54f738a07c3715e055ef2a763380e8ec31253e` to `3002c30ffd618f768031048742bcc700a2aff363` with expected-old-SHA checking and `force=false`. No tag or existing commit was rewritten.
- limitation: the local checkout still points at the old `master` because shell GitHub fetch is blocked by the current network sandbox; fetch the published `master` when connectivity is available. The AutoOS dirty integration candidate and T-08 closeout status remain unchanged.

### 2026-10-13 — revision 6 / all-repository ref audit and Jade Edit fast-forward

- review_scope: branch-convergence-closeout update; plan_id: PLAN-775; plan_revision: 6; overall status remains `executing` pending a final review and plan archival.
- refreshed_refs: GitHub API returned all 37 default branches. Only `auto-lang/master` advanced from `3002c30…` to receipt-only commit `d777ccbb227c41882ad5f6976c803b1e03b61135`, and `jade-edit/main` advanced from `4d49bbc45bd62da2724a4bc678983f6133087470` to `5fb1e9e28e5a7e379317b7dffb400bad6c94606d`.
- jade_edit: the repository has no `v0.6-dev` branch. Its old `main` was two commits behind the `v0.5.0` tag target; `main` was fast-forwarded (no force) to `5fb1e9e…`. The two included commits modify `DEBTS.md`, `docs/parity-ledger.md`, and archived Plan 039 only. `v0.5.0` now equals `main`; old main remains an ancestor.
- branch_audit: all 23 remote `v0.6-dev` tips are ancestors of their default branches (`behind_by=0`). Of 37 repositories, 11 have a resolvable `v0.5.0` tag and all 11 tags are ancestors; 26 have no resolvable tag. AutoLang compare is `v0.5.0..master = ahead 311, behind 0`.
- local_sync at refs capture: `master` and `origin/master` were both `d777ccbb…`; after publishing the receipt-only commit `45db8e25…`, the Git CLI still cannot fetch via SSH. The local `master` remains at `d777ccbb…`, while its three tracked documentation files match the published `45db8e25…` blobs; the pre-existing `auto/` untracked directory remains untouched. Existing plan worktrees remain present.
- preservation: AutoOS's dirty integration candidate remains intact (18 dirty app submodule worktrees, 2 recursive gitlink mismatches on the latest read-only inspection) because its state is not safe to remove; this does not block the published branch convergence. Feature Plans remain at their recorded statuses for separate continuation.

### 2026-10-13 — stage: review / PLAN-775 branch-convergence phase

- review_scope: phase `branch-convergence`; plan_id: PLAN-775; plan_revision: 6; outcome: pass; overall Plan status remains `executing` so preserved cross-repository WIP and unfinished feature Plans can continue separately.
- reviewer_identity/limitations: same task context as execution; no separate reviewer identity is claimed. I reconstructed the result from current GitHub refs/compare responses, the published commit/tree objects, and read-only inspection of preserved worktrees rather than relying on the earlier execution summary.
- worktree_path / branch: local `D:/autostack/auto-lang`, `master` at `d777ccbb227c41882ad5f6976c803b1e03b61135` with three documentation files matching the published review baseline blobs; review baseline on GitHub is `45db8e25f8eec63b67e21729ea00e002a8336d6d`. AutoOS preservation candidate `D:/autostack/.wt/lang-775/auto-os`, `plan-775-dev` at `853a551e681ad888f4c72c519b6ad1df94e7f313`.
- base_commit: AutoLang correction merge `3002c30ffd618f768031048742bcc700a2aff363`; reviewed remote chain is `3002c30…` → `d777ccbb…` → `45db8e25…` (the latter two are receipt-only documentation commits).
- reviewed_commit: `45db8e25f8eec63b67e21729ea00e002a8336d6d` (remote; the published revision-6 Plan and receipt blobs were read back and matched against the reviewed content).
- dependency_revisions: final refs snapshot `../reports/v06-main-convergence-20261012/final-refs-20261013.json`; no code/spec dependency moved after the captured heads except AutoLang receipt-only documentation.
- spec_inputs: SD-00 `none`, frozen in Plan revision 6 (blob `2b0f32305e83ad724e6279092983c6790464af62`); no canonical runtime Spec delta.
- acceptance_results: AC-01 pass (37 default refs refreshed; 23/23 `v0.6-dev` tips reachable; 11/11 resolvable `v0.5.0` tags are ancestors, 26 repos have no tag); AC-02 pass (Plan 742/744/773/774 and existing worktrees retained at their existing states; root `auto/` untracked directory retained); AC-03 pass (AutoLang correction merge parents verified; remote and local `v0.6-dev` tips are ancestors; AutoOS merge parents and AutoMusk `-x` source recorded); AC-04 pass (AutoOS product gitlinks and Notes `vendor/auto-down` resolve to published SHAs); AC-05 pass (reused unchanged scoped test and fresh-clone evidence from the receipt; this review changed docs/refs only, so no Cargo or docs_gen tier was run); AC-06 pass (approved deferrals and preserved WIP are explicit); AC-07 pass (all audited default branches are the v0.6 development lines; Jade Edit now equals its v0.5 tag target).
- findings: F-03, resolved — Jade Edit `main` was two docs commits behind its `v0.5.0` tag, with no `v0.6-dev` branch; non-forced fast-forward moved `main` to the tag. No open branch-convergence findings.
- pending_closeout_gates: keep the AutoOS candidate intact; latest read-only status shows 18 modified app submodule worktrees and two recursive gitlink mismatches (`apps/015-notes` and its nested `vendor/auto-down`). Do not reset, checkout, or remove it. This is preserved local WIP, not an unmerged published branch update.
- evidence: GitHub branch-ref and compare checks for 37 repos; AutoLang `v0.5.0..master` ahead 312/behind 0 and both `v0.6-dev` heads behind 0; Jade Edit `v0.5.0` vs `main` identical; AutoOS merge parents/tree gitlinks; Notes nested pin; receipt and final refs files.
- next: continue the unfinished feature Plans separately from the updated development line; keep this Plan executing until the preserved AutoOS candidate receives its own safe handoff/cleanup decision.

### 2026-10-15 — stage: review / final closeout pass

- review_scope: final `closeout`; plan_id: PLAN-775; plan_revision: 6（终审只加簿记，不改语义合同，revision 不动）; outcome: **pass**；Plan 状态 `executing` → `reviewed`。
- reviewer_identity/limitations: 关账会话在主检出 `D:/autostack/auto-lang`；与执行同体系、无独立复审者身份，结论从活体远端 refs（ls-remote）、本地血统断言、收据与 final-refs 工件重建，不依赖执行者总结。
- worktree_path / branch: 登记在案的保全候选 `D:/autostack/.wt/lang-775/auto-os`（`plan-775-dev` @ `853a551…`）已不存在——见下 disposition；本次关账仅涉 docs/plans 簿记，无需活动实现树。
- reviewed_commit: 已发布复审基线 `45db8e25…`（revision 6）经 `merge-base --is-ancestor` 重验为本地 `master b74f1b492…` 与 `origin/master 2a510751…` 的祖先；关账在其上追加 docs-only 提交。
- acceptance_results（2026-10-15 重验）: AC-01 pass——`v0.5.0` tag `e05e2d3d…` 与远端 `v0.6-dev` 尖端 `94173fe2…` 均为 `origin/master 2a510751…` 祖先（37 仓全量面以 final-refs-20261013.json 存档为准）；AC-02 pass——Plan 742/744/773/774 均经各自后续复审走到 archived（合流未强改状态、后续续做为计划本意）；AC-03 pass——纠偏 merge `3002c30…` 为现 master 祖先（本地验证），AutoOS merge `853a551…` 双父 `e7fadc47…`/`71d3b5f4…` 经 `git cat-file -p` 重验；AC-04 pass——远端抽查 `auto-os main=853a551`、`auto-notes main=1391bd7`、`auto-down master=c39ac7f`，与收据汇合点逐一相符；AC-05 pass——复用 2026-10-13 定向检查与 fresh-clone 证据（理由：本次关账只改 docs/plans，代码/依赖/测试配置输入未变）；AC-06 pass——本终审即其闭合；AC-07 pass——各默认分支仍为 v0.6 开发线，遗留保全项已按下方 disposition 关闭。
- preserved_candidate_disposition: `.wt/lang-775` 组目录（含 AutoOS 脏集成候选工作树与 `plan-775-dev` 分支）在本机已不存在，且未找到移除收据。候选分支与 `main` 同指 `853a551…`，commit 层面无损；丢失的仅是 18 个子模块的未提交 WIP。用户 2026-10-15 知悉该工作树已消失并明确授权最终关账（接受该损失）；`auto-os` 主检出 clean 且正处 `853a551…`。
- findings: 无新增合同内缺陷。REG-5（`cargo taa` 两确定性红：`aavm2_a2r::test_aavm2_goldens_check` + `aavm_runner_tests::test_aavm2_compile_corpus`，归因汇入提交 `b1230e767` 的 json.encode 泛型 adapter 规则）已登记于 KNOWN-DEBT-AND-RISKS.md，按 2026-10-02 aavm 豁免裁定非阻塞；批量回归收据 `.last-batch-regression.json`（2026-10-14，verdict `green-with-registered-new-reds`）已覆盖含 775 的窗口。
- pending_closeout_gates: 无——docs-only 关账（Category A，无 cargo 档）；本窗口批量回归已执行。
- evidence: ls-remote 输出（auto-lang master/v0.6-dev/v0.5.0、auto-os main、auto-notes main、auto-down master）；merge-base 断言；auto-os `cat-file` 父提交；[合流收据](../reports/v06-main-convergence-20261012/merge-receipt-20261013.md)；[final refs](../reports/v06-main-convergence-20261012/final-refs-20261013.json)；`.last-batch-regression.json`；KNOWN-DEBT REG-5 条目。
- next: /auto-plan:merge 沉淀（SD-00 none，簿记级）→ 归档。

### 2026-10-15 — stage: merge / 沉淀收据 PLAN-775:r6

- prepared: 终审基线=已发布复审链 `3002c30…`→`d777ccbb…`→`45db8e25…`（rev6 复审基线，经 merge-base 重验为本地 `master b74f1b492…` 与 `origin/master 2a510751…` 祖先）；Spec delta=SD-00 none（冻结于 rev6，无 canonical spec 改动）；交付=2026-10-13 已发布远端汇流，非新代码着陆。
- landed: auto-lang 侧交付链已在 master（历史性着陆，ff 语义无 merge commit 需新建）；本次关账在 master 直接追加 docs/ledger 簿记提交（AGENTS.md master 簿记例外）。auto-os `main`（本地+远端）=汇流点 `853a551…`。
- ledger_refreshed: `.autoos/specs.json` 受护栏离线投影（无 store writer 可达——127.0.0.1 全端口探测无服务；与 P770/P777 同通道先例）。新增 `reviews/P775-R7`（终审 pass）、`reports/P775-M`（本收据）；designs 零条目=SD-00 书面说明所致，非欠填；version 18→19。字节级备份 `.autoos/specs.json.bak-775`，语义 diff 仅两节新增+version，回读校验通过。
- archived: `git mv docs/plans/775-v06-main-convergence.md docs/plans/archive/`，`status: archived`，`completion_kind: delivered`；plan/ledger 工件随本提交入默认分支（非 untracked 落盘）。
- cleaned: auto-lang 侧 PLAN-775 从未建 worktree/分支（`git worktree list` 仅 lang-778/lang-782 在册）；`.wt/lang-775` 组已先行消失（处置见终审 P775-R7：无移除收据、用户 2026-10-15 知情授权、commit 层面无损）。无待清对象，wt-guard N/A。
- 批量回归到期判定：收据 `.last-batch-regression.json`（2026-10-14T11:30Z，窗口 766→774）已含 PLAN-775 合并工作并显式消费 775%5 触发；距今 <48h，归档时**不 due**（到期点 2026-10-16T11:30Z，届时由 /auto-plan:regress 主检出单实例执行）。
- 部署陈旧性检查：docs-only 关账，今日零代码着陆，无生产工件需重建（2026-10-13 着陆代码已被后续计划门禁+批量回归覆盖）。
- outcome: **pass**（delivery/ledger/archival/cleanup 全验证）。

## 待澄清事项

无。后续功能工作依各自 Plan 推进；异机本地 WIP 不在本机审计可证明范围内。

