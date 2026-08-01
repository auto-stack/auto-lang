# Plan 380：a2r 对 Rust 互操作的三项补全（impl Trait 返回 / 元组结构体构造 / extractor 参数）

> **Status**: 📐 方案设计完成，待实施（2026-08-01）
> **来源**: auto-musk Plan 014 —— 移植 axum 异步 Web 层（server.rs 2206 行）时，
> 误判"a2r 缺 async 支持"。复核 a2r 测试用例 + 逐点验证后发现：a2r 的 async/trait/
> 库调用支持相当完整，真正阻塞 server handler 全量移植的是 3 个具体转译缺陷。
> **影响仓库**: `auto-lang`（`crates/auto-lang/src/trans/rust.rs` + `parser.rs`）
> **风险**: 中 —— 触动类型解析 + 结构体构造代码生成，但每项有 golden 测试参照。
> **承接**: auto-lang Plan 379（route 保留字解除，已合并）。

---

## 0. 核心纠偏：a2r 的 Rust 互操作已经很全面

auto-musk Plan 014 早期结论"a2r 缺 async / trait object / 服务器运行时支持"是**错误的**。
经逐点验证 a2r 已支持的能力（均有 `test/a2r/16_interop/` golden 案例或实测背书）：

| 能力 | Auto 写法 | 生成 Rust | 验证 |
|---|---|---|---|
| async fn | 返回类型 `~T` | `async fn f() -> T` | `16_interop/001_async_fn` |
| `.await` | `expr.await` | `expr.await` | `002_tokio_main` |
| `#[tokio::main]` | main 体内含 `.await` 自动加 | `#[tokio::main] async fn main` | `002_tokio_main` |
| **调用 async Rust 库** | `use.rust tokio::fs::{read_to_string}` + `read_to_string(path).await` | 原生 | 实测 ✅ |
| **trait object `Box<dyn T>`** | `spec T {...}` + 字段类型用 `T` | `trait T {...}` + `Box<dyn T>` | 实测 ✅ |
| **trait impl** | `type English as Greeter {...}` | `impl Greeter for English` | 实测 ✅ |
| tokio::spawn | `expr.go` | `tokio::spawn(async move { expr.await })` | rust.rs:2963 |
| 自动 .await 插入 | 方法返回 `~Result`/`Future` 时 | 自动补 `.await` | rust.rs:6776 (Plan 373) |
| axum 路由链 | `.route("/", get(h))` | 原生 | Plan 379 + 实测 |
| extractor 作参数类型（不解构）| `fn h(p Path<str>)` | `fn h(p: Path<String>)` | 实测 ✅ |
| 具体返回类型替代 impl Trait | `fn h() ~Json<T>` | `async fn h() -> Json<T>` | 实测 ✅ |

**结论：Auto 调用 async Rust 库是全面支持的，trait object 也有完整路径（spec 系统）。**
不需要"补 async 运行时"——那从来不是问题。

## 1. 真正阻塞 server handler 全量移植的 3 个缺陷

经精确二分，剩余障碍收敛为 **3 个具体的转译缺陷**（与 async/trait 无关，是语法/构造细节）：

### 缺陷 A：元组结构体构造 `T(value)` → `T { field0: value }`（a2r-20）

**现象**：`Json("ok")` / `Some(5)` 等元组结构体（newtype pattern）构造，a2r 生成命名字段形式。
```
Json(Item_result)  →  Json { field0: Item_result }   // E0560: Json<T> 无 field0 字段
```
**影响**：axum handler 的 `Json(body)` 返回构造、Option/Result 包装构造。
**根因**：rust.rs 的结构体字面量构造代码生成只处理"命名字段"形态，对"位置参数"构造
退化成 `field0/field1/...`，未识别目标类型是元组结构体。
**位置**：rust.rs 结构体字面量（`Expr::Struct` 或构造调用）的 trans 分支。

