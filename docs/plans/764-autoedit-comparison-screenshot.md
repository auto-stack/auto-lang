---
plan_id: PLAN-764
plan_revision: 1
status: executing
feature_name: autoedit-comparison-screenshot
author: [Codex]
created_at: 2026-10-11
updated_at: 2026-10-11
supersedes_spec_components: [docs/specs/website/design/application-introductions.md]
new_spec_components: []
touched_goals: []
affects: [website]
current_step: 0
total_steps: 3
---

# [PLAN-764] AutoEdit 文件对比原图接入

## 0. 变更摘要
用户明确授权把 PixPin_2026-10-11_16-23-31.png 放入对应页面，补齐 SHOT-03，中英共用完整原图。

## 1. 目标
AutoEdit 专题比较章节显示作者提供的深色实图，可放大及查看原图。

## 2. 架构方案
沿用 CAPTURE_IMAGES → ScreenshotSlot → EvidenceImage，无新组件，无产品源码变动。

## 3. 需求分析与背景调查
docs/specs/overview.md 定义官网模块；website/project.md 与 design/application-introductions.md 规定原图与键盘放大。PLAN-757 已归档，两张图已交付，SHOT-03 因旧布局留空；新图构图清晰，不冒用旧构建身份。
授权：2026-10-11 本轮用户明确请求，涵盖图片接入、双语说明、截图清单与必要验证，无需再次确认。
基面：6da3e6201b85e47934f9527a61b205b47be1278d。工作树 D:/autostack/.wt/lang-764/auto-lang，分支 plan-764-dev。依赖外仓只读。

## 4. 详细设计
原图复制到 website/public/v05/candidate/autoedit-comparison.png，3840×2419。新 provenance 记录原路径、hash、可见状态和提供日期；未知 backend/build commit 不推断。caption 标注 dark、作者提供日期及 before/after 场景。
修订当前指南 SHOT-03 待拍状态及过期“757 未合入”说法；历史 p757 证据不改。保留 VM 深色截图政策和 Rust 界面未验收边界。

### 规范增量
SD-01 modify docs/specs/website/design/application-introductions.md：旧诊断图保留，新提供合格图填入 SHOT-03，未知构建身份单独登记，映射 AC-01/02/03。module project.md 与 plans.md 同步现状。touched_goals=[]：素材补齐不改变产品功能。

## 5. 测试设计
Category A，禁止 cargo t/docs_gen。npm website build；已有 v05-copy 图片/原图/键盘放大检查覆盖第三图；apps-introduction 图片数量更新，运行 AutoEdit 双语及多宽度/主题检查。原图 SHA-256 对拍。

## 6. 验收标准
AC-01：双语 AutoEdit 正确章节显示 SHOT-03 新图，无待拍，旧图不覆盖。
AC-02：尺寸/hash 一致，原图链接及 Enter/Escape/回焦可用，桌面手机、深浅网页无溢出。
AC-03：来源/双语说明/指南/规范一致；不伪造 runtime/commit，不宣称旧应用缺陷全部修复。

## 7. 执行步骤
- [ ] T-01 原图/共享 metadata/双语 caption，AC-01/02。
- [ ] T-02 指南/新 provenance/规范与模块索引，AC-03。
- [ ] T-03 build/定向既有 e2e/hash/diff 核查并提交，AC-01/02/03。

## 8. 风险
新图 backend/build identity 未独立确认；按提供来源呈现。原图保留比例与放大。

## 9. 复审记录
stage: new | PLAN-764:r1 | pass | next: work。用户已明确授权；复审按工件重建，同会话限制如实记录。

## 10. 待澄清事项
无影响接入的用户决策项。
