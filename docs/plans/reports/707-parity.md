# PLAN-707 T-06 兼容与 parity 报告

- 日期：2026-09-29
- 基线：plan-707-dev worktree；对照 `docs/plans/reports/707-stream-decision.md` D-4/D-5/D-7。
- 证据命令：本文各节标注；环境红按 baseline 同命令复证定责（§4）。

## 1. 兼容矩阵（AC-05 面）

| 面 | 门禁 | 结果 | 说明 |
|---|---|---|---|
| plan341（sse_get_stream for-in 内联消费） | `cargo nextest run -E 'test(plan341)'` | 15/15 绿 | 生产者迁 STREAMS + legacy 通道桥后事件语义不变（Data/Done/Error 逐字节保留） |
| plan348 | 同上 | 绿 | ASYNC_STREAMS 共用面不迁移裁定成立 |
| plan326（分段执行回归） | `test(plan326)` | 绿 | ParkedWait::HttpStream 新臂无侵扰 |
| plan702（UI 段驱动） | `test(plan702)` | 绿 | parked_wait_ready 新臂无侵扰 |
| plan705（非流式 async 生命周期） | `test(plan705)` | 24/24 绿 | managed 取消升格后既有超时/关闭/风暴语义保持 |
| plan083（消息桥 detached） | `test(plan083)` | 6/6 绿 | `submit_detached_client_job` 显式形态，无误杀 |
| generator_tests | `test(generator)` | 绿 | Generator 臂 clone→驱动→回写重构后逐值消费/暂停/终结不变 |
| sse parser 家族 | `test(/sse/)` | 绿 | legacy helper 行为原样（差异矩阵钉在 decoder 测试） |
| 017-chat publisher | 真 SSE 端点 E2E（back_proxy family） | 预存红（§4） | 与 707 改动无因果：基线同败 |

## 2. 新增面装配核对（D-4/D-5）

- 公共声明：`stdlib/auto/http.at` 新增 `HTTPStream.iter`、`stream_next/is_done/close/iter`
  自由函数形、`sse_open/sse_poll/sse_close/sse_error`；`http.vm.at` 同步。
  catalog 双表（bigvm 宏表 + NATIVE_ID_ENTRIES）配对登记（`Http.*` 命名空间别名，
  ID 与 `http_stream.*` 族共享）。
- 三形态 for-in：
  - 内联 `for c in Http.get_stream(u)`：`plan707_client_for_in_inline_raw_stream` 绿；
  - 变量 `var s = Http.get_stream(u); for c in s`：codegen `stream_vars` 分流 +
    `shim_iterator_next` 流句柄惰性消费，`plan707_client_for_in_variable_stream` 绿；
  - 生成器嵌入：`plan707_wait_generator_park_drive_count_static` 绿（~Iter<str> 体内
    for-in 流，gate 关闭驱动静态、放行唤醒取值）。
- 手动族：next（`[DONE]` 哨兵）/is_done（终态 0/1）/close（真收口出表）——
  `plan707_client_manual_next_is_done_close`、`plan707_client_close_reclaims_entry` 绿。
- SSE 新面：`sse_close`（显式释放）、`sse_error`（终态诊断非消费式）——
  `plan707_client_sse_poll_close_error_surface` 绿。

## 3. 未迁移/差异明示（不声称完成项）

1. **a2r-std / auto-man / Rust 生成服务的流客户端未迁移**（线程形态/无界队列/close
   占位仍在）——SD-05 登记差异，后续计划收敛。
2. **bus.subscribe / 文件进度流**保持 705 前线程形态（共同句柄/清理适配，不全面迁移线程）。
3. **`.iter()` 方法式调用**：VM 轨以自由函数（`Http.stream_iter`）+ 变量形态分流装配；
   `s.iter()` 方法式派发未接线（opaque dispatch 表无 HTTPStream 项）——变量形态
   codegen 分流已覆盖计划要求的三形态。
4. **run_task_loop 的流唤醒保留轮询形态**（wake source 4/4b，每轮扫描）；事件驱动
   仅覆盖 HTTP parked 泵与 UI 恢复泵（决策 D-6 范围说明）。
5. **旧 `parse_sse_chunk`/`SSEParser`**：保留原行为为 legacy adapter（含 EOF 尾事件
   兜底与重复交付 quirk，行为钉住不变）；live 流一律走增量 decoder。
6. `run_task_loop` 之外的非 request 上下文消费退出（break 后不再拉取）依赖队满背压
   停止上游拉取 + 显式 close 收口；scope 组取消覆盖 request 上下文（D-7 范围）。

## 4. 环境红定责（baseline 同命令复证）

| 测试族 | 现象 | 基线复证 | 定责 |
|---|---|---|---|
| `ui_gen::vue::test_a2vue_desktop_surface_asset` | 12s 超时/断言失败 | `git stash -u` 后同命令失败 | 预存环境红，与 707 无关 |
| `back_proxy_tests::http_e2e_back_proxy_*`（全家 ~17 例） | `start(config)` 启动失败（<0.1s） | 干净树 `cargo th` 同败 | 预存环境红（本机代理启动依赖缺失），与 707 无关；017 SSE 流端点在此环境不可复验，以 §2 新增 E2E 面替代证据 |
| plan707 E2E 三例 | — | — | 本轮新增，`-j4` 与 back_proxy 同跑稳定绿 |

## 5. E2E relay 证据（AC-02/AC-03/AC-05）

`plan707_stream_e2e_tests.rs`（`test-http-e2e` 门，`cargo th` 池；真 TCP）：

1. `http_e2e_plan707_relay_frames_timed_single_termination`：上游 SSE（真 TCP，250ms
   分时两帧）→ VM `#[api] /api/relay`（~Iter<str> generator 消费 `http.sse_get_stream`）
   → 真 TCP 下游：帧序列 JSON 形态正确、两帧间隔 ≥150ms、末尾正常终结、流表/live-op
   5s 内回基线。
2. `http_e2e_plan707_relay_downstream_disconnect_cancels_upstream`：下游 RST 后，上游
   读端在 ≤8s 观测到连接关闭（eof/reset，非 timeout）——资源组收口把上游连接真实断开；
   流表/live-op 回基线。
3. `http_e2e_plan707_stream_explicit_close_two_rounds_baseline`：显式 close 两轮，
   每轮出表、live-op 回基线（无 request scope 的显式管理路径）。
