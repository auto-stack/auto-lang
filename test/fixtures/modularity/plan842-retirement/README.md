# PLAN-842 外部消费者退役控制

在计划工作树中执行：

```text
python test/fixtures/modularity/plan842-retirement/check.py --output <日志目录>
```

该探针对每个退役 Rust 路径使用独立 `cargo check`，核对失败码及被拒绝的符号；随后实际编译包含规范 AST、诊断、解析器、会话、UI 文法和 builder 入口的消费者。依赖从相邻的计划工作树读取，使用语言仓共享 target，不依赖网络。
