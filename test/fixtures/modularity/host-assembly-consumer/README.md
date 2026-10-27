# PLAN-842 宿主装配外部消费者

以独立进程运行纯 SDK、无 UI 兼容包和 UI 默认/显式覆盖场景；命令需从语言工作树执行，使用固定锁并离线运行：

```text
cargo run --locked --offline --manifest-path test/fixtures/modularity/host-assembly-consumer/Cargo.toml -- absent
cargo run --locked --offline --manifest-path test/fixtures/modularity/host-assembly-consumer/Cargo.toml --features lang -- absent
cargo run --locked --offline --manifest-path test/fixtures/modularity/host-assembly-consumer/Cargo.toml --features ui -- default
cargo run --locked --offline --manifest-path test/fixtures/modularity/host-assembly-consumer/Cargo.toml --features ui -- explicit
```

UI 两例只通过 `Parser` 保证旧包真实链接，直接调用 SDK 装配入口，刻意不调用语言包的安装包装；因此能够区分链接兜底与显式入口。

`--features ui -- thread-exit` 在工作线程调用真实 SDK 安装入口，join 后验证 provider 仍在，并按约定返回退出码 23；default 模式还验证重复装配不替换已有 provider。
