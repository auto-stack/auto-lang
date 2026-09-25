# stdlib Plans

| Plan | 标题 | 状态 | 归档 | 一句话沉淀 |
|---|---|---|---|---|
| 696 | stdlib-http-server-runtime-hardening | ✅（reviewed→archived） | archive/ | VM HTTP 请求体完整读取、同线程 VM owner、SSE 协作调度/断连取消；记录 `.at`/VM native/a2r-std/生成 Axum/merge-split 的实际边界；详档见 [backend-assembly](design/backend-assembly.md) 与 [http-server](design/http-server.md) |
| 699 | vm-http-transport-axum-bridge | ✅（reviewed→archived） | archive/ | VM `#[api]` HTTP 迁 Axum/Hyper（专属 net 线程+owned 桥+预算面 431/408/413/503），手写解析退出调用图；优雅关闭+端口复绑；SSE 帧流 waker 接线与断连回收；详档见 [http-server](design/http-server.md) 与 [backend-assembly](design/backend-assembly.md) |
