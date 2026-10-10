---
plan_id: PLAN-757
status: executing
feature_name: AutoEdit 自仓工作区实拍
author: [Codex]
created_at: 2026-10-10
updated_at: 2026-10-10

# /auto-plan:review 结束时填写：
supersedes_spec_components: [docs/specs/website/design/application-introductions.md, docs/specs/website/design/ui-presentation.md]
new_spec_components: []
touched_goals: []             # 引用 docs/specs/goals.md 的 GOAL-NNN

affects: [website]
plan_revision: 1
current_step: 0
total_steps: 4
---

# [PLAN-757] autoedit-website-real-captures

## 0. 变更摘要
从定型 AutoEdit 源码正式编译桌面程序，以其自身仓库为 workspace，拍摄工作台、跨文件搜索、双文件比较，替换 PLAN-756 的 SHOT-01..03。

## 1. 目标
交付三张真实原图、构建/动作记录、网站双语引用和更新后的逐图拍摄指南。其他九个空位保留。不修改 AutoEdit 产品代码、不部署网站、不承诺整套 v0.5 候选验收。

## 2. 架构方案
独立组 D:/autostack/.wt/lang-757/{auto-lang,auto-down,auto-edit}；lang 分支 plan-757-dev，其余依赖 detached。正式 portable --regen 构建，禁止旧产物回退。运行 workspace 指向 D:/autostack/auto-edit，保存会话及比较版本在隔离目录。

## 3. 技术栈
Auto CLI → a2r/Iced 原生 release、现有 portable 脚本、Windows、真实窗口操作/PNG、VitePress EvidenceImage。

## 4. 需求分析与背景调查
授权：2026-10-10 用户明确允许自行编译、打开 AutoEdit 截图，并优先用它自身仓库作为 workspace；继承网站文案/空位任务的实施与合入范围，无另需确认事项。
基面：auto-lang 690e23bd4；auto-edit f9ae2a64361f74ad8565fc5df5c4a9aeb5fc10f2。PATH auto 版本含 dirty，需记录构建/二进制身份，不能当作干净工具链来源。
依据：docs/specs/website/project.md、design/application-introductions.md、design/ui-presentation.md、design/demo-capture-catalog.md、docs/reports/website-v05-capture-guide.md；auto-edit/tools/portable/build_portable.py、specs/auto-edit/pac.at、src/back/fsys.at。
主检出 blueprints 删除与 PLAN-755 WIP 非本任务，禁止纳入/重置；独立依赖副本保证构建不消费这些删除。

## 5. 详细设计
发布原图 website/public/v05/candidate/autoedit-{workspace,search,diff}.png；首页、v0.5、应用总览及专题复用工作台图，专题引用搜索/比较图。记录尺寸/hash、源版本、工具链、构建日志、exe hash、OS、后端、会话、动作、日期。比较使用实际源码 Git 两个版本或隔离副本，不编辑主仓制造修改。

### 规范增量
| ID | 动作 | 目标 | 前后规则 | 理由 | 验收 |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/website/design/application-introductions.md | AutoEdit 空位→实拍与来源；其余保留 | 定型产品实拍 | AC-01..04 |
| SD-02 | modify | docs/specs/website/design/ui-presentation.md | AutoEdit 主图共享、功能图真实来源 | 完成对应占位 | AC-03..04 |

## 6. 测试设计
Category A 不运行 Cargo tests/docs_gen；编译是用户要求的步骤。检查 PNG/hash、源码对应、网站构建及双语/移动/深浅引用、放大和原图。

## 7. 验收标准
- AC-01：官方入口从定型源码完成原生构建，记录 commit、命令、exe 完整 hash，无旧产物替代。
- AC-02：三图来自真实 AutoEdit 自仓 workspace 与源码；搜索来自实际文件；比较显示真实两个版本及差异，保存动作证据。
- AC-03：发布完整原始 PNG；双语首页/v0.5/应用总览/专题引用与实际后端/日期一致；其他九槽存在。
- AC-04：更新指南与规范，网站构建/资源/放大/移动主题通过，复审记录遗漏及债务。

## 8. 执行步骤
- [ ] T-01：独立组、官方构建与来源记录（AC-01）。
- [ ] T-02：工作台、搜索、比较原生实拍与动作证据（AC-02）。
- [ ] T-03：双语三槽与 AutoEdit 总览替图、指南/规范更新（AC-03）。
- [ ] T-04：验证、提交、独立复审、合入及按门禁归档（AC-04）。

## 9. 复审记录
stage: new | revision: 1 | outcome: pass | next: work。用户已有授权，无新增产品实现。

## 10. 待澄清事项
无需要用户决策的项。构建或展示阻塞时保存失败证据，继续独立工作，不冒充成功截图。
