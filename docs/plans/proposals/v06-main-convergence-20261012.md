---
status: drafting
feature_name: v0.6 在途收口与生态主线汇合
plan_revision: 1
author: [Codex]
created_at: 2026-10-12
updated_at: 2026-10-12
current_step: 0
total_steps: 11
supersedes_spec_components:
  - docs/specs/overview.md
new_spec_components: []
touched_goals: []
---

# 执行合同预稿：v0.6 在途收口与生态主线汇合

尚未分配正式 plan_id：主检出现在是有 WIP 的 v0.6-dev，767–769 与远端 master 同号不同义。已批准执行；正式取号前先同步主线并检查两线联合编号集合，再在安全的 master 检出运行 `scripts/new-plan.sh`。当前两线中 v0.6 的 770/771 也已占用，因此保留 master 的 767–769，迁移 v0.6 的 767/768/769 到 772/773/774，汇合 Plan 使用 775；旧 ID 通过别名表追溯。本文仍是合同预稿，正式执行以新生成的编号计划为准。

## 0. 变更摘要

完成已开展的 v0.6 工作、补齐未推送成果与 v0.5 源增量，普通合并至各仓发布后的默认主线，恢复 main/master 为唯一日常集成目标。详见[盘点报告](../../reports/v06-main-convergence-20261012/README.md)。

## 1. 目标

保留发布与共享开发历史、v0.5 tag、所有需要的本地成果；在实际修复/独立验收完成后发布新的 v0.6 组合。37 个相关 GitHub 仓逐项有明确处置；没有开发增量的仓可快进或无需代码合并。

非目标：重命名所有 master 为 main、改写/squash 已发布历史、自动删除旧恢复引用、实现未开工 769–771 或整个 roadmap。

## 2. 架构方案

Git 汇合以最新发布主线为第一父线；跨仓遵守子仓先发布、父仓后固定 SHA。OS 源码→gitlink 的跨仓映射保留 v0.5 增量。现行 Plan 身份唯一，历史证据保持字节与版本归属。具体候选代码仅在独立工作树修复；已有在途 Plan 在唯一权威树完成后再进入集成候选。

## 3. 技术栈

Git 本地 commit graph/merge-tree/worktree、GitHub refs/compare、仓内 Cargo 分档、AC PE gates、既有应用双端验证和固定 SHA 子模块。无新架构技术选择。

## 4. 需求分析与背景调查

用户本轮授权范围：所有 auto-lang 相关仓库、先收口未完成/未推送 v0.6 工作，再保留 v0.5 历史汇合主线。未要求新 agent、模型、预算、分支重命名或 force push。执行前确认来自 AGENTS 的 L1 流程。

规划依据：docs/specs/overview.md、docs/specs/README.md、docs/roadmap-v0.6.md、docs/design/strategy/v0.6-offline-development-strategy.md、各仓 AGENTS/应用 Plan。调查 heads 与远端证据已固定在报告与附件；正式执行重新固定新 SHA，不能使用历史调查头宣称新代码通过。

已确认 Reader 用户决策保留三项 required 目标，先核新框架已提供能力，缺的另立前置 Plan。其余具体缺口见报告；不以 pending 草案或缺证据的 execution_done 文本代替实现验收。

## 5. 详细设计

按报告的八阶段执行；正式分配后在 `D:/autostack/.wt/lang-775/{auto-lang,auto-down,auto-os,...}` 组建权威集成树，auto-lang Plan 工作分支使用仓规约要求的 `plan-775-dev`，其他仓采用各自工作流的专属分支。候选代码基于最新远端默认主线，并将本地 master 的唯一文档提交纳入候选。

正式 Plan 文件/簿记留默认检出；各原 Plan/应用保持唯一写入位置。编号冲突迁移只变更现行登记/引用，保留原 branch/ID/path/SHA 别名及冻结证据。候选必须包含旧发布头、接受的所有源提交；任何源变化使相关 review/gate 过期。

### 规范增量

| delta_id | add/modify/retire | target | before/after rule | rationale | AC |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/overview.md | 离线旧主线/在途状态→实际 v0.6 组合、最新模块路径与归属 | 消除过期现状 | AC-04/08 |
| SD-02 | modify（auto-os 仓） | docs/specs/apps/independent-repositories.md | 旧源快照→已吸收 v0.5 增量且可重现的组合 | 原目录替换不丢功能 | AC-05/07 |
| SD-03 | modify（路径由正式 T-01 逐 Plan 固定） | 742/744/768、各应用 Plan 指向的 docs/specs/... | 各 Plan 冻结 delta→经其独立 review 的实现规则 | 不在汇合时批量弱化承诺 | AC-02/04 |

SD-03 的具体文档由每个在途合同和 review 最终固定；本迁移不得擅自将未验收功能写入 canonical，也不得绕过 required 门。roadmap/策略/AGENTS/编号别名属操作与状态维护，不能冒称编译器架构新能力。

## 6. 测试设计

详细入口和 oracle 见报告“验证与验收边界”。高风险不变量为：原历史/tag 不变、所有接受源可追溯且为新主线祖先、v0.5 产品源增量不因 gitlink 转换消失、required 未通过时阻止发布父组合、gitlink 从远端可获取。

