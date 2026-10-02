# 外部 HTTP/SSE 流生命周期(http stream lifecycle)

> **Status**: current(PLAN-707,Design 33 阶段 C2a;PLAN-724 SD-06 修订——a2r 客户端迁移落地) | 层:vm ffi http_stream/stdlib | 2026-10-02

## 执行模型

外部流(上游 HTTP/SSE 连接)的建立、逐块读取与 SSE 逐事件等待由**统一
资源表** `http_stream::STREAMS`(全局 `Mutex<HashMap<u64, StreamHandle>>`)
承载:shim 打开流=同步登记 `Opening` 态条目后向**固定共享 async
runtime**(与 705 非流式 executor 同 runtime)提交生产者 future——
建立与读取零阻塞线程、零 per-stream runtime。生产者持**独立许可对**
(`AUTO_HTTP_STREAM_ACTIVE` 16 / `AUTO_HTTP_STREAM_QUEUE` 32,不占
705 非流式配额),经 reqwest async 读上游,解码后条目入**有界队列**
(≤`AUTO_HTTP_STREAM_MAX_QUEUED` 16 条;单条 SSE 事件
≤`AUTO_HTTP_STREAM_MAX_EVENT` 256 KiB,raw 文本块切 ≤16 KiB)并
notify 等待者;队满停在空间等待(背压传导到上游读端,不丢数据、
不无界增长)。

生产者**可取消**:abort 句柄登记于 `STREAM_ABORTS`,取消=终结状态机
+ abort(在下一 await 生效——许可等待/请求建立/读体/队满发送等待全为
await 点)+ 出表 + 通知等待者;spawn→登记窗口内的取消由插入后复查
闭合(与 705 `submit_client_job` 同款 insert-then-recheck 协议)。

## 状态机与消费协议

```text
Opening(建立中,≤10s) → Pending(已建立待数据) → (有界队列) → 消费
   ↘ Eof(自然结束,先排空队列)          ↘ Failed(建立/读/解码/预算)
   ↘ Cancelled(close/break/断连/scope 取消,释放未消费数据)
```

- **typed pull**(`stream_pull → Data|Pending|Opening|Eof|Failed`)是唯一
  推进方式;EOF 先排空已成功入队 data 再报终结;**终态单次迁移幂等**,
  迟到生产者 enqueue 前复查终态被拒(禁止 insert 重建)。
- **等待凭据**:`ParkedWait::HttpStream(stream_id)`(段 park/泵恢复)+
  `AutoTask.waiting_http_stream_id`(任务 loop 重试协议)。CALL_NAT
  rewind 重试时 shim 重新 pop——挂起臂必须**回推被弹的迭代器 id**
  (回推流 id 是不同键,707 实测致 plan341 回归);就绪出口必须清除
  等待凭据(残留标志使引擎在后续任意 CALL_NAT 误 rewind,热循环根因)。
- **generator 等待凭据化**:cooperative 驱动遇流/Future 等待立即停步;
  `next_sse_generator_value` 对流做 enable→检查→await(流 Notify +
  COMPLETION_NOTIFY 双通道),删除 yield_now 自旋——gate 关闭期间驱动
  次数不随等待时长增长。
- **manual 哨兵保留**:`HTTPStream.next` EOF 仍返回 `"[DONE]"`;
  `sse_poll` 语义逐字节保留(`""` pending / `"[DONE]"` 终结 / 无效句柄
  Err)。迭代器终结**凭流状态**而非字符串比较——合法 `"[DONE]"` 载荷与
  空 data 是业务数据不丢。新增 `sse_close`(显式释放)、`sse_error`
  (终态诊断,非消费式)。

## 增量 SSE 解码

`sse/decoder.rs`(`SseDecoder::feed/finish`,live 路径专用):流首 BOM
剥离(可跨 chunk);行终结 LF/CRLF/CR 三态(悬置 CR 等下一字节判别
CRLF 合并);字段=首个冒号切分、冒号后恰一个空格移除、其余空白保留;
多行 data `\n` 拼接;NUL id/非法 retry 忽略;空行仅在有 data 缓冲时分发
(`data:` 空值行不分发);EOF 丢未闭合事件;行/事件 256 KiB 预算超限 →
可观测终结错误。raw 流走 `Utf8Carry`(跨 chunk 码点无损;已判非法
字节按 lossy 约定替换——只修跨 chunk 拆分,不改非法字节语义)。
**legacy 保留**:`sse/parser.rs` 的 `parse_sse_chunk`/`SSEParser`(含
EOF 尾事件兜底与重复交付 quirk)行为原样,消费方不迁移;live 流一律走
decoder(两套 trim/分帧差异见 decoder 测试矩阵钉)。

## 请求资源组与所有权

- **scope 组**:handler 段(dispatch/resume,`bind_current_scope` RAII)
  内 open 的流登记进 `RequestScope.resources`;`finalize_scope`(完成/
  取消/断连/关闭)逐流 `stream_cancel`——组内资源不越过请求生命期。
- **任务第二线**:`AutoTask.owned_stream_ids`——generator 体首次 pull
  发生在 SSE serve 循环(scope 守卫已退出),open shim 同时登记到执行
  任务;收口点=`cleanup_sse_iterator`(Generator 任务回收前清空)、
  `abort_parked_request`(废弃请求的任务持有流取消)。
