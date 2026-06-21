# Plan 328: a2r HTTP Server 架构 — 把 #[api] 转译成 Axum 原生 Rust Server

> **Status**: 设计完成,待实施
> **目标**: `auto build`(Rust backend)时,把 #[api] 函数转译成 Axum handler,生成完整可运行的编译期 HTTP server(无 VM)
> **关联**: Plan 312(HTTP server MVP)、Plan 327(AutoVM HTTP server,路径 B)

## 背景

a2r 的本意是编译期转译获得最佳性能(用于最终发布)。当前 a2r(trans/rust.rs)完全不识别 #[api],
AxumGenerator(api/targets/axum.rs)已写好 handler+router 生成但未接入 build 流程,也不生成
server 启动代码。本计划让 a2r 把 #[api] 函数转成 Axum 原生 Rust server。

## 已有资产(无需重写)

- `Fn.api_attrs: Option<ApiAttrs>`(ast/fun.rs:35)——**已完成**,parser 已把 #[api] 解析到此
- `ApiAttrs { method, path }`(ast/fun.rs:41-46)——HTTP 方法和路径
- `AxumGenerator`(api/targets/axum.rs:92-255)——能生成 handler(Path/Query/Json 参数)+ router
- a2r 主体(trans/rust.rs)——已能转译业务逻辑(List→Vec, var→static, module→mod, generator→stream!)
- `CargoBuilder`(auto-man/src/builder/cargo.rs)——能写 Cargo.toml + cargo build
- `transpile_auto`(target.rs:574)——a2r 转译入口(单文件/多文件)
- `transpile_rust_project`(trans/rust.rs:11281)——多文件转译(含 mod/类型共享)

## 6 个改动环节

### 环节 1: ApiExtractor 从 Fn.api_attrs 提取端点(不再 heuristic)

**现状**: api/mod.rs:135 的 extract_endpoint 注释自承"未真正读取注解",靠函数名 heuristic。
**改动**: 改 extract_endpoint,从 Fn.api_attrs 提取 method/path(而非猜)。
**文件**: api/mod.rs:135-157

### 环节 2: AxumGenerator 补 server 启动模板

**现状**: generate(258-276)只输出 handler+router,无 main/serve。
**改动**: 加 `generate_server_main` 方法,生成:
```rust
#[tokio::main]
async fn main() {
    let app = create_api_router();
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8080").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
```
**文件**: api/targets/axum.rs(新方法)

### 环节 3: 接入 transpile_auto(在 a2r 输出后生成 server 代码)

**现状**: transpile_auto(target.rs:574)调 a2r 生成 .rs 文件,不处理 #[api]。
**改动**: 在 transpile_auto 完成后:
1. 解析所有 .at 源文件的 AST,收集 #[api] 函数
2. 用 ApiExtractor 提取端点
3. AxumGenerator.generate 输出 handlers + router
4. AxumGenerator.generate_server_main 输出 main.rs
5. 写入 rust/src/router.rs + rust/src/main.rs
**文件**: auto-man/src/target.rs(新增 generate_api_server 方法)

### 环节 4: Cargo.toml 注入 axum/tokio/serde 依赖

**现状**: CargoBuilder::setup(cargo.rs:124)靠 scan_rs_for_crates 被动扫 use 语句。
**改动**: 当有 #[api] 端点时,显式追加固定版本依赖。
**文件**: auto-man/src/builder/cargo.rs(CargoBuilder::setup)

### 环节 5: SSE handler(~Iter<T> / ~Stream<T>)

**现状**: AxumGenerator 无 SSE 分支。a2r 能把 generator 转成 `impl Stream`。
**改动**: generate_handler 检测返回类型 ~Iter/~Stream → 生成 `Sse<impl Stream>` handler。
**文件**: api/targets/axum.rs(generate_handler 加 SSE 分支)

### 环节 6: handler 返回类型映射

| Auto 返回类型 | Axum handler 返回 |
|---|---|
| `[]Note` | `Json<Vec<Note>>` |
| `?Note` | `Json<Option<Note>>` |
| `Note` | `Json<Note>` |
| `int`/`str`/`bool` | `Json<i32>` / `Json<String>` / `Json<bool>` |
| `~Iter<T>` / `~Stream<T>` | `Sse<impl Stream<Item = T>>` |
| `void` | `axum::response::StatusCode` |

## 实施顺序

1. 环节 1+2 → 编译验证
2. 环节 3+4 → auto build 验证
3. 环节 5+6 → 端到端验证
4. 全量回归 + 提交
