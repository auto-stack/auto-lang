# PLAN-705 T-07 资源生命周期报告

时点：T-06 后（worktree `D:/autostack/.wt/lang-705/auto-lang`，branch
`plan-705-dev`）。探针全部位于
`crates/auto-lang/src/tests/plan705_spike_tests.rs`（`plan705` 名族），
nextest 每测试进程隔离，单测可独立复跑（"同样配置重复可复现"）。

## 1. 取消风暴 + 晚完成回收（AC-02/04）

`plan705_e2e_cancel_storm_reclaims_deterministically`：
配置 = `AUTO_HTTP_REQUEST_TIMEOUT_MS=250` × 上游 400ms 延迟；3 并发 × 2 轮。

- 全部请求以 503 终结（deadline 收口，无悬挂）；
- 每轮结束后断言 `LIVE_OPS` 表回基线（迟到完成被 presence 守卫丢弃，
  复活零发生）、`REQUEST_SCOPES` 空；
- 两轮迭代逐字节同基线——同配置重复可复现。

deadline 单发形态（T-05 `plan705_e2e_deadline_cancels_parked_and_reclaims`）
实测：250-400ms 档 deadline 在 ~[350, 500)ms 墙钟收口 503（远小于上游
2s），live-op/scope/许可三面回基线。

## 2. 线程数量稳定（AC-03）

`plan705_client_thread_count_stable_under_load`（Windows）：
进程线程数经 `CreateToolhelp32Snapshot(TH32CS_SNAPTHREAD)` 枚举，预热稳态
后并发 8 请求（上游 40ms/conn），完成后计数。

- 实测线程数前后差 ≤2（tokio 辅助线程抖动容差内），零每请求线程；
- 结构性佐证：client 三族（JSON/handle/builder）的发射路径已无
  `std::thread::spawn`（T-03 diff 移除，队列满=终结性错误
  `plan705_client_queue_full_terminal_error_no_thread_fallback`）。

### 限额默认值与满载行为（冻结值实测）

| 限额 | 默认 | 探针 | 满载观测 |
|---|---|---|---|
| runtime 线程 | 2 | 线程稳定探针 | 恒定 |
| client 活跃 | 8 | 队满探针（cap=1） | 队满/活跃满 → `http client queue full` 终结性错误，JSON 形态 `{"error":..,"status":0}` 可消费 |
| client 队列 | 64 | 同上 | 提交即拒（返回 false），无临时线程 |
| 响应体预算 | 10 MiB | `plan705_client_body_budget_enforced`（16B 档） | 增量累计超限即终结性错误 `exceeds budget 16` |
| 单 job 总期限 | 30s | `plan705_client_total_deadline_terminal_error`（200ms 档） | ~200ms 收口 Err，不等待上游 2s |
| 生命期许可 | `AUTO_HTTP_MAX_INFLIGHT`(64) | deadline/关停探针 | 请求终结后 `available_permits` 回满 |

## 3. 完成写点与忙等核查（AC-01/02）

- 写点全查：`ASYNC_RESULTS` 旧 7 写点已全量收口 `complete_live_op`
  （T-02 diff + `grep -n "complete_live_op" stdlib.rs` 复核：io×2、json
  job、handle、rb-send、auth、bearer）；`vm.futures` 完成统一
  `complete_external_future`（delay/fail/all/race 四点）。
- 忙等门禁：`plan705_gate_default_callgraph_no_busy_wait`——
  `http_server.rs` 的 `.call_fn_by_name(` 恰 1 处且带 `legacy 同步驱动
  保留位` 标记（serve_blocking_stdnet 串行 server，非默认调用图）；
  `.call_closure(` 零命中。引擎忙等臂（call_fn_by_name 的 30s drain）
  保留给 legacy 调用图，默认路径（serve_with →
  dispatch_api_request_segment）零触及。

## 4. 时延观测（记录，非门禁）

本机（Windows，开发机）观测：预热单请求 ~67-70ms 全链路；8 并发突发下
各请求 ~2.0s（loopback 连接建立 churn，环境相关——Defender/防火墙对突发
本地连接的探 probing；非 VM/owner 串行化：8 个请求总墙钟 ≈ 单请求耗时
而非 8×）。**owner 并发面正确性由 gate 探针（health 在 park 期间完成）
承载，时延数值仅作资源画像记录。**

## 5. 资源斜率结论

全部探针零资源斜率：live-op/scope/许可三面在风暴、deadline、关停、
builder 段 park 后均回基线；线程数恒定。无 permit 漏失与假并发
（并发正确性由 gate E2E 的 health 交错断言 + 双 park 各取所得承载）。
