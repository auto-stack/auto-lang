# Plan 329: IPC SSE 支持 — Tauri Channel 流式推送

> **Status**: 设计完成,待实施
> **目标**: 让 Tauri IPC 的 #[api] handler 支持 SSE（~Iter/~Stream），用 Tauri 2 的 Channel 替代 HTTP SSE
> **关联**: Plan 328（a2r HTTP server + IPC 兼容）

## 设计

SSE handler 在 IPC 模式用 Tauri 2 的 `tauri::ipc::Channel`：
- command 接收 `on_event: Channel` 参数
- 通过 `on_event.emit(json!(item))` 推流
- 前端用 `Channel.onmessage` 接收

## 改动

1. generate_command：SSE 分支（async fn + Channel 参数 + for 循环 emit）
2. to_rust_type：~Iter/~Stream 提取内层类型
3. generate_full：use 头加 serde_json
4. generate_server_main：无改动（Channel command 注册同普通 command）
