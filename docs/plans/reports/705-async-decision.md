# PLAN-705 T-01 决策报告：异步入口矩阵、作用域/许可、client 限额、断连判据

状态：T-01 产出（2026-09-28）。基线 commit `025fb192c`（worktree
`D:/autostack/.wt/lang-705/auto-lang`，branch `plan-705-dev`；依赖组
`D:/autostack/.wt/lang-705/auto-down` detached @ `3373a5cc`）。
Spike 证据：`crates/auto-lang/src/tests/plan705_spike_tests.rs` 2/2 绿
（notify 驱动 owner loop 零轮询唤醒 + 取消/迟到完成单次终结种子）；
前置基线 `p027|engine_segment|plan702` 定向 7/7 绿。

## 1. 代码事实（核查结论，行号对应当基线）

- `serve_with`（http_server.rs:3384-3479）owner loop 同步内联
  `dispatch_api_request`；一次只有一个 handler 在跑，等待上游即占住 owner。
- `call_fn_by_name` 忙等臂（engine.rs:2414-2436）：5ms 轮询 + 30s 硬超时 +
  `drop_async_result` 回收；`call_closure`（engine.rs:1994-2111）对 Yield
  一律 continue——closure 内等待会 1M 预算内自旋，无 park 形态。
- RequestBuilder `.send` 的 CALL_SPEC 拦截（engine.rs:7224-7291）自带同构
  30s 同步 drain。两处 drain 是 705 必须退役的"handler 等待占住 owner"路径。
- `ASYNC_RESULTS` 7 个生产写点（stdlib.rs 1718/1761/5811/7133/7421/7792/7834）
  全部无条件 `map.insert`；`drop_async_result`（7071）裸 remove——取消后
  迟到完成可复活条目（源码级确认，7064 注释自认）。
- External Future：`vm.futures` DashMap + `register_external_future`/
  `resolve_external_future`（engine.rs:612-642）；生产 native 直接写字段
  （stdlib.rs 1781/1798/1869/1909），无完成通知，全靠轮询。
- `~T` = 解析期脱糖为 `Future<T>`（parser.rs:11545）；`return v` 不包
  future——值形态是否 future bits（`(id<<8)|0xF0`）只有运行期可知；
  `API_RETURN_TYPES`（http_server.rs:74-102）只对 `#[api]` 发布且仅
  Stream 分支被消费——**无 return-mode 元数据**。
- 桥回复超时（http_transport.rs:247-256）只丢 oneshot receiver；已排队
  请求仍执行，迟到 reply 静默丢弃。无 scope、无 permit、无连接级取消。
- SSE：spawn_local 生产者 + channel(1) + `SseIteratorCleanup` Drop 回收
  （http_server.rs:4005-4035）——705 保持不动。

## 2. 入口矩阵（冻结）

| 入口路径 | 现状 | 705 形态 | 段入口 |
|---|---|---|---|
| named handler `fn() T` / `fn() ~T` | 同步忙等 | `call_fn_by_name_segment`；Parked 进 owner parked 表 | 已有（702） |
| middleware 链（逐个 `call_fn_by_name`） | 同步 | 逐个段驱动；middleware park 即整个请求挂起（stage=Middleware(i)），恢复后继续链 | 复用 named 段入口 |
| `__axum:` closure（`call_closure`） | 同步、Yield 自旋 | 新增 `call_closure_segment`：同 call_closure 的帧setup + `drive_handler_segment(false)` 驱动 | T-04 新增 |
| handler 内 RequestBuilder `.send`（CALL_SPEC drain） | 同步 drain 30s | 段模式（task 段标志）跳过 drain：rewind ip 至 CALL_SPEC 起点 + `StepResult::Yield` → park 成 HttpRequest；shim 重入臂消费 | T-04 改造 |
| legacy 串行 server（`serve_blocking_stdnet` / `run_http_server_blocking` / `shim_http_server_listen`） | 同步串行 | 不迁移（非默认调用图）；支持矩阵明示为"legacy 保留" | 无 |
| SSE 生成器路由 | spawn_local 生产者 | 保持 696/698/699 语义不动 | 无 |
| `*_sync` 客户端族（post_sync/get_sync/post_bearer） | 阻塞 | 保持显式同步 API，不入 request scope（§5.3 非目标） | 无 |

无法覆盖某生产等待路径即 replan——本矩阵之外新发现的等待路径同样处理。

## 3. 请求作用域与总许可（冻结）

- `RequestScope`：`{ id, conn_id, state(AtomicU8: Queued/Running/Parked/
  Replied/Done/Cancelled), deadline, cancel 通知 }`，全局
  `REQUEST_SCOPES: Mutex<HashMap<u64, ScopeHandle>>`。
- **生命期许可**：tokio Semaphore，容量 = `AUTO_HTTP_MAX_INFLIGHT`。
  **兼容性变化（明示）**：该 env 从"队列容量"升级为"queued+running+
  parked 请求生命期总上限"；许可在桥 try_send 前获取，scope 终结时释放
  （普通回复发送后 / SSE 流结束（许可移交流） / 取消 / 关闭），幂等一次。
  mpsc 队列容量仍为同值（队满 503 先于许可上限发生）。
