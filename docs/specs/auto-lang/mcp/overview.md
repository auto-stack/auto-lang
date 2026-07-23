# mcp（AutoVM MCP Server）

> **Status**: implemented（plan-265 落地，核心 7 工具可用；sandbox/会话 GC/诊断建议为半完成状态）

## 职责

把 AutoVM 包装成 MCP（Model Context Protocol）server，让 AI agent 通过
stdio 上的 JSON-RPC 2.0 创建隔离 VM 会话、执行/校验 Auto 代码、增量补丁
定义并导出会话源码。与人类 REPL 并列，共享同一个 `AutovmReplSession` 核心
（plan-265 的"dual-mode VM"原则）。

注意边界：仓库里还有第二个 MCP server——AutoUI MCP
（`crates/auto-lang/src/ui/mcp_server.rs`，plan-278/299/314），它嵌在 iced
桌面进程里、走 HTTP、操作运行中的 UI，与本模块（操作源代码/VM 会话）是两个
独立实现，归 ui 模块管。本目录只覆盖 `crates/auto-lang/src/mcp/`。

## 现状

- 传输：行分隔 JSON-RPC 2.0 over stdio；实现 MCP 握手子集
  （`initialize` / `notifications/initialized` / `ping` / `tools/list` /
  `tools/call`），协议版本 `2024-11-05`，server 名 `autovm`。
- 工具（7 个，均在 `server.rs:tool_definitions` 注册）：
  `auto_session_create` / `auto_evaluate` / `auto_session_reset` /
  `auto_inspect` / `auto_typecheck` / `auto_patch` / `auto_snapshot`。
- CLI 入口：`auto mcp`（`crates/auto/src/main.rs` 的 `Commands::Mcp` 分支，
  ~1547 行），构造 `McpServer` 后阻塞在 stdin 循环。
- `SessionManager` 同时被 `autovm_daemon`（plan-269）复用——会话 GC
  （`cleanup_expired`）目前只有 daemon 路径会调用。
- 测试：`src/mcp/tests.rs` 只有占位注释，模块无单测。

## 关键入口

- `crates/auto-lang/src/mcp/server.rs:McpServer` — JSON-RPC 路由 + 工具分发
- `crates/auto-lang/src/mcp/server.rs:McpServer::dispatch_tool` — 7 工具分发表
- `crates/auto-lang/src/mcp/server.rs:patch_replace_definition` — auto_patch 的文本替换算法
- `crates/auto-lang/src/mcp/protocol.rs:read_message` / `write_response` — stdio 传输
- `crates/auto-lang/src/mcp/protocol.rs:ToolResult` — 工具结果 schema（text ContentBlock）
- `crates/auto-lang/src/mcp/session_manager.rs:SessionManager` — 会话生命周期 + 源码累积
- `crates/auto-lang/src/autovm_persistent.rs:AutovmReplSession` — 底层持久 VM 会话（复用核心）
- `crates/auto/src/main.rs`（`Commands::Mcp` 分支，~1547 行）— `auto mcp` CLI 入口

## 使用示例

`.claude/settings.json`（plan-265 给出的接入方式）：

```json
{ "mcpServers": { "autovm": { "command": "auto", "args": ["mcp"] } } }
```

典型会话：`auto_session_create` → `auto_evaluate`（定义+求值，成功后源码累积进
history）→ `auto_patch`（按名替换单个定义并重建会话）→ `auto_snapshot`（导出
完整 .at 源码）→ `auto_session_reset { action: "delete" }`。

## 已知坑

- `sandbox` 标志只存不生效：`VmSession.sandbox` 标了 `#[allow(dead_code)]`
  （session_manager.rs），没有任何文件 I/O / 网络限制逻辑。
- `auto_typecheck` 名不副实：只做语法解析（`parse_preserve_error`）+ 符号清单，
  不做类型推断；plan-265 设想的 infer 集成未落地。
- MCP stdio 模式无会话 GC：`cleanup_expired` 仅 `autovm_daemon` 调用，
  `McpServer::run` 循环从不清理空闲会话。
- 诊断结构单薄：只有 `severity + message`，没有 plan-265 设计的
  `code`/`span`/`suggestions`。
- `auto_patch` 是文本行匹配 + 花括号计数，不解析 AST——注释或字符串里的
  花括号、非常规排版都可能导致替换范围错误。
- 一条 JSON 解析失败即当 EOF 退出服务器（`protocol.rs:read_message` 解析失败
  返回 `None`），容错性弱。
- 每次 `auto_evaluate` 成功后整段 code 追加进 `source_history`（含纯表达式），
  `auto_snapshot` 原样拼接，导出文件未必能直接过 a2r。

## 蒸馏来源（Phase 1）

- `docs/design/14-developer-tools.md`（MCP 节：AutoVM MCP Server / AutoUI MCP Server）
- `docs/plans/old/265-autovm-mcp-server.md`（本模块奠基 plan）
- `docs/plans/archive/314-autoui-mcp-styled-vtree.md`（AutoUI MCP 侧，交叉引用）
- 代码：`crates/auto-lang/src/mcp/`（mod/protocol/server/session_manager/tests）
