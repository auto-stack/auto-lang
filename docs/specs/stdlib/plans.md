# stdlib Plans

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 696 | stdlib-http-server-runtime-hardening | ✅（reviewed→archived） | archive/ | VM HTTP 请求体完整读取、同线程 VM owner、SSE 协作调度/断连取消；记录 `.at`/VM native/a2r-std/生成 Axum/merge-split 的实际边界；详档见 [backend-assembly](design/backend-assembly.md) 与 [http-server](design/http-server.md) |
| 699 | vm-http-transport-axum-bridge | ✅（reviewed→archived） | archive/ | VM `#[api]` HTTP 迁 Axum/Hyper（专属 net 线程+owned 桥+预算面 431/408/413/503），手写解析退出调用图；优雅关闭+端口复绑；SSE 帧流 waker 接线与断连回收；详档见 [http-server](design/http-server.md) 与 [backend-assembly](design/backend-assembly.md) |
| 705 | vm-http-handler-async-lifecycle | ✅（reviewed→archived） | archive/ | VM HTTP handler 段驱动异步等待（park/resume 零轮询唤醒+middleware/closure/RequestBuilder 段入口+~T 元数据门三形态）、请求作用域（生命期许可总上限+取消三类判据+失效跳过）、结果通道单次终结（迟到完成不复活）+ 固定 async 客户端 executor（零每请求线程+五限额）；顺手修复 intercept_error 跨帧 try 恢复缺口与 706 纯 VM 形态编译断；详档见 [http-handler-async-lifecycle](design/http-handler-async-lifecycle.md) |
| 707 | vm-http-stream-async-relay | ✅（reviewed→archived） | archive/ | 外部 HTTP/SSE 流异步消费/转发/取消（统一资源表+独立许可对+有界队列背压+abort 实停——闭合 705「取消只删结果槽」缺口）、增量 SSE decoder（跨 chunk UTF-8/CR·LF·CRLF/预算）、for 三形态（内联/变量/生成器+DashMap 自死锁修复）、scope 资源组级联收口 + 真 TCP relay E2E；顺手修复 http.at 流族 VM 轨静默 no-op 与成功 job AbortHandle 慢性泄漏；详档见 [http-stream-lifecycle](design/http-stream-lifecycle.md) |
