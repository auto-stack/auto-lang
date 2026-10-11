# VM 宿主稳定域(活跃会话)——HTTP 消息桥生命周期 / 回填派发 / 退出审计

> 沉淀来源:PLAN-765(2026-10-11)。归因纪要全文见
> `docs/reports/765-t01-attribution-memos.md`(三面:帧循环分配剖面 /
> HTTP 缓冲生命周期 / exit(1) 调用点)。
> 域:iced/winit 宿主(`auto run --render vm`)的活跃会话稳定面。

## 1. HTTP 消息桥生命周期契约(`Http.get_msg`)

消费方(如 musk PollStream,500ms 拍 /page 292KB 级)按拍回填时,宿主侧
每响应的分配必须满足(PLAN-765 F-1/F-2/F-3):

1. **共享连接池客户端**——消息桥请求一律走 `shared_async_http_client()`
   (pool 8 + 30s 超时),**禁止每请求新建 `reqwest::Client`**(每 Client
   自带独立 hyper 池,高频轮询下"新 TCP+池结构用后即弃"是空闲期私有内存
   棘轮的主分配源之一)。
2. **响应体按 Content-Length 预留**——`read_body_capped` 以 CL 预建容量,
   消除逐 chunk 倍增的中间垃圾;CL 超预算预检保留。
3. **载荷单缓冲预编码**——`HttpMsgDone.event` 恒为入队侧单缓冲产出的
   `{handler}\u{1F}s\u{1F}{json}` 预编码串(事件前缀 + 借值序列化转义体
   写进同一块按容量预留的缓冲);渲染层泵 `poll_http_msgs` **零拷贝**直投
   `IcedMessage.event`。禁止恢复"裸 handler + 独立 payload 字段 + 渲染层
   format! 再拼"形态(该链 292KB 级响应全程 5-6 份全文拷贝)。
4. **诊断打点 env 门控**——`[msg-bridge]/[msg-pump]` 类常驻 eprintln 必须
   挂 `AUTOUI_HTTP_BRIDGE_DIAG`(缺省关;高频 stderr churn 与 080 在案的
   "管道关闭打印 panic"族耦合)。

## 2. 回填派发语义(seq 门控,PLAN-765 F-4)

- `DesktopMessage::HttpBackfill` 是消息桥回填的**专用通道变体**——update
  侧经 `DynamicComponent::on_http_backfill_for` 派发:以 `state_mutation_seq`
  前后比对画界,**handler 零状态写且原先不脏 → 撤销置脏**(PLAN-062 F1
  `fire_timer` 空转拍语义在回填通道的延伸),视图走缓存分支零重建。
- 普通事件派发(`on_with_input_for` → `call_handler_for` Ok 臂)**保持
  无条件置脏**——vmref-push-only handler 依赖派发置脏兜底(F1 注释在案)。
- 逃生门:`AUTOUI_MSG_BACKFILL_DIRTY=1` 恢复回填通道无条件置_dirty(回填
  handler 依赖侧通道视觉的场景兜底)。
- **回填 handler 契约**:回填 handler 应当"写状态或无事发生"——依赖派发
  置脏实现视觉更新(不写状态也要重绘)的回填 handler 会被本门控跳过重建。

## 3. 事件名推导的载荷安全性(PLAN-765 F-5)

`derive_event_name`(renderer.rs)的 `::` 路径装饰截取**只作用于首个
`\u{1F}`(PAYLOAD_SEP)之前的事件名段**——载荷体(JSON body 内代码文本
`std::process` 等)携带 `::` 时不得截坏 handler 名。历史缺陷:按全文
`rfind("::")` 截断 → 回填静默丢拍 → store 停更(musk K10"长寿实例空渲染
/特定会话内容"观察同形)。

## 4. 退出审计四分法(PLAN-765 F-7/F-8/F-9,扩展 PLAN-575 三挂点)

| 挂点 | site 串 | 覆盖 |
|---|---|---|
| vm_process_exit | `vm_process_exit` | VM `Process.exit` FFI |
| panic | `panic msg=.. loc=..` | 任意线程 Rust panic(码 101) |
| main_return | `main_return` | iced run Ok 返回(码 0) |
| main_err | `main_err` / `main_err_ai_mode` | main 返回 Err / ai_mode exit(1) |
| mem_guard 逃生 | `mem_guard_f12_escape` | 冻结态 F12 exit(42) |
| **原生崩溃** | `native_crash` | 未处理 SEH 异常(`SetUnhandledExceptionFilter`,落笔后 CONTINUE_SEARCH 保 WER 语义) |

**外部击杀指纹(判读矩阵第 4 类的完备化)**:审计册**零行 + 退出码 1**
(Windows TerminateProcess/taskkill 强杀退出码恒 1)⇒ 外部终止,进程内
无未覆盖静默退出路径。native_crash 钩覆盖边界:不覆盖外部
TerminateProcess(不可拦截)与 `__fastfail`(刻意绕过 SEH);栈溢出场景
尽力而为;后续加载模块可能覆盖 filter(best effort)。

## 5. 测量口径注记(soak 仪器判读)

- 私有内存采样锯齿(±1MB 级堆瞬态起伏)在 45s 短窗上的最小二乘"斜率"
  会虚高一个量级(基线五轮"idle 30-72MB/min"实为绝对增量 +1~2MB/45s 的
  噪声放大);**判 idle 泄漏必须用绝对增量 + ≥300s 窗的段趋势**。
- `soak-active-session.mjs` 类仪器 finally 的 `taskkill /T /F` 会在收尾
  杀 VM——console 的"VM EXITED code=1"在 `written` 之后出现时是清理
  假象;判真死亡看 JSON `vm_died` 字段与采样时间线。
