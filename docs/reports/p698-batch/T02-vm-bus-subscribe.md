# T-02 轨B 勘定与全环录证（PLAN-698 / 2026-09-23）

## T-02a 勘定结论：bridge 案①（进程内 iterator 注册表）成立

实测拓扑（http_server.rs / native.rs）：
- VM server 服务 `~Stream` 端点的方式 = 调 handler 取**返回的 iterator_id**，
  再循环 `iterator.next` 拉值写 SSE 帧（serve_blocking_stdnet 与 serve_async
  双路径同构；696 cooperative 分片臂兼容）。
- 生成 Axum 侧（api_gen）：`events.rs` 模板 = `broadcast::channel(256)` 单总线、
  **无 topic 寻址**（`bus.subscribe()` 零参、`events::broadcast(json)`）。
- 既有 `Iterator::AsyncHttpStream` 臂（Plan 341）即"异步流→迭代器"适配器：
  非阻塞 try_recv、空时 Waiting("sse") 让出、Done/Error→done+-1（696 回收语义）。

三案裁定：案① 与上述三点完全同构（stub 签名零破坏、断连回收复用 696 臂）；
案②（直连 SSE URL）在进程内拓扑下是绕远；案③（内核 pubsub）无必要。
"topic 查无响亮失败"原案随之失效——bus 无 topic；响亮失败面保留为
iterator 查无 id 的既有行为。

## T-02b 落地

- `shim_bus_subscribe`：EVENT_BUS.subscribe() → 专属转发线程
  （`blocking_recv` → mpsc，镜像生成侧 `while let Ok` = 任何 RecvError 终止流）
  → ASYNC_STREAMS 注册 → `Iterator::AsyncHttpStream` 注册 → 压 iterator_id。
- publisher 臂 `publish_post_broadcast`（两个 serve 循环插桩，200 POST 后）：
  has_sse 门控（codegen 侧信道 `record_api_return_type` 判 ~Stream 端点存在）；
  Typing/New{Type} 形态与 api_gen broadcast_event_name 逐字段对齐。
- codegen 在 #[api] fn 编译时发布返回类型（`unique_name`——Display 对 User
  类型会吐整个 type-decl s-expr，实测发现后改用 unique_name）。

## AC-02 全环录证（017-chat，VM 臂）

命令序列：`auto run -r vm --server vm -B 18499`（AutoVM HTTP server，6 routes）
→ `curl -N /api/stream` 订阅 → POST /api/typing + /api/messages → SSE 帧：

```
data: "{\"event\":\"Typing\",\"name\":\"Alice\"}"
data: "{\"id\":5,\"sender\":\"Bob\",\"text\":\"hello bus\",\"time\":\"Just now\",\"mine\":true,\"event\":\"NewMessage\"}"
```

（帧全文见 ac02-vm-sse-frames.txt；首跑曾现
`New(type-decl (name Message) (members ...))` 缺陷——即 unique_name 根修动机。）

## UI 联动录证（vite 前端 + AUTO_BACKEND_IMPL=vm）

`auto run -B 18499 -F 5177`（vue 前端 + VM API 后端）+ 浏览器走查：
1. composer 输入即触发 "You is typing…"（输入事件→POST typing→VM bus→SSE→store→指示器）。
2. Send → "hello from VM bus e2e" 上屏（POST create + SSE NewMessage store 更新）。
3. **带外** `curl POST /api/typing {"sender":"Carol"}`（UI 零发起）→ UI 渲染
   "Carol is typing…"（右上角可见，见 ac02-ui-typing-loop.png）——纯
   VM publisher→subscribe→SSE→store→UI 链路实证。

## 门禁

- `cargo t bus_subscribe` 2/2 绿（fanout+无订阅者忽略 / shim 注册+投递）。
- `cargo t plan698_publisher` 3/3 绿（Typing 形态 / New{Type} 注入 /
  非对象响应不广播）。
- 既有回归：作用域 http_server / bus_subscribe 组零新增红（T-05 全量对账兜底）。
