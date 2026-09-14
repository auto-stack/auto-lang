# AutoUI MCP 测试夹具协议

## 状态与范围

本设计服务 PLAN-623，定义 AutoUI MCP 的测试专用运行时状态夹具。
它只服务 VM 轨的确定性测试，不是应用功能，也不改变 Vue HTTP 业务链路或
Rust 组件事件模型。

夹具的唯一启用条件是进程环境中的 `AUTOUI_TEST_FIXTURES=1)。默认关闭时，
工具请求在 MCP 线程被拒绝并且不会进入动作队列。

## 设计目标

1. 测试可以注入当前 AutoUI 组件已经声明的状态字段，包括 VM 数组字段。
2. 注入结果在 iced/VM 更新线程完成后才返回 `applied`。
3. 可选的 `trigger` 在状态写入成功后、同一个更新周期内进入普通 VM handler
   入口，避免用固定 sleep 猜测时序。
4. 同一请求在 VM merged 与 no-merge 启动方式下使用相同语义。
5. Rust MCP 明确返回不支持，普通 action/keyboard/state 工具零回归。
6. 框架不包含任何 Tetris 字段名；业务夹具由调用方按 `autoui_state` 提供。

## 请求契约

```json
{
  "schema_version": 1,
  "state": {
    "board": [0, 0, 0],
    "piece": 0,
    "rotation": 1,
    "px": 7,
    "py": 16,
    "pending_lock": true,
    "phase": "playing"
  },
  "trigger": {
    "widget": "TetrisStore",
    "event": "Tick",
    "input": null
  }
}
```

- `schema_version` 必须为整数 1。
- `state` 必须为非空 JSON object。
- 字段名必须存在于当前 MCP session 的 state snapshot。
- JSON 值递归支持 null、bool、整数、浮点、字符串、数组和 object。
  数组写回使用字段现有的 VM 数组载体，不把 heap array id 当普通整数写入。
- 输入值的 JSON 类型必须与当前字段兼容；整数不能静默截断超出 i32 的数值。
- `trigger` 可省略。存在时只含 widget、event、可选 input 字符串；它只能
  调用已有 handler，不提供代码或表达式执行能力。
- 请求体大小、字段数量、递归深度和数组元素数量受固定上限约束。

响应：

```json
{
  "status": "applied",
  "request_id": 17,
  "changed": ["board", "piece", "rotation", "px", "py", "pending_lock", "phase"],
  "trigger": "TetrisStore.Tick"
}
```

失败响应使用 MCP 的 `isError=true`，文本中包含稳定错误类别：
`fixtures_disabled`、`backend_unsupported`、`invalid_schema`、
`unknown_field`、`type_mismatch`、`write_failed`、`trigger_failed`、
`ack_timeout`。

## 线程与消息模型

AutoUI MCP 线程分配单调递增 request id，构造：

- `ActionTarget::Fixture { request_id }`
- `ActionMessage.value = Some(serialized_fixture_payload)`

VM 的 `poll_mcp_actions` 将 Fixture 转成保留事件
`__mcp_fixture|<request_id>|<payload>`。该事件在 `update_inner` 的普通
`on_with_input_for` 之前消费。

消费顺序：

1. 解析并校验整个 payload。
2. 读取当前字段形态，确认所有字段可写；校验失败不触发 handler。
3. 按字段写入：数组字段走 `read_state_as_vec` /
   `write_state_vec`，其他字段走 `write_state`。
4. 设置 per-App `view_dirty`。
5. 如存在 trigger，使用现有 `on_with_input_for` 或 timer 入口派发。
6. 通过该 App 的 `mcp_shared` 写入 request-id ack。
7. MCP 工具在有界期限内轮询 ack；收到 ack 才返回 applied/error。

ack 表只保留未完成请求及最近有限数量的结果，消费后删除；超时请求不会
阻塞后续 MCP 操作。renderer 不直接持有 HTTP/MCP 锁等待工具线程，避免死锁。

Rust 的 `devtools_subscription` 对 Fixture 目标丢弃并记录不支持；MCP 工具
在提交前依据 SharedState backend capability 返回 `backend_unsupported`，
因此正常情况下不会进入 Rust DevTools 队列。

## 安全边界

- `AUTOUI_TEST_FIXTURES=1` 缺失时不发送任何 ActionMessage。
- capability 由 renderer 启动时写入 SharedState（VM 或 Rust），不依赖调用方
  自行声明的环境变量。
- 夹具只写当前运行时状态，不触发 storage/persistence API。
- HTTP server 仍使用现有本机监听配置；本设计不开放远程 fixture 端口。
- fixture 工具不出现在应用 VTree、action 列表或任何可见 UI 中。

## VM 后端兼容性

merged 和 no-merge 共用同一个 VM renderer 消费臂。no-merge 的业务 handler
仍可通过既有 HTTP backend 执行；fixture 写入本地 VM state 后再触发 handler，
因此测试可以分别验证状态注入与后端调用，不把 HTTP 结果伪装成 fixture ack。

## Tetris 移交

Plan 005 在本协议落地后提供一份 case 数据，至少覆盖 opening/lock、7×4
旋转、1–4 行消除。每个 case：

1. 用 `autoui_fixture` 写入 board、piece、rotation、px、py、pending_lock、
   phase；
2. 等待 applied ack；
3. 用 trigger Tick 或现有键盘/动作入口推进；
4. 用 `autoui_state` 读取 board、score、lines、phase、feedback；
5. 与 Rust rules golden 的同 case 结果逐项比较。

这一步留在 auto-os Plan 005，不把业务字段写入框架协议。

