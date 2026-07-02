# Plan 353: WebSocket 双向通讯

> **状态**：设计文档 / TODO
> **优先级**：🟡 中
> **难度**：高
> **依赖**：Plan 348（非阻塞 yield 机制复用）、Plan 350（WSS over TLS）

## 背景

WebSocket 补全了 HTTP 请求-响应模型之外的实时双向通讯能力。适用场景：
- 实时聊天 / 协作编辑
- 实时数据推送（股票行情、游戏状态）
- 双向控制（远程终端、IoT 设备控制）

当前 HTTP 库有 SSE（服务端→客户端单向流），但没有双向通讯。

## 方案

### Client 侧

#### `ws.connect(url: String) -> ws_handle`
建立 WebSocket 连接。返回 handle（i64，存入 `WS_CONNECTIONS` registry）。
- 支持 `ws://` 和 `wss://`（TLS）
- 内部用独立线程跑 `tungstenite` 异步连接
- 连接成功后 handle 可用于 send / on_message

#### `ws.send(handle: i64, message: String) -> bool`
发送文本消息。阻塞式（独立线程发送）。

#### `ws.send_binary(handle: i64, data: []byte) -> bool`
发送二进制消息。

#### `ws.on_message(handle: i64) -> iterator_id`
消息消费迭代器。每次 yield 一条收到的消息。
- 复用 Plan 348 的非阻塞 yield 机制
- 独立线程从 WebSocket 读消息 → channel → shim_iterator_next 非阻塞拉取
- 连接关闭时迭代结束（push -1）

用法：
```auto
let conn = ws.connect("wss://api.example.com/realtime")
ws.send(conn, "Hello, WebSocket!")
for msg in ws.on_message(conn) {
    print("Received: " + msg)
}
```

#### `ws.close(handle: i64)`
主动关闭连接。

### Server 侧

#### `http.server_ws(path: String, handler: fn)`
注册 WebSocket 路由。handler 接收一个 `ws_session` 参数，可 send/on_message。

```auto
#[ws(path = "/ws/chat")]
fn chat_handler(session: ws_session) {
    for msg in session.on_message() {
        session.send("Echo: " + msg)
    }
}
```

### 实现要点

1. **依赖引入**：`tungstenite`（同步 WebSocket 库）或 `tokio-tungstenite`（异步）。推荐 `tungstenite`（简单，适合独立线程模式）。
2. **WS_CONNECTIONS registry**：`thread_local` 或 `lazy_static Mutex<HashMap<i64, WsConnection>>`。
3. **WsConnection 结构**：
   ```rust
   struct WsConnection {
       sender: std::sync::Mutex<Option<Sender>>,  // tungstenite Writer
       receiver_channel: mpsc::Receiver<WsMessage>,
       done: AtomicBool,
   }
   ```
4. **非阻塞消息消费**：复用 Plan 348 的 `AsyncHttpStream` + `waiting_sse_stream_id` 机制。
5. **Server WebSocket**：在 `serve_async` 的 TCP accept 循环里检测 WebSocket upgrade 请求（`Upgrade: websocket` header），用 `tungstenite::accept` 升级连接。

### 数据流

```
ws.connect(url)
  → spawn 独立线程
  → tungstenite::connect(url)
  → 循环读消息 → channel 推送
  → 返回 handle

for msg in ws.on_message(handle)
  → shim_iterator_next 从 channel try_recv
  → 有数据 → push 消息字符串
  → 无数据 → Waiting("ws") + yield（Plan 348 非阻塞）
  → run_task_loop 检查 channel → 唤醒

ws.send(handle, msg)
  → 通过 Sender 发送（同步，快速）
```

### 关键文件
- `crates/auto-lang/src/vm/ffi/websocket.rs` — 新模块，WsConnection + 5 个 shim
- `crates/auto-lang/src/vm/ffi/stdlib.rs` — 注册 native
- `crates/auto-lang/src/vm/ffi/http_server.rs` — serve_async 加 WebSocket upgrade
- `crates/auto-lang/src/vm/native_catalog.rs` — 注册新 native ID
- `crates/auto-lang/Cargo.toml` — 加 `tungstenite` 依赖

### 用法示例

#### Client（实时聊天）
```auto
let conn = ws.connect("wss://chat.example.com/ws")
for msg in ws.on_message(conn) {
    print("Friend: " + msg)
    ws.send(conn, "Got it!")
}
ws.close(conn)
```

#### Server（Echo WebSocket）
```auto
#[ws(path = "/ws/echo")]
fn echo_handler(session) {
    for msg in session.on_message() {
        session.send("Echo: " + msg)
    }
}
```

## 不在范围
- 心跳/keepalive（应用层 ping/pong）—— 可后续加
- 自动重连 —— 可后续加
- WebSocket 压缩（permessage-deflate）
- 多路复用（一个连接跑多个频道）