代表控制：两个同号不同义 Plan 均正确定位；Tetris/Solitaire 改源与产品既有功能同时存在；本地独有 master 文档与 Musk 两提交可追溯；Blog 修复后的等待期/失败保存/恢复/迁移真实负例；Reader 三个原 required 场景。远端发生并发更新或子仓 push 失败时，候选不能绕过新基线/父组合门。

work 负责实现/实际运行，review 用同树独立上下文逐条重建证据并绑定 revision/HEAD，merge 只负责已验证候选的发布与沉淀。未指定额外 agent/model；禁止以 merge 阶段替代实现正确性验收。

## 7. 验收标准

| AC | owner_stage | 可观察结果/验证 |
|---|---|---|
| AC-01 | work | 37 仓远端与可见本地/工作树有完整 SHA、独有提交/WIP 和处置；恢复引用/必要证据可重建 |
| AC-02 | work | 742/744/768、Notes/Blog/Launcher/Weather/Reader 等已开展工作按原合同关闭 required 实现与验证；未开工草案保持草案；阻塞不伪装 pass |
| AC-03 | work | 发布主线、所有接受开发源/本地独有提交均为候选祖先；原 v0.5 tags 与历史 SHA 不变；无 force/rebase/squash |
| AC-04 | work | 代码/路径/状态/Spec 冲突语义解决；Plan ID 现行唯一，旧身份和冻结 evidence 可追溯；联合代码/原型/应用门通过 |
| AC-05 | work | Tetris/Solitaire v0.5 源增量进入产品仓，Syslog 更新保留；其它产品源判定逐项留证；OS manifest/启动和 fixed-SHA 子模块可重现 |
| AC-06 | review | 独立 AC01–05/SD/遗漏/延期/workaround 审计绑定最终各仓 HEAD，零未处置硬失败；明确 flake/coverage 边界 |
| AC-07 | merge | 每仓普通 push 后远端包含候选源；父 gitlink 仅指向可远端获取的提交；各仓完成/失败/重试收据准确 |
| AC-08 | merge | 默认主线成为实际开发检出/集成目标，操作规则/Spec/ledger/index/正式 Plan archive 完整；guard clean 后才清理临时树，原恢复成果保留 |

## 8. 执行步骤

| Task | owner_stage | 依赖 | 操作与成果 | AC |
|---|---|---|---|---|
| T-01 [ ] | work | 本方案已批准 | 保全 refs/WIP/原证据；在 master 更新到发布主线后，按两线联合编号检查迁移号 772–774，并由 `scripts/new-plan.sh` 正式创建 Plan 775；固定37仓清单与 SD 具体路径 | 01/04 |
| T-02 [ ] | work | T-01 | 基于最新各仓发布主线建立兄弟集成树；固定原工作树和唯一写入者；重新预演源/基线 | 01/03 |
| T-03 [ ] | work | T-02 | 742/744 原合同反例、剩余VM门和状态收口；768 final review/原计划合入；保留已完成 AC/ABI | 02/04 |
| T-04 [ ] | work | T-02/03 | 核查新版框架的Reader三项前置；缺失的正式设计/Plan交付，再完成 Reader required 终验；不降范围 | 02/04 |
| T-05 [ ] | work | T-03；Reader依T-04 | Notes/Blog/Launcher/Weather等完成各自 work/review/规范沉淀与源成果 push；检查Explorer/既有应用固定点 | 02/05 |
| T-06 [ ] | work | T-02 | 吸收本地master文档/Musk两提交/有效恢复roadmap；按报告解决语言三代码冲突、路径整理、Plan身份与知识记录；快进型仓处理 | 03/04 |
| T-07 [ ] | work | T-02/05 | Tetris/Solitaire源delta迁产品仓并原门验证、普通push；其他源逐项对账；父OS语义合并/固定gitlinks | 05 |
| T-08 [ ] | work | T-03..07 | 最终组合重新构建/原型/语言分档/应用双端/fresh checkout；复审输入与source SHA冻结，生成proposed SD与全部证据 | 02..05 |
| T-09 [ ] | review | T-08，写入暂停 | 同树独立AC/SD/反例/历史/源迁移/required/债务审计；绑定revision+各repo HEAD，失败回work | 06 |
| T-10 [ ] | merge | T-09 pass | 重新核远端、普通ff/push，先子后父；验远端祖先/tag/fixed-SHA可获取；逐仓发表准确收据 | 07 |
| T-11 [ ] | merge | T-10 | 回主线、现行workflow/Spec/ledger/index更新、正式Plan归档、回归到期判定与guard清理 | 08 |

## 9. 复审记录

2026-10-12 stage:new-preflight；revision2；outcome:user approved execution. 已完成37仓分支盘点、核心heads同步、21 source-sync祖先核验、lang/OS/Musk三仓对象级冲突预演。因联合编号核对发现 v0.6 770/771 已占用，正式方案调整为迁移 767/768/769→772/773/774、汇合 Plan 775；未宣称 work/review/merge pass。

next：用户确认后按T-01安全正式登记，再进入 auto-plan-work。没有遗漏合并动作被隐式批准或执行，没有创建 agent/新聊天。

## 10. 待澄清事项

本次执行合同已由用户确认。保留现有 main/master 名称、Reader 原 required 目标和三个未开工草案的状态。当前机器之外的未push成果不可由本审计证明不存在，后续遇到新源需加入清单并重新验证，不能报告未验证范围已完成。
