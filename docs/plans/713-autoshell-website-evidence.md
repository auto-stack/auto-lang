---
plan_id: PLAN-713
status: reviewed
feature_name: AutoShell website evidence and presentation
author: [Codex]
created_at: 2026-09-30
updated_at: 2026-09-30

plan_revision: 1
supersedes_spec_components: [docs/specs/website/project.md]
new_spec_components: []
touched_goals: []

affects: [website]
current_step: 4
total_steps: 4
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
| SD-01 | modify | docs/specs/website/project.md | Add shared EN/ZH AutoShell page, evidence/provenance, responsive screenshot/script interactions | Record enduring presentation contract | AC-01..04 |

## 6. 测试设计

VitePress/Vue compile; inspect ZH/EN desktop/mobile, light/dark, no document overflow, assets/links, script selection and screenshot zoom/Esc. Shared-style homepage smoke. Execute portable scripts using existing ash binary. Record environment/baseline issues separately.

## 7. 验收标准

- AC-01: Both locales match current observed behavior, valid examples, no obsolete Agent/F4 claims; source/content checks.
- AC-02: Shell/pipeline/F2/policy use cases navigable, actual screenshot evidence with provenance and accessible zoom; browser checks.
- AC-03: Retain visual language, no mobile overflow, light/dark readability and homepage shared-style consistency; browser/style checks.
- AC-04: Quick-start/sample links work and report contains concrete website recommendations; build/link/sample validation.

## 8. 执行步骤

- [x] T-01 (AC-01/04): Create website/.vitepress/theme/components/AutoShellLanding.vue, replace EN/ZH wrappers; copy selected assets and portable samples.
- [x] T-02 (AC-02/03): Screenshot enlargement, script selection, section anchors, responsive presentation.
- [x] T-03 (AC-01/03): Correct EN/ZH home/apps entries and bounded landing.css; prepare SD-01 and review report.
- [x] T-04 (AC-01..04): Compile/browser/example validation, commit implementation and record results.

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