### 缺陷 B：`impl Trait` 返回类型不被解析（"Expected type, got impl"）

**现象**：`fn health() ~impl IntoResponse` 在类型位置遇到 `impl` 关键字报错。
**影响**：axum handler 常用 `-> impl IntoResponse` 返回（多态返回）。
**可绕开性**：**高**——用具体类型 `~Json<T>` / `~String` 替代（它们都 impl IntoResponse）。
auto-musk server.rs 大部分 handler 可改具体返回类型；少数多态返回的需保留手写或引入枚举。
**是否必须修**：**非必须**（可绕开），但修复后更忠实。优先级低。

### 缺陷 C：extractor 参数**解构** `Path(id)` 不被解析（"unexpected token"）

**现象**：`fn h(Path(id) ~Path<str>)` 参数位置的 `Path(id)` 解构模式失败。
**可绕开性**：**高**——用 `fn h(p Path<str>)`（extractor 作整体参数类型），
函数体内 `p.0` 或 `p.into_inner()` 取值。axum 的 extractor 都支持"不解构直接接收"。
**是否必须修**：**非必须**（可绕开）。修复需扩展参数语法的解构模式，工作量大。
**优先级**：低。

### 缺陷 D：类型参数位置的 `dyn` 不支持（`Arc<dyn Trait>` 字段）

**现象**：`type AppState { client Arc<dyn Client> }` 报 "Expected term, got RBrace"——
类型解析器在 `<dyn Client>` 的 `dyn` 处卡住。
**影响**：auto-musk 的 `AppState.client: Arc<dyn Client>`（共享的 LLM 客户端 trait object）。
**对照**：`spec Client` 做字段类型能生成 `Box<dyn Client>`（已验证 ✅），但显式写
`Arc<dyn T>` / `Box<dyn T>` 不行——`<` 内的 `dyn` 关键字不被类型解析器接受。
**修复**：类型解析（parser `parse_type`）在解析泛型参数 `<...>` 时，允许 `dyn Trait`
形态（`dyn` 后跟一个类型名），生成 `dyn Trait`。这样 `Arc<dyn Client>` 直接可用，
无需走 spec 字段间接。

## 2. 补全方案

### 实施优先级（按 ROI + 解锁面）

| 优先级 | 缺陷 | 工作量 | 解锁 |
|---|---|---|---|
| **P0** | A（元组结构体构造）| 小 | axum `Json(v)` 返回 + Option/Result 包装；解锁绝大多数 server handler |
| **P1** | D（`Arc<dyn T>` 字段）| 小 | AppState 的 `Arc<dyn Client>` 共享 trait object 字段 |
| P2 | B（impl Trait 返回）| 中 | 多态返回 handler（非必须，可绕开）|
| P3 | C（extractor 解构）| 大 | 解构写法（非必须，可绕开）|

**建议**：只实施 P0 + P1（小工作量，解锁 server.rs handler 主体），P2/P3 记录但暂不做
（都可绕开，ROI 低）。

### P0：修复元组结构体构造（缺陷 A）

**方案**：在 rust.rs 的结构体构造代码生成处，增加"位置参数构造"识别：
1. 当构造调用 `T(args...)` 的**所有参数都是位置的**（无 `field:` 命名），且：
   - 目标类型 T 在 Universe/type_store 里已知是**元组结构体**（字段为 `_0/_1/...` 或
     无命名字段），或
   - 目标类型是已知的 newtype（axum::Json / std 的 Some/Ok/Err 等），
2. 则生成 `T(arg0, arg1, ...)`（位置构造），而非 `T { field0: arg0, ... }`。

**识别依据**：type_store 的 TypeDecl 有无命名字段。若 TypeDecl 字段为空或字段名形如
`_0/_1`，按元组结构体处理。对外部类型（use.rust 导入的 Json/Option/Result），用一个
已知 newtype 白名单（`Json`/`Some`/`None`/`Ok`/`Err`/`Box`/`Arc`/`Vec` 等单字段元组）。

