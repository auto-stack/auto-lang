---
plan_id: PLAN-713
status: reviewed
feature_name: AutoShell website evidence and presentation
author: [Codex]
created_at: 2026-09-30
updated_at: 2026-09-30

plan_revision: 2
supersedes_spec_components: [docs/specs/website/project.md]
new_spec_components: []
touched_goals: []

affects: [website]
current_step: 5
total_steps: 5
---

# [PLAN-713] autoshell-website-evidence

## 0. 变更摘要

Correct the AutoShell page against current code and observed 2026-09-29 runs; organize both locales around practical use and show the existing screenshots with enlargement and F2 examples. Preserve the website's shared visual style.

## 1. 目标

Accurate, useful, responsive product documentation. Improve bounded shared landing styles and record broader website suggestions. No shell/runtime changes or production deployment.

## 2. 架构方案

Shared AutoShellLanding Vue component for EN/ZH pages; existing FeatureCard/ShowcaseSection and theme tokens. Selected copied evidence assets and portable samples. Native dialog image enlargement; keyboard-accessible script selection and page anchors. Implementation worktree: D:/autostack/.wt/lang-713/auto-lang, plan-713-dev.

## 3. 技术栈

VitePress 1.6 / Vue 3 / shared CSS. Scoped browser/build/link/sample checks only; AGENTS Category A excludes Cargo suites and docs_gen.

## 4. 需求分析与背景调查

- User authorized reviewing ../auto-lang/website/zh/apps/autoshell, reorganizing information, website UI suggestions, and direct optimization when necessary. Existing authorization covers this bounded implementation.
- Main base 4243345d715afe838b46c1bbef7e2c76e2dcaa47. Specs: docs/specs/overview.md and docs/specs/website/project.md. Current page is from 4a20d3df6; no matching active plan found.
- Evidence: D:/autostack/auto-shell/ash-cli-demo-2026-09-29/{README.md,shell-analysis.md,f2-script-run-session.txt,shell-runs.json}; auto-shell/ash/ash/src/main.rs and frontend/editor_overlay/mod.rs.
- Errors: nonexistent 79-agent-tool CLI, retired F4, unsupported filter example, unverified cross-platform equality. Copied screenshots are unused; CTA leads away before explaining usage.
- Main has another session's untracked screenshot assets and unrelated files. Preserve originals; copy selected assets into the dedicated worktree.
- Managed worktree tool is bound to this chat's auto-shell repo and has no target-repository parameter; use auto-lang's Git worktree convention. No junction/symlink.

## 5. 详细设计

Order: daily shell, field pipelines, F2 multiline scripts, automation/policies, quick start. Correct shortcuts and examples; label runtime version/date/platform and screenshot provenance. Separate current CLI startup flags from persistent session commands. Keep homepage/apps entries consistent in both languages.

Shared styling: narrow hero-title selector, readable code metadata/output, visible keyboard focus, reduced-motion respect, mobile grids. Broader navbar/search redesign remains a report recommendation.

### 规范增量

| delta_id | operation | target | before/after rule | rationale | acceptance |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/project.md | Shared EN/ZH AutoShell page, preserve native overview captures and visible F1/F2/F3 forms, evidence/provenance, responsive screenshot/script interactions | Record enduring presentation contract | AC-01..04 |

## 6. 测试设计

VitePress/Vue compile; inspect ZH/EN desktop/mobile, light/dark, no document overflow, assets/links, script selection and screenshot zoom/Esc. Shared-style homepage smoke. Execute portable scripts using existing ash binary. Record environment/baseline issues separately.

## 7. 验收标准

- AC-01: Both locales match current observed behavior, valid examples, no obsolete Agent/F4 claims; source/content checks.
- AC-02: Shell/pipeline/F2/policy use cases navigable, actual screenshot evidence with provenance and accessible zoom; browser checks.
- AC-03: Retain visual language, no mobile overflow, light/dark readability and homepage shared-style consistency; browser/style checks.
- AC-04: Quick-start/sample links work and report contains concrete website recommendations; build/link/sample validation.

- AC-05: 展示用户最初提供的 ash-01 / ash-2 原生截图并说明内容；常见 F1/F2/F3 形式各有可见图示和真实来源说明，中英文同步。

## 8. 执行步骤

- [x] T-01 (AC-01/04): Create website/.vitepress/theme/components/AutoShellLanding.vue, replace EN/ZH wrappers; copy selected assets and portable samples.
- [x] T-02 (AC-02/03): Screenshot enlargement, script selection, section anchors, responsive presentation.
- [x] T-03 (AC-01/03): Correct EN/ZH home/apps entries and bounded landing.css; prepare SD-01 and review report.
- [x] T-04 (AC-01..04): Compile/browser/example validation, commit implementation and record results.

- [x] T-05 (AC-05): 恢复两个原生截图，新增 F1/F2/F3 图示区；保留现有实跑图和脚本，更新证据/规范并检查显示。

