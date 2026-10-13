---
plan_id: PLAN-775
status: executing
feature_name: v06-main-convergence
plan_revision: 5
author: [Codex]
created_at: 2026-10-13
updated_at: 2026-10-13
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
| auto-lang | `master` `e35f9e99b6711ff6f1990ea980729d1fa521327e` | merge `77495105284b4ba657c1bd1aa3b864d72aeddb17`：第一父线保留发布 `master`，第二父线为本地 v0.6-dev `2e9777b191f32683e8a3bbbe0b993675e38cce63`；原远端 v0.6-dev `94173fe2c175ef007a825a25050a2f04fb5ed98a` 未改写 |
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

其他仓的分支同步/无开发独有提交判定以初始调查表为准；T-08 已经通过 GitHub API 刷新全部 37 个相关仓的默认分支头，完整 SHA 清单见 [最终远端 refs](../reports/v06-main-convergence-20261012/final-refs-20261013.json)。AutoOS 候选的根提交已推送；在独立验证副本中确认 recursive 子模块无 SHA 不匹配，`verify-app-links.py` 通过（21 个新产品、24 个子模块，另核对 3 个既有应用）。

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
| AC-01 | merge | 既有提交与 tag 均未改写；AutoLang 的 `v0.5.0` tag 经追加 merge 成为 `master` 祖先，37 个相关仓默认分支头已刷新并留有收据。 |
| AC-02 | work | 在途功能 Plan 原状态、原 required 目标和独立工作树继续保留；本次不要求其先完成。 |
| AC-03 | merge | AutoLang 纠偏 merge `3002c30ffd618f768031048742bcc700a2aff363` 的第一父为 `v0.5.0` tag commit `e05e2d3d7832c7f1d7e29a09a371648b5a53f692`，第二父为原 `master` `9e54f738a07c3715e055ef2a763380e8ec31253e`；AutoOS 父顺序仍按原收据核验；其他开发更新可追溯，AutoMusk 两提交带 `-x` 来源记录。 |
| AC-04 | merge | Tetris/Klondike 的 v0.5 源增量已先推产品仓；AutoOS 指针指向已推提交；Notes 的 nested pin 指向已推 AutoDown 修复。 |
| AC-05 | review | 变更仓按范围检查通过；独立 AutoOS 检出 recursive 子模块无 SHA mismatch，`verify-app-links.py` 通过。 |
| AC-06 | review | 本阶段复核历史、冲突解决、来源、debt/未完成项和工作区边界；结论限定为 branch-convergence phase，不宣称整体 Plan 最终通过。 |
| AC-07 | merge | 默认 `main`/`master` 作为 v0.6 后续开发线；37 仓远端 refs 已记录。AutoLang 远端已包含纠偏提交，但本地 checkout 仍在旧头 `9e54f73…`，待网络恢复后快进；AutoOS dirty 候选仍保留，T-08 closeout 未结束。 |

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
| T-08 | merge | 刷新所有未变化仓远端 refs；只快进无 WIP 的本地默认检出；在干净 AutoOS checkout 运行 `verify-app-links.py`/recursive submodule 验证；完成最终收据和通过 wt-guard 的安全清理。 | [ ] 37 仓远端 refs、AutoOS fresh-clone recursive/app-link 验证已完成；AutoLang 远端纠偏后本地 checkout 尚待 fetch/快进。AutoOS 集成候选根状态仍含 20 个 dirty 子模块、2 处 recursive gitlink mismatch；按保全要求留存，安全清理待处理。 |

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

## 待澄清事项

无。后续功能工作依各自 Plan 推进；异机本地 WIP 不在本机审计可证明范围内。
