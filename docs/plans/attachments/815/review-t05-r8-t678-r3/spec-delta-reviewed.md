# PLAN-815 T-05 R8 / T-06～T-08 提前 R3 规范增量核验

2026-10-22，Plan r3，review_scope=phase（五项修复复审，非T-09终审）。冻结HEAD c050e718e09b66ebb6d7925c1f4e0ce89bb10dc5，code e351cb2a73a78f353457e56ea20cb82f61c1ac88，base e3fdcdc76。输入/当前canonical版本见baseline.json.txt，交接正文见frozen-handoff.md。实际bin SHA256 f21db18136403d1124ecbc1338f61c5603544401c434e7d12fad20a60c37dfb5与交接一致，起止输入零漂移。未写canonical/live ledger。

| SD / target | 当期核验与发布边界 |
|---|---|
| SD-01 docs/specs/auto-ui-tools/project.md | Rust请求声明已传至生成/增量判定/交付源选择/运行CWD；默认、自定义、node与prop及默认目录冲突、同项目切换已有目录、选中目录内容变化的实际产物/派发控制通过。Ark原样生成ID与缺失/不可读失败通过。此为修复范围验收，不外推完整792发行/真实设备运行已验证。 |
| SD-02 docs/specs/tool-protocol/project.md | 五项修复未改wire/预算/依赖配置；tool_process21+8保持，先前捕获/取消具名失败与闭包门证据按未变输入继承。完整协议消费者四种服务端回执及深度控制仍属T-07待办。 |
| SD-03 docs/specs/auto-man/project.md | legacy Jet/Ark两个公开入口正控保持；不以协议严失败改变旧手工语义。新front helper保留legacy探测链。 |
| SD-04 docs/specs/project-model/project.md | 当前请求front优先级已在Rust实际生成/派发验证，不再仅准入。PM合法bun回退、工具跳磁盘和legacy缓存稳定分开验证，新鲜子进程只证明首次磁盘链；Win/Linux完整pkg模块9/9。完整字段矩阵仍pending。 |
| SD-05 docs/specs/auto-lang/ui/overview.md | 新增调用不改变公开包方向/源owner；本轮没有以接口不变推断完整平台/资源已通过，资源/许可审计继续待办。 |
| SD-06 docs/specs/repository-boundaries/generated-ui-distribution.md | 777本分支独立strict0 violations/3 grandfathered；不是master并集/792/fresh/locked发行。真实Vue构建测试通过，dist验收器仍把CSS引用当应用bundle引用，F-815-T07-R1-01部分未闭，不能称可消费入口验证已经完整。 |
| SD-07 docs/specs/auto-ui-runtime/project.md | 本轮未修改runtime源；既有VM生命周期关闭边界保留，tool_process实际会话启动/输入/退出/重启保持。Linux本轮pkg9/9不能代替完整Linux生命周期/旧real_tree可移植矩阵。 |

T-08源/bin和本轮front/Ark-ID事实更新相符；完整T-07前置与SD最终发布性尚未成立。SD-01/02既有A规范需按当前canonical增补、保留来源，不能用历史before覆盖已经发布的A内容（原待办继续保留，不重复编号）。计划supersedes/new/touched_goals沿r3保持，本轮不变更合同/修订号、不标整体reviewed，不发布816-B能力。T-09及其后merge-owned冻结门仍pending。