## 9. 复审记录

stage: new | revision: 1 | outcome: pass | next: work. Existing direct-optimization authorization covers scope. Final review in this session will reconstruct evidence; no delegated reviewer claimed.

## 10. 待澄清事项

None blocking. Preserve unrelated work. Production publishing is outside the request.

## 执行完成记录

Implementation commit: 17a28f12a82b9ebdfd0f6912f7914977f2aff72a. Branch plan-713-dev, worktree D:/autostack/.wt/lang-713/auto-lang. Category A: no Cargo suites/docs_gen. Final VitePress build exit 0, 161.46s. Existing Auto-highlighter fallback and >500kB bundle warnings recorded in report; no new runtime errors observed. Three portable scripts matched expected outputs. EN/ZH desktop/mobile, theme, keyboard tabs, clipboard and native-dialog checks are recorded in docs/reports/autoshell-website-review-2026-09-30.md and its three screenshots. Selected screenshot/sample asset existence and imports pass; git diff --check pass. JSON sample explicitly exempted in website/.gitignore to survive fresh checkout.

Untracked preview copies and build log moved reversibly to D:/autostack/.wt/lang-713/preview-content-backup/verification-extra; original main-checkout assets and other sessions' changes preserved. No semantic scope change.

## 复审结论（当前会话，基于产物重建）

stage: review | plan_id: PLAN-713 | plan_revision: 1 | outcome: pass
reviewed_commit: 17a28f12a82b9ebdfd0f6912f7914977f2aff72a
base_commit: 8d917c469931f809bffa1d163f5765c29703f9a1

- dependency_revisions: auto-shell HEAD fdc0f7839003d0f74c6a6e744982ebc67c778449 plus observed F5 working-tree overlay SHA256 B3289102BA1C9484DCC2304C5A558CA317FBF29B359FEE7242BD8A425A6921C9; ash.exe SHA256 51344FB1CE5C8CEEC2B49F1BA5CC2EC551DBC49DCB0C697D95476ACCD42AB7BF; main.rs SHA256 49AAE6D62FEAAFE9D4BE2F6364B3CC0D6157D06628D6F01D229EF15B0DB01B5A. Website package-lock Git blob 2d6f81791d5291c3c07a1131d4af59de6147f9de.
- spec_inputs: docs/specs/website/project.md base Git blob 490c7e523402f6922af6367d9eb86fa28493cad6.
- frozen SD-01: reviewed commit's docs/specs/website/project.md Git blob 5997aeaed592477b888b68e4d915223eca69ab90; file SHA256 2AD2A99FE97B7F9202A40883DE889206AE3AC8E31F4C7225FDD26317EF9A844E. Delta is exactly the shared page/evidence/interactions paragraph at line 12; baseline/after are recoverable with git diff base reviewed_commit.
- AC-01 / T-01,T-03: pass. Read committed bilingual data, CLI routes and shortcut implementation; obsolete 79/four-mode/filter claims removed. Source/observed-vs-unverified boundaries explicit.
- AC-02 / T-02: pass. Browser tab selection and key navigation, clipboard contents, images loaded, native dialog close focus and Esc return verified. Figure source is explicitly PTY reconstruction.
- AC-03 / T-02,T-03: pass. 1440/390/360 layouts and both themes checked, scrollWidth below viewport. Homepage hero/card sizes preserved. Shared variables/components and scoped accent rules inspected.
- AC-04 / T-01,T-04: pass. All 5 screenshot assets and 4 sample files now tracked, wrapper imports resolve, three actual scripts match output. VitePress build exit 0, 161.46s. Recommendations/report/screenshots committed.
- Health: git diff base..reviewed_commit --check passes; worktree status clean. Existing site-wide Auto highlighting fallback and chunk-size warnings are documented. No new console errors or debug print additions. No Rust implementation touched.
- Evidence reuse: final amendment only removed blank EOF lines; git diff ade8b2fb..17a28f12 --ignore-blank-lines is empty. Final build/browser/sample evidence remains applicable to unchanged executable source and dependencies.
- Findings resolved before verdict: dark scoped selector corrected; explicit close-button focus added; JSON sample exempted from ignore and committed; EOF whitespace normalized. No deferred item inside AC scope.
- touched_goals: [] because this bounded website documentation/presentation change does not advance a separately registered GOAL. Broader site redesign is report-only advice, outside this Plan.
- Limitation: same implementation session; artifact/code/runtime evidence reconstructed, no independent agent claimed.
- next: merge after preserving concurrent main-checkout work; production deployment remains outside scope.

## 合入收据 PLAN-713:r1

stage: merge | outcome: blocked (post-landing consolidation only)

