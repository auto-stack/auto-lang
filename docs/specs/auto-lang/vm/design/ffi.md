# Native 与 FFI

## 范围

native 函数注册体系、Rust/C FFI 动态加载、标准库 shim。对应代码：`vm/native.rs`、`vm/native_registry.rs`、`vm/native_catalog.rs`、`vm/qualified_name.rs`、`vm/ffi/`（mod.rs、c_ffi.rs、rust_stdlib.rs、stdlib.rs、convert.rs、http_server.rs、websocket.rs、c_bindings/）。

## 原则

- 单一注册：编译期元数据与运行时 shim 同源，消除双注册表（ADR-09）。
- 限定名解析：函数以 `QualifiedName` + import scope 解析，不留短名别名（plan-203）。
- FFI 错误经 `VMError::FFI`（engine.rs:182）统一上报。

## 细节

### Native 注册体系

- `AutoVMNativeRegistry`（native_registry.rs:42）：编译期全局注册表，存函数 ID 与返回类型（`NativeRetType`，native_registry.rs:31）；`register_builtin_natives()`（native_registry.rs:455）集中注册内置函数。
- `NativeInterface`（native.rs:33）：运行时 shim 表，CALL_NAT 按 ID 派发。
- plan-249 落地单一注册架构：惰性注册 + catalog 宏（native_catalog.rs），消除历史双注册表（BIGVM_NATIVES vs shim registry）。
- plan-198 使 native 元数据从 `#[vm]` 源声明派生，消除硬编码。
- plan-203 引入 `QualifiedName`/`resolve_qualified`/import scope，消除约 137 个短名别名，单态派发随之重构（Phase 5f 仍 deferred）。

### Rust FFI

- 混合桥：`#[rust_fn]` 宏声明 shim，plan-094 完成 43 个 shim；plan-092 建立沙箱约定。
- 动态加载端到端：依赖（如 serde_json）→ `cargo build` cdylib → AutoVM `load` .dll → 调用（plan-212b，MVP 为 string→string）。
- `ffi/rust_stdlib.rs` 为 Rust 实现的标准库函数提供 shim 注册。

### C FFI

- `CFfiRuntime`（ffi/c_ffi.rs:19）基于 libloading 装载动态库；`load_builtin_manifest`（c_ffi.rs:449）从 C 头文件生成绑定清单。
- plan-216 把 auto-bindgen 接入 CLI 构建管线（4 个阶段完成）：头文件 → 绑定 → 编译 → VM 调用。
- `ffi/convert.rs` 负责 Value ↔ C ABI 类型转换；`ffi/error.rs` 定义 FFI 错误面。

### 内置服务 shim

- `ffi/http_server.rs`、`ffi/websocket.rs` 把 HTTP/WebSocket 服务暴露为 native 函数（plan-312/313/349/350 系列）；任务挂起配合 `waiting_http_request_id`/`waiting_sse_stream_id`（见 concurrency.md）。

## 显式非目标

- 多语言 FFI 插件系统（`Plugin` trait、handle 表生命周期）：design/05 Open Questions，未实现。
- Python FFI 运行时（CALL_PY 等）：属 python 集成线（plan-300/369），不在本主题展开。

> 来源: docs/plan-reports/07-vm-runtime.md（Plan Index 198/203/212b/216/249 行）；crates/auto-lang/src/vm/{native,native_registry,native_catalog,qualified_name}.rs、ffi/ 目录；plan-092/094/198/203/212b/216/249
