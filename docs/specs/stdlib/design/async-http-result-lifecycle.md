# 异步 HTTP 结果通道生命周期(async result channel lifecycle)

> **Status**: proposed(SD-01,PLAN-027) | 层:vm ffi stdlib | 2026-09-22

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

1. **pending 标记不得覆写完成态**:登记 pending 一律
   `entry(req_id).or_insert(None)`——worker 先完成的 `Some` 不被
   `None` 覆盖(覆写=永 pending→30s 超时→泄漏,PLAN-027 缺陷 B)。
2. **放弃路径必须回收条目**:任何不再消费 req_id 的路径(超时放弃、
   任务丢弃)必须调 `drop_async_result(req_id)`。engine 两处同步
   drain 超时臂(call_fn_by_name Yield 臂 / request-builder send 臂)
   为规范消费方(PLAN-027 缺陷 A)。
3. **Err 条目必须可终结**:消费函数对 `Err` 变体映射为可解析错误
   body(`{"error":..,"status":0}` 契约,与 simple_http_json 对齐),
   不得当作"仍在等待"(PLAN-027 缺陷 C:remove 已发生,None 语义
   = shim 永久 Waiting)。
4. **发射路径零线程 churn**:api.* json 请求经常驻微池
   (2 worker × 容量 64 mpsc)执行;队满退化为按需 spawn(突发兜底)。
   禁止回归"每请求 spawn"(双层时代 160 线程/s;实测泄漏斜率
   ∝ 线程创建数,2.23MB/min → 池化后噪声水位,evidence/027)。
   panic 隔离用 `catch_unwind`,不得以新线程替代。
5. **回归钉**:斜率断言脚本(auto-term scripts/repro/027-mem-slope.ps1,
   阈值 0.5MB/min)+ 契约单测(p027_check_async_result_err_maps_to_
   error_body / p027_drop_async_result_removes_entry)。

## 关联

- PLAN-027(evidence/027:L0 审计/L1 判别/坍缩半减/池化终局四轮数据)
- http-server.md(服务侧)
- AUTO_VM_MEM(engine 池观测,VM 侧面;本通道为 rust 层,AUTO_VM_MEM
  不可见——观测依赖斜率钉)
