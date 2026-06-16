# Plan 313: AutoVM TCP Flush + 服务端 SSE 推送

> **Status**: ✅ Phase 1-2 Delivered(2026-06-16)
> **依赖**: Plan 312(`#[api]` + VM HTTP server)—— SSE 分帧挂在 Plan 312 的 HTTP server 请求处理流程上。
> **方向澄清**: 本任务只补**服务端方向**(生产 SSE)。客户端方向(消费外部 LLM 的 text/event-stream)已实现(`http_stream.*` + `parse_sse`,stdlib.rs:2506-2814)。

---

## §1 背景与动机

auto-musk/auto-forge 的核心特性是**流式聊天**——后端保持长连接,把 LLM token 用 SSE 逐块推给前端(`/api/forge/chats/{sid}/stream`)。需 AutoVM 后端能作为 SSE 服务端推流。

### 1.1 当前状态(已验证)

| 组件 | 现状 | 证据 |
|---|---|---|
| **无 flush 原语** | `shim_net_tcp_stream_write_str` 用 `write_all`,无 flush | `stdlib.rs:1813-1828` |
| **全文无 TCP flush** | vm/ 下 flush 只有 CSV Writer no-op + 文件 IO | `stdlib.rs:4379`,`io.rs:221` |
| **read_line 每次新建 BufReader** | 多行连续读取丢字节 | `stdlib.rs:1795-1809` |
| **无 TCP_NODELAY** | accept/connect 后未设 nodelay,小包被 Nagle 缓存 ~40ms | `stdlib.rs:1658`,`1703` |
| **无服务端 SSE 辅助** | 全仓库零匹配 `text/event-stream`/`chunked`/`keep-alive` | vm/ 目录 |
| **accept 是阻塞单线程** | 无 epoll/多路复用,多客户端靠多线程 | `stdlib.rs:1639` |
| **唯一 TCP 示例是 echo** | `Connection: close`,不支持长连接 | `examples/http_server/server.at:16` |

### 1.2 两个关键隐患(验证中发现,非用户原始描述)

1. **`tcp_stream_read_line` 每次新建 BufReader**(`stdlib.rs:1801`):每次调用 `BufReader::new(stream)` 包装一次裸 stream,内部缓冲区在读到一行后**丢弃**,下次读会丢数据。服务端要逐行解析 HTTP 请求头(多行)时**不可用**。这是 SSE server 的前置依赖,必须先修。

2. **`TcpStream::flush()` 是 no-op**:socket 没有用户态缓冲,`std::io::Write::flush` 对 `TcpStream` 不做任何事。真正影响 SSE 推送时延的是 **Nagle 算法**(`TCP_NODELAY` 未设置),小数据包会被缓存最多 ~40ms。**单加 flush 原语解决不了 SSE 时延,必须配合 `TCP_NODELAY`。**

---

## §2 设计方案

### 2.1 三层改动

| 层 | 改动 | 依赖 |
|---|---|---|
| **L1: TCP flush + nodelay 原语** | 新增 `Net.tcp_stream_flush` + `Net.tcp_stream_set_nodelay`;修复 read_line BufReader 缓存 | 无 |
| **L2: 服务端 SSE 分帧** | 响应头 + 帧格式 + 长连接管理(Auto 层 helper + native 最小原语) | L1 |
| **L3: 与 Plan 312 集成** | `#[api]` handler 如何返回"流式"响应 | Plan 312 + L1/L2 |

### 2.2 flush 的真实语义

**关键澄清**:`TcpStream::flush()` 对 raw socket 是 no-op(std 的 `Write::flush` 对 TcpStream 不做任何事,因为 socket 无用户态缓冲)。要实现真实的"立即送达":

| 措施 | 效果 | 必须? |
|---|---|---|
| `stream.flush()` | no-op(但语义清晰,且将来加 BufWriter 时有用) | 提供 |
| `stream.set_nodelay(true)` | 关闭 Nagle,小包立即发送 | **必须**(SSE 时延的关键) |

**建议**:flush 原语 + `set_nodelay` 都做。flush 提供语义清晰性(API 完整性),`set_nodelay` 才是 SSE 低延迟的真正解法。可考虑 accept/connect 后**默认开启 nodelay**。

---

## §3 分阶段实施计划

### Phase 1 — TCP flush + nodelay + read_line 修复(P0,前置)

**目标**:提供 flush 原语,修复 BufReader 缓存缺陷,开启 nodelay。

**改动**:

| 文件 | 改动 |
|---|---|
| `stdlib.rs` 新增 `shim_net_tcp_stream_flush` | `stream.flush()`(语义 + 将来 BufWriter 兼容) |
| `stdlib.rs` 新增 `shim_net_tcp_stream_set_nodelay` | `stream.set_nodelay(bool)` |
| `stdlib.rs:1658` accept 后 | 默认 `set_nodelay(true)` |
| `stdlib.rs:1703` connect 后 | 默认 `set_nodelay(true)` |
| `stdlib.rs:1795-1809` read_line 修复 | BufReader 持久化存储(改 `TCP_STREAMS` value 类型,或单独维护 `HashMap<u64, BufReader<TcpStream>>`) |
| `native_catalog.rs` | 注册 `Net.tcp_stream_flush` + `Net.tcp_stream_set_nodelay` opcode |
| `stdlib.rs:3580-3591` | 注册 native |

