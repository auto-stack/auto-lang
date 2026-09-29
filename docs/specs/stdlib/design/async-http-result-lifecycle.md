# 异步 HTTP 结果通道生命周期(async result channel lifecycle)

> **Status**: current(SD-01,PLAN-027;PLAN-705 SD-02 修订) | 层:vm ffi stdlib | 2026-09-29

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
   只对仍 live 的令牌投递。
2. **单次终结,取消不复活(PLAN-705 SD-02)**:live-op 表状态机
   Pending→Completed→(take 消费移除 | cancel 终结移除)。完成端唯一入口
   `complete_live_op`:仅 Pending→Completed 一次转变,已取消/已消费/缺席
   的迟到完成**丢弃数据禁止 insert 重建**(旧裸表无条件 insert 可在取消后
   复活条目——req_id 单调不复用,复活即永驻泄漏,该缺陷已修)。任何放弃
   路径调 `drop_async_result`(=cancel 幂等终结);take 只消费 Completed,
   Pending 探测/重入不得删除条目。
3. **Err 条目必须可终结**:消费函数对 `Err` 变体映射为可解析错误
   body(`{"error":..,"status":0}` 契约,与 simple_http_json 对齐),
   不得当作"仍在等待"(PLAN-027 缺陷 C:remove 已发生,None 语义
   = shim 永久 Waiting)。
4. **发射路径零线程 churn(PLAN-705 收口)**:非流式客户端三族
   (json/handle/builder,含 auth/bearer/msg-bridge)统一经**固定 async
   executor**(线程数 `AUTO_HTTP_ASYNC_WORKERS` 默认 2)——旧"2 worker 微池
   + 队满每 job spawn 兜底"形态退役。有限边界与满载终结性错误:
   活跃 `AUTO_HTTP_CLIENT_MAX_ACTIVE`(8)/队列 `AUTO_HTTP_CLIENT_QUEUE`(64)
   满载即拒(零临时线程)/响应体 `AUTO_HTTP_CLIENT_BODY_LIMIT`(10 MiB
   增量预算)/单 job 总期限 `AUTO_HTTP_CLIENT_TIMEOUT_MS`(30s);
   重试退避为 async sleep,取消随 future 丢弃,绝不继续发送。
   禁止回归"每请求 spawn"或"队满临时 spawn"(PLAN-027 缺陷面 + AC-03)。
5. **回归钉**:契约单测(p027 两项——已随 PLAN-705 更新为协议面断言)+
   `plan705` 探针族(单次终结/竞态/迟到完成/线程稳定/体预算/总期限,
   `plan705_spike_tests.rs`)+ 资源基线断言(取消风暴后 live-op/scope/
   许可回基线,见 plans/reports/705-resource-lifecycle.md)。

## 关联

- PLAN-027(evidence/027:L0 审计/L1 判别/坍缩半减/池化终局四轮数据)
- PLAN-705(reports/705-async-decision.md 冻结 + verification 复审)
- [http-handler-async-lifecycle](http-handler-async-lifecycle.md)(服务侧生命周期)
- http-server.md(服务侧)
- AUTO_VM_MEM(engine 池观测,VM 侧面;本通道为 rust 层,AUTO_VM_MEM
  不可见——观测依赖斜率钉)
