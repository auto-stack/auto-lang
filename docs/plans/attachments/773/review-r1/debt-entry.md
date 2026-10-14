### Plan 773（2026-10-14，独立 final review R1）

历史实现已合入，当前独立验收 **needs_fix / executing 1/10**。以下是合同内阻断，
不是允许延期的完成项；正常 pilot 24/24 与协议测试 57/57 不抵消反例。
完整源码锚点、AC/task、原始输入/结果、修复方向见
[独立复审报告](attachments/773/review-r1/REVIEW.md)；修复树为 lang-773/plan-773-dev。

| id | 级别 | 缺口 |
|---|---|---|
| F-773-R1-01 | high / blocking | a2r 成功发射可满足 Error oracle，真实 CLI PASS/0 |
| F-773-R1-02 | high / blocking | 已接受的 adapter 观测和错误出口流断言静默忽略，IMPOSSIBLE 仍 PASS |
| F-773-R1-03 | high / blocking | snapshot FAIL 错误 BLOCKED actual 交接；成功路径消费者未消费实际 bytes |
| F-773-R1-04 | high / blocking | selector 全部执行后改 NOT_RUN；坏 adapter selector 未报合同 ERROR |
| F-773-R1-05 | high / blocking | out/json-out 覆盖 fixture/输入；输出域/父 reparse 链校验遗漏 |
| F-773-R1-06 | high / blocking | oracle sidecar bytes 漂移身份不变；compiler/parser/config/旧 report 核查未接线 |
| F-773-R1-07 | high / blocking | max_expanded_checks_per_run 仅定义，无加载/执行门；字节准入读取边界遗漏 |
| F-773-R1-08 | high / blocking | I/O 被归语言 run；嵌套 MultipleErrors 未完整递归，阶段/基础设施未独立观测 |
| F-773-R1-09 | medium / blocking | lint 缺 expected/ref resolve 仍 exit 0 |
| F-773-R1-10 | medium / blocking | pilot 聚合入口未在计划规定模块；tv filter 不包含 test_cases_tests |
| F-773-R1-11 | medium / blocking | legacy hash/context 对账、proposal 旧新 hash/diff/共享消费者/失败传播未齐 |

修复保留原 21 case/24 check 和既有 fence/strict schema 绿项；在同一树提供闭合表、
永久控制和新提交，再跑 §6 完整 final 门禁。未写 canonical Spec/ledger，未归档或清理。

