# 异步 HTTP 结果通道生命周期(async result channel lifecycle)

> **Status**: current(SD-01,PLAN-027;PLAN-705 SD-02 修订;PLAN-707 SD-02 修订——取消实停;PLAN-724 SD-04 修订——错误消费分层与登记协议钉) | 层:vm ffi stdlib | 2026-10-02

## 背景

VM 侧 `http.get_json/post_json/put_json/delete_json`(#[api] 契约改写的
消费面)与 handle 型 natives 共用全局结果表 `ASYNC_RESULTS:
Mutex<HashMap<u64, Option<Result<AsyncResult, String>>>>`:
`None`=pending(已发射未完成),`Some`=worker 已完成。消费协议 =
发射方置 pending → engine 同步 drain(`async_http_result_ready` 探测
+ shim 重入 `check_async_http_result`/`_handle` 移除)。
PLAN-027 定罪该通道的三个缺陷与稳态泄漏源(每请求线程 churn),本篇
固化修复后的生命周期规则。

## 规则

1. **pending 标记不得覆写完成态**:登记一律 `register_live_op(req_id)`
   (or 语义,PLAN-027 缺陷 B 沿袭),且**必须先于提交 worker**——完成端
   只对仍 live 的令牌投递。**登记协议是提交方义务**(PLAN-724 钉):
   直驱 `spawn_async_http_handle`/`spawn_async_http` 等汇聚点的调用方
   (生产 shim 与测试同责)必须先 register 再 submit——未登记令牌的
   managed 提交会被 707 取消竞态守卫(still-live 闭合)中止,请求恒不到线
   (e4 `default_headers_reach_wire_on_plain_get` 30s 恒红实证,修复=
   测试走生产协议 register→submit→wire→consume/cancel 配对;守卫本身
   正确,不得削弱)。
2. **单次终结,取消不复活(PLAN-705 SD-02)**:live-op 表状态机
   Pending→Completed→(take 消费移除 | cancel 终结移除)。完成端唯一入口
   `complete_live_op`:仅 Pending→Completed 一次转变,已取消/已消费/缺席
   的迟到完成**丢弃数据禁止 insert 重建**(旧裸表无条件 insert 可在取消后
   复活条目——req_id 单调不复用,复活即永驻泄漏,该缺陷已修)。任何放弃
   路径调 `drop_async_result`(=cancel 幂等终结);take 只消费 Completed,
   Pending 探测/重入不得删除条目。
3. **槽终结与消费接口分离(PLAN-724 SD-04 修订)**:`Err` 落表 = 通道
   **终结**(take 可取、不复活——PLAN-027 缺陷 C 的"永久 Waiting"已修),
   但 Err 到消费面的映射**按接口分层,不再是统一 `{"error":..,"status":0}`
   旧表述**:json 族 shim 按 712 错误分层(SD-10,`ui/overview.md`)——
   **网络失败可 catch**(try 臂接住错误形状值,不抛穿),**非 2xx 是错误
   形状值**(kind=error 的 HttpResponse/tuple 值,不 throw);handle 族
   携带 `(status, headers, body)` 自行判读;auth 族 `(0, 错误文案)` tuple。
   文档消费边界以 712 分层为准,历史"Err 统一折 status:0 JSON"表述退休。
4. **发射路径零线程 churn(PLAN-705 收口)**:非流式客户端三族
   (json/handle/builder,含 auth/bearer/msg-bridge)统一经**固定 async
   executor**(线程数 `AUTO_HTTP_ASYNC_WORKERS` 默认 2)——旧"2 worker 微池
   + 队满每 job spawn 兜底"形态退役。有限边界与满载终结性错误:
   活跃 `AUTO_HTTP_CLIENT_MAX_ACTIVE`(8)/队列 `AUTO_HTTP_CLIENT_QUEUE`(64)
   满载即拒(零临时线程)/响应体 `AUTO_HTTP_CLIENT_BODY_LIMIT`(10 MiB
   增量预算)/单 job 总期限 `AUTO_HTTP_CLIENT_TIMEOUT_MS`(30s);
   禁止回归"每请求 spawn"或"队满临时 spawn"(PLAN-027 缺陷面 + AC-03)。
   Rust a2r 面的对应内核(`a2r_std::http::client`)协议同构、实现独立
   (见 [http-client-runtime](../../a2r-std/design/http-client-runtime.md))。
5. **取消=实际停止执行体(PLAN-707 SD-02 修订)**:705 原文"取消随
   future 丢弃"在实现面不成立——cancel 当时只删结果槽。现在
   `cancel_live_op` 同时 abort 该 req_id 的 managed job future
   (`JOB_ABORTS` 登记;abort 在下一 await 生效:许可等待/请求建立/
   重试退避 sleep/读体——**queued/active/retry 三阶段均可打断**,许可
   随 wrapper task 丢弃归还;spawn→登记窗口内的取消由插入后复查闭合)。
   abort 句柄随 job 完成/取消双路径出表(成功 job 滞留句柄=慢性泄漏,
   已修)。**detached 显式分离**:`submit_detached_client_job`(消息桥
   fire-and-forget)不写 live-op、不登记 abort——`cancel_live_op` 对其
   零影响,LIVE_OPS 缺席不构成误杀面。Future 取消不承诺撤销对端已
   接受的 POST/OS 已缓冲数据;只证明本地 job 停止读/重试、许可归还。
6. **回归钉**:契约单测(p027 两项——已随 PLAN-705 更新为协议面断言)+
   `plan705` 探针族(单次终结/竞态/迟到完成/线程稳定/体预算/总期限,
   `plan705_spike_tests.rs`)+ 资源基线断言(取消风暴后 live-op/scope/
   许可回基线,见 plans/reports/705-resource-lifecycle.md)+ PLAN-707
   取消探针族 `plan707_cancel`(queued/active/retry 三阶段实停、许可
   归还、迟到不复活、detached 反例,`plan707_cancel_tests.rs`)+
   PLAN-724 登记协议钉(e4 `default_headers_reach_wire_on_plain_get`
   登记→提交→消费配对 + 未登记反例:abort 句柄回收+无令牌无完成)。

## 关联

- PLAN-027(evidence/027:L0 审计/L1 判别/坍缩半减/池化终局四轮数据)
- PLAN-705(reports/705-async-decision.md 冻结 + verification 复审)
- PLAN-707(reports/707-stream-decision.md D-1 冻结 + 707-verification.md;
  取消执行体闭合的本轮)
- [http-handler-async-lifecycle](http-handler-async-lifecycle.md)(服务侧生命周期)
- http-server.md(服务侧)
- AUTO_VM_MEM(engine 池观测,VM 侧面;本通道为 rust 层,AUTO_VM_MEM
  不可见——观测依赖斜率钉)
