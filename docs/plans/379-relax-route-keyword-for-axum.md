# Plan 379：放宽 `route` 关键字 — 让 axum `Router::route()` 可作为方法名调用

> **Status**: ✅ 已完成（2026-08-01）
> **来源**: auto-musk Plan 014（Rust 后端移植到 Auto）—— 移植 axum 异步 Web 层时，
> 发现 `.route()` 被 a2r 当成保留字，词法层面无法作为方法名调用。
> **影响仓库**: `auto-lang`（`crates/auto-lang/src/token.rs`、`lexer.rs`）
> **风险**: 低 — `route`（单数）是"死保留字"：词法化为 `TokenKind::Route`，但 parser
> 从不消费它（只有 `routes`/`outlet`/`link` 在 view DSL block 解析里有实际用途）。

---

## 1. 问题

auto-musk 的 Rust 后端用 axum 构建 Web 服务器，核心模式是：

```rust
Router::new().route("/", get(handler)).route("/api/x", get(x))
```

移植到 Auto 时，`.route("/", ...)` 报 `Expected term, got Route`：

```
use.rust axum::Router
fn build(r Router) Router { let r2 = r.route("/", h); return r2 }
                                                          ↑ Expected term, got Route
```

根因：`route` 是 Plan 105 引入的前端路由 DSL 保留字（`TokenKind::Route`）。axum 的
`.route()` 方法名与之冲突。`#[rs]` 逃生舱也救不了 —— `#[rs]` 仍走 Auto 词法分析，
`route` 仍是保留字 token。

## 2. 根因调查

`token.rs` 的关键字表把 `route` 映射到 `TokenKind::Route`：

```rust
"routes" => Some(TokenKind::Routes), // Plan 105: router keywords
"outlet" => Some(TokenKind::Outlet),
"link" => Some(TokenKind::Link),
"route" => Some(TokenKind::Route),
```

但 grep 全 parser/lexer，**`TokenKind::Route` 从未被 parser 消费**：

- `routes`（复数）在 `parse_routes_block_inner`（parser.rs:11441）作为 view DSL block 关键字使用 —— 必须保留。
- `outlet`/`link` 在 view DSL 元素解析里使用 —— 保留。
- `route`（单数）—— 无任何 parser 分支消费它。是 Plan 105 引入但从未接入的"死保留字"。

对照先例：**Plan 354** 已经把 `nav` 从保留字改成普通 Ident（注释在 token.rs:405），
理由完全相同（让 `nav(...)` 可作为函数调用）。`route` 的处理与 `nav` 一致。

## 3. 修复

`token.rs`：把 `"route"` 的映射改成 `None`（词法化为 Ident）：

```rust
"routes" => Some(TokenKind::Routes),
"outlet" => Some(TokenKind::Outlet),
"link" => Some(TokenKind::Link),
// Plan 379: 'route' 不再是关键字（词法化为 Ident），可作为方法名（如 axum
// Router::route）。单数 route 从未被 parser 消费，移除安全。参照 Plan 354 nav。
// "route" => Some(TokenKind::Route),
```

`TokenKind::Route` 变体本身保留（避免 match 穷尽性连锁修改；它仍在 Display impl 里
被引用，编译无 warning）。

`lexer.rs`：更新 `test_router_keywords` 断言（`route` 现在词法化为 `<ident:route>`，
与 `nav` 一致）。该测试在 master 上本就因 Plan 354 的 nav 改动而失败，本次一并修正。

## 4. 验证

- ✅ `cargo build --bin auto` 编译通过。
- ✅ axum 链式调用现在可解析：`Router.new().route("/", h)`、`.route("/x", "y")`、
  `.merge(r)` 等均生成正确的原生 Rust（保留字障碍解除）。
- ✅ `test_router_keywords` + `test_routes_token_text` 通过。
- ✅ 无新增回归：master 上既有的失败集（`dstr_tests`、`route::discovery`、
  `perf_benchmark` 栈溢出）与本 worktree 完全一致 —— 这些是既有问题，非本次引入。

## 5. 后续（非本计划范围）

放宽 `route` 后，移植 axum 层还有**一个独立障碍**：函数引用作为方法实参
（`.route("/", handler)` 里 `handler` 被当成未定义变量）。这是 a2r 名称解析的另一个
缺陷，不影响本计划的保留字修复，留待后续计划处理（可用闭包 `() => handler()` 临时绕开）。
