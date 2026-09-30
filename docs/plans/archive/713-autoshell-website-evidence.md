---
plan_id: PLAN-713
status: archived
feature_name: AutoShell website evidence and presentation
author: [Codex]
created_at: 2026-09-30
updated_at: 2026-09-30

plan_revision: 3
supersedes_spec_components: [docs/specs/website/project.md]
new_spec_components: []
touched_goals: []

affects: [website]
current_step: 6
total_steps: 6
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
| SD-01 | modify | docs/specs/website/project.md | Shared EN/ZH AutoShell page, preserve native overview captures and visible F1/F2/F3 forms, evidence/provenance, responsive screenshot/script interactions and shared native colored hero for product/v0.5 | Record enduring presentation contract | AC-01..06 |

## 6. 测试设计

VitePress/Vue compile; inspect ZH/EN desktop/mobile, light/dark, no document overflow, assets/links, script selection and screenshot zoom/Esc. Shared-style homepage smoke. Execute portable scripts using existing ash binary. Record environment/baseline issues separately.

## 7. 验收标准

- AC-01: Both locales match current observed behavior, valid examples, no obsolete Agent/F4 claims; source/content checks.
- AC-02: Shell/pipeline/F2/policy use cases navigable, actual screenshot evidence with provenance and accessible zoom; browser checks.
- AC-03: Retain visual language, no mobile overflow, light/dark readability and homepage shared-style consistency; browser/style checks.
- AC-04: Quick-start/sample links work and report contains concrete website recommendations; build/link/sample validation.

- AC-05: 展示用户最初提供的 ash-01 / ash-2 原生截图并说明内容；常见 F1/F2/F3 形式各有可见图示和真实来源说明，中英文同步。

- AC-06: AutoShell 与 v0.5 的 EN/ZH 主介绍使用原生终端彩色命令/表格截图；删除主图中的模拟输出与 F2/F5 标注，保留正文原图和快捷键说明；截图可放大且移动端不溢出。

## 8. 执行步骤

- [x] T-01 (AC-01/04): Create website/.vitepress/theme/components/AutoShellLanding.vue, replace EN/ZH wrappers; copy selected assets and portable samples.
- [x] T-02 (AC-02/03): Screenshot enlargement, script selection, section anchors, responsive presentation.
- [x] T-03 (AC-01/03): Correct EN/ZH home/apps entries and bounded landing.css; prepare SD-01 and review report.
- [x] T-04 (AC-01..04): Compile/browser/example validation, commit implementation and record results.

- [x] T-05 (AC-05): 恢复两个原生截图，新增 F1/F2/F3 图示区；保留现有实跑图和脚本，更新证据/规范并检查显示。

- [x] T-06 (AC-06): 共享原生彩色表格主图，同步 v0.5 双语展示，更新规范/证据并构建检查。

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

## revision 3 用户纠正（2026-09-30）

用户要求主介绍及 v0.5 宣传使用真实彩色命令/表格截图，移除主图内无意义 F2/F5 说明。复用已保留 ash-01 原图，以 CSS 视窗展示 ls 表格，放大仍显示完整原图。沿用授权和工作区；r1/r2 证据保留但不覆盖 AC-06。SD-01 追加真实彩色主图共享契约。stage: new | revision: 3 | outcome: pass | next: work.

## revision 3 执行完成

stage: work | plan_id: PLAN-713 | plan_revision: 3 | outcome: pass | code_commit: e728fa3f8995968f268e5c5a2654ff9eef272229 | task_ids: T-06 | evidence: docs/reports/autoshell-native-hero-2026-09-30.md plus four captures | blockers: none for delivery | next: review.

VitePress exit 0, 180.93s; EN/ZH product/v05, 390px/default desktop, light/dark, full-image zoom/Esc verified. Worktree clean. Historical r2 record revision restored to 2 after a broad frontmatter replacement; semantic history unchanged.

## revision 3 复审（当前会话，产物重建）

stage: review | plan_id: PLAN-713 | plan_revision: 3 | outcome: pass
reviewed_commit: e728fa3f8995968f268e5c5a2654ff9eef272229
base_commit: 712778972

- AC-06 / T-06: pass. Committed shared AutoShellPreview references the unchanged native ash-01 source; EvidenceImage's optional viewport preserves pixels and opens full original. Both locales and both showcase locations use that component. Fake hero output/F2-F5 line/tags removed, no body figure removed. Native color/table visible in four committed browser captures.
- AC-01..05: pass. Shell/runtime/examples not changed; r1/r2 execution evidence reused for identical commands/dependencies. Shared EvidenceImage retains dialog/focus/keyboard paths; live zoom/Esc return verified. Originals/mode images loaded, layout and source inspected. EN/ZH default desktop and 390px, light/dark checked. No new console errors observed.
- Build gate: exit 0, 180.93s. Category A no Cargo suites/docs_gen. Existing highlighting/bundle warnings retained, no new build error. git diff base..reviewed_commit --check passes; worktree clean, no debug additions or unmet AC.
- Frozen SD-01: docs/specs/website/project.md blob 9eb3ca3990aa0a2cb5121cd681e8ef95867c3e0c; prior blob 975544b94c783ff35804dd47647fd1cd6936f429. Exact paragraph records shared native colored hero for product/v05 plus full-image enlargement. No new runtime requirement or global theme redesign.
- Dependencies: website lockfile unchanged from r1/r2; original native image unchanged; no new shell execution claimed. touched_goals remains [] (presentation correction, no separately registered Goal).
- Limitation: same-session reconstruction against committed artifacts, no independent agent claimed. Evidence: docs/reports/autoshell-native-hero-2026-09-30.md. No deferred acceptance item. next: linear landing; unavailable store-mediated ledger writer continues to block archival only.

