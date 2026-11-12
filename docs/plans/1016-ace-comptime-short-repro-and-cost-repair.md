---
plan_id: PLAN-1016
status: executing
feature_name: ACE comptime short reproduction and measured cost repair
author: [Codex]
created_at: 2026-11-12
updated_at: 2026-11-12
plan_revision: 1
current_step: 1
total_steps: 5
supersedes_spec_components: [docs/specs/auto-cte/frozen-cache.md]
new_spec_components: []
touched_goals: []
---

# [PLAN-1016] ACE comptime 短复现与测量驱动优化

## 0. 变更摘要
定位1201秒测试，用短实验区分编译/AV执行/发布/GC成本，依据证据优化，不循环重跑1800秒全组。

## 1. 目标
保持闭包执行、冻结准入、预算与GC合同；新增几十秒内可复现同一成本路径的探针，对主耗时进行可测优化。原32MiB/512修订/两轮64MiB水位、正式样本与门限全部保留。

## 2. 架构方案
先测量后选择等价优化；不换执行lane，不删除预算/残留检查，不用缩小用例替代正式验收。排除不必要的共享存储协议重构。

## 3. 技术栈
Rust，existing comptime conformance harness/Obs，ManagedStore，Cargo/nextest，Python有界进程运行。

## 4. 需求分析与背景调查
用户2026-11-12已明确授权定位、建计划、独立worktree、短复现、分阶段测量后优化，无需再次请求相同范围确认。
来源：AGENTS.md；docs/specs/overview.md、auto-cte/{project,frozen-cache}.md、auto-test-case/ace-test-policy.md；comptime/tests/cases/cte_resources_tests.rs、comptime/src/{driver,av_driver}.rs、artifacts/src/store/mod.rs、auto-cache/src/storage.rs。
直接证据：D:/autostack/plan-artifacts/plan-1013/t07/capture-log.json failed[group=ace-comptime].stderr：resources_real_watermark_gc_physical_delete_and_converge SLOW>900s，PASS 1201.821s (52/149)，随后父进程deadline exit124 wall_ms=1800009.09。因此1201秒为通过的GC单测时间，1800秒为整个组的父截止，非该单测nextest超时。
观察：GC测试两次填充64MiB，每批50 calls/300 helpers；Closure lane每call重新前端编译；put_inner每次扫描CAS临时残留并space遍历。主导占比待T-02测量，不预判。
实现树：D:/autostack/.wt/lang-1016/auto-lang，branch codex/plan-1016-dev；主树仅计划簿记，当前其他文档WIP不纳入提交。
材料：D:/autostack/plan-artifacts/plan-1016/，owner PLAN-1016，in-flight，探针日志/临时存储/测量与构建缓存；merge时按规清除。

## 5. 详细设计
新增短探针复用原gc_deck及实际CteDriver，先保持300 helpers仅缩减call/batch，再独立改变helpers与存量；另测相同真实store发布路径。固定输入和断言，记录阶段耗时及发布数量/字节，计量编译准备与运行分开。短探针每次执行60秒上限，超出不盲重试，先减工作量或读阶段记录。优化方案由测量决定；源码hash、engine/lane、values、hit/miss、无效输入拒绝、预算和生命周期不变。

### 规范增量
| delta_id | add/modify/retire | target | before/after rule | rationale | acceptance IDs |
|---|---|---|---|---|---|
| SD-01 | modify | docs/specs/auto-cte/frozen-cache.md | 补充成本探针入口及实测等价优化；原资源/性能门不变 | 可快速定位回归 | AC-02..04 |

## 6. 测试设计
| case | invariant | entry/oracle | boundary |
|---|---|---|---|
| V-01 | 同一真实编译/落盘路径 | 原gc_deck的固定短批次，逐值闭式结果、ct_execute/cold miss、实际存储增长、分阶段ms | 不冒充64MiB正式验收 |
| V-02 | 优化语义等价 | 优化前后同输入，值/manifest/engine不变，warm hit无执行；相关既有负控 | 不改变lane/预算 |
| V-03 | 改善主导成本 | 同形状基线与修后阶段记录；次数/结构计数辅助，不以严格毫秒断言抗噪 | 冷编译单列 |
完整职责组候选ace-comptime；若真实触及共享store则ace-artifacts及auto-cache与必要消费者另核。work只运行当前finding的定向反例和检查。冻结完整组/必要原压力验收为后续review/merge所有，不因调试反复跑。用户明确禁止反复1800秒全组；原规模验证仅在短探针已证优化后安排一次，未执行明确pending。
实现与内部检查本agent顺序执行；独立复审按仓流程另行进行，不创建未授权子agent。

## 7. 验收标准
- [x] AC-01 (work) 日志定位1201秒完整测试身份，区分单测结果/组截止。
- [ ] AC-02 (work) 保留同一路径的短探针，运行<=60秒，测编译/执行/发布/GC阶段，逐值oracle。
- [ ] AC-03 (work) 有优化前后同输入证据，主导成本下降且原语义、预算和资源合同不变；相关定向检查通过。
- [ ] AC-04 (work) 短探针永久可运行，原重测规模/断言不减；交付提交与未跑门清晰记录。
- [ ] AC-05 (review/merge) 独立审查及冻结职责并集、原规模证据，规范沉淀后清理；未执行不充PASS。

## 8. 执行步骤
- [x] T-01 (work, AC-01) 追溯capture-log，记录测试/父截止身份。
- [ ] T-02 (work, AC-02, depends T-01) 独立树新增短探针与阶段观测，采基线，冻结瓶颈选择。
- [ ] T-03 (work, AC-03, depends T-02) 依据测量等价优化及必要正负控，不扩大合同。
- [ ] T-04 (work, AC-04, depends T-03) 定向验证、格式/告警检查、提交实现，规范增量及交接。
- [ ] T-05 (review/merge, AC-05, depends T-04) 独立复审、原规模/冻结并集，沉淀收尾。

## 9. 复审记录
2026-11-12 stage:new outcome:pass next:work；用户范围授权覆盖本计划；T-01完成。没有运行长测，没有修改其他会话实现。

## 10. 待澄清事项
无范围决策阻塞；主耗时尚待测量，完整压力门待优化后安排。
