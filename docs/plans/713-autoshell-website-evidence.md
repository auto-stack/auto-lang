---
plan_id: PLAN-713
status: executing
feature_name: AutoShell website evidence and presentation
author: [Codex]
created_at: 2026-09-30
updated_at: 2026-09-30

plan_revision: 1
supersedes_spec_components: [docs/specs/website/project.md]
new_spec_components: []
touched_goals: []

affects: [website]
current_step: 0
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

- [ ] T-01 (AC-01/04): Create website/.vitepress/theme/components/AutoShellLanding.vue, replace EN/ZH wrappers; copy selected assets and portable samples.
- [ ] T-02 (AC-02/03): Screenshot enlargement, script selection, section anchors, responsive presentation.
- [ ] T-03 (AC-01/03): Correct EN/ZH home/apps entries and bounded landing.css; prepare SD-01 and review report.
- [ ] T-04 (AC-01..04): Compile/browser/example validation, commit implementation and record results.

## 9. 复审记录

stage: new | revision: 1 | outcome: pass | next: work. Existing direct-optimization authorization covers scope. Final review in this session will reconstruct evidence; no delegated reviewer claimed.

## 10. 待澄清事项

None blocking. Preserve unrelated work. Production publishing is outside the request.