## 合入收据 PLAN-713:r3

stage: merge | outcome: blocked (post-landing ledger/archival only)

- prepared: current r3 review pass and frozen SD-01; worktree clean. Default Windows bash resolved to WSL and could not address D:/; reran the repository guard with C:/Program Files/Git/bin/bash.exe, exit 0 / clean, no reparse points.
- landed: main 0662c2cff2f664893f2e3f3677230f428ed27ab5 → f515252ea4e996e75dee36206b1c6abb3eb22723 via --ff-only. Rebase e728fa3f8995968f268e5c5a2654ff9eef272229 → f515252ea4e996e75dee36206b1c6abb3eb22723; range-diff equal (=), no implementation or Spec conflict. Browser/build evidence remains applicable.
- integrity: main and retained preview worktree have identical shared-preview blob 710d3b22cd2a2a5c6f9523e541130b309f7a24d9 and canonical Spec blob 9eb3ca3990aa0a2cb5121cd681e8ef95867c3e0c. Original screenshot unchanged. Other session's Plan 712 edits and untracked images/files preserved.
- delivery complete: both product locales and both v0.5 promotional sections share the native colored ls table capture. Existing body screenshots/shortcut explanations remain. Production deployment not performed.
- ledger_refreshed / archived / cleaned: pending same store-mediated writer limitation as r1/r2. Current available-tool inventory contains no read_specs/write_spec/update_spec; no direct ledger write. auto-plan-merge requires verified store-mediated publication before archival, so Plan stays reviewed and worktree retained for preview.

## 合入收据 PLAN-713:r3 — 收尾完成（2026-09-30 续会话，merge completion_kind: delivered）

stage: merge | outcome: pass | delivery_commit: 1cd9765df | ledger_commit: 3d4c7b009

- prepared: r3 review pass（reviewed_commit e728fa3f8，工件重建裁定在案）+ frozen SD-01（docs/specs/website/project.md blob 9eb3ca399…，主检出复核未变）；worktree clean；前置核实=落地祖先链（f515252ea ∈ master）、canonical spec 零漂移、批量回归不到期。
- landed: r1/r2/r3 落地链全 ff-only 零合并提交（17a28f12a→7b1a31f88、41cf4d434→50b020d7e、e728fa3f8→f515252ea，各轮 range-diff 全等，收据见上）；本收尾 delivery=1cd9765df——plan-713-dev 先 ff 至 master tip 0680ec694（纯快进零重写，无 range-diff 负担）后 §5 module 回写（website/plans.md 713 行 + INDEX 再生 no-op〔EOL 噪声，add 规范化后与 HEAD 全等〕；projection-only 检定=实现/依赖零变化）再 ff-only 落地。
- ledger_refreshed: 3d4c7b009——designs 段 P713-1（SD-01 现行知识投影，docsha:55e0cbc54360a1ae=sha256 前 16 位，方法经 714 已知值反推验证）+reviews 段 P713-2（r3 合入收尾收据）。store 写者不可用（8080 不在线、本会话无 spec 工具），循 711/714 在仓先例外科插入：committed 形（indent1+LF，HEAD blob 1b0d14549，designs 117→118 / reviews 179→180）与 worktree 形（indent2+CRLF，reviews 180→181）双向回读断言全过；字节 roundtrip 守卫=逐插入点逆删除后与原文件逐字节全等；既有条目语义零扰动；P712-1（计划 712 会话在途 WIP）字节保全、未裹挟提交。
- archived: docs/plans/archive/713-autoshell-website-evidence.md，status: archived（本提交）。
- cleaned: pending（紧随本提交执行 wt-guard → worktree/branch/组目录移除，guard 结果与移除证据补记于下）。
- batch regression: 不到期——713%5=3 非整除；.last-batch-regression.json last_covered_plan_id=712 @2026-09-30T08:04:49Z 当日新鲜（<48h），无触发条件。
- artifacts/deployment: VitePress dist 为实现 worktree 预览态；生产 bundle 与线上站点未随本计划重建/发布（r1 起明确范围外，Category A 纯网站文档，无 Rust/release 产物消费面，PLAN-092 陈旧产物教训不适用）。
- 后续：站点级导航/搜索/排版重构建议由 PLAN-715（website-ui-refresh，drafting）承接；本计划工作区预留的 preview/main-assets 备份随 worktree 移除一并失效（其内容均已入库或属他会话资产）。
