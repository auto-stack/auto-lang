---
plan_id: PLAN-722
status: reviewed
feature_name: Website Apps 总览、四应用介绍与系统 Demo 展示方案
author: [Codex]
created_at: 2026-10-01
updated_at: 2026-10-01
plan_revision: 1
current_step: 5
total_steps: 5
supersedes_spec_components: [docs/specs/website/project.md]
new_spec_components: [docs/specs/website/design/application-introductions.md, docs/specs/website/design/application-demos.md]
touched_goals: []
affects: [website]
---

# [PLAN-722] Apps 介绍与系统 Demo 展示方案

## 0. 变更摘要

统一四主应用 AutoEdit / AutoShell / AutoMusk / JadeEdit 与系统应用两组；撰写 EN/ZH 阅读专题，截图留空；制作无后台依赖的系统示例介绍入口及后续真实图/选择性交互设计。保留已有 AutoShell 实跑为独立 guide，旧 AutoDown/AutoUI 资料不删除。

## 1. 目标

G-01 总览说明四产品用途分工并直达专题；G-02 专题依据当前源码/规范，介绍操作、能力、运行条件与边界；G-03 设计约28系统 Demo 的展示，不全量制作 mock。
不做：六栏导航、AI 双视角、应用源码/服务、拍摄或替换图片、v0.5重排、部署公网、Cargo测试。

## 2. 架构方案

VitePress 默认阅读布局承载详细 Markdown；SSR Vue 卡片承载总览。主介绍不插入任何图片，截图槽用 HTML 注释注明素材意图，无破图及面向访客 TODO。Shell 原生 ls 图及脚本交互保留在新 /apps/autoshell/guide/；发布页 AutoShellPreview 不变。系统 Demo 新介绍不嵌入运行画廊/iframe。

## 3. 技术栈

VitePress 1.6 / Vue 3 / Playwright，沿用全站 tokens；D:/autostack/.wt/lang-722/auto-lang，plan-722-dev。

## 4. 需求分析与背景调查

授权：用户本轮明确要求先写 Apps 主介绍、四专题（截图留空），再设计28 Demo 展示；已授权上述实施，无重复确认/预算条件。Musk 等另会话示例准备，Jade 等 Milestone；不新增图片。

依据：docs/specs/overview.md、website/project.md、design/ui-presentation.md；base 7153a4a6683b39d02bf1317c1113e460e5dfb85e，tracked clean，保留 unrelated .tmp-vm*。
现有 apps.md/zh/apps.md 仍列 Shell/Musk/AutoDown/AutoUI；Shell 实跑组件必须保留。
只读兄弟仓：auto-edit f9a16e5add5ebb4ecfe6ea9b72f75a00a2ffca9e（docs/specs/00-overview.md、modules/diff-view.md、strategy/002-north-star-v2.md）；jade-edit 2b25cefe6c78d42ff519481f24b79ebe53231ba3（docs/README.md、ARCHITECTURE.md、specs/editor-safety.md、draft-recovery.md，032 executing 不冒领）；auto-musk 6e6b6cc847639805c796e0e5a7cc6091b96f4319（README.md，非100%Auto/无人值守/重启恢复）；auto-os 73d02b50f535543bd635c6f6f49dad6835efff45（apps.manifest、icons/mapping.json、gallery registry/AppViewport）。
当前 gallery registry 35项含基础组件；网站旧包与现势ID不一致（012-stopwatch/025-dashboard vs clock/sys-monitor），图标预留槽不等于应用数。28是候选展示集，不承诺固定28已集成/在线可玩。

## 5. 详细设计

/apps 双语：四产品2×2简介卡片、系统分类入口、OS/UI/AI关联、原资料链接。
新 autoedit/jadeedit 双语路由，重写 automusk/autoshell 介绍为默认阅读页，每页一H1/目录/正文/相关链接；Shell guide 新路由完整保留原使用组件。新 /apps/demos/ 双语分类介绍，截图注释槽不显示空框、不调业务API。
系统展示：分类静态目录→真实截图与操作说明→逐项验证的无后台体验。依赖服务的应用给本地运行条件；fixture仅作为少量后续片，标注演示数据/重置与持久化边界。28候选表列源码与依赖核查，不将loadable当可用证明。

### 规范增量

| delta_id | add/modify/retire | docs/specs target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | add | docs/specs/website/design/application-introductions.md | 无统一四产品契约→两组产品、双语阅读、当前状态、注释截图槽、Shell guide | 统一口径保留资料 | AC-01..04 |
| SD-02 | add | docs/specs/website/design/application-demos.md | 统一在线体验→静态分类优先、逐项验证与28候选/服务核查 | 无后台仍可读 | AC-05..06 |
| SD-03 | modify | docs/specs/website/project.md | Shell主介绍承载全部图→介绍链接guide，新增四应用契约 | 与交付一致 | AC-03..06 |

## 6. 测试设计

npm run build；apps-introduction.spec.ts 验证双语SSR/链接/无图无业务API/目录locale/guide功能/五宽度两主题；site-ui、os-ai-introduction、desktop-showcase、spa-routes触面回归。14路由×390/1440×两主题截图+人工检查；全几何10新介绍路由+2系统页×5宽度×两主题。Category A 不运行Cargo/docs_gen；端口独占，不终止其他会话服务。

