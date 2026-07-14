# Plan 355: a2r async/await 转译（从 Plan 344 拆出）

> **状态**：设计文档 / TODO
> **来源**：从 Plan 344（统一 HTTP 通讯架构）拆出，Plan 344 VM 侧已完成

## 背景

Plan 344 的 VM 侧四象限已全部实现（同步/异步 × 流式/非流式）。
但 a2r 侧只有同步转译（ureq / reqwest::blocking），缺少 async/await 转译。

## 未完成的部分（a2r 侧）

| 项 | 说明 |
|---|---|
| **async fn 转译** | Auto 的 handler fn → Rust `async fn` |
| **`.await` 转译** | VM 的非阻塞 yield → Rust 的 `.await` |
| **`for await` → `while let`** | SSE/stream 迭代 → `while let Some(chunk) = stream.next().await` |
| **reqwest async client** | a2r 生成 `reqwest::Client`（异步）替代 `ureq`（同步） |

## 改动范围

- `crates/auto-lang/src/ast.rs` — `Await` / `ForAwait` AST 节点（新增）
- `crates/auto-lang/src/parser.rs` — `await` / `for await` 语法解析
- `crates/auto-lang/src/ui_gen/rust.rs` — a2r RustGenerator 支持 async/await
- `crates/auto-man/src/rust_ui.rs` — 生成 reqwest async client 代码

## 优先级

🟡 中——现有同步 a2r + VM 异步已覆盖所有运行模式，a2r async 是性能优化。