- prepared: review pass for revision 1 and frozen SD-01. Worktree clean and wt-guard clean (no reparse points).
- landed: main 1cd928407 → 7b1a31f884e2c450e0b8110e7f8def45587d1868 via git merge --ff-only plan-713-dev. Rebase mapping 17a28f12a82b9ebdfd0f6912f7914977f2aff72a → 7b1a31f884e2c450e0b8110e7f8def45587d1868; git range-diff old_base..old_reviewed master_before_landing..delivery shows one equal (=) patch. Only concurrent Plan bookkeeping was inherited; implementation/Spec/dependencies unchanged.
- integration smoke: main page, dialog, users.json and canonical Spec hashes equal delivery branch; worktree browser reload displays the page, no console errors. Website build/runtime verification remains on the implementation worktree per AGENTS. Main's unrelated dirty Plans 711/712 and untracked files preserved.
- asset preservation: five preexisting untracked main images matched reviewed bytes and were moved to D:/autostack/.wt/lang-713/main-assets-backup before fast-forward; their original paths now hold the identical tracked delivery assets. Other images untouched.
- ledger_refreshed: pending. No read_specs/write_spec/update_spec tools are registered in this context and no configured store endpoint was identified. No direct .autoos/specs.json write attempted.
- archived/cleaned: pending ledger verification. Plan remains reviewed; worktree/branch retained. No new approval required, no UI acceptance item remains open. Follow-up should finish only store-mediated projection, repository module/index bookkeeping if required, archival and guarded cleanup.
- production artifacts: not deployed. Fresh VitePress dist exists in retained worktree; main deployment bundle and hosted website were not rebuilt/published by this task. No runtime/backend changes.

## revision 2 用户纠正（2026-09-30）

用户指出筛选导致图示覆盖不足，明确要求 F1/F2/F3 与最初两图都展示。沿用同一未归档 Plan/工作区；r1 交付及证据保留，但不覆盖新 AC-05。原生资源实际文件名为 ash-01.png 和 ash-2.png（用户称 ash-02），保留文件名不改写原图。追加常见形式图示区和原生界面介绍，未发起 AI 模型请求的截图只描述入口。规范增量 SD-01 扩展为保留原生界面总览和常见快捷键形式；无需重新申请已获授权的页面补充。

## revision 2 复审与执行完成

stage: review | plan_id: PLAN-713 | plan_revision: 2 | outcome: pass
reviewed_commit: 41cf4d434db4d436ae3965bd84a475dad6999b19
base_commit: f8395b1eb0fc5dcb79689f6694eabdc6082cc60e

- AC-05 / T-05: pass. Commit includes byte-preserved ash-01.png/ash-2.png plus three separately visible F1/F2/F3 figures and both locales' explanations. Actual F1, F2/F5, F3 entry/Esc interactions performed; no AI question sent. PTY reconstructions explicitly labeled; conversation history not published. New report docs/reports/autoshell-modes-2026-09-30.md and two browser captures bind evidence.
- AC-01..04: pass. Original scripts, policies and shared components unchanged (r1 evidence reused with that reason). New page source/Spec inspected; whole-site VitePress build exit 0, 75.79s; EN/ZH, 1440/390/360, light/dark and zoom/Esc checks passed. Existing script labels still load summary and JSON captures. No new console errors; git diff --check and clean worktree verified.
- Environment: build initially lacked already-known v05 preview assets; restored physical copies, final build passed, moved copies back outside worktree. No unrelated assets committed. Category A: no Cargo/tests/docs_gen.
- Frozen SD-01 delta: docs/specs/website/project.md Git blob 975544b94c783ff35804dd47647fd1cd6936f429 at reviewed_commit, baseline 5997aeaed592477b888b68e4d915223eca69ab90. Adds native/F-key screenshot retention/provenance to existing paragraph, no runtime or global navigation redesign. Dependency versions/hash baseline unchanged from r1.
- Same-session artifact-based review; no independent agent claimed. No remaining AC item or new debt. next: fast-forward landing, preserve other sessions' main work; existing ledger-publication blocker still governs archival.

## 合入收据 PLAN-713:r2

- stage: merge | outcome: blocked (post-landing ledger/archival only).
- landed: main 6bcd6fd06 → 50b020d7ee26993aa574e4694f68c8e393a0b0ef via --ff-only; mapping 41cf4d434 → 50b020d7e, range-diff shows equal (=) implementation patch. wt-guard clean, no reparse points.
- Main source, Spec and both original image hashes equal delivery. Main's original untracked ash-01/ash-2 matched bytes, preserved in D:/autostack/.wt/lang-713/main-assets-backup before landing; original paths now contain identical tracked assets. Other sessions' modifications preserved. Browser/build evidence valid after identical rebase.
- Current user request complete: originals and F1/F2/F3 visible with explanations, no previously delivered screenshot removed. Both locales synchronized.
- Ledger/archive/cleanup remain pending for the same unavailable store-mediated writer as r1; no direct JSON write, no production deployment. Worktree remains for preview and receipts.
