# PLAN-856 合入门预算诊断

冻结代码 `4150a8bea7b66aff9eca87a319d5cd1c6b4b50a7`、SDK `7b987ca8b12f2eec3c076a1bfbf3ff4663aa8467`；855交付基面 `05fdc81c546aff735eefdb6c39fbc5fb35dc546a` 已含同一断言，计划856未改动它。

- 生产端完整门 300/300 通过。消费端第一次完整门异常退出 4294967295，日志在 zero_window 开始处截断，无错误诊断；该单例和完整11例入口隔离通过，首次中断的根因未知。
- 消费端第二次完整门 exit 101：`tests/segment_budget.rs:199` 要求 `cumulative CPU budget exhausted`，实际是 `instruction budget exhausted ... per-request cumulative budget 100000 steps; AUTO_HTTP_STEP_BUDGET`。原始证据 `D:/autostack/plan-artifacts/plan-856/merge-r1/consumer-test-attempt2.log.txt`。
- 固定 SDK `vm/engine.rs` 的 `DriveBudget::CpuSlice` 将累计上限传给片内检查；片内超限和片尾达到上限是不同错误出口。`src/lib.rs` 的预算为每片1000步、2ms、每64步查时钟、累计100000步；不同切片边界可触发不同出口。片内错误将 CPU slice 报为 HTTP 配置，是既有 SDK 诊断缺陷。该现象与是否设置 `AUTO_HTTP_STEP_BUDGET` 无关，当前环境该变量未设置。
- 同一个已构建的消费端测试二进制在相同冻结内容/配置下，原失败用例隔离重放连续 12/12 通过；日志与退出码 `D:/autostack/plan-artifacts/plan-856/merge-r1/consumer-budget-diagnostics.json.txt`。这证明出现过不同结果，不能证明首次中断原因，也不证明将来不再出现。
- 下一步：依据现有定向结果进行一次同基线完整门，保留所有尝试；不得把定向结果替代完整门或把测试不稳定描述成已修复。无论通过与否，此既有脆弱断言及 SDK 误导性文案应作为明确风险记录。
