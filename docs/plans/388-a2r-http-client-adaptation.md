---
plan: 388
title: a2r-http-client-adaptation
affects: [auto-man, auto-lang/a2r, a2r-std]
status: in-progress # draft | in-progress | complete
---

# Plan 388: a2r HTTP Client 适配 — 与 VM 的 HTTP 能力对齐

> **For Claude:**
> - 构建/测试命令：`cargo test -p auto-lang`（回归）、`cargo test -p auto-man`（codegen 相关，若有测试）、
>   `cargo test -p a2r-std`（runtime 单测）、生成代码手动编译运行验证。
> - 前置：Plan 349（HTTP Roadmap，VM 侧已完成，本计划是其 a2r 适配的正式化）、Plan 350（WebSocket VM 侧）。
> - 来源：`docs/plans/349-http-roadmap.md` "待实现步骤 1-8" 的 a2r 部分（步骤 1-4/6）。
> - **Out of scope**：步骤 7（普通 HTTP 异步化 = VM engine AWAIT_FUTURE 外部 future，Plan 344 路径 B，VM 侧改动）；
>   步骤 8（Cookie/重试/压缩/CORS 易用性）；VM 侧 native 已有能力不重写。

## §1 Goal / 目标

让 auto-man 的 Rust 前端（`crates/auto-man/src/rust_ui.rs`）生成的 API client 具备与 VM 侧
对齐的 HTTP 能力：**HTTPS 自定义 TLS 配置、multipart 文件上传、文件下载+断点续传+进度、
WebSocket 客户端**。VM 侧 native 均已实现并通过测试（Plan 349/350），本计划只做 a2r 侧
代码生成适配，使 Auto 前端生成的 Rust 代码在 API 层面与 VM 行为一致。

**非目标**：VM 异步化改造（步骤 7，Plan 344 路径 B）、易用性增强（步骤 8）、
HTTP server 侧（Plan 328 已覆盖）。

## §2 背景 / 现状缺口（调查已确认）

| 维度 | 现状 | 引用 |
|---|---|---|
| 生成 Cargo.toml 依赖 | **reqwest (blocking+multipart)、tungstenite (native-tls) 已声明**；ureq (json) 也在 | `rust_ui.rs:1481-1483,1698` |
| API client codegen | **全用 ureq**：GET/POST/PUT/DELETE `generate_get_fn_body`/`generate_write_fn_body` | `rust_ui.rs:696-890` |
| TLS 配置 | 仅生成 `_tls_skip_verify()` 读 `AUTO_TLS_SKIP_VERIFY` env，**端点函数未接线**（ureq 无 client builder） | `rust_ui.rs:680-694` |
| multipart | `generate_http_utility_functions` 已有 reqwest multipart **通用工具**（`upload_file`/`multipart_upload` 等），但无 endpoint 驱动生成、无 `RequestBuilder.multipart_file/text` 链式对应 | `rust_ui.rs:1130-1160` |
| 下载/断点续传/进度 | **无生成函数** | `rust_ui.rs` grep 无 download/resume/progress |
| WebSocket 客户端 | **无生成函数**（tungstenite 依赖已声明但未用） | `rust_ui.rs` grep 无 ws_ |
| a2r-std | `crates/a2r-std/src/http.rs` 只有基础 request/bearer/post_stream，无 TLS/multipart/download/ws | `http.rs` 全文 |

## §3 Scope / 范围

**W1 — TLS 配置适配（ureq → reqwest::blocking 迁移）**
- `generate_get_fn_body`/`generate_write_fn_body`/`generate_delete_fn_body` 从 `ureq::X(...)` 改为
  `reqwest::blocking::Client`（支持 `ClientBuilder.danger_accept_invalid_certs` /
  `add_root_certificate`）。
- `generate_http_client_helper` 扩展：按 `AUTO_TLS_SKIP_VERIFY` / `AUTO_TLS_CA_CERT` 生成
  TLS-aware `fn _client() -> reqwest::blocking::Client`（lazy static 或每次构建）。
- 生成代码可编译、行为与 ureq 等价（GET/POST JSON、headers、timeout、fire-and-forget 线程模式保持）。

**W2 — multipart 上传适配**
- 从 api.at endpoint 参数推导文件参数（`File`/`binary` 类型），生成上传函数；
- 补齐 `RequestBuilder.multipart_file(field, path)` / `multipart_text(field, value)` 链式生成
  （对齐 VM `shim_request_builder_send` 的 multipart 优先逻辑）。

**W3 — 文件下载 + 断点续传 + 进度**
- 生成 `download(url, file_path)` / `download_resume(url, file_path, offset)`（Range header）/
  `download_with_progress(url, file_path)`（进度迭代器）。
- 对齐 VM native（`http.download` / `http.download_resume` / `http.download_with_progress`）。

**W4 — WebSocket 客户端 codegen**
- 生成 `ws_connect/ws_send/ws_on_message/ws_close`（tungstenite）。
- 对齐 VM `ws.connect/send/on_message/close`（Plan 350）。

**W5 — 测试**
- VM 侧：TLS skip_verify / multipart upload / download+resume / progress / WS echo 用例
  （`plan349_tests.rs` 或扩展现有 VM 测试，验证 native 行为基线）。
- a2r 侧：生成代码编译+运行验证（TLS client 生成、upload/download/ws 函数生成）；
  `auto run --render rust` 对示例项目（015-notes 或专用 fixture）编译通过；
  文本黄金/包含性断言（生成的 Rust 含 TLS/multipart/download/ws 代码）。

## §4 Verification / 验证

- `cargo test -p auto-man`（如有）全绿；`cargo test -p auto-lang` 不退化。
- 生成项目 `cargo build` + `cargo run` 通过；行为与 VM 对应 native 一致。
- 文本断言：生成的 Rust 源码包含 TLS/multipart/download/ws 函数。

## §5 风险与缓解

| 风险 | 缓解 |
|---|---|
| ureq → reqwest 迁移破坏现有生成 UI 编译 | W1 先改 codegen + 生成 fixture 编译回归；保留 fire-and-forget 线程模式 |
| reqwest blocking 在 UI 主线程阻塞 | 现状已是阻塞（ureq）；异步化是步骤 7 单独计划 |
| WS 依赖 native-tls 平台差异 | 已声明 `tungstenite = { features = ["native-tls"] }`；测试用本地 echo server |

## §6 关联 / References

- **Plan 349**（active，roadmap）：VM 侧已完成；本计划的 a2r 部分形式化
- **Plan 350**（archive）：WebSocket VM 侧实现
- **Plan 344**：普通 HTTP 异步化设计（步骤 7，本计划 out of scope）
- **Plan 328**（archive）：a2r HTTP server 架构（AxumGenerator）

---

## §7 实施进度

> 计划骨架 commit 后开始 W1。各 WI 独立可合并，按依赖排序。
> house style 用 `### W1 — 待办` / `### W1 — landed` 记录。
