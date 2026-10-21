# T-04 提前定向审查：规范增量冻结

2026-10-21；Plan 815 r3；代码 90aed7fc7df875987cf9014a37fdc256e83fdae6；原合同/四份输入 Spec 的 SHA256 见 baseline.json.txt。审查对象为现有 §5「规范增量」与 contract-baseline.md；本文件不是新合同或 canonical 发布。

| ID | 当前原提案 | T-04 判断 |
|---|---|---|
| SD-01 | auto-ui-tools/project.md：独立工具/资源、兼容与生命周期 | 生成与资源主体已有代码；交付闭包/公开兼容仍失败，不能把完整 B 写成已交付。A 窄能力历史规则保留。 |
| SD-02 | tool-protocol/project.md：I-TOOL 协议 | 本轮无协议扩能；进程描述仍只声明 describe/validate_project，不能声称 build/run 可用。 |
| SD-03 | auto-man/project.md：旧兼容与非 UI adapter | auto-api-gen 是共享单算法 owner；运行兼容模块暂留 Man 合同允许，但旧公开路径必须补齐。 |
| SD-04 | project-model/project.md：已交付 I-PROJECT 消费 | 请求构造仍归 Man adapter，工具消费轻请求；本轮未扩静态准入或重新声明类型。 |
| SD-05 | auto-lang/ui/overview.md：消费公开编译域 | app_registry/i18n_lookup/store_deps 移至 auto-ui-compiler；store_deps 旧路径与测试依赖未完整保留。 |
| SD-06 | repository-boundaries/generated-ui-distribution.md：source/embed/producer 闭包 | 当前 23 项清单不满足自包含，不能宣称 fresh/locked 已成立。 |

十项偏差的裁量：1/2/3 的共享单算法方向与无 Man 反依赖目标相容，本次未发现需要另造第二实现的理由；T-08/T-09 应如实描述 api-gen 的服务桥、sidecar 效果 adapter、pkg 共用方向及 owner。4 的公共兼容声明被本轮反例否定。5 的 shadcn 组合胶水消费同一 ui_gen 算法，完整兼容仍归后续验收。6 发行清单实际失败。7 从禁列移除 auto-media-core 与已发布 M814-A 合法 SDK 闭包一致，精确四个大库禁列保留。8 请求缝定向测试通过；legacy 配置读取改用 man-core 声明模型，未另造 I-PROJECT 构造器。9 nextest 包测试不能覆盖进程内所有组合，全局态的历史限制保留。10 三项工具红的测试体经归一后与基面相同，但本轮没有通过 stash 在旧基面执行；不能把新增 plan734 金样红混入原“独立编译 5 错”归因。

现有 spec-impact 路径/GOAL-003、GOAL-007 在阶段内保留。T-08 最终增量还须纳入 auto-ui-compiler 与 auto-api-gen 的实际 owner/公开面及准确路径元数据；本轮不把未完成 B 规则写入 canonical，不写 live ledger。原 M815-A/M815-L 已发布范围与历史 pass 保留。完整 Spec 增量和全 AC 终审仍归 T-08/T-09。
