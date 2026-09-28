# PLAN-705 T-06 兼容性对账报告（parity）

基线：worktree `D:/autostack/.wt/lang-705/auto-lang`，branch `plan-705-dev`，
T-06 时点 commit `bad060695`（T-01..05 已合入）。所有命令均以
`cargo nextest run -p auto-lang --lib --features test-http-e2e` 形式执行
（nextest 每测试进程隔离；端口类 E2E 用 `-j1/-j2` 控并发）。

## 1. 支持矩阵（T-01 决策报告入口矩阵 × 实际证据）

| 入口路径 | 705 形态 | 验证 | 结果 |
|---|---|---|---|
| named handler（`fn() T`，等待 HTTP） | 段驱动 park/resume | `plan705_e2e_upstream_gate_health_two_parked`（gate 期间 health 完成 + 双 park 各取所得） | PASS |
| named handler（`fn() ~T` 返回 future） | 元数据门 + AwaitReturnFuture / 内部体驱动 / AsyncReturnBody | `plan705_e2e_async_return_final_value_and_int_counterexample`（External future 150ms → 最终值 `150`） | PASS |
| 普通 int 反例（位形态 0xF0=240） | 元数据门拒绝位模式猜 | 同上（`plain_int` 返回 240 → 原样 `240`） | PASS |
| 链式 await（多 park） | 段恢复链 | `plan705_e2e_chained_await_and_error_recovery`（chain-plain 双 park + second body） | PASS |
| 错误恢复（park 后深帧异常） | `intercept_error` 跨帧展开（PLAN-705 引擎修复） | 同上 catch 臂 + `plan705_engine_deep_frame_try_recovery` | PASS |
| middleware 链（逐个段驱动） | middleware park 挂起整请求，恢复续链 | `plan705_e2e_middleware_park_then_chain`（gate middleware 等 250ms 上游 → null 放行 → handler 到达） | PASS |
| `__axum:` closure（`call_closure` 自旋面） | `call_closure_segment`（帧建立同 call_closure + 段驱动） | `plan705_engine_closure_segment_parks_and_resumes`（fn-ref closure 合成注册，250ms 内 park、resume 取最终值） | PASS |
| handler 内 RequestBuilder `.send` | 段模式退役 30s 同步 drain（rewind+Yield park） | `plan705_builder_send_parks_in_segment_mode`（350ms 内 park、恢复取 body） | PASS |
| client JSON 族（get/post/put/delete/patch_json） | 固定 async executor + 有限队列 | plan446/plan349 矩阵 + `plan705_client_queue_full_terminal_error_no_thread_fallback`（队满终结性错误） | PASS |
| client handle 族（get/post/put/delete + auth/bearer） | 同上（每请求线程退役） | plan446 e1/e2 + plan349 e2e（63/63） | PASS |
| builder 属性矩阵 | header/timeout/cookie/gzip/brotli/multipart/retry/TLS 原样迁 async | plan446_batch2 e1/e2 + plan349 builder 链（含 retry 消费形态） | PASS |
| 响应体预算 / 总期限 / 重试取消 | `read_body_capped` + job 总期限 + async 退避可弃 | `plan705_client_body_budget_enforced` + `plan705_client_total_deadline_terminal_error` | PASS |
| SSE 生成器路由 | 696/698 语义保持（spawn_local 生产者/channel(1)/cleanup） | plan326 SSE 族（79/79，含 `e2e_concurrent_sse`） | PASS |
| 回复 deadline | scope 幂等终结（从"只丢接收端"升级） | `plan705_e2e_deadline_cancels_parked_and_reclaims`（~400ms 收口 + live-op/scope/许可回基线） | PASS |
| 关闭 | 排水窗内自然完成；窗尽取消 parked + 资源回收 | `plan705_e2e_shutdown_cancels_parked_side_effect_kept` | PASS |
| 半关闭 | hyper `half_close(true)`——不判取消，正常响应 | `plan705_e2e_half_close_still_responds` | PASS |
| 失效队列 | 出队跳过业务函数（零 handler 任务） | `plan705_e2e_invalid_scope_skips_handler_dispatch` | PASS |
| 生命期许可 | queued+running+parked 总上限（升级语义明示） | 队满/许可满双 503 + deadline/关停探针中许可回基线断言 | PASS |

## 2. 回归面（命名跑，T-06 时点实测）

| 面 | 命令（`-E` 过滤） | 结果 |
|---|---|---|
| plan326 server E2E（含 696 real_015/017/023 parity、SSE、关闭、端口门禁） | `test(/plan326/)` `-j2` | **79/79** |
| 702 engine 段 + UI 桥 + 705 全族 | `test(/plan702|plan705/)` | **21/21** |
| client/builder/auth 矩阵（plan446/plan349/plan394/p027） | `test(/plan446|plan349|plan394|p027/)` `-j2` | **63/63** |
| 广谱（上四族 + try/catch/bridge） | `test(/plan705|plan702|plan326|plan394|plan349|plan446|p027|try|catch|bridge/)` `-j4` | **522/522** |

696/698/699 的 SSE 生产者/channel(1) 背压/SseIteratorCleanup 未改动（diff
零触碰 `produce_sse_frames`/`SseIteratorCleanup`/`FrameStream` 帧泵语义；
`FrameStream` 仅新增 scope 代持字段）；702 的重入/UI tick 策略未触碰
（`resume_fn_by_name_segment` 仅增加段标志置位/复位）。

## 3. 已知边界（明示，非缺陷）

- back_proxy E2E 14 项（socket 10013 PermissionDenied）与
  `ui_gen::vue::test_a2vue_desktop_surface_asset` 为本机环境/基线预存红
  （base `025fb192c` stash 复证，见 T-04 复审记录）。
- `e2e_concurrent_sse` 在 j4 高并发下偶发（端口压测敏感），隔离复跑绿。
- legacy 串行 server（`serve_blocking_stdnet` / `run_http_server_blocking` /
  `shim_http_server_listen`）按决策报告 §2 保留同步形态，不在 705 迁移面。
- `__axum:` 全链路 fixture（musk backend）依赖 sibling checkout（plan442
  `#[ignore]` 形态）；705 以合成 fn-ref closure 单元覆盖同一 `call_closure_segment`
  入口，全链路留给后续真实消费方复验。

## 4. 语义增量（进 SD-01..04 沉淀）

- 单 owner 段执行：等待上游期间 health 可服务（gate E2E）；多请求各自
  独立任务栈与续体；跨 await 非原子事务（副作用不回滚——关停探针 hits=1）。
- 取消三类确证判据：桥 deadline / 关停 / 连接任务终结；半关闭与输入 EOF
  不判取消（half_close 探针）。
- 取消不复活：live-op 终结后 worker 迟到完成被 presence 守卫丢弃
  （`plan705_spike_cancel_and_late_completion_single_finalization` +
  deadline 探针基线断言）。
