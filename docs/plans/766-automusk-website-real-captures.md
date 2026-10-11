---
plan_id: PLAN-766
status: executing
feature_name: automusk-website-real-captures
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12
plan_revision: 1
supersedes_spec_components: [docs/specs/website/design/application-introductions.md, docs/specs/website/project.md]
new_spec_components: []
touched_goals: []
affects: [website]
current_step: 0
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
- [ ] T-01：专用 lang-766 工作树新增两图与来源目录，更新 capture-slots.ts / AppsOverview.vue / 双语专题 / capture-guide；关联 AC-01/02/03。
- [ ] T-02（依赖 T-01）：更新既有两套测试并构建、定向回归；准备 SD-01/02 与冻结规范增量、证据报告；关联全部 AC。
- [ ] T-03（依赖 T-02）：独立证据复核、提交、合入/规范索引/归档，按既有 deploy-website.ps1 部署策略备份切换并线上验证；关联 AC-03。

## 9. 复审记录
stage: new | PLAN-766 r1 | outcome: pass | next: work。现有用户授权覆盖完整 scope，无新增待决事项。

## 10. 待澄清事项
无；截图不提供运行身份，以 unknown 保留。
