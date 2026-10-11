---
plan_id: PLAN-764
plan_revision: 1
status: reviewed
feature_name: autoedit-comparison-screenshot
author: [Codex]
created_at: 2026-10-11
updated_at: 2026-10-11
supersedes_spec_components: [docs/specs/website/design/application-introductions.md, docs/specs/website/project.md, docs/specs/website/plans.md]
new_spec_components: []
touched_goals: []
affects: [website]
current_step: 3
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
- [x] T-01 原图/共享 metadata/双语 caption，AC-01/02。
- [x] T-02 指南/新 provenance/规范与模块索引，AC-03。
- [x] T-03 build/定向既有 e2e/hash/diff 核查并提交，AC-01/02/03。

## 8. 风险
新图 backend/build identity 未独立确认；按提供来源呈现。原图保留比例与放大。

## 9. 复审记录
stage: new | PLAN-764:r1 | pass | next: work。用户已明确授权；复审按工件重建，同会话限制如实记录。

## 10. 待澄清事项
无影响接入的用户决策项。

### Work r1
stage: work | PLAN-764:r1 | pass | implementation: 89071f738cb39ebd6c6cae344c3ab181b3af25e1 | T-01/02/03 complete | next: review。
原图/hash/共享 SHOT-03 元数据、指南、规范和模块索引均接入；build 230.63s；8/8 定向 e2e（1.1min）；网页桌面/手机截图目视通过；Category A 无 Cargo。证据 docs/reports/p764-autoedit-comparison-verification.md、p764-autoedit-comparison-catalog.json、p764-spec-delta.json 和 p764-comparison/。

### Review r1
stage: review | PLAN-764:r1 | outcome: pass | reviewed_commit: 5fa9caefef5933c2a3e8950dd831be1486a0ca5e | base_commit: d42c4d497 | dependency_revisions: 外仓不变/未依赖构建。
AC-01 pass：双语 SSR 和实际水合页面显示正确章节的新图；原 SHOT-01/02 保持，SHOT-03 不再待拍。
AC-02 pass：3840×2419、原图/发布复制品同 hash；8/8 定向 e2e 包含三图 Enter/Escape/原图/回焦、双语导航和五宽度双主题；1440/390 PNG 逐图目视通过。复用同一网站实现、配置与依赖下的该次 gate，不无依据重跑。
AC-03 pass：来源单独登记、双语 caption 日期准确、指南旧待拍状态修正、SD-01/模块 project/plans 回写一致。frozen delta sha256 5152881f0d6d9b0fd49752fce8084493968d2b25410477294baa5f95fd5b15fa，三 target hash 逐一匹配。
遗漏/延后/workaround 扫描：无新增；既存 P757 产品问题不据新截图宣称关闭。零 Rust 触面；diff --check 通过，无新 debug。新旧原图/历史诊断边界明确。touched_goals=[] 的素材-only 理由保留。
独立性限制：同实施会话复核，无授权外委；从已提交 diff、原始文件、frozen manifest、durable e2e 日志和页面图片重建结论。
next: merge。
