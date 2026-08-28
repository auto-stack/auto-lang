# a2r-actor-tests

> **Status**: test-only
> 路径：`crates/a2r-actor-tests`  | 技术栈：Rust（cargo 嵌套构建）

actor 行为 parity 慢测试（Plan 387）：把 `.at` actor 程序转译成 Rust → cargo 编译运行 →
与 AutoVM actor 套件的 stdout 对拍。**慢**（嵌套 cargo build），测试标 `#[ignore]`
默认不跑，按需手动执行。

## 目标与范围

- 守护 a2r 转译在 task/actor 并发语义上与 VM 行为一致。
- 不做：常规回归（那是 `cargo t` / parity workspace 的职责）。

## 模块清单

| 模块 | 职责 | 状态 |
|---|---|---|
| lib | 测试支撑（转译→构建→运行→对拍） | test-only |
| tests/actor_parity | actor 套件对拍用例（a2r_std 以 path 引入） | test-only |

## plans

- **plan-387** a2r actor task translation ✅ archived——actor/task 转译 + parity 测试族