- owner 在 handler 派发前与每次 park/resume 后检查 scope 状态与 deadline：
  已取消/过期 → 不再调用业务函数，回 503，parked task 丢弃、live op 取消。
- **owner 唤醒协议（spike 已证）**：全局 `COMPLETION_NOTIFY: Notify`；
  owner loop 每轮 `notified().enable()` → 检查就绪 parked 项 → await；
  完成端写点 notify_waiters。零固定间隔轮询；另加最早 deadline 定时臂与
  取消通道臂。parked 表扫描只发生在通知/定时臂触发后（事件驱动）。

## 4. 默认 client 限额（冻结）

| 限额 | 默认 | env | 满载行为（现有返回形式） |
|---|---|---|---|
| 固定 async runtime 线程 | 2 | `AUTO_HTTP_ASYNC_WORKERS` | —（线程数恒定，零每请求线程） |
| client 活跃 job | 8 | `AUTO_HTTP_CLIENT_MAX_ACTIVE` | 终结性错误进入排队前拒绝 |
| client 等待队列 | 64 | `AUTO_HTTP_CLIENT_QUEUE` | 队满立即终结性错误：JSON `{error,status:0}`；handle/builder 走既有 Err 通道 |
| 响应体预算 | 10 MiB | `AUTO_HTTP_CLIENT_BODY_LIMIT` | Content-Length 预检 + 增量累计，超限终结性错误 |
| 单 job 总期限 | 30s | `AUTO_HTTP_CLIENT_TIMEOUT_MS` | 超限终结性错误（与既有 30s client 超时对齐） |
| 重试 | 既有默认不变 | — | 退避 sleep 改 async（可被取消丢弃），绝不队满临时 spawn |

覆盖 JSON（`run_http_json_job` 族）、handle（`spawn_async_http_handle`/
auth/bearer）、RequestBuilder send 三条非流式客户端路径；文件 IO worker
与消息桥（msg_get）不在本轮迁移面（分别保留同步完成写点经统一协议、
fire-and-forget 语义）。

## 5. Hyper 断连判据（冻结）

- **确认为取消**（仅此三类）：
  1. 回复 deadline 到期（`AUTO_HTTP_REQUEST_TIMEOUT_MS`，桥侧 select 超时）；
  2. shutdown 信号；
  3. **连接任务确证终结**：net 侧 watcher 挂在每连接 serve_connection 的
     JoinHandle 上，连接任务结束（client 主动关/RST/解析错，即任务退出）
     → 取消该 conn_id 名下全部 scope。同连接多请求一并取消。
- **不判取消**：请求输入 EOF/半关闭（客户端可能半关写端仍读响应）。
- TCP 黑洞类无法即时观测的断连由回复 deadline 有界收口——不声称任意
  FIN 立即中止。
- owner 侧：取消到达时 parked 等待废弃、live op 终结；**已发生副作用不
  回滚**（单执行段顺序执行、跨 await 非原子事务，写入新 Spec SD-01）。

## 6. `~T` 返回表示与元数据（冻结）

- **元数据门，禁位模式猜**：codegen 为全部 `Stmt::Fn` 发布
  `API_ASYNC_RETURNS: Mutex<HashSet<String>>`（声明 ret 为 `Future<T>` 的
  函数名集）；closure 由编译期登记 closure_id → is_async 静态集。
- 编组时：fn/closure 异步声明 **且** 返回 nv 呈 future bits → 解码
  future_id，请求挂 AwaitReturnFuture(future_id)，完成取最终 T（Ready→
  result，Failed→null）；异步声明但非 future bits（`return v` 普通值）→
  按普通终值处理。非异步声明 → 绝不解释位模式（普通 int 240 反例由
  元数据门挡住，AC-01 反例测试锚定）。
- 结果表**不物理合并**：ASYNC_RESULTS 升级为统一协议的 live-op 表
  （Pending→Completed→Delivered/Cancelled 单次终结，presence-guard 拒
  复活）；`vm.futures` 保持独立 + 补完成通知。两类只共享协议入口形态
  （register/complete/take/cancel + notify），不扩 ABI。

## 7. 风险与边界登记

- CALL_SPEC rewind 语义需在 T-04 实测确认（CALL_NAT 的 `ip = pre_call_ip-3`
  模式是否对 CALL_SPEC 同构）；不可行则改 shim 首轮直接置 waiting + 返回
  Yield（等效重入语义），仍禁忙等。
- middleware try_lock 失败静默跳过（http_server.rs:3634）保留现语义；
  段模式下单 owner 无竞争，该分支成为死路防护。
- 既有 30s 引擎超时臂（忙等）保留给 legacy 调用图；`cargo t` 语料/回归
  若依赖忙等时序，由 702 同款"段路径不回收"断言族守护。
- 许可升级语义（§3）若影响真实消费者（外部脚本依赖 64 并发排队），
  按 §10.2 呈报修订 plan_revision。