- **非 request 上下文**(UI/CLI 程序):流归显式 close 管理;break 后
  队满背压已停止上游拉取,无泄漏增长(连接保持为文档化语义)。
- **级联**:下游断连/关闭 → scope 收口 → 上游流取消(本地连接真实
  关闭,上游读端可观测);SSE 输出终结 → generator 任务 owned 流回收。
  「等待可取消」与「关闭登记」分别证明(scope 收口≠生产者实停,
  后者由 abort 承载)。

## 限额(env 覆盖,首次使用读一次)

| 项 | 默认 | env |
|---|---|---|
| 流 active / queue | 16 / 32 | `AUTO_HTTP_STREAM_ACTIVE` / `_QUEUE` |
| 每流排队条数 | 16 | `AUTO_HTTP_STREAM_MAX_QUEUED` |
| 单事件(SSE) | 256 KiB | `AUTO_HTTP_STREAM_MAX_EVENT` |
| raw 文本块 | 16 KiB(固定) | — |
| 建立期限 | 10 s | `AUTO_HTTP_STREAM_OPEN_TIMEOUT_MS` |
| 上游读空闲 | 60 s(每 chunk 重置;队满背压等待不计时) | `AUTO_HTTP_STREAM_IDLE_TIMEOUT_MS` |

内存上界:每流 ≤16×256 KiB 队列 + ≤256 KiB carry ≈ 4.25 MiB(SSE 极端);
16 活跃流 ≤~68 MiB。SSE GET 状态 ≥400 → `Failed("sse upstream status N")`
(Content-Type 不强制);raw 流保留读取非 2xx body 的既有可观察行为;
headers/POST body 原样透传,不对 POST 自动重试/重连。

## for-in 装配与声明面

- 三形态全支持:`for c in Http.get_stream(u)`(内联,名含 stream/sse_ 的
  Call 源迭代器通道)、`var s = Http.get_stream(u); for c in s`(codegen
  `stream_vars` 分流——RHS 为流打开调用的赋值登记持流变量)、生成器体内
  for-in(`shim_iterator_next` 流句柄惰性消费臂,不进 iterators 表零
  id 碰撞)。
- 声明面:`http.at`/`http.vm.at` 的 `get_stream/post_stream(+headers)/
  next/is_done/close/iter` + `sse_open/sse_poll/sse_close/sse_error`;
  catalog 双表 `Http.*` 别名(与 `http_stream.*` 族同 ID)。句柄为
  单槽 i32(与 sse_open 先例一致,int 变量链/`_iterator` 局部同链)。
- `http.sse_get_stream` 生产端迁新执行器,经 **legacy 通道桥**
  (`spawn_legacy_channel_bridge`:STREAMS → AsyncStreamEvent Data/Done/
  Error)供给 `Iterator::AsyncHttpStream` try_recv 消费面——事件语义
  逐字节保留;rx drop(消费端清理)→ 泵退出并 stream_cancel。

## 支持矩阵(边界明示)

- **已支持**:上述全部(探针族 `plan707` 31 项 + E2E 三例真 TCP relay:
  分时帧/断连级联取消/close 基线,见 plans/reports/707-{parity,
  resource-lifecycle,verification}.md)。
- **Rust 客户端已迁移(PLAN-724 SD-06)**:a2r 两 facade 的外部流消费
  收敛到共享 async 内核(有界队列/背压/建立与 idle 期限/真取消/SSE 共享
  解码),本篇的 707 状态机/预算/单次终结语义在
  [http-client-runtime](../../a2r-std/design/http-client-runtime.md)
  以 Rust 形态同构落地;**协议差异明示**——VM 轨 = park/resume 凭据 +
  引擎泵恢复(本篇),Rust 轨 = async await/通知等待(无 park 概念),
  两者不共享资源表与许可。
- **非目标**:auto-man 生成服务的任意外部流 handler(事件总线模板为主,
  重写另案);bus.subscribe/文件进度流线程形态全面改造(共同句柄
  清理适配而已);通用 WS/TLS/HTTP2;自动 SSE 重连/Last-Event-ID 重放;
  async-for 语法;`HTTPStream.iter()` 方法式派发(自由函数
  `Http.stream_iter` + 变量形态已覆盖);run_task_loop 流唤醒的事件化
  (保留每轮扫描 wake source 4/4b)。
- **legacy 保留**:SSEParser/parse_sse_chunk(一次性 helper);
  `http.sse_get_stream` iterator 形态;`[DONE]` 哨兵契约。

## 关联

- PLAN-707(reports:707-stream-decision / 707-parity / 707-resource-lifecycle
  / 707-verification)
- [async-http-result-lifecycle](async-http-result-lifecycle.md)(managed
  job 取消执行体 abort——流取消的基础)
- [http-handler-async-lifecycle](http-handler-async-lifecycle.md)(段驱动
  与请求作用域;本篇为其外部流延伸)
- [networking-stdlib](../../auto-lang/runtime/design/networking-stdlib.md)
  (C2a 落地面)
