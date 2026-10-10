---
plan_id: PLAN-756
status: reviewed
feature_name: website-v05-copy-and-capture-slots
author: [agent]
created_at: 2026-10-10
updated_at: 2026-10-10
plan_revision: 1
current_step: 4
total_steps: 4
supersedes_spec_components: [docs/specs/website/design/ui-presentation.md, docs/specs/website/design/application-introductions.md, docs/specs/website/project.md]
new_spec_components: []
touched_goals: []
affects: [website]
---

# [PLAN-756] Website / v0.5 文案同步与截图待拍清单

## 0. 变更摘要
修正双语首页与 v0.5 专题的产品、发布状态、在线体验和文字入口；未准备好的应用截图留明确文字空位，交付逐图拍摄合同。

## 1. 目标
中英文语义一致；四主应用与应用总览一致；不将滚动里程碑写成冻结发行包，不把介绍数量写成全应用在线运行覆盖。中文条目不整句误加粗。已有桌面与 Shell 证据保留。仅文字/入口与必要截图槽，不做目录布局、压图、实际拍摄或部署。

## 2. 架构方案
复用 ReleaseLanding 与作者数据；文字型截图槽不渲染 img。共用 localeHref 处理共享 SPA。截图清单放 docs/reports，空位绑定清单 ID。

## 3. 技术栈
VitePress/Vue/TypeScript/Markdown；Category A，零 Rust 改动，不跑 Cargo/docs_gen。

## 4. 需求分析与背景调查
依据 docs/specs/overview.md、website/project.md、website/design/{ui-presentation,application-introductions}.md、docs/language/overview.cn.md、v05-release-closeout.md 和网站源码。基础 master a424dc18b0273521c3d065311ea8c3ee92ed007d。main 有其他会话 blueprints 删除和临时文件，禁止包含。
授权：用户已明确批准“先修复文本，截图留下空档，最后列出每个截图应该怎么去截”。旧规范禁止空位，本轮用户指令覆盖：允许有说明的文字待拍槽，不放假图/破图。同范围不再请求授权。

## 5. 详细设计
首页/发布页统一 AutoEdit、AutoShell、AutoMusk、JadeEdit。Musk/Jade 待拍；AutoEdit 新增展示处待拍，已批准专题/总览原图不移除。保留 AutoDown/Jade Garden 资料出口。快速开始直达语言概览运行与构建；增加 AutoOS、Playground 近端出口。理念按钮写为阅读详情。修正中文分隔符。写清准备状态、后端依赖、历史统计时点和长期路线边界。
收口清单同步 738 archived，不勾选应用/候选验收。

### 规范增量
| ID | 操作 | 目标 | before/after | 理由 | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/design/ui-presentation.md | 无空图位 → 本轮允许有 ID/说明的文字待拍槽；既有有效原图保留 | 用户指令 | AC-03/04 |
| SD-02 | modify | docs/specs/website/design/application-introductions.md | 首页/发布页统一四主应用，Musk/Jade 文字待拍槽 | 内容一致性 | AC-01/03 |

## 6. 测试设计
worktree npm run build；适用 Playwright/浏览器路由与文字检查；360/390/768/1024/1440 双语首页/发布页几何无新溢出；git diff --check。截图未拍不是完成证据；后端服务部署不在本轮。

## 7. 验收标准
- AC-01：双语首页/发布页四主应用、Musk 实现口径、里程碑/未冻结、28 项介绍/后端依赖一致；历史计数保留时点；实际渲染复核。
- AC-02：中文 Playground 保持 locale；共享画廊清单无 zh 且非 404；快速开始/AutoOS/Playground 有真实近端出口；中文条目仅名称加粗、无悬空破折号。
- AC-03：待拍处有 ID，无假图/破图/旧 Musk 或 Jade Garden 冒充新应用；桌面/Shell 和已批准 AutoEdit 原图保留。
- AC-04：逐图清单含页面位置、fixture、动作、构图、运行环境/后端、文件名、commit/tag；冻结依赖不假填；收口清单同步 738。
- AC-05：构建、实际行为、双语五宽度和补丁卫生通过；零 Rust 测试；完成内容提交。

