# 协议与 stdio 传输

## 范围

`mcp/protocol.rs` 的 JSON-RPC 2.0 / MCP 握手子集与行分隔 stdio 传输；
`mcp/server.rs` 的请求路由。这是本模块对外的唯一协议面。

## 原则

- 手写最小实现，不依赖 rmcp 等框架（architecture.md ADR-01）。
- 输出一切机器可读：工具结果一律是 JSON 字符串包在 text ContentBlock 里，
  不需要任何文本刮取。
- 日志只走 stderr（`eprintln!`），stdout 专属于协议帧。

## 细节

### 帧格式

- 行分隔 JSON：一行一个完整 JSON-RPC 消息；空行跳过（`read_message`
  递归读取下一行）。
- 不实现 MCP 规范的 `Content-Length` 帧头，也没有 HTTP/SSE 传输。
- EOF（读到 0 字节）即服务器主循环退出条件。

### 消息类型

- `JsonRpcRequest { jsonrpc, id: Option<Id>, method, params }`，
  `Id` 为 untagged 的 `Number(i64) | String(String)`。
- `JsonRpcResponse { jsonrpc, id?, result?, error? }`，`Option` 字段
  序列化时省略；`JsonRpcError { code, message, data? }`。
- 构造走 `JsonRpcResponse::success` / `::error` 两个工厂。

### MCP 握手与路由（server.rs:handle_request）

| method | 行为 |
|---|---|
| `initialize` | 置 `initialized=true`，返回 `InitializeResult`：protocol_version `2024-11-05`，capabilities 只声明 tools，server_info `{name: "autovm", version: "0.1.0"}` |
| `notifications/initialized` | 客户端确认，返回空 result（注意：通知本无 id，这里仍回一帧） |
| `ping` | 返回 `{}` |
| `tools/list` | 返回 `tool_definitions()` 的 7 个 `ToolDefinition{name, description, input_schema}` |
| `tools/call` | 解析 `ToolCallParams{name, arguments}` → `dispatch_tool`；参数解析失败返回 `ToolResult::error` |
| 其它 | `-32601 Method not found` |

### 工具结果 schema

`ToolResult { content: Vec<ContentBlock>, is_error: Option<bool> }`，
`ContentBlock` 只有 `Text{text}` 一种变体。所有工具把结构化 JSON
`serde_json::to_string` 后塞进 text 字段——即"文本块里装 JSON"。

### 不变量

- 每个请求恰好产生一行 stdout 响应（`run` 循环 handle 后立即 flush）。
- 工具级错误（会话不存在、缺参数）走 `ToolResult::error`（is_error=true），
  协议级错误（未知方法）走 JSON-RPC error——两层不混用。

### 已知弱点（与代码一致，非计划）

- 单行 JSON 解析失败时 `read_message` 返回 `None`，被主循环当成 EOF，
  服务器直接退出——一条坏消息就能杀掉进程。
- `notifications/initialized` 按通知语义不该有响应，实现却回了一帧
  空 result（id=None）；客户端宽容则无碍。

## 显式非目标

- MCP resources / prompts / sampling（plan-265 列为 Future Extensions）。
- Content-Length 帧、HTTP/SSE/WebSocket 传输（远程访问属 plan-265 的
  Future Extensions，未立项）。
- 协议版本协商：固定回 `2024-11-05`，不看客户端请求的版本。

> 来源: `crates/auto-lang/src/mcp/protocol.rs`、`crates/auto-lang/src/mcp/server.rs`、`docs/plans/old/265-autovm-mcp-server.md`
