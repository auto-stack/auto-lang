---
plan_id: PLAN-766
status: archived
feature_name: automusk-website-real-captures
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12
plan_revision: 2
supersedes_spec_components: [docs/specs/website/design/application-introductions.md, docs/specs/website/project.md]
new_spec_components: []
touched_goals: []
affects: [website]
current_step: 3
total_steps: 3
---

# [PLAN-766] AutoMusk 用户实拍接入网站

## 0. 变更摘要
接入用户指定的会话主图与计划页面原图，更新双语网站并部署。

## 1. 目标
主图进入首页、v0.5、应用总览和 AutoMusk 专题；计划图进入专题 SHOT-05。保留完整 PNG 和原图放大操作。

## 2. 架构方案
复用 CAPTURE_IMAGES、ScreenshotSlot、EvidenceImage；共享 SHOT-04 主图。SHOT-06 保持待拍。

## 3. 技术栈
现有 VitePress/Vue/Playwright；资产与文档 Category A，禁止 Cargo 与 docs_gen。

## 4. 需求分析与背景调查
授权：用户明确指定 PixPin_2026-10-11_17-13-23.png 为主图、17-13-32.png 为计划页面，此前明确要求最新版网站部署。范围仅 auto-lang 网站素材/相关规范及静态站部署。
来源：docs/specs/overview.md、docs/specs/website/project.md、design/application-introductions.md（读取 2026-10-12）；capture-slots.ts、AppsOverview.vue、双语 automusk/index.md、apps-introduction.spec.ts、v05-copy.spec.ts。
两原图经视觉检查均 2560×1600 深色；同一密码生成器示例，计划图显示 PLAN-001 与 0/5，不能证明执行/复审通过。精确 backend/build 未独立确认。
取号时 .next-id 落后于已有 PLAN-765；保留既有 765，移除本次误分配空骨架，再由 new-plan.sh 独占分配 766、推进至 767。

## 5. 详细设计
新增 website/public/v05/candidate/musk-workspace.png、musk-plan.png；源字节不变。共享 metadata 增 SHOT-04/05，SHOT-05 改为“计划页面”及真实场景。AppsOverview 增主图；双语介绍、拍摄清单、来源目录同步。

### 规范增量
| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/design/application-introductions.md | Musk 待拍 → SHOT-04/05 用户原图；SHOT-06 待拍，计划草稿不冒领复审 | 用户指定素材 | AC-01/02/03 |
| SD-02 | modify | docs/specs/website/project.md | 增两图来源与网站共享位置现状 | 当前模块知识 | AC-01/03 |

## 6. 测试设计
工作树 website 内 bun install --frozen-lockfile、npm run build（AUTO_BOOK_ROOT=D:/autostack/book）。现有 apps-introduction / v05-copy 更新期望并验证双语原图/键盘放大；五宽度双主题阅读回归。源/目标 SHA256 相等。部署后 HTTP 及线上两图验证。

## 7. 验收标准
- AC-01：双语首页/v0.5/总览/专题主图为 SHOT-04；专题 SHOT-05 为计划图；SSR/路由/浏览器加载正确。
- AC-02：两图保留 2560×1600 原始字节，hash 一致；Enter 放大、原图链接、Esc 返回焦点可用；五宽度双主题无横向溢出。
- AC-03：来源目录如实记录提供时间、文件/hash、未知运行身份；计划页面不标已完成复审，SHOT-06 待拍；build/相关回归通过，静态部署可验证并有回滚备份。

## 8. 执行步骤
- [x] T-01：专用 lang-766 工作树新增两图与来源目录，更新 capture-slots.ts / AppsOverview.vue / 双语专题 / capture-guide；关联 AC-01/02/03。
- [x] T-02（依赖 T-01）：更新既有两套测试并构建、定向回归；准备 SD-01/02 与冻结规范增量、证据报告；关联全部 AC。
- [x] T-03（依赖 T-02）：按既有 deploy-website.ps1 部署策略备份切换并线上验证；交付后独立复审与合入归档单独记录；关联 AC-03。

T-01 证据：lang-766 工作树新增两原图，源/目标字节相等，PNG 2560×1600；SHOT-04 主图共享与 SHOT-05 场景改写完成。

T-02 证据：npm run build 152.67s EXIT0；apps-introduction + v05-copy 23/23（双语/五宽度/双主题），8 项截图 landing/overflow=0；提交 38ef90ade0ba145e4ca03d221d39c9484ec00977，base c7fc18d56。durable report=docs/reports/p766-website-verification.json；冻结 SD=docs/reports/p766-spec-delta.json。

T-03 证据：release 20261012-38ef90ade（备份同 release）；IP HTTP 8 路由/domain HTTPS 2 路由精确文件校验通过，线上6/6。首次校验收到默认虚拟主机跳转并自动回滚，指定 Host 后切换成功。

## 9. 复审记录
stage: new | PLAN-766 r1 | outcome: pass | next: work。现有用户授权覆盖完整 scope，无新增待决事项。

r2：T-03 明确为部署验证，复审/合入归档是其后生命周期操作，避免在复审前要求先合入的循环；目标/验收/授权范围不变。

stage: work | PLAN-766 r2 | outcome: pass | code_commit: 730c33f1d7f5f68631abdff67085a5104d6e1116（38ef90ade 等价 rebase） | T-01/02/03 完成 | next: review。

stage: review | PLAN-766 r2 | outcome: pass | reviewed_commit: 730c33f1d7f5f68631abdff67085a5104d6e1116 | base: c7fc18d56 | AC-01/02/03: pass | SD-01/02 冻结 hash 匹配 | findings: 无遗漏/未批准延后/workaround/新增债务 | evidence: docs/reports/p766-review.md / p766-website-verification.json | next: merge。同一实现会话证据重建，非独立上下文；逐项直接复核已提交差异、原图 hash 与运行结果。

status transition: executing → execution_done（work）→ reviewed（review）。delivery metadata descendant 41e47d73c 仅规范投影/报告/索引，无 website 实现变更。

## 10. 待澄清事项
无；截图不提供运行身份，以 unknown 保留。

## spec-sync 回写记录

PLAN-766:r2 completion_kind: delivered。
- prepared：复审 730c33f1d，38ef90ade → 730c33f1d 的 range-diff 完全等价；SD-01/02 冻结 hash 与目标一致。
- landed：master ff-only 到 41e47d73c5b2db183beaea9195eae91043e85b65；其 metadata descendant 无 website 实现变化。
- ledger_refreshed：D:/autostack/auto-lang/.autoos/specs.json；用户仓库兼容手工回退（docs/specs/README.md §4–6），仅 P766-1/P766-2/P766-R/P766-Report；原 840 对象/未知兼容字段保持，现 844。
- canonical：docs/specs/website/design/application-introductions.md、project.md；plans.md 添加交付行；spec-index.py 已再生（26 projects，内容未变）。
- archived：docs/plans/archive/766-automusk-website-real-captures.md，delivered。
- production：20261012-38ef90ade；线上6/6、IP/HTTPS 精确文件校验通过；备份 /home/visus/auto-website-backup-20261012-38ef90ade。
- batch due-check：共享收据 last_covered_plan_id=761，前序 PLAN-765 已由其会话落地。本任务 Category A 按仓库明确禁止 Cargo/docs_gen，仅目标验证；不在资产交付中重复启动前序 Rust 批量维护。
- cleanup：待守卫后移除 lang-766，仅本计划工作树/分支。