**golden 测试**（新增 `test/a2r/16_interop/004_tuple_struct/`）：
```auto
use.rust axum::Json
fn ok() ~Json<str> {
    return Json("ok".to_string())
}
```
期望生成：
```rust
async fn ok() -> Json<String> {
    return Json("ok".to_string());
}
```
外加 cargo check（带 axum 依赖）验证可编译。

### P1：修复 `Arc<dyn T>` 字段（缺陷 D，已确认）

**方案**：扩展类型解析器（parser `parse_type`）在解析泛型参数 `<...>` 时，
接受 `dyn Trait` 形态（`dyn` 关键字后跟类型名）。这样字段类型 `Arc<dyn Client>` /
`Box<dyn Client>` 可直接写，生成 `Arc<dyn Client>`。

测试用例（`test/a2r/16_interop/005_dyn_field/`）：
```auto
use.rust std::sync::Arc
spec Client { fn fetch(url str) str }
type AppState { client Arc<dyn Client> }
```
期望生成：
```rust
use std::sync::Arc;
trait Client { fn fetch(&self, url: &str) -> String; }
struct AppState { pub client: Arc<dyn Client> }
```

**注**：已有的 `spec T` 做字段类型 → `Box<dyn T>` 路径继续保留（对自定义 trait 友好）；
P1 补的是"直接写 `Arc<dyn>`/`Box<dyn>` 引用外部 Rust trait"的能力。

### P2/P3：记录但不实施

缺陷 B（impl Trait 返回）和 C（extractor 解构）都有 100% 可绕开的等价写法
（具体返回类型 / extractor 作整体参数）。修复它们的语法扩展工作量大、ROI 低，
本计划仅记录，留给后续按需推进。

## 3. 修复后的 server handler 移植形态（auto-musk 侧）

修复 P0 后，auto-musk 的 axum handler 可这样移植（示例）：
```auto
use.rust axum::{Router, Json}
use.rust axum::routing::{get, post}
use.rust axum::extract::{Path, State}
use.rust serde::{Serialize, Deserialize}

type AppState { count int }   // 或用 spec Client 表达 Arc<dyn Client>

#[derive(Serialize, Deserialize)]
type ItemBody { name str }

// handler: 具体返回类型 Json<Item>（非 impl IntoResponse），Json 构造用位置参数
fn create_item(body Json<ItemBody>) ~Json<ItemBody> {
    return Json(ItemBody(name: body.name.clone()))
}

fn build_router() Router {
    return Router.new()
        .route("/items", post(create_item))
        .route("/health", get(health))
}
```
这是 100% 可编译的原生 axum Rust。auto-musk server.rs 的 ~50 个 handler 可按此模式
全量移植（少数多态返回 handler 用具体类型重写或保留手写）。

## 4. 验证基线

每项缺陷修复须过：
1. 新增 golden `.at` → `.expected.rs` 测试（`test/a2r/16_interop/` 下）
2. 临时 crate + 真实 axum 0.8 依赖 → `cargo check` 0 错误
3. 无新增回归：master 既有失败集（dstr_tests/route::discovery/perf 栈溢出）不变
4. auto-musk 侧：server.at 的 build_router() + 一个真实 extractor handler cargo check 通过

## 5. 与 auto-musk Plan 014 的关系

本计划是 Plan 014 的**上游依赖**：
- Plan 379（已合并）解除 route 保留字 → server 路由表可移植
- **Plan 380 P0**（本计划）修复元组结构体构造 → server handler 返回值可移植
- 两者完成后，auto-musk server.rs 的路由表 + handler 主体可全量移植到 Auto
- auto-musk Plan 014 的 a2r-20/21 限制可随之降级/移除

实施顺序：先 Plan 380 P0（本 worktree），合并后回 auto-musk 全量移植 server handler。
