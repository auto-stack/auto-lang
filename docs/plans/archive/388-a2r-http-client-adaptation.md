---
plan: 388
title: a2r-http-client-adaptation
affects: [auto-man, auto-lang/a2r, a2r-std]
status: complete # draft | in-progress | complete
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

- `cargo test -p auto-man` 全绿（182 测试，含 7 个 W1-W4 字符串级回归单测）。
- 端到端：015-notes split 模式（`AUTO_VM_MERGE=0`）生成的 API client 在临时 crate
  （reqwest blocking+json+multipart + tungstenite + lazy_static）编译通过；
  运行时冒烟（mock HTTP server + tungstenite echo）：multipart 链式上传 ✅、
  download_with_progress 进度事件+文件完整 ✅、ws echo（send→on_message）✅。
- 文本断言：生成的 Rust 源码含 `_http_client()`/TLS 接线、`multipart_form`、
  `download_with_progress`、`ws_connect/send/on_message/close`，无 `ureq::` 残留。

## §5 验收（Acceptance）

- [x] W1 TLS 适配：API client 从 ureq 迁移到 reqwest::blocking；`AUTO_TLS_SKIP_VERIFY` /
      `AUTO_TLS_CA_CERT` 生效；生成代码编译通过（015-notes split 模式）。
- [x] W2 multipart 链式 builder（`multipart_form().text().file().send()`）运行时验证通过。
- [x] W3 `download_with_progress` 流式进度事件 + 文件完整（100KB / 14 事件）运行时验证通过。
- [x] W4 `ws_on_message` 帧投递 + 读超时死锁修复，echo 端到端通过。
- [x] W5 回归单测（7 个）+ 生成代码编译/运行时验证；auto-man 182 全绿。
- 后续（roadmap 步骤 7/8，本计划 out of scope）：普通 HTTP 异步化（Plan 344 路径 B）、
  Cookie/重试/压缩/CORS 易用性。

## §6 风险与缓解

| 风险 | 缓解 |
|---|---|
| ureq → reqwest 迁移破坏现有生成 UI 编译 | W1 先改 codegen + 生成 fixture 编译回归；保留 fire-and-forget 线程模式 |
| reqwest blocking 在 UI 主线程阻塞 | 现状已是阻塞（ureq）；异步化是步骤 7 单独计划 |
| WS 依赖 native-tls 平台差异 | 已声明 `tungstenite = { features = ["native-tls"] }`；测试用本地 echo server |

## §7 关联 / References

- **Plan 349**（active，roadmap）：VM 侧已完成；本计划的 a2r 部分形式化
- **Plan 350**（archive）：WebSocket VM 侧实现
- **Plan 344**：普通 HTTP 异步化设计（步骤 7，本计划 out of scope）
- **Plan 328**（archive）：a2r HTTP server 架构（AxumGenerator）

---

## §8 实施进度

> 计划骨架 commit 后开始 W1。各 WI 独立可合并，按依赖排序。
> house style 用 `### W1 — 待办` / `### W1 — landed` 记录。

### W1 — landed（TLS 适配，ureq → reqwest::blocking）
- 交付（`crates/auto-man/src/rust_ui.rs`，commit `108b40bb`）：
  - `generate_http_client_helper` 新增 `_http_client()`：`OnceLock` 缓存
    `reqwest::blocking::Client`，读 `AUTO_TLS_SKIP_VERIFY`（danger_accept_invalid_certs）/
    `AUTO_TLS_CA_CERT`（Certificate::from_pem + add_root_certificate）。
  - `generate_get_fn_body` / `generate_delete_fn_body` / `generate_write_fn_body`：
    `ureq::X(url)` → `_http_client().X(url).send()`；`.send_json(body)` → `.json(&body).send()`；
    `.into_json::<T>()` → `.json::<T>()`。
  - **URL 统一 `.to_string()`**：无路径参数时 `url_expr` 是字面量 → `let url = "..."` 得到 `&str`，
    `&url` 是 `&&str`，不满足 reqwest `IntoUrl`（ureq 的 `AsRef<str>` 接受，reqwest 不接受）。
  - 生成 Cargo.toml 模板 reqwest features 加 `"json"`（`Response::json()`/`RequestBuilder::json()` 必需，
    旧模板只有 blocking+multipart）。
- 验证：
  - 3 个 W1 单测（helper TLS 接线 / GET 各返回类型 / write+delete 无 `ureq` 残留）全绿。
  - **端到端**：015-notes split 模式（`AUTO_VM_MERGE=0`）生成的 API client 在临时 crate
    （reqwest blocking+json+multipart + tungstenite + lazy_static）`cargo build` 通过。
- 顺带确认：`generate_http_utility_functions`（upload/download，W2/W3 模板）与
  `generate_ws_functions`（W4 模板）已存在，随 W1 一起编译通过。