**read_line 修复方案**(关键):`TCP_STREAMS` 的 value 从 `StdTcpStream` 改为包装类型:
```rust
struct TcpStreamEntry {
    stream: StdTcpStream,
    read_buf: Option<BufReader<StdTcpStream>>,  // 持久化 BufReader
}
```
或更简单:把 BufReader 存到独立的 thread-local `TCP_READERS: HashMap<u64, BufReader<StdTcpStream>>`。read_line 时优先从 reader 读,write 时从 stream 写(需 `try_clone()`)。

**验收**:
- `Net.tcp_stream_flush(handle)` 可调用,不报错
- `Net.tcp_stream_set_nodelay(handle, true)` 关闭 Nagle
- read_line 连续调用多次不丢字节(用 echo server 测试多行请求头)

### Phase 2 — 服务端 SSE 分帧(P0)

**目标**:提供 SSE 服务端发送能力(Auto 层 helper + native 最小原语)。

**SSE 协议**:
```
HTTP/1.1 200 OK\r\n
Content-Type: text/event-stream\r\n
Cache-Control: no-cache\r\n
Connection: keep-alive\r\n
\r\n
data: {"token":"hello"}\n\n
data: {"token":" world"}\n\n
data: [DONE]\n\n
```

**Native 最小原语**(不提供 SSE 帧格式 native,保持最小):
- 复用 Phase 1 的 `write_str` + `flush` + `set_nodelay`
- SSE 分帧逻辑在 **Auto 层**实现(标准库或示例)

**Auto 层 SSE helper**(新增 `stdlib/auto/sse_server.at`):
```auto
#[vm]
pub fn sse_send_event(stream int, data str) {
    Net.tcp_stream_write_str(stream, "data: " + data + "\n\n")
    Net.tcp_stream_flush(stream)
}

#[vm]
pub fn sse_send_named(stream int, event str, data str) {
    Net.tcp_stream_write_str(stream, "event: " + event + "\ndata: " + data + "\n\n")
    Net.tcp_stream_flush(stream)
}

#[vm]
pub fn sse_write_headers(stream int) {
    Net.tcp_stream_write_str(stream, "HTTP/1.1 200 OK\r\nContent-Type: text/event-stream\r\nCache-Control: no-cache\r\nConnection: keep-alive\r\n\r\n")
    Net.tcp_stream_flush(stream)
}
```

**验收**:最小 AutoVM SSE server(某端点定时推 `data: ...`);浏览器 EventSource 持续收到;验证 flush 实时生效(帧实时到达而非最后一次性发)。

### Phase 3 — 与 Plan 312 集成(后置,依赖 Plan 312)

**目标**:定义 `#[api]` handler 如何返回"流式"响应。

**方案**:两种推荐写法,选一种:

| 方案 | 写法 | 优势 |
|---|---|---|
| **A. 特殊返回类型** | `#[api] fn stream() ~sse { ... }` 返回 SSE 流类型,listen 循环检测到 ~sse 不关连接 | 声明式,清晰 |
| **B. sse 构造器 + 回调** | handler 拿到底层连接句柄,循环 `sse_send_event` + `flush` | 显式,灵活 |

**推荐 B**(sse 构造器),因为它不引入新返回类型语义,且与 Plan 312 的 handler 调用模型兼容(handler 拿到 `stream` 参数,循环写帧)。

**并发**:SSE 长连接占 handler 较久,用 `std::thread::spawn`(独立 OS 线程,见 Plan 312 §2.3),避免占满 spawn_blocking 池。确认不阻塞其他请求(每连接独立线程)。

**验收**:auto-forge 风格的 SSE 端点(`/api/forge/chats/{sid}/stream`),前端 EventSource 实时收到 token 流。

---

## §4 风险与缓解

| # | 风险 | 缓解 |
|---|---|---|
| 1 | **read_line BufReader 修复涉及 TCP_STREAMS 类型重构** | 影响面大(所有 TCP native 都访问 TCP_STREAMS)。用独立 `TCP_READERS` map 避免改 value 类型 |
| 2 | **accept 阻塞单线程,SSE 长连接占满** | Phase 3 用 `std::thread::spawn` 每连接一线程;或依赖 Plan 312 的 listen 循环 spawn 模型 |
| 3 | **nodelay 默认开启可能影响大批量传输性能** | 提供 `set_nodelay(false)` 让用户关闭;默认开启只针对 server accept 的连接 |
| 4 | **SSE 帧格式 Auto 层 helper 可能不够用**(event:/id:/retry:) | Phase 2 只提供 data + event 两帧;named event + id 留后续;保持最小 |

---

## §5 验收标准

1. `Net.tcp_stream_flush(handle)` 可调用(语义 flush)
2. `Net.tcp_stream_set_nodelay(handle, bool)` 关闭/开启 Nagle
3. read_line 连续多行读取不丢字节(BufReader 修复)
4. accept/connect 后默认 nodelay=true
5. 最小 AutoVM SSE server:浏览器 EventSource 持续收到帧,连接不断
6. flush 实时生效(帧实时到达,非最后一次性发)
7. SSE 长连接不阻塞其他请求(并发验证)

---

## §6 实现顺序建议

```
Plan 312 Phase 1-3(CRUD HTTP server)→ Plan 313 Phase 1(flush/nodelay/read_line)
→ Plan 313 Phase 2(SSE 分帧)→ Plan 312 Phase 4(启动方式)
→ Plan 313 Phase 3(SSE + #[api] 集成)
```

Plan 313 Phase 1 的 read_line 修复是 Plan 312 Phase 3 listen 循环的潜在前置(如果 listen 用 read_line 解析请求头)。但 Plan 312 可以用自己的请求解析逻辑(不依赖 read_line),两者解耦。建议 Plan 312 先行(不阻塞),Plan 313 Phase 1 独立做。
