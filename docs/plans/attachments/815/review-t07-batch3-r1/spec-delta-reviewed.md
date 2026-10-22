# PLAN-815 T-07 子批3 SD 受影响范围核验

冻结 9aee8659d8571ad889d5ccc46086c5102d8f64d6，base503c9f67f，Plan r3。phase review needs_fix，仅原 B2-R1-03 尚未完整关闭。正文冻结副本 frozen-handoff.md、frozen-field-profile-matrix.md；当前 Spec 哈希见 baseline.json.txt，终核无漂移。未发布 canonical 或改 live ledger。

| 增量 | 受影响判断 |
|---|---|
| SD-01 auto-ui-tools/project.md | 本批无生产行为变更；工具源包 fresh/locked、重定位实际消费独立33/33，支持既有发行保证。字段矩阵错误不改写成新产品限制。 |
| SD-02 tool-protocol/project.md | I-TOOL v1/protocol/预算/控制和日志边界代码未变；本轮真包五操作describe、未知版本generate零副作用、坏帧非零退出成立，既有生命周期修复证据保持其原范围。 |
| SD-03 auto-man/project.md | 门面/非UI owner代码未变，复用既有审查，不声称本轮重跑所有兼容面。 |
| SD-04 project-model/project.md | 不改变809合同。canonical design/generation-request.md明列PacSource window/title/api/dep_roots消费，不能被错误携带表覆盖。需修T07矩阵后再作完整T08/T09依据。 |
| SD-05 auto-lang/ui/overview.md | 公开SDK依赖/owned子进程实现未变，既有闭项不重开。 |
| SD-06 repository-boundaries/generated-ui-distribution.md | 工具源包A/B实际构建和资源来源通过；四发行路径区分正确。实际debug二进制身份依构建而异，不宣称无生产改动即相同哈希；T08最终交接绑定实际冻结源/产物。 |
| SD-07 auto-ui-runtime/project.md | Inter/OFL真实owner和随包资源已纠正、独立逐字节核对通过；宿主退出API未改，复用既有证据。 |

spec-impact 保持主Plan：supersedes runtime/Man/UIoverview/model/distribution 五路径，new tools/protocol 两路径，GOAL-003/007。阶段范围不授权整Plan reviewed/合入/发布/归档。最终merge-owned门禁pending。