## 8. 执行步骤
- [✅] T-01：双语首页/release 数据/ReleaseLanding/HomeDemo/learning-navigation/prepare-content 修文案与入口（AC-01/02）。
- [✅] T-02：截图文字槽接首页/发布页与必要专题；新建 docs/reports/website-v05-capture-guide.md（AC-03/04）。
- [✅] T-03：worktree 准备两处 Spec 增量；main 修正发布收口簿记（AC-04）。
- [✅] T-04：worktree 构建/行为/几何验证/提交，复核 AC（AC-05）。

执行：用户授权后于独立 worktree 开始；T-01/02/03 已完成，T-04 已完成。源码提交 e516408212f35893e63438ae20df41adda8daf2a；worktree clean。

## 9. 复审记录
stage: new | revision: 1 | outcome: pass | authorization: 用户本轮明确批准 | next: work。
同会话依据实际 diff/产物重建评审证据，不声称外部独立审查。

stage: work | plan_id: PLAN-756 | plan_revision: 1 | outcome: pass | code_commit: e516408212f35893e63438ae20df41adda8daf2a | task_ids: T-01..04 | evidence: npm run build 156.77s；78项首轮77通过/1测试URL断言错误，修正后v05-copy 5/5通过；截图ID/清单12/12匹配；git diff --check通过 | blockers: none | next: review。

stage: review | plan_id: PLAN-756 | plan_revision: 1 | outcome: pass | reviewed_commit: e516408212f35893e63438ae20df41adda8daf2a | base_commit: d6e4819ba686298238237ae6f473da77b22478bf | dependency_revisions/spec_inputs/acceptance_results: [review](reports/756-website-copy-review.md) | findings: P756-D1/D2 nonblocking/outside scope | frozen_delta: evidence/756/spec-delta.patch | next: merge。

## 10. 待澄清事项
无阻塞。拍摄、候选冻结、部署不属本轮。手机目录布局/图片负载留后续 UI 工作，不声称已修复。


## 11. 合入与规范同步收据

stage: merge | key: PLAN-756:r1 | outcome: blocked (ledger only) | prepared: 2 reviewed design rules + faithful project/plans summary; spec-index regenerated with no content diff | landed: 0cc62c92380d9381e59788762609f5b9172b4a1d on master via --ff-only | ledger_refreshed: pending, no spec writer tool available; GET http://127.0.0.1:8080/api/specs?workspace=auto-lang refused connection | archived: pending per skill | cleaned: pending, worktree retained.

Rebase mapping: e516408212f35893e63438ae20df41adda8daf2a → c61d182bd18a604dae7dfe077393961be782120e; a8edba7962f48747fc30f24229c1a11f27f639de → 0cc62c92380d9381e59788762609f5b9172b4a1d. git range-diff showed both commits equal (=); no source, dependency or reviewed-rule conflict. Main canonical files match the worktree; frozen consolidation patch SHA256 79ff99388ea6cde3c602d07da20dfdf9d3d5e6aeba9ceec2a4dbb687180df516.

Canonical paths: docs/specs/website/design/ui-presentation.md, docs/specs/website/design/application-introductions.md, docs/specs/website/project.md; module navigation docs/specs/website/plans.md. SD-01/02 unchanged; project/plans lines only derive the same delivered rules and provenance. Pending projection: reuse designs items by canonical file and append PLAN-756 review/report source references through the SpecsStore writer; resolve actual item IDs on successful store read. No direct .autoos/specs.json edit or lifecycle bypass occurred.

Integration smoke: main/worktree capture source, guide and canonical Spec bytes equal; source IDs match 12 guide rows; shared localeHref remains correct; git diff --check passed. Production observation: website/.vitepress/dist/index.html on main last written 2026-10-10 17:12:01 before landing; not rebuilt/deployed by this task. Verified bundle is in plan worktree. Rust/backend binaries untouched and Category A prohibits Cargo/docs_gen. No claim of public deployment, frozen artifacts, screenshot completion or W-02 acceptance.

Before landing guard: wt-guard clean (zero reparse point); worktree Git clean. Current main foreign blueprints deletions and Plan 755 WIP preserved. No cleanup/archive until store-mediated projection readback succeeds. Per auto-plan-merge/SKILL.md: “If no store-mediated writer is reachable, record blocked; keep the Plan active.”
