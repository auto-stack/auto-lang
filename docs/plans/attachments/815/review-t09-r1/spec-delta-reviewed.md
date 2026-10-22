# PLAN-815 T09 R1 Spec逐项终核

review_scope=final，HEAD a67f114fda8abf52630126d4aa52eeacd6d3248a，Plan r3，outcome needs_fix(F-815-T09-01/P2)。冻结handoff/字段矩阵见本目录，当前七目标hash见baseline，终核七目标无漂移；无关graphics四文档并发变化另行对账。未发布canonical或live ledger。

| SD | target / before→after与终核 |
|---|---|
| 01 | docs/specs/auto-ui-tools/project.md。A窄子集→B完整生成/build/run/资源owner、兼容、单源与生命周期。方向与依赖闭包相容，但公开shell再生器不满足可消费生成物，F01闭合前不能以完整工具已验收沉淀；保留A事实。 |
| 02 | docs/specs/tool-protocol/project.md。保留v1同源codec/122帧深度/预算、扩B五操作/控制与日志/交互取消/终态/退出；原真实进程/Linux和边界控制成立，未变输入可复用。A文件已存在，应受管增补。 |
| 03 | docs/specs/auto-man/project.md。UI算法/资产退出到tools，旧门面/legacy结果不静默变更，非UI单算法归auto-api-gen。实际公开转发保持；F01修复不能靠删除原生成器来规避。 |
| 04 | docs/specs/project-model/project.md。809单源请求、静态边界不扩大、具名适配器和主声明快照独立；字段表已校正真实消费，与现有design/generation-request.md不冲突；本轮没有请求协议变更。 |
| 05 | docs/specs/auto-lang/ui/overview.md。消费814公开SDK/owned执行，主体已验证；shell生成manifest缺canonical双直依赖违反当前runtime明确生成工程规则，是F01关联规范不满足处。 |
| 06 | docs/specs/repository-boundaries/generated-ui-distribution.md。26源码闭包、根输入/producer/锁/资源/失败矩阵保持原33/33和377文件实证。源提交与目录digest/精确artifact分层不承诺可重建同字节；F01生产改动后必须重绑新源和实际参考artifact，不沿用旧release指纹作新代码交付。 |
| 07 | docs/specs/auto-ui-runtime/project.md。宿主退出代计数/活动窗口API、初始化期请求与跨会话陈旧请求、VM会话进程隔离与原CLI边界已实证；按冻结正文可形成增量，整体发布仍待F01与最终pass。 |

spec-impact维持原五supersedes路径、两new组件路径和GOAL-003/007。本轮发现为原规则缺口，不擅自改变合同或plan_revision；不得用“预存/罕见/只在再生器”豁免明确迁入的公开生成入口。A/L/T04历史通过保留；T06/07/08受影响范围重开，T09未过、merge-owned全门pending。
