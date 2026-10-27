# PLAN-842 外部消费者退役控制

在计划工作树中执行：

```text
python test/fixtures/modularity/plan842-retirement/check.py --output <日志目录>
```

该探针对每个退役 Rust 路径使用独立 `cargo check`，核对失败码及被拒绝的符号；随后实际编译包含规范 AST、诊断、解析器、会话、UI 文法和 builder 入口的消费者。依赖从相邻的计划工作树读取，使用语言仓共享 target，不依赖网络。

版本迁移阶段还会分别验证三个 `=0.1.0` 依赖不能误绑定到 0.2.0 源，以及三者均为 `=0.2.0` 时规范消费者可编译。

T-06 控制还验证 `use-bigvm` 已被具名拒绝、移除后默认 VM 真正执行程序。
