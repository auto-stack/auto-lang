# Plan 154: 真正的 HTTP 流式实现

**Status**: Completed
**Created**: 2026-04-01
**Completed**: 2026-04-01
**Priority**: High (Plan 153 AI Agent 阻塞性依赖)

## 目标

将 Plan 152 的占位 HTTP 流式实现替换为真正的流式读取，使用 `reqwest::blocking` 实现逐 chunk 读取 HTTP 响应。

## 背景

Plan 152 标记为 "Completed"，但 HTTP 流式功能是占位实现：
- `shim_http_stream_next()` 总是立即返回 `"[DONE]"`
- `HttpStreamData` 不存储 `reqwest::Response`
- `simple_http_request()` 使用原始 `std::net::TcpStream`，一次读完所有数据

## 方案

使用 `reqwest::blocking`（workspace 已有依赖），因为它：
1. 自带内部 tokio runtime，不与 VM 的全局 runtime 冲突
2. `blocking::Response` 实现 `std::io::Read`，可逐块读取
3. 在 FFI shim 内同步调用，无需改变 VM 调度架构

## 实施总结

### 修改的文件

| 文件 | 变更 |
|------|------|
| `crates/auto-lang/Cargo.toml` | reqwest 改为非 optional，features=["blocking"] |
| `crates/auto-lang/src/vm/ffi/stdlib.rs` | HttpStreamData + 3个 shim 函数重写 |

### 关键变更

1. **HttpStreamData**: 存储 `reqwest::blocking::Response`，添加 `status_code` 字段
2. **shim_http_get_stream**: 使用 `reqwest::blocking::Client::get().send()` 发起请求
3. **shim_http_post_stream**: 使用 `reqwest::blocking::Client::post().body().send()` 发起请求
4. **shim_http_stream_next**: 使用 `std::io::Read::read()` 逐 8KB 块读取响应

### 验证结果

- `cargo build -p auto-lang` 编译通过 ✅
- SSE 测试全部通过 (19/19) ✅
- 创建 `tmp/demo_real_stream.at` 测试脚本

### 后续工作

- Plan 153 可以使用此真实流式实现来调用 LLM API
- 未来可实现 `AWAIT_EXT` opcode 实现真正的非阻塞 I/O