## 7. 验收标准

- AC-01：双语总览恰列四产品，专题/旧资料链接有效，SSR与浏览器走查。
- AC-02：四产品双语覆盖定位/操作/现状/运行条件/关系，一H1、目录、locale正确；规范事实比对。
- AC-03：主介绍无img/iframe/破图/可见TODO，截图注释槽在源，v0.5图不改。
- AC-04：guide保留ls原图、脚本标签/复制/下载；介绍↔guide；原回归迁移后绿。
- AC-05：系统介绍无后台可读，28候选有真实源码/服务核查，loadable不冒充无后台。
- AC-06：设计明确目录/截图/交互/fixture职责、数量口径及逐项验收门，无mock实现及全可玩宣称。
- AC-07：五宽度双主题双语无横溢出，截图人工绿、构建/触面测试绿；来源和风险入报告。

## 8. 执行步骤

- [x] T-01（AC-02,05,06）：核对四产品规范/源码和gallery清单，事实及28候选入设计/报告。
- [x] T-02（AC-01,03）：新 AppsOverview.vue/数据、双语apps.md，四卡片+系统入口，无图/API。
- [x] T-03（AC-02..04）：四专题双语、Shell guide互链、截图注释槽。
- [x] T-04（AC-05,06）：双语系统介绍与设计、SD-01..03准备。
- [x] T-05（AC-01..07）：构建/触面测试/几何/截图/链接/事实复核，提交实现与报告，独立复审。

## 9. 复审记录

stage: new | PLAN-722:r1 | outcome: pass | next: work
范围与本轮明确授权一致，直接在worktree执行。复审将说明同会话的独立性限制并重建证据。

work-started: D:/autostack/.wt/lang-722/auto-lang (plan-722-dev), base 992d730de4ad71a1579f2ce5d50ebfd9e7b0975c. T-01..04 implemented: bilingual overview, eight topics, two retained guides, two demo pages, 28 verified candidate paths, prepared SD-01..03. T-05 verification pending.

work-completed: T-01..05 complete. Website baseline a4456ad520027d382ade7b4116f77ed248e1b1da; build exit 0; scoped Playwright 88/88; visual 140 geometry checks and 56 captures complete. Durable evidence: docs/reports/p722-apps-introduction.md and companion directory. Category A, zero Cargo. stage: work | PLAN-722:r1 | outcome: pass | next: review.

stage: review | plan_id: PLAN-722 | plan_revision: 1 | outcome: pass
reviewed_commit: bb8bcefd389589f0522fc0b23951843bb975a8ab
base_commit: 992d730de4ad71a1579f2ce5d50ebfd9e7b0975c
website_tested_commit: a4456ad520027d382ade7b4116f77ed248e1b1da (reviewed HEAD differs only by committed evidence; website diff empty)
dependency_revisions: docs/reports/p722-source-baseline.json exact 10 source hashes/revisions and 28 candidate paths; investigation revisions preserved above, later snapshot wins current facts.
spec_inputs: docs/specs/website/design/application-introductions.md sha256=4b4e27141ce6b2941ea897f108edb562a54f693c84c461d6e9edd926a05ca957; docs/specs/website/design/application-demos.md sha256=f9ce55be1ac6e75cc357f76306e9801865e43bced5387caf25cf3a9b25754104; docs/specs/website/project.md sha256=f94bd932e21c2d3247d54994b0208dbd6a74744dd05f787d3220fc8fa5232aa0
acceptance_results: AC-01..07 pass, mapped to T-01..05 in docs/reports/p722-apps-introduction.md.
verification: build exit 0; 88 Playwright passed (5.2m); 140 route/width/theme checks; 56 PNG captures reviewed through eight contact sheets plus full-size ZH Apps; clean committed worktree, git diff --check and frozen SD hash checks pass. Runtime evidence reused because exact website code, lockfile and test configuration unchanged from tested baseline; report-only descendant commit. Category A, no Cargo/docs_gen.
independence: 同实施会话；重新读取真实差异、测试断言、规范与来源，并逐AC重建结论，不宣称独立会话/独立agent。
findings: P722-D1 low, existing gallery ID drift, outside agreed scope; no missing required sub-item or unapproved deferral/workaround. Main KNOWN-DEBT-AND-RISKS.md recorded. SD-01..03 pass and frozen in durable evidence.
spec-impact: supersedes/new paths finalized in frontmatter; touched_goals=[] because website current canonical Spec has prose goals, no stable goal IDs; do not fabricate IDs from this Plan's local G-01..03.
evidence: docs/reports/p722-apps-introduction.md, p722-apps-introduction/{manifest.json,playwright-results.txt,reviewed-spec-delta.json,56 captures,8 contact sheets}, p722-source-baseline.json.
next: merge when landing is requested; keep worktree/preview for user inspection. 本轮写作及设计工作完成，不执行master合入/ledger/归档，不将计划已归档冒领。

## 10. 待澄清事项

无阻塞用户问题。精确发布集合/无后台体验覆盖为设计内后续逐项验证项，本轮不承诺固定28项全可用。

