# PLAN-815 T-05 R6 / T-06 / T-08 当期规范核验

Plan r3，phase定向；代码c6fdaaaf1ecc2799d5ad225db3922a121c90c383，交接bb4f0a1caeb252ccb82d722659ee827703b82fc1。实际交接与修复表冻结在frozen-handoff.md/frozen-fixes.md，Unix实施证据另冻结frozen-unix-evidence.md；当前canonical来源hash见baseline.json.txt。本文件不是完整B发布稿，未改canonical/live ledger。

独立重建bin SHA256 c95748c80b512215d89d6c987d3b8b708f697d102a24a62333f7666051c42b61与交接一致。源/bin/支持指纹、RunOk/BuildOk双终态、SD-01..07正文均保持已补状态。

| SD / target | R6核验 |
|---|---|
| SD-01 / docs/specs/auto-ui-tools/project.md | 请求包管理器三态与guard接线成立，原wire与磁盘冲突例选npm；无字段合法bun回退也成立。旧公开Jet/Ark build返回与协议严格失败分离成立。生产取消原例已闭，但捕获超时/截断仍吞成completed，不能沉淀“完整失败语义已统一”。 |
| SD-02 / docs/specs/tool-protocol/project.md | 取消回执与终态分类正确；DrainEnd五分类只在helper内存在，Jet调用方把BoundOverrun/Truncated/IoError及异常退出折成false，客户端未获具名错误。Unix可停读不再只是Windows外推，本轮冻结源码离线Linux4/4通过。 |
| SD-03 / docs/specs/auto-man/project.md | 旧公开Jet失败Ok/Ark缺wrapper Ok独立验证，Vscode同类门控源码已读，原兼容回归关闭。不能把统一owned Command形态当作客户端错误回报已经成立。 |
| SD-04 / docs/specs/project-model/project.md | 当前请求raw pkg优先，缺字段只走外部global/detect，legacy才保留主pac扫描。新PM测试的Bun值不等断言误拒合法来源，必须修正验证器，不能删bun能力或削减回退规则。模型/codec本轮未改。 |
| SD-05 / docs/specs/auto-lang/ui/overview.md | 分层方向保持；owned树终止与reader真停原例成立。调用方仍须诚实传播捕获结束原因，不能把一次失败探测等同于完整探测得到空设备列表。 |
| SD-06 / docs/specs/repository-boundaries/generated-ui-distribution.md | 当前冻结指纹准确；完整777/792并集/fresh/locked/资源与最新master并集仍T-07/merge后续责任，未以本轮局部测试替代。 |
| SD-07 / docs/specs/auto-ui-runtime/project.md | 退出窗口/VM执行本轮未改，真实VM交互、串行外部→本地与缺DLL对照复跑保持；Unix4项包括会话环境与fatal marker，不能外推到Linux图形呈现或全库。 |

T-08草稿继续partial：取消/平台/legacy旧缺口有进展，但“捕获有界失败具名披露”仍不成立，且PM验收自身误拒合法环境。后续正文须与修复后真实失败矩阵对齐；T-07依赖、完整T-09与merge-owned冻结门仍pending。impact保持runtime/Man/UIoverview/model/distribution、tools/protocol、GOAL-003/007；不改AC或plan_revision，不覆盖A/L与T-04历史发布。