### W2 + W3 + W4 — landed（multipart 链式 / 下载进度 / WebSocket on_message）
- 交付（`crates/auto-man/src/rust_ui.rs`，commit 待填）：
  - **W2**：`generate_http_utility_functions` 新增链式 builder
    `multipart_form().text(field, value).file(field, path).send(url)`（`MultiPart` struct），
    对齐 VM `RequestBuilder.multipart_file/multipart_text`。原有 `upload_file`/
    `upload_file_with_fields` 保留。
  - **W3**：新增 `download_with_progress(url, file_path) -> mpsc::Receiver<Value>`——
    独立线程 + `copy_to` 流式写盘（非 buffer-all）+ `ProgressWriter` 发进度事件
    （`{"done": bool, "written", "total"}`），对齐 VM `http.download_with_progress`
    的非阻塞迭代器。`download_file`/`download_file_resume` 保留。
  - **W4**：`generate_ws_functions` 补 `ws_on_message(handle) -> Vec<String>`（非阻塞
    drain 每连接入站队列）；`WsConn` 增加入站 `Receiver`；reader 线程把 Text/Binary
    帧真实投递到队列（原来直接丢弃）。**修复死锁**：阻塞 `read()` 会让出站 channel
    永不被轮询（echo 会话互等）——加 50ms 读超时（`MaybeTlsStream::Plain`），超时/
    WouldBlock 视为等待而非断连。修掉未用的 `Arc` 导入。
- 验证：
  - 4 个字符串级单测（multipart 链式 / progress / ws 四函数+投递 / 无 Arc 残留）全绿；
    auto-man 全套 182 测试通过。
  - **端到端运行时冒烟**（015-notes split 模式生成代码 + 临时 crate，`include!`
    模拟真实生成布局）：mock HTTP server + tungstenite echo server——
    `multipart_form().text().file().send()` 服务端收到正确 multipart body ✅；
    `download_with_progress` 100KB 下载 14 个进度事件 + 文件完整 ✅；
    `ws_connect → ws_send → ws_on_message` 收到 `echo:ping` ✅；**ALL PASS**。

### W5 — landed（验证基建，见各 WI）
- W1/W2/W3/W4 的字符串级回归单测内嵌 rust_ui.rs tests 模块（7 个 W 测试）；
  端到端编译+运行时验证通过临时 crate 完成（不入库，`#[ignore]` 级重型验证）。
- 生成 Cargo.toml 模板 reqwest features 补 `"json"`（W1 起生效）。

---

## §9 归档前复审（2026-08-05）

对全部 W1-W4 代码 + 生成模板的三路审计（codegen / 生成代码可编译性 / 运行时行为），
发现并修复以下问题（commit 待填）：

### §9.1 修复的 workaround / 遗漏

| # | 问题 | 位置 | 修复 |
|---|---|---|---|
| R1 | **W4 读超时只覆盖 `ws://`（Plain 变体）**——`wss://` 仍是阻塞 `read()`，出站 channel 死锁依旧 | ws 模板 `socket.get_mut()` 分支 | 改用 `socket.get_ref()` + match Plain/NativeTls 统一设 50ms 读超时；`_` 通配满足 `#[non_exhaustive]`（rustls 启用时也编译通过，不设超时——残余限制，已注释） |
| R2 | **W2/W3 的 upload/download 用裸 `Client::new()`/`blocking::get`**——忽略 `AUTO_TLS_*`，与 W1 的 TLS-aware 端点不一致（自签证书环境上传/下载会失败） | 6 处 utility 函数 | 全部改用 `_http_client()` |
| R3 | **W4 `ws_close` 未真正关闭连接**——只断 send 路径，reader 线程持有 socket 继续读 | `ws_close` | `WsConn` 加 `close: Arc<AtomicBool>`；`ws_close` 置位 → reader 循环检测后退出 → socket drop 真正关闭连接 |
| R4 | **ureq 死依赖残留**——迁移后生成 Cargo.toml 两个模板仍带 `ureq` | 模板 line ~1641/1857 | 移除；同步更新 "ureq" 过时注释 |
| R5 | `unwrap_or_default()` 静默回退未说明 | `_http_client()` | 注释：builder 失败时回退更严格（验证保持开启/CA 缺失→连接报错），绝不更弱 |

### §9.2 记录的设计偏差（非 bug）

- **W2 endpoint 驱动上传未做**：api.at 的 `ApiParam` 无文件参数类型（`api/types.rs:178`），
  按 349 roadmap 的"或生成通用 fn upload"分支实现（通用函数 + 链式 builder），
  非逐 endpoint 生成。需 endpoint 文件参数时后续 plan 先扩展 api.at 类型。
- **`_http_client()` 每次调用克隆**：`OnceLock` 缓存 + `Client` Arc 克隆，每请求一次 clone，
  开销可忽略；避免生命周期/线程安全复杂度。
- **ws_on_message 用 `Vec<String>` drain**：非阻塞逐次取帧；消息未被及时 drain 时
  `mpsc::channel` 无界累积（UI 轮询场景可接受，文档注明）。

### §9.3 验证

- 8 个 W 测试全绿（W1×3 + W2 + W3 + W4×2，含 R1-R3 守护断言）。
- 015-notes split 模式生成代码编译 + 运行时冒烟**ALL PASS**（新增 ws_close 断言：
  `ws_send` 关闭后返回 false + echo server 检测对端 socket 关闭）。
- auto-man 全套 182 测试通过。
